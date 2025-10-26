//! Full Query implementation with bidirectional control protocol

use futures::stream::StreamExt;
use serde_json::json;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::sync::{mpsc, oneshot, Mutex};

use crate::errors::{ClaudeError, ControlProtocolError, Result, TransportError};
use crate::types::hooks::{HookCallback, HookContext, HookInput, HookMatcher};
use crate::types::mcp::McpSdkServerConfig;
use tracing::{debug, instrument};

use super::transport::Transport;

/// Control request from SDK to CLI
#[allow(dead_code)]
#[derive(Debug, serde::Serialize)]
struct ControlRequest {
    #[serde(rename = "type")]
    type_: String,
    request_id: String,
    request: serde_json::Value,
}

/// Control response from CLI to SDK
#[derive(Debug, serde::Deserialize)]
struct ControlResponse {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    type_: String,
    response: ControlResponseData,
}

#[derive(Debug, serde::Deserialize)]
struct ControlResponseData {
    #[allow(dead_code)]
    subtype: String,
    request_id: String,
    #[serde(flatten)]
    data: serde_json::Value,
}

/// Control request from CLI to SDK
#[derive(Debug, serde::Deserialize)]
struct IncomingControlRequest {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    type_: String,
    request_id: String,
    request: serde_json::Value,
}

/// Full Query implementation with bidirectional control protocol
pub struct QueryFull {
    pub(crate) transport: Arc<Mutex<Box<dyn Transport>>>,
    hook_callbacks: Arc<Mutex<HashMap<String, HookCallback>>>,
    sdk_mcp_servers: Arc<Mutex<HashMap<String, McpSdkServerConfig>>>,
    next_callback_id: Arc<AtomicU64>,
    request_counter: Arc<AtomicU64>,
    pending_responses: Arc<Mutex<HashMap<String, oneshot::Sender<serde_json::Value>>>>,
    message_tx: mpsc::UnboundedSender<serde_json::Value>,
    pub(crate) message_rx: Arc<Mutex<mpsc::UnboundedReceiver<serde_json::Value>>>,
    // Direct access to stdin for writes (bypasses transport lock)
    pub(crate) stdin: Option<Arc<Mutex<Option<tokio::process::ChildStdin>>>>,
    // Store initialization result for get_server_info()
    initialization_result: Arc<Mutex<Option<serde_json::Value>>>,
    // Timeout for control requests
    control_request_timeout: Duration,
}

impl QueryFull {
    /// Create a new Query
    pub fn new(transport: Box<dyn Transport>, control_request_timeout: Duration) -> Self {
        let (message_tx, message_rx) = mpsc::unbounded_channel();

        Self {
            transport: Arc::new(Mutex::new(transport)),
            hook_callbacks: Arc::new(Mutex::new(HashMap::new())),
            sdk_mcp_servers: Arc::new(Mutex::new(HashMap::new())),
            next_callback_id: Arc::new(AtomicU64::new(0)),
            request_counter: Arc::new(AtomicU64::new(0)),
            pending_responses: Arc::new(Mutex::new(HashMap::new())),
            message_tx,
            message_rx: Arc::new(Mutex::new(message_rx)),
            stdin: None,
            initialization_result: Arc::new(Mutex::new(None)),
            control_request_timeout,
        }
    }

    /// Set stdin for direct write access (called from client after transport is connected)
    pub fn set_stdin(&mut self, stdin: Arc<Mutex<Option<tokio::process::ChildStdin>>>) {
        self.stdin = Some(stdin);
    }

    /// Set SDK MCP servers
    pub async fn set_sdk_mcp_servers(&mut self, servers: HashMap<String, McpSdkServerConfig>) {
        *self.sdk_mcp_servers.lock().await = servers;
    }

