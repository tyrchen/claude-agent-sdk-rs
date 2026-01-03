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
use crate::types::messages::{Message, UserContentBlock};

/// Client for bidirectional streaming interactions with Claude
///
/// This client provides the same functionality as Python's ClaudeSDKClient,
/// supporting bidirectional communication, streaming responses, and dynamic
/// control over the Claude session.
///
/// # Example
///
/// ```no_run
/// use claude_agent_sdk_rs::{ClaudeClient, ClaudeAgentOptions};
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
    /// use claude_agent_sdk_rs::{ClaudeClient, ClaudeAgentOptions};
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
    /// # use claude_agent_sdk_rs::{ClaudeClient, ClaudeAgentOptions};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = ClaudeClient::new(ClaudeAgentOptions::default());
    /// # client.connect().await?;
    /// client.query("What is 2 + 2?").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query(&mut self, prompt: impl Into<String>) -> Result<()> {
        self.query_with_session(prompt, "default").await
    }

    /// Send a query to Claude with a specific session ID
    ///
    /// This sends a new user prompt to Claude. Different session IDs maintain
    /// separate conversation contexts.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The user prompt to send
    /// * `session_id` - Session identifier for the conversation
    ///
    /// # Errors
    ///
    /// Returns an error if the client is not connected or if sending fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use claude_agent_sdk_rs::{ClaudeClient, ClaudeAgentOptions};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = ClaudeClient::new(ClaudeAgentOptions::default());
    /// # client.connect().await?;
    /// // Separate conversation contexts
    /// client.query_with_session("First question", "session-1").await?;
    /// client.query_with_session("Different question", "session-2").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query_with_session(
        &mut self,
        prompt: impl Into<String>,
        session_id: impl Into<String>,
    ) -> Result<()> {
        let query = self.query.as_ref().ok_or_else(|| {
            ClaudeError::InvalidConfig("Client not connected. Call connect() first.".to_string())
        })?;

        let prompt_str = prompt.into();
        let session_id_str = session_id.into();

        // Format as JSON message for stream-json input format
        let user_message = serde_json::json!({
            "type": "user",
            "message": {
                "role": "user",
                "content": prompt_str
            },
            "session_id": session_id_str
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

    /// Send a query with structured content blocks (supports images)
    ///
    /// This method enables multimodal queries in bidirectional streaming mode.
    /// Use it to send images alongside text for vision-related tasks.
    ///
    /// # Arguments
    ///
    /// * `content` - A vector of content blocks (text and/or images)
    ///
    /// # Errors
    ///
    /// Returns an error if the client is not connected or if sending fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use claude_agent_sdk_rs::{ClaudeClient, ClaudeAgentOptions, UserContentBlock};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = ClaudeClient::new(ClaudeAgentOptions::default());
    /// # client.connect().await?;
    /// let base64_data = "iVBORw0KGgo..."; // base64 encoded image
    /// client.query_with_content(vec![
    ///     UserContentBlock::text("What's in this image?"),
    ///     UserContentBlock::image_base64("image/png", base64_data)?,
    /// ]).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query_with_content(
        &mut self,
        content: impl Into<Vec<UserContentBlock>>,
    ) -> Result<()> {
        self.query_with_content_and_session(content, "default")
            .await
    }

    /// Send a query with structured content blocks and a specific session ID
    ///
    /// This method enables multimodal queries with session management for
    /// maintaining separate conversation contexts.
    ///
    /// # Arguments
    ///
    /// * `content` - A vector of content blocks (text and/or images)
    /// * `session_id` - Session identifier for the conversation
    ///
    /// # Errors
    ///
    /// Returns an error if the client is not connected or if sending fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use claude_agent_sdk_rs::{ClaudeClient, ClaudeAgentOptions, UserContentBlock};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = ClaudeClient::new(ClaudeAgentOptions::default());
    /// # client.connect().await?;
    /// client.query_with_content_and_session(
    ///     vec![
    ///         UserContentBlock::text("Analyze this chart"),
    ///         UserContentBlock::image_url("https://example.com/chart.png"),
    ///     ],
    ///     "analysis-session",
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query_with_content_and_session(
        &mut self,
        content: impl Into<Vec<UserContentBlock>>,
        session_id: impl Into<String>,
    ) -> Result<()> {
        let query = self.query.as_ref().ok_or_else(|| {
            ClaudeError::InvalidConfig("Client not connected. Call connect() first.".to_string())
        })?;

        let content_blocks: Vec<UserContentBlock> = content.into();
        let session_id_str = session_id.into();

        // Format as JSON message for stream-json input format
        // Content is an array of content blocks, not a simple string
        let user_message = serde_json::json!({
            "type": "user",
            "message": {
                "role": "user",
                "content": content_blocks
            },
            "session_id": session_id_str
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
    /// # use claude_agent_sdk_rs::{ClaudeClient, ClaudeAgentOptions};
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
    /// # use claude_agent_sdk_rs::{ClaudeClient, ClaudeAgentOptions, Message};
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

    /// Rewind tracked files to their state at a specific user message.
    ///
    /// This is analogous to Python's `client.rewind_files()`.
    ///
    /// # Requirements
    ///
    /// - `enable_file_checkpointing=true` in options to track file changes
    /// - `extra_args={"replay-user-messages": None}` to receive UserMessage
    ///   objects with `uuid` in the response stream
    ///
    /// # Arguments
    ///
    /// * `user_message_id` - UUID of the user message to rewind to. This should be
    ///   the `uuid` field from a `UserMessage` received during the conversation.
    ///
    /// # Errors
    ///
    /// Returns an error if the client is not connected or if sending fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use claude_agent_sdk_rs::{ClaudeClient, ClaudeAgentOptions, Message};
    /// # use std::collections::HashMap;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let options = ClaudeAgentOptions::builder()
    ///     .enable_file_checkpointing(true)
    ///     .extra_args(HashMap::from([("replay-user-messages".to_string(), None)]))
    ///     .build();
    /// let mut client = ClaudeClient::new(options);
    /// client.connect().await?;
    ///
    /// client.query("Make some changes to my files").await?;
    /// let mut checkpoint_id = None;
    /// {
    ///     let mut stream = client.receive_response();
    ///     use futures::StreamExt;
    ///     while let Some(Ok(msg)) = stream.next().await {
    ///         if let Message::User(user_msg) = &msg {
    ///             if let Some(uuid) = &user_msg.uuid {
    ///                 checkpoint_id = Some(uuid.clone());
    ///             }
    ///         }
    ///     }
    /// }
    ///
    /// // Later, rewind to that point
    /// if let Some(id) = checkpoint_id {
    ///     client.rewind_files(&id).await?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn rewind_files(&self, user_message_id: &str) -> Result<()> {
        let query = self.query.as_ref().ok_or_else(|| {
            ClaudeError::InvalidConfig("Client not connected. Call connect() first.".to_string())
        })?;

        let query_guard = query.lock().await;
        query_guard.rewind_files(user_message_id).await
    }

    /// Get server initialization info including available commands and output styles
    ///
    /// Returns initialization information from the Claude Code server including:
    /// - Available commands (slash commands, system commands, etc.)
    /// - Current and available output styles
    /// - Server capabilities
    ///
    /// This is analogous to Python's `client.get_server_info()`.
    ///
    /// # Returns
    ///
    /// Dictionary with server info, or None if not connected
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use claude_agent_sdk_rs::{ClaudeClient, ClaudeAgentOptions};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = ClaudeClient::new(ClaudeAgentOptions::default());
    /// # client.connect().await?;
    /// if let Some(info) = client.get_server_info().await {
    ///     println!("Commands available: {}", info.get("commands").map(|c| c.as_array().map(|a| a.len()).unwrap_or(0)).unwrap_or(0));
    ///     println!("Output style: {:?}", info.get("output_style"));
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_server_info(&self) -> Option<serde_json::Value> {
        let query = self.query.as_ref()?;
        let query_guard = query.lock().await;
        query_guard.get_initialization_result().await
    }

    /// Start a new session by switching to a different session ID
    ///
    /// This is a convenience method that creates a new conversation context.
    /// It's equivalent to calling `query_with_session()` with a new session ID.
    ///
    /// To completely clear memory and start fresh, use `ClaudeAgentOptions::builder().fork_session(true).build()`
    /// when creating a new client.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The new session ID to use
    /// * `prompt` - Initial message for the new session
    ///
    /// # Errors
    ///
    /// Returns an error if the client is not connected or if sending fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use claude_agent_sdk_rs::{ClaudeClient, ClaudeAgentOptions};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = ClaudeClient::new(ClaudeAgentOptions::default());
    /// # client.connect().await?;
    /// // First conversation
    /// client.query("Hello").await?;
    ///
    /// // Start new conversation with different context
    /// client.new_session("session-2", "Tell me about Rust").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new_session(
        &mut self,
        session_id: impl Into<String>,
        prompt: impl Into<String>,
    ) -> Result<()> {
        self.query_with_session(prompt, session_id).await
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
            eprintln!(
                "Warning: ClaudeClient dropped without calling disconnect(). Resources may not be cleaned up properly."
            );
        }
    }
}
