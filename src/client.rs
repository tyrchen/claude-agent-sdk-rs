//! ClaudeClient for bidirectional streaming interactions with hook support

use futures::stream::Stream;
use std::pin::Pin;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

use crate::errors::{ClaudeError, Result};
use crate::internal::message_parser::MessageParser;
use crate::internal::query_full::QueryFull;
use crate::internal::transport::subprocess::QueryPrompt;
use crate::internal::transport::{SubprocessTransport, Transport};
use crate::types::config::{ClaudeAgentOptions, PermissionMode};
use crate::types::hooks::HookEvent;
use crate::types::messages::Message;

/// Client for bidirectional streaming interactions with Claude
///
/// This client provides the same functionality as Python's ClaudeSDKClient,
/// supporting bidirectional communication, streaming responses, and dynamic
/// control over the Claude session.
///
/// # Example
///
/// ```no_run
/// use claude_agent_sdk::{ClaudeClient, ClaudeAgentOptions};
/// use futures::StreamExt;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut client = ClaudeClient::new(ClaudeAgentOptions::default());
///
///     // Connect to Claude
///     client.connect().await?;
///
///     // Send a query
///     client.query("Hello Claude!").await?;
///
///     // Receive response as a stream
///     {
///         let mut stream = client.receive_response();
///         while let Some(message) = stream.next().await {
///             println!("Received: {:?}", message?);
///         }
///     }
///
///     // Disconnect
///     client.disconnect().await?;
///     Ok(())
/// }
/// ```
pub struct ClaudeClient {
    options: ClaudeAgentOptions,
    query: Option<Arc<Mutex<QueryFull>>>,
    connected: bool,
}

impl ClaudeClient {
    /// Create a new ClaudeClient
    ///
    /// # Arguments
    ///
    /// * `options` - Configuration options for the Claude client
    ///
    /// # Example
    ///
    /// ```no_run
    /// use claude_agent_sdk::{ClaudeClient, ClaudeAgentOptions};
    ///
    /// let client = ClaudeClient::new(ClaudeAgentOptions::default());
    /// ```
    pub fn new(options: ClaudeAgentOptions) -> Self {
        Self {
            options,
            query: None,
            connected: false,
        }
    }

