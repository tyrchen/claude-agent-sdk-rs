# Claude Agent SDK for Rust

[![Crates.io](https://img.shields.io/crates/v/claude-agent-sdk-rs.svg)](https://crates.io/crates/claude-agent-sdk-rs)
[![Documentation](https://docs.rs/claude-agent-sdk-rs/badge.svg)](https://docs.rs/claude-agent-sdk-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE.md)

[English](README.md) | [中文](README_zh-CN.md)

Rust SDK for interacting with Claude Code CLI, enabling programmatic access to Claude's capabilities with **full bidirectional streaming support**.

**Status**: ✅ 100% feature parity with Python SDK - Production Ready

## ✨ Features

- 🚀 **Simple Query API**: One-shot queries for stateless interactions with both collecting and streaming modes
- 🔄 **Bidirectional Streaming**: Real-time streaming communication with `ClaudeClient`
- 🎛️ **Dynamic Control**: Interrupt, change permissions, switch models mid-execution
- 🪝 **Hooks System**: Intercept and control Claude's behavior with ergonomic builder API
- 🛠️ **Custom Tools**: In-process MCP servers with ergonomic `tool!` macro
- 🔌 **Plugin System**: Load custom plugins to extend Claude's capabilities
- 🔐 **Permission Management**: Fine-grained control over tool execution
- 💰 **Cost Control**: Budget limits and fallback models for production reliability
- 🧠 **Extended Thinking**: Configure maximum thinking tokens for complex reasoning
- 📊 **Session Management**: Separate contexts and memory clearing with fork_session
- 🦀 **Type Safety**: Strongly-typed messages, configs, hooks, and permissions
- ⚡ **Zero Deadlock**: Lock-free architecture for concurrent read/write
- 📚 **Comprehensive Examples**: 23 complete examples covering all features
- 🖼️ **Multimodal Input**: Send images alongside text with base64 or URL sources
- 🧪 **Well Tested**: Extensive test coverage with unit and integration tests

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
claude-agent-sdk-rs = "0.5"
tokio = { version = "1", features = ["full"] }
```

Or use cargo-add:

```bash
cargo add claude-agent-sdk-rs
cargo add tokio --features full
```

## 🎯 Prerequisites

- **Rust**: 1.90 or higher
- **Claude Code CLI**: Version 2.0.0 or higher ([Installation Guide](https://docs.claude.com/claude-code))
- **API Key**: Anthropic API key set in environment or Claude Code config

## 🚀 Quick Start

### Simple Query (One-shot)

```rust
use claude_agent_sdk_rs::{query, Message, ContentBlock};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Simple query with default options
    let messages = query("What is 2 + 2?", None).await?;

    for message in messages {
        if let Message::Assistant(msg) = message {
            for block in msg.message.content {
                if let ContentBlock::Text(text) = block {
                    println!("Claude: {}", text.text);
                }
            }
        }
    }

    Ok(())
}
```

With custom options:

```rust
use claude_agent_sdk_rs::{ClaudeAgentOptions, query};

let options = ClaudeAgentOptions {
    model: Some("sonnet".to_string()),  // Use Sonnet for lower cost
    max_turns: Some(5),
    tools: Some(["Read", "Write"].into()),
    ..Default::default()
};

let messages = query("Create a hello.txt file", Some(options)).await?;
```

### Tool Configuration: `tools` vs `allowed_tools`

The SDK provides two different parameters for tool configuration:

| Parameter | CLI Flag | Purpose |
|-----------|----------|---------|
| `tools` | `--tools` | **Restricts** which tools Claude can use |
| `allowed_tools` | `--allowedTools` | **Adds permissions** for specific tools (mainly for MCP tools) |

**Use `tools`** when you want to limit Claude to specific built-in tools:

```rust
// Claude can ONLY use Read, Write, and Bash
let options = ClaudeAgentOptions {
    tools: Some(["Read", "Write", "Bash"].into()),
    ..Default::default()
};
```

**Use `allowed_tools`** when you need to grant permission for custom MCP tools:

```rust
// Allow custom MCP tools (format: mcp__{server}__{tool})
let options = ClaudeAgentOptions {
    allowed_tools: vec!["mcp__my-tools__greet".to_string()],
    ..Default::default()
};
```

### Streaming Query (Memory-Efficient)

For large conversations or real-time processing, use `query_stream()`:

```rust
use claude_agent_sdk_rs::{query_stream, Message, ContentBlock};
use futures::stream::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Get a stream of messages instead of collecting them all
    let mut stream = query_stream("What is 2 + 2?", None).await?;

    // Process messages as they arrive (O(1) memory)
    while let Some(result) = stream.next().await {
        let message = result?;
        if let Message::Assistant(msg) = message {
            for block in msg.message.content {
                if let ContentBlock::Text(text) = block {
                    println!("Claude: {}", text.text);
                }
            }
        }
    }

    Ok(())
}
```

**When to use:**
- `query()`: Small to medium conversations, need all messages for post-processing
- `query_stream()`: Large conversations, real-time processing, memory constraints

### Bidirectional Conversation (Multi-turn)

```rust
use claude_agent_sdk_rs::{ClaudeClient, ClaudeAgentOptions, Message, ContentBlock};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = ClaudeClient::new(ClaudeAgentOptions::default());

    // Connect to Claude
    client.connect().await?;

    // First question
    client.query("What is the capital of France?").await?;

    // Receive response
    loop {
        match client.receive_message().await? {
            Some(Message::Assistant(msg)) => {
                for block in msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        println!("Claude: {}", text.text);
                    }
                }
            }
            Some(Message::Result(_)) => break,
            Some(_) => continue,
            None => break,
        }
    }

    // Follow-up question - Claude remembers context!
    client.query("What's the population of that city?").await?;

    loop {
        match client.receive_message().await? {
            Some(Message::Assistant(msg)) => {
                for block in msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        println!("Claude: {}", text.text);
                    }
                }
            }
            Some(Message::Result(_)) => break,
            Some(_) => continue,
            None => break,
        }
    }

    client.disconnect().await?;
    Ok(())
}
```

### Custom Tools (SDK MCP Servers)

Create custom in-process tools that Claude can use:

```rust
use claude_agent_sdk_rs::{tool, create_sdk_mcp_server, ToolResult, McpToolResultContent};
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

    // Note: Use `allowed_tools` for MCP tools (not `tools`)
    // - allowed_tools: Grants permission for custom MCP tools
    // - tools: Restricts built-in tools (Read, Write, Bash, etc.)
    let options = ClaudeAgentOptions {
        mcp_servers: McpServers::Dict(mcp_servers),
        allowed_tools: vec!["mcp__my-tools__greet".to_string()],
        model: Some("sonnet".to_string()),  // Use Sonnet for lower cost
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

**Note**: MCP tools must be explicitly allowed using `allowed_tools` with format `mcp__{server_name}__{tool_name}`.

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

## Session Management & Memory Clearing

The SDK provides multiple ways to manage conversation context and clear memory:

### Using Session IDs (Separate Contexts)

Different session IDs maintain completely separate conversation contexts:

```rust
let mut client = ClaudeClient::new(ClaudeAgentOptions::default());
client.connect().await?;

// Session 1: Math conversation
client.query_with_session("What is 2 + 2?", "math-session").await?;

// Session 2: Programming conversation (different context)
client.query_with_session("What is Rust?", "programming-session").await?;

// Back to Session 1 - Claude remembers math context
client.query_with_session("What about 3 + 3?", "math-session").await?;
```

### Fork Session (Fresh Start)

Use `fork_session` to start completely fresh without any history:

```rust
let options = ClaudeAgentOptions::builder()
    .fork_session(true)  // Each resumed session starts fresh
    .build();

let mut client = ClaudeClient::new(options);
client.connect().await?;
```

### Convenience Method

Use `new_session()` for quick session switching:

```rust
client.new_session("session-2", "Tell me about Rust").await?;
```

See [examples/16_session_management.rs](examples/16_session_management.rs) for complete examples.

## Type System

The SDK provides strongly-typed Rust interfaces for all Claude interactions:

- **Messages**: `Message`, `ContentBlock`, `TextBlock`, `ToolUseBlock`, etc.
- **Configuration**: `ClaudeAgentOptions`, `SystemPrompt`, `PermissionMode`
- **Hooks**: `HookEvent`, `HookCallback`, `HookInput`, `HookJsonOutput`
- **Permissions**: `PermissionResult`, `PermissionUpdate`, `CanUseToolCallback`
- **MCP**: `McpServers`, `SdkMcpServer`, `ToolHandler`, `ToolResult`

## 📚 Examples

The SDK includes **23 comprehensive examples** demonstrating all features with 100% parity to Python SDK. See [examples/README.md](examples/README.md) for details.

### Quick Examples

```bash
# Basic usage
cargo run --example 01_hello_world        # Simple query with tool usage
cargo run --example 02_limit_tool_use     # Restrict allowed tools
cargo run --example 03_monitor_tools      # Monitor tool execution

# Streaming & Conversations
cargo run --example 06_bidirectional_client  # Multi-turn conversations
cargo run --example 14_streaming_mode -- all # Comprehensive streaming patterns

# Hooks & Control
cargo run --example 05_hooks_pretooluse      # PreToolUse hooks
cargo run --example 15_hooks_comprehensive -- all  # All hook types
cargo run --example 07_dynamic_control       # Runtime control

# Custom Tools & MCP
cargo run --example 08_mcp_server_integration  # In-process MCP servers

# Configuration
cargo run --example 09_agents               # Custom agents
cargo run --example 11_setting_sources -- all  # Settings control
cargo run --example 13_system_prompt        # System prompt configs

# Production Features
cargo run --example 17_fallback_model       # Fallback model for reliability
cargo run --example 18_max_budget_usd       # Budget control
cargo run --example 19_max_thinking_tokens  # Extended thinking limits
cargo run --example 20_query_stream         # Streaming query API

# Plugin System
cargo run --example 21_custom_plugins       # Load custom plugins
cargo run --example 22_plugin_integration   # Real-world plugin usage

# Multimodal
cargo run --example 23_image_input          # Image input with text queries

# Session Management
cargo run --example 16_session_management   # Session clearing and management
```

### Example Categories

| Category        | Examples | Description                                         |
| --------------- | -------- | --------------------------------------------------- |
| **Basics**      | 01-03    | Simple queries, tool control, monitoring            |
| **Advanced**    | 04-07    | Permissions, hooks, streaming, dynamic control      |
| **MCP**         | 08       | Custom tools and MCP server integration             |
| **Config**      | 09-13    | Agents, settings, prompts, debugging                |
| **Patterns**    | 14-16    | Comprehensive streaming, hooks, and sessions        |
| **Production**  | 17-20    | Fallback models, budgets, thinking limits, streaming|
| **Plugins**     | 21-22    | Custom plugin loading and integration               |
| **Multimodal**  | 23       | Image input alongside text                          |

## 📖 API Overview

### Core Types

```rust
// Main client for bidirectional streaming
ClaudeClient

// Simple query functions for one-shot interactions
query(prompt: &str, options: Option<ClaudeAgentOptions>) -> Vec<Message>
query_stream(prompt: &str, options: Option<ClaudeAgentOptions>) -> Stream<Item = Result<Message>>
query_with_content(content: Vec<UserContentBlock>, options: Option<ClaudeAgentOptions>) -> Vec<Message>
query_stream_with_content(content: Vec<UserContentBlock>, options: Option<ClaudeAgentOptions>) -> Stream<Item = Result<Message>>

// Configuration
ClaudeAgentOptions {
    model: Option<String>,
    fallback_model: Option<String>,      // Backup model for reliability
    max_budget_usd: Option<f64>,         // Cost control
    max_thinking_tokens: Option<u32>,    // Extended thinking limit
    plugins: Vec<SdkPluginConfig>,       // Custom plugin loading
    max_turns: Option<u32>,
    tools: Option<Tools>,                // Restrict available tools (--tools)
    allowed_tools: Vec<String>,          // Grant MCP tool permissions (--allowedTools)
    system_prompt: Option<SystemPromptConfig>,
    hooks: Option<HashMap<String, Vec<HookMatcher>>>,
    mcp_servers: Option<HashMap<String, McpServer>>,
    // ... and more
}

// Messages
Message::Assistant(AssistantMessage)
Message::User(UserMessage)
Message::System(SystemMessage)
Message::Result(ResultMessage)
```

### ClaudeClient (Bidirectional Streaming)

```rust
// Create and connect
let mut client = ClaudeClient::new(options);
client.connect().await?;

// Send queries
client.query("Hello").await?;

// Receive messages
loop {
    match client.receive_message().await? {
        Some(Message::Assistant(msg)) => { /* Handle */ }
        Some(Message::Result(_)) => break,
        None => break,
        _ => continue,
    }
}

// Session management - separate conversation contexts
client.query_with_session("First question", "session-1").await?;
client.query_with_session("Different context", "session-2").await?;
client.new_session("session-3", "Fresh start").await?;

// Dynamic control (mid-execution)
client.interrupt().await?;  // Stop current operation
// Client will handle the interrupt automatically

// Disconnect
client.disconnect().await?;
```

### Hooks System

```rust
use claude_agent_sdk_rs::{Hook, HookMatcher, HookInput, HookContext, HookJSONOutput};

async fn my_hook(
    input: HookInput,
    tool_use_id: Option<String>,
    context: HookContext,
) -> anyhow::Result<HookJSONOutput> {
    // Block dangerous commands
    if let Some(command) = input.get("tool_input")
        .and_then(|v| v.get("command"))
        .and_then(|v| v.as_str())
    {
        if command.contains("rm -rf") {
            return Ok(serde_json::json!({
                "hookSpecificOutput": {
                    "permissionDecision": "deny",
                    "permissionDecisionReason": "Dangerous command blocked"
                }
            }));
        }
    }
    Ok(serde_json::json!({}))
}

let mut hooks = HashMap::new();
hooks.insert("PreToolUse".to_string(), vec![
    HookMatcher {
        matcher: Some("Bash".to_string()),
        hooks: vec![Hook::new(my_hook)],
    }
]);

let options = ClaudeAgentOptions {
    hooks: Some(hooks),
    ..Default::default()
};
```

## 🧪 Development

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Code Quality

```bash
# Check code with clippy
cargo clippy --all-targets --all-features

# Format code
cargo fmt

# Check formatting
cargo fmt -- --check
```

### Building

```bash
# Build library
cargo build

# Build with release optimizations
cargo build --release

# Build all examples
cargo build --examples

# Build documentation
cargo doc --open
```

## 🔧 Troubleshooting

### Common Issues

**"Claude Code CLI not found"**

- Install Claude Code CLI: <https://docs.claude.com/claude-code>
- Ensure `claude` is in your PATH

**"API key not configured"**

- Set `ANTHROPIC_API_KEY` environment variable
- Or configure via Claude Code CLI settings

**"Permission denied" errors**

- Use `permission_mode: PermissionMode::AcceptEdits` for automated workflows
- Or implement custom permission callbacks

### Debug Mode

Enable debug output to see what's happening:

```rust
let options = ClaudeAgentOptions {
    stderr_callback: Some(Arc::new(|msg| eprintln!("DEBUG: {}", msg))),
    extra_args: Some({
        let mut args = HashMap::new();
        args.insert("debug-to-stderr".to_string(), None);
        args
    }),
    ..Default::default()
};
```

## Python SDK Comparison

The Rust SDK closely mirrors the Python SDK API:

| Python                                        | Rust                                        |
| --------------------------------------------- | ------------------------------------------- |
| `async with ClaudeClient() as client:`     | `client.connect().await?`                   |
| `await client.query("...")`                   | `client.query("...").await?`                |
| `async for msg in client.receive_response():` | `while let Some(msg) = stream.next().await` |
| `await client.interrupt()`                    | `client.interrupt().await?`                 |
| `await client.disconnect()`                   | `client.disconnect().await?`                |

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/tyrchen/claude-agent-sdk-rs
cd claude-agent-sdk-rs

# Install dependencies
cargo build

# Run tests
cargo test

# Run examples
cargo run --example 01_hello_world
```

### Guidelines

- Follow Rust conventions and idioms
- Add tests for new features
- Update documentation and examples
- Run `cargo fmt` and `cargo clippy` before submitting

This SDK is based on the [claude-agent-sdk-python](https://github.com/anthropics/claude-agent-sdk-python) specification.

## License

This project is distributed under the terms of MIT.

See [LICENSE.md](LICENSE.md) for details.

## 🔗 Related Projects

- [Claude Code CLI](https://docs.claude.com/claude-code) - Official Claude Code command-line interface
- [Claude Agent SDK for Python](https://github.com/anthropics/claude-agent-sdk-python) - Official Python SDK
- [Anthropic API](https://www.anthropic.com/api) - Claude API documentation

## ⭐ Show Your Support

If you find this project useful, please consider giving it a star on GitHub!

## 📝 Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history and changes.
