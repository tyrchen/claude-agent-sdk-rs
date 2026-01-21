//! MCP (Model Context Protocol) types for Claude Agent SDK

use async_trait::async_trait;
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::errors::Result;

/// MCP servers configuration
#[derive(Clone, Default)]
pub enum McpServers {
    /// No MCP servers
    #[default]
    Empty,
    /// Dictionary of server configurations
    Dict(HashMap<String, McpServerConfig>),
    /// Path to MCP servers configuration file
    Path(PathBuf),
}

/// MCP server configuration
#[derive(Clone)]
pub enum McpServerConfig {
    /// Stdio-based MCP server
    Stdio(McpStdioServerConfig),
    /// SSE-based MCP server
    Sse(McpSseServerConfig),
    /// HTTP-based MCP server
    Http(McpHttpServerConfig),
    /// SDK (in-process) MCP server
    Sdk(McpSdkServerConfig),
}

/// Stdio MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpStdioServerConfig {
    /// Command to execute
    pub command: String,
    /// Command arguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    /// Environment variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
}

/// SSE MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpSseServerConfig {
    /// Server URL
    pub url: String,
    /// HTTP headers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
}

/// HTTP MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpHttpServerConfig {
    /// Server URL
    pub url: String,
    /// HTTP headers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
}

/// SDK (in-process) MCP server configuration
#[derive(Clone)]
pub struct McpSdkServerConfig {
    /// Server name
    pub name: String,
    /// Server instance
    pub instance: Arc<dyn SdkMcpServer>,
}

/// Trait for SDK MCP server implementations
#[async_trait]
pub trait SdkMcpServer: Send + Sync {
    /// Handle an MCP message
    async fn handle_message(&self, message: serde_json::Value) -> Result<serde_json::Value>;
}

/// Tool handler trait
pub trait ToolHandler: Send + Sync {
    /// Handle a tool invocation
    fn handle(&self, args: serde_json::Value) -> BoxFuture<'static, Result<ToolResult>>;
}

/// Tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Result content
    pub content: Vec<ToolResultContent>,
    /// Whether this is an error
    #[serde(default)]
    pub is_error: bool,
}

/// Tool result content types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ToolResultContent {
    /// Text content
    Text {
        /// Text content
        text: String,
    },
    /// Image content
    Image {
        /// Base64-encoded image data
        data: String,
        /// MIME type
        mime_type: String,
    },
}

/// SDK MCP tool definition
pub struct SdkMcpTool {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// JSON schema for tool input
    pub input_schema: serde_json::Value,
    /// Tool handler
    pub handler: Arc<dyn ToolHandler>,
}

/// Create an in-process MCP server
pub fn create_sdk_mcp_server(
    name: impl Into<String>,
    version: impl Into<String>,
    tools: Vec<SdkMcpTool>,
) -> McpSdkServerConfig {
    let server = DefaultSdkMcpServer {
        name: name.into(),
        version: version.into(),
        tools: tools.into_iter().map(|t| (t.name.clone(), t)).collect(),
    };

    McpSdkServerConfig {
        name: server.name.clone(),
        instance: Arc::new(server),
    }
}

/// Default implementation of SDK MCP server
struct DefaultSdkMcpServer {
    name: String,
    version: String,
    tools: HashMap<String, SdkMcpTool>,
}

#[async_trait]
impl SdkMcpServer for DefaultSdkMcpServer {
    async fn handle_message(&self, message: serde_json::Value) -> Result<serde_json::Value> {
        // Parse the MCP message
        let method = message["method"]
            .as_str()
            .ok_or_else(|| crate::errors::ClaudeError::Transport("Missing method".to_string()))?;

        let message_id = message.get("id").cloned();

        match method {
            "initialize" => {
                // Return JSONRPC response for initialize
                Ok(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": message_id,
                    "result": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {
                            "tools": {}
                        },
                        "serverInfo": {
                            "name": self.name,
                            "version": self.version
                        }
                    }
                }))
            }
            "tools/list" => {
                // Return list of tools in JSONRPC format
                let tools: Vec<_> = self
                    .tools
                    .values()
                    .map(|t| {
                        serde_json::json!({
                            "name": t.name,
                            "description": t.description,
                            "inputSchema": t.input_schema
                        })
                    })
                    .collect();

                Ok(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": message_id,
                    "result": {
                        "tools": tools
                    }
                }))
            }
            "tools/call" => {
                // Execute a tool
                let params = &message["params"];
                let tool_name = params["name"].as_str().ok_or_else(|| {
                    crate::errors::ClaudeError::Transport("Missing tool name".to_string())
                })?;
                let arguments = params["arguments"].clone();

                let tool = self.tools.get(tool_name).ok_or_else(|| {
                    crate::errors::ClaudeError::Transport(format!("Tool not found: {}", tool_name))
                })?;

                let result = tool.handler.handle(arguments).await?;

                Ok(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": message_id,
                    "result": {
                        "content": result.content,
                        "isError": result.is_error
                    }
                }))
            }
            "notifications/initialized" | "notifications/cancelled" => {
                // Notifications don't require a response
                // Return null for notifications
                Ok(serde_json::json!(null))
            }
            _ => Err(crate::errors::ClaudeError::Transport(format!(
                "Unknown method: {}",
                method
            ))),
        }
    }
}

/// Macro to create a tool
#[macro_export]
macro_rules! tool {
    ($name:expr, $desc:expr, $schema:expr, $handler:expr) => {{
        struct Handler<F>(F);

        impl<F, Fut> $crate::types::mcp::ToolHandler for Handler<F>
        where
            F: Fn(serde_json::Value) -> Fut + Send + Sync,
            Fut: std::future::Future<Output = anyhow::Result<$crate::types::mcp::ToolResult>>
                + Send
                + 'static,
        {
            fn handle(
                &self,
                args: serde_json::Value,
            ) -> futures::future::BoxFuture<
                'static,
                $crate::errors::Result<$crate::types::mcp::ToolResult>,
            > {
                use futures::FutureExt;
                let f = &self.0;
                let fut = f(args);
                async move { fut.await.map_err(|e| e.into()) }.boxed()
            }
        }

        $crate::types::mcp::SdkMcpTool {
            name: $name.to_string(),
            description: $desc.to_string(),
            input_schema: $schema,
            handler: std::sync::Arc::new(Handler($handler)),
        }
    }};
}
