//! Full Query implementation with bidirectional control protocol

use dashmap::DashMap;
use futures::stream::StreamExt;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::oneshot;

use crate::errors::{ClaudeError, Result};
use crate::types::hooks::{HookCallback, HookContext, HookInput, HookMatcher};
use crate::types::mcp::McpSdkServerConfig;
use crate::types::permissions::{
    CanUseToolCallback, PermissionResult, PermissionResultDeny, PermissionUpdate,
    ToolPermissionContext,
};

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
    /// Transport for communication - uses &self methods via internal sync
    pub(crate) transport: Arc<dyn Transport>,
    /// Hook callbacks - concurrent access via DashMap
    hook_callbacks: Arc<DashMap<String, HookCallback>>,
    /// SDK MCP servers - concurrent access via DashMap
    sdk_mcp_servers: Arc<DashMap<String, McpSdkServerConfig>>,
    /// Tool permission callback - handles AskUserQuestion and other tool permissions
    can_use_tool: Option<CanUseToolCallback>,
    next_callback_id: Arc<AtomicU64>,
    request_counter: Arc<AtomicU64>,
    /// Pending control request responses - concurrent access via DashMap
    pending_responses: Arc<DashMap<String, oneshot::Sender<serde_json::Value>>>,
    message_tx: flume::Sender<serde_json::Value>,
    /// Message receiver - cloneable without mutex thanks to flume
    pub(crate) message_rx: flume::Receiver<serde_json::Value>,
    /// Initialization result - set once during initialize(), read many times
    initialization_result: OnceLock<serde_json::Value>,
}

impl QueryFull {
    /// Create a new Query
    pub fn new(transport: Box<dyn Transport>) -> Self {
        let (message_tx, message_rx) = flume::unbounded();

        Self {
            transport: Arc::from(transport),
            hook_callbacks: Arc::new(DashMap::new()),
            sdk_mcp_servers: Arc::new(DashMap::new()),
            can_use_tool: None,
            next_callback_id: Arc::new(AtomicU64::new(0)),
            request_counter: Arc::new(AtomicU64::new(0)),
            pending_responses: Arc::new(DashMap::new()),
            message_tx,
            message_rx,
            initialization_result: OnceLock::new(),
        }
    }

    /// Create a new Query with a pre-existing Arc transport (for testing)
    #[cfg(feature = "testing")]
    pub fn new_with_transport(transport: Arc<dyn Transport>) -> Self {
        let (message_tx, message_rx) = flume::unbounded();

        Self {
            transport,
            hook_callbacks: Arc::new(DashMap::new()),
            sdk_mcp_servers: Arc::new(DashMap::new()),
            can_use_tool: None,
            next_callback_id: Arc::new(AtomicU64::new(0)),
            request_counter: Arc::new(AtomicU64::new(0)),
            pending_responses: Arc::new(DashMap::new()),
            message_tx,
            message_rx,
            initialization_result: OnceLock::new(),
        }
    }

    /// Set the tool permission callback
    ///
    /// This callback is invoked when Claude requests permission to use a tool,
    /// including the `AskUserQuestion` tool. For `AskUserQuestion`, the callback
    /// should return `PermissionResultAllow` with `updated_input` containing the
    /// `answers` field populated with user responses.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use std::sync::Arc;
    /// use claude_agent_sdk_rs::types::permissions::*;
    ///
    /// let callback: CanUseToolCallback = Arc::new(|tool_name, input, _ctx| {
    ///     Box::pin(async move {
    ///         if tool_name == "AskUserQuestion" {
    ///             // Handle user questions by providing answers
    ///             let mut updated = input.clone();
    ///             if let Some(obj) = updated.as_object_mut() {
    ///                 obj.insert("answers".to_string(), serde_json::json!({
    ///                     "Which option?": "Option A"
    ///                 }));
    ///             }
    ///             PermissionResult::Allow(PermissionResultAllow {
    ///                 updated_input: Some(updated),
    ///                 ..Default::default()
    ///             })
    ///         } else {
    ///             // Allow other tools
    ///             PermissionResult::Allow(PermissionResultAllow::default())
    ///         }
    ///     })
    /// });
    /// ```
    pub fn set_can_use_tool(&mut self, callback: Option<CanUseToolCallback>) {
        self.can_use_tool = callback;
    }

