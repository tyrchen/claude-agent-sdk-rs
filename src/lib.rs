//! # Claude Agent SDK for Rust
//!
//! Rust SDK for interacting with Claude Code CLI, enabling programmatic access to Claude's capabilities.
//!
//! ## Features
//!
//! - **Simple Query Interface**: One-shot queries for stateless interactions
//! - **Streaming Client**: Bidirectional, stateful conversations with full control
//! - **Custom Tools (SDK MCP Servers)**: In-process tool definitions callable by Claude
//! - **Hooks System**: Intercept and control Claude's behavior at runtime
//! - **Permission Management**: Fine-grained control over tool execution
//! - **Session Management**: Resume, fork, and manage conversation sessions
//!
//! ## Quick Start (Coming Soon)
//!
//! The `query()` function will enable simple one-shot queries:
//!
//! ```rust,ignore
//! use claude_agent_sdk::{query, Message, ContentBlock};
//! use futures::StreamExt;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let mut messages = query("What is 2 + 2?", None).await?;
//!
//!     while let Some(message) = messages.next().await {
//!         match message? {
//!             Message::Assistant(msg) => {
//!                 for block in msg.content {
//!                     if let ContentBlock::Text(text) = block {
//!                         println!("Claude: {}", text.text);
//!                     }
//!                 }
//!             }
//!             _ => {}
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod errors;
mod internal;
pub mod query;
pub mod types;
pub mod version;

// Re-export commonly used types
pub use errors::{ClaudeError, Result};
pub use types::{
    config::*,
    hooks::*,
    mcp::{
        create_sdk_mcp_server, McpServerConfig, McpServers, SdkMcpServer, SdkMcpTool, ToolHandler,
        ToolResult, ToolResultContent as McpToolResultContent,
    },
    messages::*,
    permissions::*,
};

// Re-export public API
pub use client::ClaudeClient;
pub use query::query;