    /// Connect to Claude (analogous to Python's __aenter__)
    ///
    /// This establishes the connection to the Claude Code CLI and initializes
    /// the bidirectional communication channel.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Claude CLI cannot be found or started
    /// - The initialization handshake fails
    /// - Hook registration fails
    pub async fn connect(&mut self) -> Result<()> {
        if self.connected {
            return Ok(());
        }

        // Create transport in streaming mode (no initial prompt)
        let prompt = QueryPrompt::Streaming;
        let mut transport = SubprocessTransport::new(prompt, self.options.clone())?;

        // Don't send initial prompt - we'll use query() for that
        transport.connect().await?;

        // Extract stdin for direct access (avoids transport lock deadlock)
        let stdin = Arc::clone(&transport.stdin);

        // Create Query with hooks
        let mut query = QueryFull::new(Box::new(transport));
        query.set_stdin(stdin);

        // Extract SDK MCP servers from options
        let sdk_mcp_servers =
            if let crate::types::mcp::McpServers::Dict(servers_dict) = &self.options.mcp_servers {
                servers_dict
                    .iter()
                    .filter_map(|(name, config)| {
                        if let crate::types::mcp::McpServerConfig::Sdk(sdk_config) = config {
                            Some((name.clone(), sdk_config.clone()))
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                std::collections::HashMap::new()
            };
        query.set_sdk_mcp_servers(sdk_mcp_servers).await;

        // Convert hooks to internal format
        let hooks = self.options.hooks.as_ref().map(|hooks_map| {
            hooks_map
                .iter()
                .map(|(event, matchers)| {
                    let event_name = match event {
                        HookEvent::PreToolUse => "PreToolUse",
                        HookEvent::PostToolUse => "PostToolUse",
                        HookEvent::UserPromptSubmit => "UserPromptSubmit",
                        HookEvent::Stop => "Stop",
                        HookEvent::SubagentStop => "SubagentStop",
                        HookEvent::PreCompact => "PreCompact",
                    };
                    (event_name.to_string(), matchers.clone())
                })
                .collect()
        });

        // Start reading messages in background FIRST
        // This must happen before initialize() because initialize()
        // sends a control request and waits for response
        query.start().await?;

        // Initialize with hooks (sends control request)
        query.initialize(hooks).await?;

        self.query = Some(Arc::new(Mutex::new(query)));
        self.connected = true;

        Ok(())
    }

    /// Send a query to Claude
    ///
    /// This sends a new user prompt to Claude. Claude will remember the context
    /// of previous queries within the same session.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The user prompt to send
    ///
    /// # Errors
    ///
    /// Returns an error if the client is not connected or if sending fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use claude_agent_sdk::{ClaudeClient, ClaudeAgentOptions};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = ClaudeClient::new(ClaudeAgentOptions::default());
    /// # client.connect().await?;
    /// client.query("What is 2 + 2?").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query(&mut self, prompt: impl Into<String>) -> Result<()> {
        let query = self.query.as_ref().ok_or_else(|| {
            ClaudeError::InvalidConfig("Client not connected. Call connect() first.".to_string())
        })?;

        let prompt_str = prompt.into();

        // Format as JSON message for stream-json input format
        let user_message = serde_json::json!({
            "type": "user",
            "message": {
                "role": "user",
                "content": prompt_str
            }
        });

        let message_str = serde_json::to_string(&user_message).map_err(|e| {
            ClaudeError::Transport(format!("Failed to serialize user message: {}", e))
        })?;

        // Write directly to stdin (bypasses transport lock)
        let query_guard = query.lock().await;
        let stdin = query_guard.stdin.clone();
        drop(query_guard);

        if let Some(stdin_arc) = stdin {
            let mut stdin_guard = stdin_arc.lock().await;
            if let Some(ref mut stdin_stream) = *stdin_guard {
                stdin_stream
                    .write_all(message_str.as_bytes())
                    .await
                    .map_err(|e| ClaudeError::Transport(format!("Failed to write query: {}", e)))?;
                stdin_stream.write_all(b"\n").await.map_err(|e| {
                    ClaudeError::Transport(format!("Failed to write newline: {}", e))
                })?;
                stdin_stream
                    .flush()
                    .await
                    .map_err(|e| ClaudeError::Transport(format!("Failed to flush: {}", e)))?;
            } else {
                return Err(ClaudeError::Transport("stdin not available".to_string()));
            }
        } else {
            return Err(ClaudeError::Transport("stdin not set".to_string()));
        }

        Ok(())
    }

    /// Receive all messages as a stream (continuous)
    ///
    /// This method returns a stream that yields all messages from Claude
    /// indefinitely until the stream is closed or an error occurs.
    ///
    /// Use this when you want to process all messages, including multiple
    /// responses and system events.
    ///
    /// # Returns
    ///
    /// A stream of `Result<Message>` that continues until the connection closes.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use claude_agent_sdk::{ClaudeClient, ClaudeAgentOptions};
    /// # use futures::StreamExt;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = ClaudeClient::new(ClaudeAgentOptions::default());
    /// # client.connect().await?;
    /// # client.query("Hello").await?;
    /// let mut stream = client.receive_messages();
    /// while let Some(message) = stream.next().await {
    ///     println!("Received: {:?}", message?);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn receive_messages(&self) -> Pin<Box<dyn Stream<Item = Result<Message>> + Send + '_>> {
        let query = match &self.query {
            Some(q) => Arc::clone(q),
            None => {
                return Box::pin(futures::stream::once(async {
                    Err(ClaudeError::InvalidConfig(
                        "Client not connected. Call connect() first.".to_string(),
                    ))
                }));
            }
        };

        Box::pin(async_stream::stream! {
            let rx: Arc<Mutex<tokio::sync::mpsc::UnboundedReceiver<serde_json::Value>>> = {
                let query_guard = query.lock().await;
                Arc::clone(&query_guard.message_rx)
            };

            loop {
                let message = {
                    let mut rx_guard = rx.lock().await;
                    rx_guard.recv().await
                };

                match message {
                    Some(json) => {
                        match MessageParser::parse(json) {
                            Ok(msg) => yield Ok(msg),
                            Err(e) => {
                                eprintln!("Failed to parse message: {}", e);
                                yield Err(e);
                            }
                        }
                    }
                    None => break,
                }
            }
        })
    }

    /// Receive messages until a ResultMessage
    ///
    /// This method returns a stream that yields messages until it encounters
    /// a `ResultMessage`, which signals the completion of a Claude response.
    ///
    /// This is the most common pattern for handling Claude responses, as it
    /// processes one complete "turn" of the conversation.
    ///
    /// # Returns
    ///
    /// A stream of `Result<Message>` that ends when a ResultMessage is received.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use claude_agent_sdk::{ClaudeClient, ClaudeAgentOptions, Message};
    /// # use futures::StreamExt;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = ClaudeClient::new(ClaudeAgentOptions::default());
    /// # client.connect().await?;
    /// # client.query("Hello").await?;
    /// let mut stream = client.receive_response();
    /// while let Some(message) = stream.next().await {
    ///     match message? {
    ///         Message::Assistant(msg) => println!("Assistant: {:?}", msg),
    ///         Message::Result(result) => {
    ///             println!("Done! Cost: ${:?}", result.total_cost_usd);
    ///             break;
    ///         }
    ///         _ => {}
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn receive_response(&self) -> Pin<Box<dyn Stream<Item = Result<Message>> + Send + '_>> {
        let query = match &self.query {
            Some(q) => Arc::clone(q),
            None => {
                return Box::pin(futures::stream::once(async {
                    Err(ClaudeError::InvalidConfig(
                        "Client not connected. Call connect() first.".to_string(),
                    ))
                }));
            }
        };

        Box::pin(async_stream::stream! {
            let rx: Arc<Mutex<tokio::sync::mpsc::UnboundedReceiver<serde_json::Value>>> = {
                let query_guard = query.lock().await;
                Arc::clone(&query_guard.message_rx)
            };

            loop {
                let message = {
                    let mut rx_guard = rx.lock().await;
                    rx_guard.recv().await
                };

                match message {
                    Some(json) => {
                        match MessageParser::parse(json) {
                            Ok(msg) => {
                                let is_result = matches!(msg, Message::Result(_));
                                yield Ok(msg);
                                if is_result {
                                    break;
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to parse message: {}", e);
                                yield Err(e);
                            }
                        }
                    }
                    None => break,
                }
            }
        })
    }

    /// Send an interrupt signal to stop the current Claude operation
    ///
    /// This is analogous to Python's `client.interrupt()`.
    ///
    /// # Errors
    ///
    /// Returns an error if the client is not connected or if sending fails.
    pub async fn interrupt(&self) -> Result<()> {
        let query = self.query.as_ref().ok_or_else(|| {
            ClaudeError::InvalidConfig("Client not connected. Call connect() first.".to_string())
        })?;

        let query_guard = query.lock().await;
        query_guard.interrupt().await
    }

    /// Change the permission mode dynamically
    ///
    /// This is analogous to Python's `client.set_permission_mode()`.
    ///
    /// # Arguments
    ///
    /// * `mode` - The new permission mode to set
    ///
    /// # Errors
    ///
    /// Returns an error if the client is not connected or if sending fails.
    pub async fn set_permission_mode(&self, mode: PermissionMode) -> Result<()> {
        let query = self.query.as_ref().ok_or_else(|| {
            ClaudeError::InvalidConfig("Client not connected. Call connect() first.".to_string())
        })?;

        let query_guard = query.lock().await;
        query_guard.set_permission_mode(mode).await
    }

    /// Change the AI model dynamically
    ///
    /// This is analogous to Python's `client.set_model()`.
    ///
    /// # Arguments
    ///
    /// * `model` - The new model name, or None to use default
    ///
    /// # Errors
    ///
    /// Returns an error if the client is not connected or if sending fails.
    pub async fn set_model(&self, model: Option<&str>) -> Result<()> {
        let query = self.query.as_ref().ok_or_else(|| {
            ClaudeError::InvalidConfig("Client not connected. Call connect() first.".to_string())
        })?;

        let query_guard = query.lock().await;
        query_guard.set_model(model).await
    }

    /// Get server initialization info
    ///
    /// Returns information about the Claude Code CLI session.
    pub async fn get_server_info(&self) -> Option<serde_json::Value> {
        self.query.as_ref()?;
        // TODO: Store and return initialization result
        None
    }

    /// Disconnect from Claude (analogous to Python's __aexit__)
    ///
    /// This cleanly shuts down the connection to Claude Code CLI.
    ///
    /// # Errors
    ///
    /// Returns an error if disconnection fails.
    pub async fn disconnect(&mut self) -> Result<()> {
        if !self.connected {
            return Ok(());
        }

        if let Some(query) = self.query.take() {
            // Close stdin first (using direct access) to signal CLI to exit
            // This will cause the background task to finish and release transport lock
            let query_guard = query.lock().await;
            if let Some(ref stdin_arc) = query_guard.stdin {
                let mut stdin_guard = stdin_arc.lock().await;
                if let Some(mut stdin_stream) = stdin_guard.take() {
                    let _ = stdin_stream.shutdown().await;
                }
            }
            let transport = Arc::clone(&query_guard.transport);
            drop(query_guard);

            // Give background task a moment to finish reading and release lock
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            let mut transport_guard = transport.lock().await;
            transport_guard.close().await?;
        }

        self.connected = false;
        Ok(())
    }
}

impl Drop for ClaudeClient {
    fn drop(&mut self) {
        // Note: We can't run async code in Drop, so we can't guarantee clean shutdown
        // Users should call disconnect() explicitly
        if self.connected {
            eprintln!("Warning: ClaudeClient dropped without calling disconnect(). Resources may not be cleaned up properly.");
        }
    }
}
