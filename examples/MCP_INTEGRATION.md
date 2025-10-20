# MCP Server Integration Example

This document explains how to integrate custom MCP (Model Context Protocol) servers with the Claude Agent SDK for Rust.

## Overview

The Claude Agent SDK supports extending Claude's capabilities through custom tools via MCP servers. MCP servers can run:

1. **In-process (SDK MCP servers)** - Tools that run directly in your application
2. **External process (Stdio MCP servers)** - Separate processes communicating via stdio
3. **Remote (SSE/HTTP MCP servers)** - Network-based MCP servers

This example focuses on **SDK MCP servers** (in-process tools).

## Quick Start

### 1. Create Custom Tools

Use the `tool!` macro to define custom tools:

```rust
use claude_agent_sdk::{tool, ToolResult, McpToolResultContent};
use serde_json::json;

async fn calculator_handler(args: serde_json::Value) -> anyhow::Result<ToolResult> {
    let a = args["a"].as_f64().unwrap();
    let b = args["b"].as_f64().unwrap();
    let operation = args["operation"].as_str().unwrap();

    let result = match operation {
        "add" => a + b,
        "subtract" => a - b,
        "multiply" => a * b,
        "divide" => a / b,
        _ => return Err(anyhow::anyhow!("Unknown operation"))
    };

    Ok(ToolResult {
        content: vec![McpToolResultContent::Text {
            text: format!("Result: {}", result),
        }],
        is_error: false,
    })
}

let calculator_tool = tool!(
    "calculator",
    "Perform arithmetic operations",
    json!({
        "type": "object",
        "properties": {
            "operation": { "type": "string", "enum": ["add", "subtract", "multiply", "divide"] },
            "a": { "type": "number" },
            "b": { "type": "number" }
        },
        "required": ["operation", "a", "b"]
    }),
    calculator_handler
);
```

### 2. Create an MCP Server

Combine tools into an MCP server:

```rust
use claude_agent_sdk::create_sdk_mcp_server;

let server = create_sdk_mcp_server(
    "my-tools",           // Server name
    "1.0.0",              // Server version
    vec![calculator_tool] // List of tools
);
```

### 3. Configure ClaudeClient

Add the MCP server to your client configuration:

```rust
use claude_agent_sdk::{ClaudeAgentOptions, ClaudeClient, McpServers, McpServerConfig};
use std::collections::HashMap;

let mut mcp_servers = HashMap::new();
mcp_servers.insert("my-tools".to_string(), McpServerConfig::Sdk(server));

let options = ClaudeAgentOptions {
    mcp_servers: McpServers::Dict(mcp_servers),
    permission_mode: Some(claude_agent_sdk::PermissionMode::AcceptEdits),
    ..Default::default()
};

let mut client = ClaudeClient::new(options);
```

### 4. Use the Tools

Connect and query Claude - it will automatically use your custom tools:

```rust
client.connect().await?;
client.query("Calculate 42 multiplied by 7").await?;

let mut stream = client.receive_response();
while let Some(msg) = stream.next().await {
    // Handle messages
}
```

## Tool Handler Signature

Tool handlers must have this signature:

```rust
async fn handler_name(args: serde_json::Value) -> anyhow::Result<ToolResult>
```

- **Input**: JSON object containing tool parameters
- **Output**: `ToolResult` with content and error flag

## ToolResult Structure

```rust
pub struct ToolResult {
    pub content: Vec<McpToolResultContent>,
    pub is_error: bool,
}

pub enum McpToolResultContent {
    Text { text: String },
    Image { data: String, mime_type: String },
}
```

## Best Practices

### Error Handling

Return errors as tool results rather than panicking:

```rust
async fn safe_handler(args: serde_json::Value) -> anyhow::Result<ToolResult> {
    let value = match args["number"].as_f64() {
        Some(v) => v,
        None => return Ok(ToolResult {
            content: vec![McpToolResultContent::Text {
                text: "Error: Invalid number".to_string(),
            }],
            is_error: true,
        }),
    };

    // Process value...
}
```

### Input Validation

Validate inputs in your handler:

```rust
async fn validated_handler(args: serde_json::Value) -> anyhow::Result<ToolResult> {
    let min = args["min"].as_i64().unwrap_or(0);
    let max = args["max"].as_i64().unwrap_or(100);

    if min >= max {
        return Ok(ToolResult {
            content: vec![McpToolResultContent::Text {
                text: "Error: min must be less than max".to_string(),
            }],
            is_error: true,
        });
    }

    // Continue processing...
}
```

### Schema Definitions

Use detailed JSON schemas to help Claude understand your tools:

```rust
json!({
    "type": "object",
    "properties": {
        "numbers": {
            "type": "array",
            "items": { "type": "number" },
            "description": "Array of numbers to analyze",
            "minItems": 1
        }
    },
    "required": ["numbers"]
})
```

## Example: Math Tools Server

See `examples/08_mcp_server_integration.rs` for a complete example that demonstrates:

- Creating multiple tools (calculator, statistics, random number generator)
- Building an MCP server with all tools
- Integrating with ClaudeClient
- Handling tool results in conversation
- Composing multiple tools to solve complex tasks

Run the example:

```bash
cargo run --example 08_mcp_server_integration
```

## Advanced: External MCP Servers

For external MCP servers (stdio, SSE, HTTP), use different configuration types:

### Stdio MCP Server

```rust
use claude_agent_sdk::{McpServerConfig, McpStdioServerConfig};
use std::collections::HashMap;

let server_config = McpServerConfig::Stdio(McpStdioServerConfig {
    command: "node".to_string(),
    args: Some(vec!["path/to/server.js".to_string()]),
    env: Some(HashMap::new()),
});
```

### SSE MCP Server

```rust
use claude_agent_sdk::{McpServerConfig, McpSseServerConfig};

let server_config = McpServerConfig::Sse(McpSseServerConfig {
    url: "http://localhost:3000/sse".to_string(),
    headers: None,
});
```

## Message Flow

When Claude uses your tools:

1. **Assistant Message** - Claude decides to use a tool (contains `ToolUse` block)
2. **User Message** - Tool result returned (contains `ToolResult` block)
3. **Assistant Message** - Claude processes the result and responds

Example message handling:

```rust
match message? {
    Message::Assistant(msg) => {
        for block in msg.message.content {
            match block {
                ContentBlock::ToolUse(tool) => {
                    println!("Using tool: {}", tool.name);
                }
                ContentBlock::Text(text) => {
                    println!("Claude: {}", text.text);
                }
                _ => {}
            }
        }
    }
    Message::User(user_msg) => {
        if let Some(blocks) = &user_msg.content {
            for block in blocks {
                if let ContentBlock::ToolResult(result) = block {
                    println!("Tool result received");
                }
            }
        }
    }
    _ => {}
}
```

## Troubleshooting

### Tool Not Being Called

- Ensure the tool name is descriptive and clear
- Provide a detailed description explaining when to use the tool
- Use comprehensive JSON schema with descriptions
- Check that the MCP server is properly registered in `ClaudeAgentOptions`

### Permission Denied

Set appropriate permission mode:

```rust
permission_mode: Some(PermissionMode::AcceptEdits)
```

### Tool Execution Errors

- Check your handler's error handling
- Return `is_error: true` in `ToolResult` for graceful errors
- Log errors to stderr for debugging

## Resources

- [MCP Specification](https://modelcontextprotocol.io/)
- [Claude Agent SDK Documentation](../README.md)
- [Example Code](./08_mcp_server_integration.rs)