    /// Set SDK MCP servers
    pub fn set_sdk_mcp_servers(&mut self, servers: HashMap<String, McpSdkServerConfig>) {
        self.sdk_mcp_servers.clear();
        for (name, config) in servers {
            self.sdk_mcp_servers.insert(name, config);
        }
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
                        self.hook_callbacks.insert(callback_id.clone(), callback);
                        callback_ids.push(callback_id);
                    }

                    let mut matcher_json = json!({
                        "matcher": matcher.matcher,
                        "hookCallbackIds": callback_ids
                    });

                    // Add timeout if specified
                    if let Some(timeout) = matcher.timeout {
                        matcher_json["timeout"] = json!(timeout);
                    }

                    event_matchers.push(matcher_json);
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

        // Store initialization result for get_server_info() (set once, read many)
        let _ = self.initialization_result.set(response.clone());

        Ok(response)
    }

    /// Start reading messages in background
    ///
    /// Returns a receiver that signals when the background task completes.
    /// The caller should store this and await it during disconnect.
    pub async fn start(&self) -> Result<oneshot::Receiver<()>> {
        let transport = Arc::clone(&self.transport);
        let transport_for_hooks = Arc::clone(&self.transport);
        let hook_callbacks = Arc::clone(&self.hook_callbacks);
        let sdk_mcp_servers = Arc::clone(&self.sdk_mcp_servers);
        let can_use_tool = self.can_use_tool.clone();
        let pending_responses = Arc::clone(&self.pending_responses);
        let message_tx = self.message_tx.clone();

        // Create a channel to signal when background task is ready
        let (ready_tx, ready_rx) = oneshot::channel();

        // Create a channel to signal when background task completes
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        tokio::spawn(async move {
            // No lock needed - Transport uses &self methods with internal sync
            let mut stream = transport.read_messages();

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
                                    // DashMap remove returns Option<(K, V)>
                                    if let Some((_, tx)) =
                                        pending_responses.remove(&response.response.request_id)
                                    {
                                        let _ = tx.send(response.response.data);
                                    }
                                }
                            }
                            Some("control_request") => {
                                // Handle incoming control request (e.g., hook callback, MCP message, permission request)
                                if let Ok(request) = serde_json::from_value::<IncomingControlRequest>(
                                    message.clone(),
                                ) {
                                    let transport_clone = Arc::clone(&transport_for_hooks);
                                    let hook_callbacks_clone = Arc::clone(&hook_callbacks);
                                    let sdk_mcp_servers_clone = Arc::clone(&sdk_mcp_servers);
                                    let can_use_tool_clone = can_use_tool.clone();

                                    tokio::spawn(async move {
                                        if let Err(e) = Self::handle_control_request(
                                            request,
                                            transport_clone,
                                            hook_callbacks_clone,
                                            sdk_mcp_servers_clone,
                                            can_use_tool_clone,
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

            // Signal that background task has completed
            let _ = shutdown_tx.send(());
        });

        // Wait for background task to be ready before returning
        ready_rx
            .await
            .map_err(|_| ClaudeError::Transport("Background task failed to start".to_string()))?;

        Ok(shutdown_rx)
    }

    /// Handle incoming control request from CLI
    ///
    /// IMPORTANT: This function MUST always send a response back to the CLI,
    /// even when an error occurs. If we return early without sending a response,
    /// the CLI will hang waiting forever, eventually timing out with "Stream closed".
    async fn handle_control_request(
        request: IncomingControlRequest,
        transport: Arc<dyn Transport>,
        hook_callbacks: Arc<DashMap<String, HookCallback>>,
        sdk_mcp_servers: Arc<DashMap<String, McpSdkServerConfig>>,
        can_use_tool: Option<CanUseToolCallback>,
    ) -> Result<()> {
        let request_id = request.request_id.clone();
        let request_data = request.request;

        // Process the request and get either success data or an error message
        let result = Self::process_control_request(
            &request_id,
            &request_data,
            &hook_callbacks,
            &sdk_mcp_servers,
            can_use_tool,
        )
        .await;

        // Always send a response, even on error
        let response = match result {
            Ok(response_data) => {
                json!({
                    "type": "control_response",
                    "response": {
                        "subtype": "success",
                        "request_id": request_id,
                        "response": response_data
                    }
                })
            }
            Err(e) => {
                tracing::error!("Control request {} failed: {}", request_id, e);
                json!({
                    "type": "control_response",
                    "response": {
                        "subtype": "error",
                        "request_id": request_id,
                        "error": e.to_string()
                    }
                })
            }
        };

        let response_str = serde_json::to_string(&response)
            .map_err(|e| ClaudeError::Transport(format!("Failed to serialize response: {}", e)))?;

        // Write via transport - stdin/stdout have separate locks, no deadlock
        transport.write(&response_str).await?;

        Ok(())
    }

    /// Process the control request and return either success data or an error.
    /// This is separated from handle_control_request to allow proper error handling.
    async fn process_control_request(
        _request_id: &str,
        request_data: &serde_json::Value,
        hook_callbacks: &Arc<DashMap<String, HookCallback>>,
        sdk_mcp_servers: &Arc<DashMap<String, McpSdkServerConfig>>,
        can_use_tool: Option<CanUseToolCallback>,
    ) -> Result<serde_json::Value> {
        let subtype = request_data
            .get("subtype")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ClaudeError::ControlProtocol("Missing subtype".to_string()))?;

        match subtype {
            "hook_callback" => {
                // Execute hook callback
                let callback_id = request_data
                    .get("callback_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ClaudeError::ControlProtocol("Missing callback_id".to_string())
                    })?;

                // Clone the callback Arc to release the DashMap guard before async call
                let callback = hook_callbacks
                    .get(callback_id)
                    .map(|r| r.clone())
                    .ok_or_else(|| {
                        ClaudeError::ControlProtocol(format!(
                            "Hook callback not found: {}",
                            callback_id
                        ))
                    })?;

                // Parse hook input
                let input_json = request_data.get("input").cloned().unwrap_or(json!({}));
                let hook_input: HookInput = serde_json::from_value(input_json).map_err(|e| {
                    ClaudeError::ControlProtocol(format!("Failed to parse hook input: {}", e))
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
                    ClaudeError::ControlProtocol(format!("Failed to serialize hook output: {}", e))
                })
            }
            "mcp_message" => {
                // Handle SDK MCP message
                let server_name = request_data
                    .get("server_name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ClaudeError::ControlProtocol(
                            "Missing server_name for mcp_message".to_string(),
                        )
                    })?;

                let mcp_message = request_data.get("message").ok_or_else(|| {
                    ClaudeError::ControlProtocol("Missing message for mcp_message".to_string())
                })?;

                let mcp_response = Self::handle_sdk_mcp_request(
                    sdk_mcp_servers.clone(),
                    server_name,
                    mcp_message.clone(),
                )
                .await?;

                Ok(json!({"mcp_response": mcp_response}))
            }
            "can_use_tool" => {
                // Handle tool permission request (including AskUserQuestion)
                let tool_name = request_data
                    .get("tool_name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ClaudeError::ControlProtocol(
                            "Missing tool_name in can_use_tool request".to_string(),
                        )
                    })?;

                let tool_input = request_data.get("input").cloned().unwrap_or(json!({}));

                let suggestions: Vec<PermissionUpdate> = request_data
                    .get("suggestions")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();

                let tool_use_id = request_data
                    .get("tool_use_id")
                    .and_then(|v| v.as_str())
                    .map(String::from);

                Self::handle_permission_request(
                    tool_name,
                    tool_input,
                    suggestions,
                    tool_use_id,
                    can_use_tool,
                )
                .await
            }
            _ => Err(ClaudeError::ControlProtocol(format!(
                "Unsupported control request subtype: {}",
                subtype
            ))),
        }
    }

    /// Send control request to CLI
    async fn send_control_request(&self, request: serde_json::Value) -> Result<serde_json::Value> {
        let request_id = format!(
            "req_{}_{}",
            self.request_counter.fetch_add(1, Ordering::SeqCst),
            uuid::Uuid::new_v4().simple()
        );

        // Create oneshot channel for response
        let (tx, rx) = oneshot::channel();
        self.pending_responses.insert(request_id.clone(), tx);

        // Build and send request
        let control_request = json!({
            "type": "control_request",
            "request_id": request_id,
            "request": request
        });

        let request_str = serde_json::to_string(&control_request)
            .map_err(|e| ClaudeError::Transport(format!("Failed to serialize request: {}", e)))?;

        // Write via transport - stdin/stdout have separate locks, no deadlock
        self.transport.write(&request_str).await?;

        // Wait for response
        let response = rx.await.map_err(|_| {
            ClaudeError::ControlProtocol("Control request response channel closed".to_string())
        })?;

        Ok(response)
    }

    /// Receive messages
    #[allow(dead_code)]
    pub async fn receive_messages(&self) -> Vec<serde_json::Value> {
        let mut messages = Vec::new();
        let rx = self.message_rx.clone();

        while let Ok(message) = rx.recv_async().await {
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

    /// Rewind tracked files to their state at a specific user message.
    ///
    /// Requires:
    /// - `enable_file_checkpointing=true` to track file changes
    /// - `extra_args={"replay-user-messages": None}` to receive UserMessage
    ///   objects with `uuid` in the response stream
    ///
    /// # Arguments
    /// * `user_message_id` - UUID of the user message to rewind to. This should be
    ///   the `uuid` field from a `UserMessage` received during the conversation.
    pub async fn rewind_files(&self, user_message_id: &str) -> Result<()> {
        let request = json!({
            "subtype": "rewind_files",
            "user_message_id": user_message_id
        });

        self.send_control_request(request).await?;
        Ok(())
    }

    /// Get server initialization info
    ///
    /// Returns the initialization result that was obtained during connect().
    /// This includes information about available commands, output styles, and server capabilities.
    /// This is lock-free since initialization_result uses OnceLock.
    pub fn get_initialization_result(&self) -> Option<serde_json::Value> {
        self.initialization_result.get().cloned()
    }

    /// Handle SDK MCP request by routing to the appropriate server
    async fn handle_sdk_mcp_request(
        sdk_mcp_servers: Arc<DashMap<String, McpSdkServerConfig>>,
        server_name: &str,
        message: serde_json::Value,
    ) -> Result<serde_json::Value> {
        // Clone the server config to release the DashMap guard before async call
        let server_config = sdk_mcp_servers
            .get(server_name)
            .map(|r| r.clone())
            .ok_or_else(|| {
                ClaudeError::ControlProtocol(format!("SDK MCP server not found: {}", server_name))
            })?;

        // Call the server's handle_message method
        server_config
            .instance
            .handle_message(message)
            .await
            .map_err(|e| ClaudeError::ControlProtocol(format!("MCP server error: {}", e)))
    }

    /// Handle tool permission request by invoking the can_use_tool callback
    async fn handle_permission_request(
        tool_name: &str,
        tool_input: serde_json::Value,
        suggestions: Vec<PermissionUpdate>,
        tool_use_id: Option<String>,
        can_use_tool: Option<CanUseToolCallback>,
    ) -> Result<serde_json::Value> {
        // If no callback is configured, deny by default
        let callback = match can_use_tool {
            Some(cb) => cb,
            None => {
                return serde_json::to_value(PermissionResult::Deny(PermissionResultDeny {
                    message: "No can_use_tool callback configured".to_string(),
                    interrupt: false,
                }))
                .map_err(|e| {
                    ClaudeError::ControlProtocol(format!(
                        "Failed to serialize permission result: {}",
                        e
                    ))
                });
            }
        };

        // Build the context with tool_use_id
        let context = ToolPermissionContext {
            signal: None,
            suggestions,
            tool_use_id,
        };

        // Call the callback
        let result = callback(tool_name.to_string(), tool_input, context).await;

        // Convert to JSON
        serde_json::to_value(&result).map_err(|e| {
            ClaudeError::ControlProtocol(format!("Failed to serialize permission result: {}", e))
        })
    }
}
