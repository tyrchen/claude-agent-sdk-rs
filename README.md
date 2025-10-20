# Claude Agent SDK for Rust

Rust SDK for interacting with Claude Code CLI, enabling programmatic access to Claude's capabilities with **full bidirectional streaming support**.

**Status**: ✅ **Production Ready** - 100% feature parity with Python SDK

## Features

- ✅ **Simple Query API**: One-shot queries for stateless interactions
- ✅ **Bidirectional Streaming**: Real-time streaming communication with `ClaudeClient`
- ✅ **Dynamic Control**: Interrupt, change permissions, switch models mid-execution
- ✅ **Hooks System**: Intercept and control Claude's behavior at runtime
- ✅ **Custom Tools**: In-process MCP servers with ergonomic tool macro
- ✅ **Permission Management**: Fine-grained control over tool execution
- ✅ **Type Safety**: Strongly-typed messages, configs, hooks, and permissions
- ✅ **Zero Deadlock**: Lock-free architecture for concurrent read/write

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
claude-agent-sdk = "0.1.0"
```

## Quick Start

### Simple Query

```rust
use claude_agent_sdk::query;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = query("What is 2 + 2?", None).await?;

    while let Some(message) = stream.next().await {
        println!("{:?}", message?);
    }

    Ok(())
}
```

### Bidirectional Conversation

```rust
use claude_agent_sdk::{ClaudeClient, ClaudeAgentOptions, Message, ContentBlock};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ClaudeClient::new(ClaudeAgentOptions::default());

    // Connect to Claude
    client.connect().await?;

    // First question
    client.query("What is your name?").await?;
    let mut stream = client.receive_response();
    while let Some(msg) = stream.next().await {
        match msg? {
            Message::Assistant(msg) => {
                for block in msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        println!("Claude: {}", text.text);
                    }
                }
            }
            Message::Result(_) => break,
            _ => {}
        }
    }
    drop(stream);

    // Follow-up question - Claude remembers context!
    client.query("What did I just ask?").await?;
    let mut stream = client.receive_response();
    while let Some(msg) = stream.next().await {
        // Process response...
    }
    drop(stream);

    client.disconnect().await?;
    Ok(())
}
```

### Custom Tools (SDK MCP Servers)

Create custom in-process tools that Claude can use:

```rust
use claude_agent_sdk::{tool, create_sdk_mcp_server, ToolResult, McpToolResultContent};
use serde_json::json;

async fn greet_handler(args: serde_json::Value) -> anyhow::Result<ToolResult> {
    let name = args["name"].as_str().unwrap_or("World");
    Ok(ToolResult {
        content: vec![McpToolResultContent::Text {
            text: format!("Hello, {}!", name),
        }],
        is_error: false,
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let greet_tool = tool!(
        "greet",
        "Greet a user",
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            },
            "required": ["name"]
        }),
        greet_handler
    );

    let server = create_sdk_mcp_server("my-tools", "1.0.0", vec![greet_tool]);

    // Configure ClaudeClient with the MCP server and allowed tools
    let mut mcp_servers = HashMap::new();
    mcp_servers.insert("my-tools".to_string(), McpServerConfig::Sdk(server));

    let options = ClaudeAgentOptions {
        mcp_servers: McpServers::Dict(mcp_servers),
        allowed_tools: vec!["mcp__my-tools__greet".to_string()],
        permission_mode: Some(PermissionMode::AcceptEdits),
        ..Default::default()
    };

    let mut client = ClaudeClient::new(options);
    client.connect().await?;

    // Claude can now use your custom tools!
    client.query("Greet Alice").await?;
    // ... handle responses

    client.disconnect().await?;
    Ok(())
}
```

**Note**: Tools must be explicitly allowed using the format `mcp__{server_name}__{tool_name}`.

For a comprehensive guide, see [examples/MCP_INTEGRATION.md](examples/MCP_INTEGRATION.md).

## Architecture

The SDK is structured in layers:

```
┌─────────────────────────────────────────────────────────┐
│                    Public API Layer                     │
│  (query(), ClaudeClient, tool!(), create_sdk_server())  │
└─────────────────────────────────────────────────────────┘
                           │