    /// Initialize with hooks
    pub async fn initialize(
        &self,
        hooks: Option<HashMap<String, Vec<HookMatcher>>>,
    ) -> Result<serde_json::Value> {
        // Build hooks configuration
        let mut hooks_config: HashMap<String, Vec<serde_json::Value>> = HashMap::new();

        if let Some(hooks_map) = hooks {
            for (event, matchers) in hooks_map {
                let mut event_matchers = Vec::new();

                for matcher in matchers {
                    let mut callback_ids = Vec::new();

                    for callback in matcher.hooks {
                        let callback_id = format!(
                            "hook_{}",
                            self.next_callback_id.fetch_add(1, Ordering::SeqCst)
                        );
                        self.hook_callbacks
                            .lock()
                            .await
                            .insert(callback_id.clone(), callback);
                        callback_ids.push(callback_id);
                    }

                    event_matchers.push(json!({
                        "matcher": matcher.matcher,
                        "hookCallbackIds": callback_ids
                    }));
                }

                hooks_config.insert(event, event_matchers);
            }
        }

        // Send initialize request
        let request = json!({
            "subtype": "initialize",
            "hooks": if hooks_config.is_empty() { json!(null) } else { json!(hooks_config) }
        });

        let response = self.send_control_request(request).await?;

        // Store initialization result for get_server_info()
        *self.initialization_result.lock().await = Some(response.clone());

        Ok(response)
    }

