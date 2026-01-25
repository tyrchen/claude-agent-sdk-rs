# Claude Agent SDK API Reference

This document provides comprehensive documentation for the Claude Agent SDK for Rust.

## Table of Contents

- [Getting Started](#getting-started)
- [ClaudeAgentOptions](#claudeagentoptions)
- [ClaudeClient](#claudeclient)
- [Query Functions](#query-functions)
- [Messages and Content](#messages-and-content)
- [Hooks System](#hooks-system)
- [MCP Server Integration](#mcp-server-integration)
- [Permission Management](#permission-management)
- [Session Management](#session-management)
- [Efficiency Hooks](#efficiency-hooks)
- [Multimodal Input](#multimodal-input)
- [Error Handling](#error-handling)

---

## Getting Started

### Installation

```toml
[dependencies]
claude-agent-sdk-rs = "0.6"
tokio = { version = "1", features = ["full"] }
```

### Basic Usage

```rust
use claude_agent_sdk_rs::{query, ClaudeClient, ClaudeAgentOptions, Message, ContentBlock};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // One-shot query
    let messages = query("What is 2 + 2?", None).await?;

    // Or use ClaudeClient for multi-turn conversations
    let mut client = ClaudeClient::new(ClaudeAgentOptions::default());
    client.connect().await?;
    client.query("Hello!").await?;
    // ... process responses
    client.disconnect().await?;

    Ok(())
}
```

---

## ClaudeAgentOptions

`ClaudeAgentOptions` configures the behavior of Claude. Use the builder pattern for ergonomic configuration.

### Builder Pattern

```rust
use claude_agent_sdk_rs::{ClaudeAgentOptions, PermissionMode};

let options = ClaudeAgentOptions::builder()
    .model("sonnet")
    .max_turns(10)
    .max_budget_usd(5.0)
    .permission_mode(PermissionMode::AcceptEdits)
    .tools(["Read", "Write", "Bash"])
    .build();
```

### Struct Initialization

```rust
let options = ClaudeAgentOptions {
    model: Some("sonnet".to_string()),
    max_turns: Some(10),
    tools: Some(["Read", "Write"].into()),
    ..Default::default()
};
```

### Complete Options Reference

#### Model Configuration

| Field | Type | Description |
|-------|------|-------------|
| `model` | `Option<String>` | Primary model: `"sonnet"`, `"opus"`, `"haiku"`, or full ID |
| `fallback_model` | `Option<String>` | Backup model if primary fails |
| `max_thinking_tokens` | `Option<u32>` | Max tokens for extended thinking |

```rust
let options = ClaudeAgentOptions::builder()
    .model("opus")
    .fallback_model("sonnet")
    .max_thinking_tokens(4000)
    .build();
```

#### Execution Limits

| Field | Type | Description |
|-------|------|-------------|
| `max_turns` | `Option<u32>` | Maximum conversation turns |
| `max_budget_usd` | `Option<f64>` | Cost limit in USD |

```rust
let options = ClaudeAgentOptions::builder()
    .max_turns(20)
    .max_budget_usd(10.0)
    .build();
```

#### Tool Configuration

| Field | Type | Description |
|-------|------|-------------|
| `tools` | `Option<Tools>` | Restrict available tools |
| `allowed_tools` | `Vec<String>` | Grant MCP tool permissions |
| `disallowed_tools` | `Vec<String>` | Blacklist specific tools |

```rust
// Restrict to specific tools
let options = ClaudeAgentOptions::builder()
    .tools(["Read", "Write", "Bash", "Glob", "Grep"])
    .build();

// Allow all tools except some
let options = ClaudeAgentOptions::builder()
    .disallowed_tools(vec!["Bash".to_string()])
    .build();

// Grant MCP tool permissions (format: mcp__{server}__{tool})
let options = ClaudeAgentOptions::builder()
    .allowed_tools(vec![
        "mcp__my-server__my-tool".to_string(),
        "mcp__db__query".to_string(),
    ])
    .build();
```

#### Permission Modes

| Mode | Description |
|------|-------------|
| `Default` | Ask for each tool |
| `AcceptEdits` | Auto-accept Edit tools |
| `Plan` | Plan mode (limited execution) |
| `BypassPermissions` | Allow all tools without asking |

```rust
use claude_agent_sdk_rs::PermissionMode;

let options = ClaudeAgentOptions::builder()
    .permission_mode(PermissionMode::AcceptEdits)
    .build();
```

#### System Prompt

```rust
// Simple text prompt
let options = ClaudeAgentOptions::builder()
    .system_prompt("You are a helpful coding assistant.")
    .build();

// Append to default prompt
use claude_agent_sdk_rs::SystemPrompt;

let options = ClaudeAgentOptions::builder()
    .system_prompt(SystemPrompt::Preset {
        preset: "claude_code".to_string(),
        append: Some("Always use TypeScript.".to_string()),
    })
    .build();
```

#### Working Directory and Environment

```rust
use std::path::PathBuf;
use std::collections::HashMap;

let options = ClaudeAgentOptions::builder()
    .cwd(PathBuf::from("/path/to/project"))
    .add_dirs(vec![PathBuf::from("/additional/dir")])
    .env(HashMap::from([
        ("MY_VAR".to_string(), "value".to_string()),
    ]))
    .build();
```

#### Session Configuration

```rust
let options = ClaudeAgentOptions::builder()
    .resume("session-id-123")           // Resume existing session
    .fork_session(true)                  // Start fresh context
    .continue_conversation(true)         // Continue conversation
    .enable_file_checkpointing(true)     // Track file changes
    .build();
```

#### Settings Sources

```rust
use claude_agent_sdk_rs::SettingSource;

let options = ClaudeAgentOptions::builder()
    .setting_sources(Some(vec![
        SettingSource::User,      // ~/.claude/settings.json
        SettingSource::Project,   // .claude/settings.json
        SettingSource::Local,     // .claude/settings.local.json
    ]))
    .build();
```

#### Debug and Logging

```rust
use std::sync::Arc;

let options = ClaudeAgentOptions::builder()
    .stderr_callback(Some(Arc::new(|msg| eprintln!("DEBUG: {}", msg))))
    .extra_args(HashMap::from([
        ("debug-to-stderr".to_string(), None),
    ]))
    .build();
```

---

## ClaudeClient

`ClaudeClient` provides bidirectional streaming for multi-turn conversations.

### Lifecycle

```rust
use claude_agent_sdk_rs::{ClaudeClient, ClaudeAgentOptions};

let mut client = ClaudeClient::new(ClaudeAgentOptions::default());

// Connect to Claude Code CLI
client.connect().await?;

// Send queries and receive responses
client.query("Hello!").await?;

// Disconnect when done
client.disconnect().await?;
```

### Sending Queries

```rust
// Simple text query
client.query("What is Rust?").await?;

// Query with specific session
client.query_with_session("Tell me about Rust", "rust-session").await?;

// Start a new session with a query
client.new_session("new-session", "Let's start fresh").await?;

// Query with multimodal content
use claude_agent_sdk_rs::UserContentBlock;

client.query_with_content(vec![
    UserContentBlock::text("What's in this image?"),
    UserContentBlock::image_base64("image/png", base64_data)?,
]).await?;
```

### Receiving Responses

```rust
use futures::StreamExt;

// Receive until Result message (single turn)
let mut stream = client.receive_response();
while let Some(message) = stream.next().await {
    match message? {
        Message::Assistant(msg) => {
            for block in msg.message.content {
                match block {
                    ContentBlock::Text(text) => println!("{}", text.text),
                    ContentBlock::ToolUse(tool) => println!("Using tool: {}", tool.name),
                    ContentBlock::Thinking(thinking) => println!("Thinking: {}", thinking.thinking),
                    _ => {}
                }
            }
        }
        Message::Result(result) => {
            println!("Cost: ${:.4}", result.total_cost_usd.unwrap_or(0.0));
            println!("Turns: {}", result.num_turns);
            break;
        }
        Message::System(sys) => {
            println!("System event: {:?}", sys);
        }
        _ => {}
    }
}
drop(stream); // Important: drop stream before next query
```

### Dynamic Control

```rust
// Interrupt current execution
client.interrupt().await?;

// Change permission mode mid-execution
client.set_permission_mode(PermissionMode::Plan).await?;

// Switch model
client.set_model(Some("opus")).await?;
client.set_model(None).await?;  // Reset to default

// Rewind files to checkpoint (requires enable_file_checkpointing)
client.rewind_files("user-message-uuid").await?;

// Get server info
if let Some(info) = client.get_server_info() {
    println!("Server capabilities: {:?}", info);
}
```

---

## Query Functions

For simple one-shot interactions without managing client lifecycle.

### query

Collects all messages in memory.

```rust
use claude_agent_sdk_rs::{query, ClaudeAgentOptions};

// With default options
let messages = query("What is 2 + 2?", None).await?;

// With custom options
let options = ClaudeAgentOptions::builder()
    .model("sonnet")
    .max_turns(5)
    .build();
let messages = query("Create a file", Some(options)).await?;
```

### query_stream

Memory-efficient streaming for large responses.

```rust
use claude_agent_sdk_rs::query_stream;
use futures::StreamExt;

let mut stream = query_stream("Write a long essay", None).await?;

while let Some(result) = stream.next().await {
    let message = result?;
    // Process each message as it arrives
}
```

### query_with_content

Query with multimodal content (images + text).

```rust
use claude_agent_sdk_rs::{query_with_content, UserContentBlock};

let messages = query_with_content(vec![
    UserContentBlock::text("Describe this image"),
    UserContentBlock::image_url("https://example.com/image.png"),
], None).await?;
```

### query_stream_with_content

Streaming with multimodal content.

```rust
use claude_agent_sdk_rs::query_stream_with_content;

let mut stream = query_stream_with_content(vec![
    UserContentBlock::text("Analyze this diagram"),
    UserContentBlock::image_base64("image/png", encoded_data)?,
], None).await?;
```

---

## Messages and Content

### Message Types

```rust
pub enum Message {
    Assistant(AssistantMessage),  // Claude's response
    System(SystemMessage),        // System events
    Result(ResultMessage),        // Query completion with metrics
    StreamEvent(StreamEvent),     // Stream events
    User(UserMessage),            // User prompts
    ControlCancelRequest(Value),  // Internal control
}
```

### AssistantMessage Structure

```rust
pub struct AssistantMessage {
    pub message: AssistantMessageInner,
    pub parent_tool_use_id: Option<String>,
    pub session_id: Option<String>,
    pub uuid: Option<String>,  // For file checkpointing
}

pub struct AssistantMessageInner {
    pub content: Vec<ContentBlock>,
    pub model: Option<String>,
    pub id: Option<String>,
    pub stop_reason: Option<String>,
    pub usage: Option<Value>,
    pub error: Option<AssistantMessageError>,
}
```

### ContentBlock Types

```rust
pub enum ContentBlock {
    Text(TextBlock),           // Text response
    Thinking(ThinkingBlock),   // Extended thinking
    ToolUse(ToolUseBlock),     // Tool invocation
    ToolResult(ToolResultBlock), // Tool result
    Image(ImageBlock),         // Image in response
}
```

### ResultMessage

Contains execution metrics:

```rust
pub struct ResultMessage {
    pub subtype: String,
    pub duration_ms: u64,
    pub duration_api_ms: u64,
    pub is_error: bool,
    pub num_turns: u32,
    pub session_id: String,
    pub total_cost_usd: Option<f64>,
    pub usage: Option<Value>,
    pub result: Option<String>,
    pub structured_output: Option<Value>,
}
```

---

## Hooks System

Hooks intercept Claude's behavior at various points. The SDK supports 6 hook types.

### Hook Types

| Hook | When Fired | Use Case |
|------|------------|----------|
| `PreToolUse` | Before tool execution | Block/allow/modify tools |
| `PostToolUse` | After tool execution | Log, analyze, add context |
| `UserPromptSubmit` | When user submits prompt | Inject context |
| `Stop` | When execution stops | Provide feedback |
| `SubagentStop` | When subagent stops | Handle subagent completion |
| `PreCompact` | Before conversation compaction | Custom compaction logic |

### Hook Input Types

```rust
pub enum HookInput {
    PreToolUse(PreToolUseInput),
    PostToolUse(PostToolUseInput),
    UserPromptSubmit(UserPromptSubmitInput),
    Stop(StopInput),
    SubagentStop(SubagentStopInput),
    PreCompact(PreCompactInput),
}
```

### Creating Hooks

```rust
use claude_agent_sdk_rs::{
    HookEvent, HookMatcher, HookInput, HookContext, HookJsonOutput,
    SyncHookJsonOutput, HookSpecificOutput, PreToolUseHookSpecificOutput,
};
use std::collections::HashMap;
use std::sync::Arc;

// Define a hook function
async fn my_pretooluse_hook(
    input: HookInput,
    _tool_use_id: Option<String>,
    _context: HookContext,
) -> HookJsonOutput {
    if let HookInput::PreToolUse(pre_tool) = input {
        // Block dangerous commands
        if pre_tool.tool_name == "Bash" {
            let command = pre_tool.tool_input
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if command.contains("rm -rf") {
                return HookJsonOutput::Sync(SyncHookJsonOutput {
                    hook_specific_output: Some(HookSpecificOutput::PreToolUse(
                        PreToolUseHookSpecificOutput {
                            permission_decision: Some("deny".to_string()),
                            permission_decision_reason: Some("Dangerous command blocked".to_string()),
                            ..Default::default()
                        }
                    )),
                    ..Default::default()
                });
            }
        }
    }
    HookJsonOutput::Sync(SyncHookJsonOutput::default())
}

// Register hooks
let mut hooks: HashMap<HookEvent, Vec<HookMatcher>> = HashMap::new();

hooks.insert(
    HookEvent::PreToolUse,
    vec![HookMatcher {
        matcher: Some("Bash".to_string()),  // Only match Bash tool
        hooks: vec![Arc::new(|input, tool_use_id, context| {
            Box::pin(my_pretooluse_hook(input, tool_use_id, context))
        })],
        timeout: None,
    }],
);

let options = ClaudeAgentOptions::builder()
    .hooks(Some(hooks))
    .build();
```

### Hook Output Fields

```rust
pub struct SyncHookJsonOutput {
    pub continue_: Option<bool>,       // Continue execution?
    pub stop_reason: Option<String>,   // Reason for stopping
    pub reason: Option<String>,        // General reason
    pub system_message: Option<String>,// Inject system message
    pub hook_specific_output: Option<HookSpecificOutput>,
}
```

### PreToolUse Hook Specific Output

```rust
pub struct PreToolUseHookSpecificOutput {
    pub permission_decision: Option<String>,  // "allow" or "deny"
    pub permission_decision_reason: Option<String>,
    pub updated_tool_input: Option<Value>,    // Modify tool input
}
```

### PostToolUse Hook Specific Output

```rust
pub struct PostToolUseHookSpecificOutput {
    pub additional_context: Option<String>,  // Add context to response
}
```

### UserPromptSubmit Hook Specific Output

```rust
pub struct UserPromptSubmitHookSpecificOutput {
    pub additional_context: Option<String>,  // Inject context into prompt
}
```

### Complete Hook Example

```rust
use claude_agent_sdk_rs::*;
use std::collections::HashMap;
use std::sync::Arc;

// PreToolUse: Block dangerous commands
async fn security_hook(input: HookInput, _: Option<String>, _: HookContext) -> HookJsonOutput {
    if let HookInput::PreToolUse(pre) = input {
        if pre.tool_name == "Bash" {
            let cmd = pre.tool_input.get("command").and_then(|v| v.as_str()).unwrap_or("");
            if cmd.contains("rm -rf /") {
                return HookJsonOutput::Sync(SyncHookJsonOutput {
                    hook_specific_output: Some(HookSpecificOutput::PreToolUse(
                        PreToolUseHookSpecificOutput {
                            permission_decision: Some("deny".to_string()),
                            permission_decision_reason: Some("Blocked destructive command".to_string()),
                            ..Default::default()
                        }
                    )),
                    ..Default::default()
                });
            }
        }
    }
    HookJsonOutput::Sync(SyncHookJsonOutput::default())
}

// UserPromptSubmit: Add context
async fn context_hook(input: HookInput, _: Option<String>, _: HookContext) -> HookJsonOutput {
    if let HookInput::UserPromptSubmit(_) = input {
        return HookJsonOutput::Sync(SyncHookJsonOutput {
            hook_specific_output: Some(HookSpecificOutput::UserPromptSubmit(
                UserPromptSubmitHookSpecificOutput {
                    additional_context: Some("Current project uses TypeScript.".to_string()),
                }
            )),
            ..Default::default()
        });
    }
    HookJsonOutput::Sync(SyncHookJsonOutput::default())
}

// PostToolUse: Log tool usage
async fn logging_hook(input: HookInput, _: Option<String>, _: HookContext) -> HookJsonOutput {
    if let HookInput::PostToolUse(post) = input {
        println!("Tool '{}' completed", post.tool_name);
    }
    HookJsonOutput::Sync(SyncHookJsonOutput::default())
}

// Build hooks map
let mut hooks: HashMap<HookEvent, Vec<HookMatcher>> = HashMap::new();

hooks.insert(HookEvent::PreToolUse, vec![
    HookMatcher {
        matcher: Some("Bash".to_string()),
        hooks: vec![Arc::new(|i, t, c| Box::pin(security_hook(i, t, c)))],
        timeout: None,
    },
]);

hooks.insert(HookEvent::UserPromptSubmit, vec![
    HookMatcher {
        matcher: None,  // Match all prompts
        hooks: vec![Arc::new(|i, t, c| Box::pin(context_hook(i, t, c)))],
        timeout: None,
    },
]);

hooks.insert(HookEvent::PostToolUse, vec![
    HookMatcher {
        matcher: None,  // Match all tools
        hooks: vec![Arc::new(|i, t, c| Box::pin(logging_hook(i, t, c)))],
        timeout: None,
    },
]);

let options = ClaudeAgentOptions::builder()
    .hooks(Some(hooks))
    .build();
```

---

## MCP Server Integration

Create custom in-process tools using the Model Context Protocol.

### Creating Tools with the `tool!` Macro

```rust
use claude_agent_sdk_rs::{tool, ToolResult, McpToolResultContent};
use serde_json::json;

// Simple tool
async fn add_handler(args: serde_json::Value) -> anyhow::Result<ToolResult> {
    let a = args["a"].as_f64().unwrap_or(0.0);
    let b = args["b"].as_f64().unwrap_or(0.0);

    Ok(ToolResult {
        content: vec![McpToolResultContent::Text {
            text: format!("{}", a + b),
        }],
        is_error: false,
    })
}

let add_tool = tool!(
    "add",
    "Add two numbers",
    json!({
        "type": "object",
        "properties": {
            "a": { "type": "number", "description": "First number" },
            "b": { "type": "number", "description": "Second number" }
        },
        "required": ["a", "b"]
    }),
    add_handler
);
```

### Creating an MCP Server

```rust
use claude_agent_sdk_rs::{create_sdk_mcp_server, McpServers, McpServerConfig};
use std::collections::HashMap;

// Create server with tools
let server = create_sdk_mcp_server("math-tools", "1.0.0", vec![add_tool]);

// Configure in options
let mut mcp_servers = HashMap::new();
mcp_servers.insert("math-tools".to_string(), McpServerConfig::Sdk(server));

let options = ClaudeAgentOptions::builder()
    .mcp_servers(McpServers::Dict(mcp_servers))
    .allowed_tools(vec!["mcp__math-tools__add".to_string()])
    .permission_mode(PermissionMode::AcceptEdits)
    .build();
```

### MCP Server Configuration Types

```rust
pub enum McpServers {
    Empty,                                    // No servers
    Dict(HashMap<String, McpServerConfig>),   // Server map
    Path(PathBuf),                            // Config file path
}

pub enum McpServerConfig {
    Stdio(McpStdioServerConfig),   // Subprocess via stdio
    Sse(McpSseServerConfig),       // Server-sent events
    Http(McpHttpServerConfig),     // HTTP REST
    Sdk(McpSdkServerConfig),       // In-process (Rust SDK)
}
```

### External MCP Server

```rust
use claude_agent_sdk_rs::{McpStdioServerConfig, McpServerConfig, McpServers};

let external_server = McpServerConfig::Stdio(McpStdioServerConfig {
    command: "npx".to_string(),
    args: vec!["-y".to_string(), "@modelcontextprotocol/server-sqlite".to_string()],
    env: HashMap::new(),
});

let mut mcp_servers = HashMap::new();
mcp_servers.insert("sqlite".to_string(), external_server);

let options = ClaudeAgentOptions::builder()
    .mcp_servers(McpServers::Dict(mcp_servers))
    .allowed_tools(vec!["mcp__sqlite__query".to_string()])
    .build();
```

### Tool Result Content Types

```rust
pub enum McpToolResultContent {
    Text { text: String },
    Image { data: String, mime_type: String },
    Resource { uri: String, mime_type: Option<String>, text: Option<String> },
}
```

---

## Permission Management

### Permission Modes

```rust
use claude_agent_sdk_rs::PermissionMode;

// Ask for each tool
let options = ClaudeAgentOptions::builder()
    .permission_mode(PermissionMode::Default)
    .build();

// Auto-accept edit operations
let options = ClaudeAgentOptions::builder()
    .permission_mode(PermissionMode::AcceptEdits)
    .build();

// Plan mode - limited execution
let options = ClaudeAgentOptions::builder()
    .permission_mode(PermissionMode::Plan)
    .build();

// Allow all tools
let options = ClaudeAgentOptions::builder()
    .permission_mode(PermissionMode::BypassPermissions)
    .build();
```

### Dynamic Permission Callback

```rust
use claude_agent_sdk_rs::{CanUseToolCallback, PermissionResult, PermissionResultAllow, PermissionResultDeny};
use std::sync::Arc;

let can_use_tool: CanUseToolCallback = Arc::new(|tool_name, args, _ctx| {
    Box::pin(async move {
        // Custom permission logic
        if tool_name == "Bash" {
            let cmd = args.get("command").and_then(|v| v.as_str()).unwrap_or("");
            if cmd.contains("rm") {
                return PermissionResult::Deny(PermissionResultDeny {
                    message: "rm commands are not allowed".to_string(),
                    interrupt: false,
                });
            }
        }

        PermissionResult::Allow(PermissionResultAllow {
            updated_input: None,
            updated_permissions: None,
        })
    })
});

let options = ClaudeAgentOptions::builder()
    .can_use_tool(Some(can_use_tool))
    .build();
```

---

## Session Management

### Session IDs

Different session IDs maintain separate conversation contexts:

```rust
let mut client = ClaudeClient::new(ClaudeAgentOptions::default());
client.connect().await?;

// Session 1: Math context
client.query_with_session("What is 2 + 2?", "math-session").await?;
// ... process response

// Session 2: Different context
client.query_with_session("What is Rust?", "rust-session").await?;
// ... process response

// Back to session 1 - remembers math context
client.query_with_session("What about 3 + 3?", "math-session").await?;
```

### Fork Session

Start completely fresh without history:

```rust
let options = ClaudeAgentOptions::builder()
    .fork_session(true)  // Each resumed session starts fresh
    .build();
```

### Resume Session

Continue an existing session:

```rust
let options = ClaudeAgentOptions::builder()
    .resume("previous-session-id")
    .build();
```

### File Checkpointing

Track and rewind file changes:

```rust
let options = ClaudeAgentOptions::builder()
    .enable_file_checkpointing(true)
    .extra_args(HashMap::from([
        ("replay-user-messages".to_string(), None),
    ]))
    .build();

let mut client = ClaudeClient::new(options);
client.connect().await?;

// Later, rewind to a checkpoint
client.rewind_files("user-message-uuid").await?;
```

---

## Efficiency Hooks

Built-in hooks for execution optimization and metrics tracking.

### Enable Efficiency Features

```rust
use claude_agent_sdk_rs::EfficiencyConfig;

// All features enabled
let options = ClaudeAgentOptions::builder()
    .efficiency(EfficiencyConfig::enabled())
    .build();

// Only CWD reminder
let options = ClaudeAgentOptions::builder()
    .efficiency(EfficiencyConfig::cwd_reminder_only())
    .build();

// Only stop tips
let options = ClaudeAgentOptions::builder()
    .efficiency(EfficiencyConfig::stop_tips_only())
    .build();

// Metrics + stop tips
let options = ClaudeAgentOptions::builder()
    .efficiency(EfficiencyConfig::with_metrics())
    .build();

// Custom CWD
let options = ClaudeAgentOptions::builder()
    .efficiency(EfficiencyConfig::enabled().with_cwd("/my/project"))
    .build();
```

### Efficiency Features

| Feature | Description |
|---------|-------------|
| `inject_cwd_reminder` | Adds working directory reminder at prompt |
| `inject_stop_tips` | Provides feedback on execution patterns |
| `track_metrics` | Collects execution metrics |

### Tracked Metrics

- Total tool calls by type
- Edit calls per file (detect fragmentation)
- Directory checks (pwd/cd usage)
- Build/test attempts
- TodoWrite call frequency

### Efficiency Warnings

The SDK generates warnings for common inefficiencies:

- **FRAGMENTED EDITS**: >3 edits to same file
- **DIRECTORY CONFUSION**: >2 pwd/cd commands
- **BUILD ITERATIONS**: >2 build/test attempts
- **EXCESSIVE TODO UPDATES**: >8 TodoWrite calls

---

## Multimodal Input

Send images alongside text in queries.

### Supported Formats

- JPEG (`image/jpeg`)
- PNG (`image/png`)
- GIF (`image/gif`)
- WebP (`image/webp`)
- Maximum: 15MB base64 (20MB decoded)

### Image from Base64

```rust
use claude_agent_sdk_rs::UserContentBlock;

let image_data = std::fs::read("image.png")?;
let base64_data = base64::encode(&image_data);

let content = vec![
    UserContentBlock::text("What's in this image?"),
    UserContentBlock::image_base64("image/png", base64_data)?,
];

let messages = query_with_content(content, None).await?;
```

### Image from URL

```rust
let content = vec![
    UserContentBlock::text("Describe this diagram"),
    UserContentBlock::image_url("https://example.com/diagram.png"),
];
```

### With ClaudeClient

```rust
client.query_with_content(vec![
    UserContentBlock::text("Analyze this chart"),
    UserContentBlock::image_base64("image/png", chart_data)?,
]).await?;
```

---

## Error Handling

### Error Types

```rust
pub enum ClaudeError {
    Connection(ConnectionError),           // CLI connection failed
    Process(ProcessError),                 // CLI process error
    JsonDecode(JsonDecodeError),          // JSON parse failure
    MessageParse(MessageParseError),      // Message structure error
    Transport(String),                    // Transport layer error
    ControlProtocol(String),              // Hook/control protocol error
    InvalidConfig(String),                // Configuration error
    CliNotFound(CliNotFoundError),        // Claude CLI not found
    ImageValidation(ImageValidationError), // Image size/format error
    Io(std::io::Error),                   // I/O error
    Other(anyhow::Error),                 // Catch-all
}
```

### Error Handling Pattern

```rust
use claude_agent_sdk_rs::{query, ClaudeError};

match query("Hello", None).await {
    Ok(messages) => {
        // Process messages
    }
    Err(ClaudeError::CliNotFound(e)) => {
        eprintln!("Claude CLI not found: {}", e);
        eprintln!("Install from: https://docs.claude.com/claude-code");
    }
    Err(ClaudeError::Connection(e)) => {
        eprintln!("Connection error: {}", e);
    }
    Err(ClaudeError::Process(e)) => {
        eprintln!("Process error (exit code {:?}): {}", e.exit_code, e.stderr);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

### Common Issues

**CLI Not Found**
```rust
// Check CLI availability
match claude_agent_sdk_rs::get_claude_code_version().await {
    Ok(version) => println!("CLI version: {}", version),
    Err(_) => println!("CLI not installed"),
}
```

**Permission Denied**
```rust
// Use appropriate permission mode
let options = ClaudeAgentOptions::builder()
    .permission_mode(PermissionMode::AcceptEdits)
    .build();
```

**API Key Issues**
```rust
// Set environment variable
std::env::set_var("ANTHROPIC_API_KEY", "your-key");

// Or configure via CLI
// claude config set api_key your-key
```

---

## Utility Functions

### Get CLI Version

```rust
let version = claude_agent_sdk_rs::get_claude_code_version().await?;
println!("Claude Code CLI version: {}", version);
```

### Skip Version Check

For faster startup in production:

```rust
let options = ClaudeAgentOptions::builder()
    .skip_version_check(true)
    .build();
```

---

## Examples Reference

| Example | Description |
|---------|-------------|
| `01_hello_world` | Basic query with tool usage |
| `02_limit_tool_use` | Restrict available tools |
| `03_monitor_tools` | Monitor tool execution |
| `04_permission_callbacks` | Dynamic permission control |
| `05_hooks_pretooluse` | PreToolUse hooks |
| `06_bidirectional_client` | Multi-turn conversations |
| `07_dynamic_control` | Runtime control (interrupt, model switch) |
| `08_mcp_server_integration` | Custom MCP tools |
| `09_agents` | Custom agent definitions |
| `10_include_partial_messages` | Partial message handling |
| `11_setting_sources` | Settings configuration |
| `12_stderr_callback` | Debug output |
| `13_system_prompt` | System prompt configuration |
| `14_streaming_mode` | Streaming patterns |
| `15_hooks_comprehensive` | All hook types |
| `16_session_management` | Session handling |
| `17_fallback_model` | Model fallback |
| `18_max_budget_usd` | Budget control |
| `19_max_thinking_tokens` | Extended thinking |
| `20_query_stream` | Streaming query API |
| `21_custom_plugins` | Plugin loading |
| `22_plugin_integration` | Plugin usage |
| `23_image_input` | Multimodal queries |
| `24_efficiency_hooks` | Execution optimization |

Run examples:
```bash
cargo run --example 01_hello_world
cargo run --example 15_hooks_comprehensive -- all
```