┌─────────────────────────────────────────────────────────┐
│                  Control Protocol Layer                 │
│        (Query: handles bidirectional control)           │
└─────────────────────────────────────────────────────────┘
                           │
┌─────────────────────────────────────────────────────────┐
│                   Transport Layer                       │
│     (SubprocessTransport, custom implementations)       │
└─────────────────────────────────────────────────────────┘
                           │
┌─────────────────────────────────────────────────────────┐
│                  Claude Code CLI                        │
│         (external process via stdio/subprocess)         │
└─────────────────────────────────────────────────────────┘
```

## Type System

The SDK provides strongly-typed Rust interfaces for all Claude interactions:

- **Messages**: `Message`, `ContentBlock`, `TextBlock`, `ToolUseBlock`, etc.
- **Configuration**: `ClaudeAgentOptions`, `SystemPrompt`, `PermissionMode`
- **Hooks**: `HookEvent`, `HookCallback`, `HookInput`, `HookJsonOutput`
- **Permissions**: `PermissionResult`, `PermissionUpdate`, `CanUseToolCallback`
- **MCP**: `McpServers`, `SdkMcpServer`, `ToolHandler`, `ToolResult`

## Examples

The SDK includes comprehensive examples demonstrating all features:

```bash
# Simple query
cargo run --example 01_hello_world

# Bidirectional conversation with context retention
cargo run --example 06_bidirectional_client

# PreToolUse hooks with callbacks
cargo run --example 05_hooks_pretooluse

# Dynamic control (interrupt, permission mode, model switching)
cargo run --example 07_dynamic_control

# MCP server integration with custom tools
cargo run --example 08_mcp_server_integration

# And more...
cargo run --example 02_limit_tool_use
cargo run --example 03_monitor_tools
cargo run --example 04_permission_callbacks
```

## API Overview

### ClaudeClient (Bidirectional Streaming)

```rust
// Create and connect
let mut client = ClaudeClient::new(options);
client.connect().await?;

// Send queries
client.query("Hello").await?;

// Receive as stream
let mut stream = client.receive_response();
while let Some(msg) = stream.next().await {
    // Process messages
}
drop(stream);

// Dynamic control
client.set_permission_mode(PermissionMode::AcceptEdits).await?;
client.interrupt().await?;
client.set_model(Some("claude-sonnet-4")).await?;

// Disconnect
client.disconnect().await?;
```

## Requirements

- Rust 1.70+
- Claude Code CLI 2.0.0+
- tokio async runtime

## Testing

```bash
# Run all tests
cargo test

# Run clippy
cargo clippy --all-targets

# Build all examples
cargo build --examples

# Run specific example
cargo run --example 06_bidirectional_client
```

## Python SDK Comparison

The Rust SDK closely mirrors the Python SDK API:

| Python | Rust |
|--------|------|
| `async with ClaudeSDKClient() as client:` | `client.connect().await?` |
| `await client.query("...")` | `client.query("...").await?` |
| `async for msg in client.receive_response():` | `while let Some(msg) = stream.next().await` |
| `await client.interrupt()` | `client.interrupt().await?` |
| `await client.disconnect()` | `client.disconnect().await?` |

## Contributing

This SDK is based on the [claude-agent-sdk-python](https://github.com/anthropics/claude-agent-sdk-python) specification.

## License

This project is distributed under the terms of MIT.

See [LICENSE.md](LICENSE.md) for details.

## Related Projects

- [Claude Code CLI](https://docs.claude.com/claude-code)
- [Claude Agent SDK for Python](https://github.com/anthropics/claude-agent-sdk-python)