    /// Start reading messages in background
    pub async fn start(&self) -> Result<()> {
        let transport = Arc::clone(&self.transport);
        let hook_callbacks = Arc::clone(&self.hook_callbacks);
        let sdk_mcp_servers = Arc::clone(&self.sdk_mcp_servers);
        let pending_responses = Arc::clone(&self.pending_responses);
        let message_tx = self.message_tx.clone();
        let stdin = self.stdin.clone();

        // Create a channel to signal when background task is ready
        let (ready_tx, ready_rx) = oneshot::channel();

        tokio::spawn(async move {
            let mut transport_guard = transport.lock().await;
            let mut stream = transport_guard.read_messages();

            // Signal that we're ready to receive messages
            let _ = ready_tx.send(());

            while let Some(result) = stream.next().await {
                match result {
                    Ok(message) => {
                        let msg_type = message.get("type").and_then(|v| v.as_str());

                        match msg_type {
                            Some("control_response") => {
                                // Handle control response
                                if let Ok(response) =
                                    serde_json::from_value::<ControlResponse>(message.clone())
                                {
                                    let mut pending = pending_responses.lock().await;
                                    if let Some(tx) = pending.remove(&response.response.request_id)
                                    {
                                        let _ = tx.send(response.response.data);
                                    }
                                }
                            }
                            Some("control_request") => {
                                // Handle incoming control request (e.g., hook callback, MCP message)
                                if let Ok(request) = serde_json::from_value::<IncomingControlRequest>(
                                    message.clone(),
                                ) {
                                    let stdin_clone = stdin.clone();
                                    let hook_callbacks_clone = Arc::clone(&hook_callbacks);
                                    let sdk_mcp_servers_clone = Arc::clone(&sdk_mcp_servers);

                                    tokio::spawn(async move {
                                        if let Err(e) = Self::handle_control_request_with_stdin(
                                            request,
                                            stdin_clone,
                                            hook_callbacks_clone,
                                            sdk_mcp_servers_clone,
                                        )
                                        .await
                                        {
                                            eprintln!("Error handling control request: {}", e);
                                        }
                                    });
                                }
                            }
                            _ => {
                                // Regular message - send to stream
                                let _ = message_tx.send(message);
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        // Wait for background task to be ready before returning
        ready_rx
            .await
            .map_err(|_| TransportError::ConnectionClosed)?;

        Ok(())
    }

    /// Handle incoming control request from CLI (new version using stdin directly)
    async fn handle_control_request_with_stdin(
        request: IncomingControlRequest,
        stdin: Option<Arc<Mutex<Option<tokio::process::ChildStdin>>>>,
        hook_callbacks: Arc<Mutex<HashMap<String, HookCallback>>>,
        sdk_mcp_servers: Arc<Mutex<HashMap<String, McpSdkServerConfig>>>,
    ) -> Result<()> {
        let request_id = request.request_id;
        let request_data = request.request;

        let subtype = request_data
            .get("subtype")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ControlProtocolError::InvalidRequest {
                field: "subtype".to_string(),
            })?;

        let response_data: serde_json::Value = match subtype {
            "hook_callback" => {
                // Execute hook callback
                let callback_id = request_data
                    .get("callback_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ControlProtocolError::InvalidRequest {
                        field: "callback_id".to_string(),
                    })?;

                let callbacks = hook_callbacks.lock().await;
                let callback = callbacks.get(callback_id).ok_or_else(|| {
                    ControlProtocolError::HookNotFound {
                        callback_id: callback_id.to_string(),
                    }
                })?;

                // Parse hook input
                let input_json = request_data.get("input").cloned().unwrap_or(json!({}));
                let hook_input: HookInput = serde_json::from_value(input_json).map_err(|e| {
                    ClaudeError::InvalidConfig(format!("Failed to parse hook input: {}", e))
                })?;

                let tool_use_id = request_data
                    .get("tool_use_id")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let context = HookContext::default();

                // Call the hook
                let hook_output = callback(hook_input, tool_use_id, context).await;

                // Convert to JSON
                serde_json::to_value(&hook_output).map_err(|e| {
                    ClaudeError::InvalidConfig(format!("Failed to serialize hook output: {}", e))
                })?
            }
            "mcp_message" => {
                // Handle SDK MCP message
                let server_name = request_data
                    .get("server_name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ControlProtocolError::InvalidRequest {
                        field: "server_name".to_string(),
                    })?;

                let mcp_message = request_data.get("message").ok_or_else(|| {
                    ControlProtocolError::InvalidRequest {
                        field: "message".to_string(),
                    }
                })?;

                let mcp_response =
                    Self::handle_sdk_mcp_request(sdk_mcp_servers, server_name, mcp_message.clone())
                        .await?;

                json!({"mcp_response": mcp_response})
            }
            _ => {
                return Err(ControlProtocolError::UnknownSubtype {
                    subtype: subtype.to_string(),
                }
                .into());
            }
        };

        // Send success response
        let response = json!({
            "type": "control_response",
            "response": {
                "subtype": "success",
                "request_id": request_id,
                "response": response_data
            }
        });

        let response_str = serde_json::to_string(&response).map_err(|e| {
            ClaudeError::InvalidConfig(format!("Failed to serialize response: {}", e))
        })?;

        // Write directly to stdin (bypasses transport lock)
        if let Some(ref stdin_arc) = stdin {
            let mut stdin_guard = stdin_arc.lock().await;
            if let Some(ref mut stdin_stream) = *stdin_guard {
                use tokio::io::AsyncWriteExt;
                stdin_stream
                    .write_all(response_str.as_bytes())
                    .await
                    .map_err(TransportError::StdinWrite)?;
                stdin_stream
                    .write_all(b"\n")
                    .await
                    .map_err(TransportError::StdinWrite)?;
                stdin_stream
                    .flush()
                    .await
                    .map_err(TransportError::StdinWrite)?;
            } else {
                return Err(TransportError::StdinUnavailable.into());
            }
        } else {
            return Err(TransportError::StdinUnavailable.into());
        }

        Ok(())
    }

    /// Send control request to CLI
    #[instrument(skip(self, request), fields(request_id))]
    async fn send_control_request(&self, request: serde_json::Value) -> Result<serde_json::Value> {
        let start = std::time::Instant::now();

        let request_id = format!(
            "req_{}_{}",
            self.request_counter.fetch_add(1, Ordering::SeqCst),
            uuid::Uuid::new_v4().simple()
        );

        tracing::Span::current().record("request_id", request_id.as_str());
        debug!(request_id = %request_id, "Sending control request");

        // Create oneshot channel for response
        let (tx, rx) = oneshot::channel();
        self.pending_responses
            .lock()
            .await
            .insert(request_id.clone(), tx);

        // Build and send request
        let control_request = json!({
            "type": "control_request",
            "request_id": request_id,
            "request": request
        });

        let request_str = serde_json::to_string(&control_request).map_err(|e| {
            ClaudeError::InvalidConfig(format!("Failed to serialize request: {}", e))
        })?;

        // Write directly to stdin (bypasses transport lock held by background reader)
        if let Some(ref stdin) = self.stdin {
            let mut stdin_guard = stdin.lock().await;
            if let Some(ref mut stdin_stream) = *stdin_guard {
                stdin_stream
                    .write_all(request_str.as_bytes())
                    .await
                    .map_err(TransportError::StdinWrite)?;
                stdin_stream
                    .write_all(b"\n")
                    .await
                    .map_err(TransportError::StdinWrite)?;
                stdin_stream
                    .flush()
                    .await
                    .map_err(TransportError::StdinWrite)?;
            } else {
                return Err(TransportError::StdinUnavailable.into());
            }
        } else {
            return Err(TransportError::StdinUnavailable.into());
        }

        // Wait for response with timeout
        let response = tokio::time::timeout(self.control_request_timeout, rx)
            .await
            .map_err(|_| ControlProtocolError::Timeout {
                timeout_ms: self.control_request_timeout.as_millis() as u64,
            })?
            .map_err(|_| TransportError::ConnectionClosed)?;

        let duration = start.elapsed();
        debug!(
            duration_ms = duration.as_millis(),
            "Control request completed"
        );

        Ok(response)
    }

    /// Receive messages
    #[allow(dead_code)]
    pub async fn receive_messages(&self) -> Vec<serde_json::Value> {
        let mut messages = Vec::new();
        let mut rx = self.message_rx.lock().await;

        while let Some(message) = rx.recv().await {
            messages.push(message);
        }

        messages
    }

    /// Send interrupt signal to Claude
    pub async fn interrupt(&self) -> Result<()> {
        let request = json!({
            "subtype": "interrupt"
        });

        self.send_control_request(request).await?;
        Ok(())
    }

    /// Change permission mode dynamically
    pub async fn set_permission_mode(
        &self,
        mode: crate::types::config::PermissionMode,
    ) -> Result<()> {
        let mode_str = match mode {
            crate::types::config::PermissionMode::Default => "default",
            crate::types::config::PermissionMode::AcceptEdits => "acceptEdits",
            crate::types::config::PermissionMode::Plan => "plan",
            crate::types::config::PermissionMode::BypassPermissions => "bypassPermissions",
        };

        let request = json!({
            "subtype": "set_permission_mode",
            "mode": mode_str
        });

        self.send_control_request(request).await?;
        Ok(())
    }

    /// Change AI model dynamically
    pub async fn set_model(&self, model: Option<&str>) -> Result<()> {
        let request = json!({
            "subtype": "set_model",
            "model": model
        });

        self.send_control_request(request).await?;
        Ok(())
    }

    /// Get server initialization info
    ///
    /// Returns the initialization result that was obtained during connect().
    /// This includes information about available commands, output styles, and server capabilities.
    pub async fn get_initialization_result(&self) -> Option<serde_json::Value> {
        self.initialization_result.lock().await.clone()
    }

    /// Handle SDK MCP request by routing to the appropriate server
    async fn handle_sdk_mcp_request(
        sdk_mcp_servers: Arc<Mutex<HashMap<String, McpSdkServerConfig>>>,
        server_name: &str,
        message: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let servers = sdk_mcp_servers.lock().await;
        let server_config =
            servers
                .get(server_name)
                .ok_or_else(|| ControlProtocolError::McpServerNotFound {
                    server_name: server_name.to_string(),
                })?;

        // Call the server's handle_message method
        server_config
            .instance
            .handle_message(message)
            .await
            .map_err(|e| ClaudeError::InvalidConfig(format!("MCP server error: {}", e)))
    }
}
