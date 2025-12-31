# Claude Agent SDK: Python vs Rust Feature Parity Specification

**Generated:** 2025-12-30
**Python SDK Version:** Latest (vendors/claude-agent-sdk-python)
**Rust SDK Version:** Current (claude-agent-sdk-rs)

## Executive Summary

This document provides a comprehensive comparison between the Python Claude Agent SDK and the Rust implementation, identifying feature parity status, implementation differences, and areas requiring attention.

| Category | Python | Rust | Status |
|----------|--------|------|--------|
| Core API | ✅ | ✅ | **Full Parity** |
| Type System | ✅ | ✅ | **Full Parity** |
| Transport Layer | ✅ | ✅ | **Full Parity** |
| Hook System | ✅ | ✅ | **Full Parity** |
| MCP Support | ✅ | ✅ | **Full Parity** |
| Error Handling | ✅ | ✅ | **Full Parity** |
| Streaming | ✅ | ✅ | **Full Parity** |
| File Checkpointing | ✅ | ✅ | **Full Parity** |
| Sandbox Config | ✅ | ✅ | **Full Parity** |
| Agent Definitions | ✅ | ✅ | **Full Parity** |
| Beta Features | ✅ | ✅ | **Full Parity** |

---

## ✅ ALL FEATURES NOW IMPLEMENTED (as of 2025-12-30)

All previously missing features have been implemented:

| Feature | Python | Rust | Status |
|---------|--------|------|--------|
| File Checkpointing | ✅ | ✅ | **IMPLEMENTED** |
| `rewind_files()` method | ✅ | ✅ | **IMPLEMENTED** |
| `StreamEvent` message type | ✅ | ✅ | **IMPLEMENTED** |
| `include_partial_messages` option | ✅ | ✅ | **IMPLEMENTED** |
| `SdkBeta` / `betas` option | ✅ | ✅ | **IMPLEMENTED** |
| `AgentDefinition` type | ✅ | ✅ | **IMPLEMENTED** |
| `agents` option | ✅ | ✅ | **IMPLEMENTED** |
| `SandboxSettings` type | ✅ | ✅ | **IMPLEMENTED** |
| `sandbox` option | ✅ | ✅ | **IMPLEMENTED** |
| `setting_sources` option | ✅ | ✅ | **IMPLEMENTED** |
| `settings` option | ✅ | ✅ | **IMPLEMENTED** |
| `add_dirs` option | ✅ | ✅ | **IMPLEMENTED** |
| `tools` / `ToolsPreset` option | ✅ | ✅ | **IMPLEMENTED** |
| `permission_prompt_tool_name` | ✅ | ✅ | **IMPLEMENTED** |
| `user` option | ✅ | ✅ | **IMPLEMENTED** |
| `UserMessage.uuid` field | ✅ | ✅ | **IMPLEMENTED** |

---

## 1. Core API

### 1.1 One-Shot Query Function

| Feature | Python | Rust | Notes |
|---------|--------|------|-------|
| Basic query | `query(prompt, options)` | `query(prompt, options)` | ✅ Identical API |
| Async execution | `async def query()` | `async fn query()` | ✅ Both async |
| Options parameter | Optional | Optional | ✅ Matching |
| Return type | `list[Message]` | `Vec<Message>` | ✅ Equivalent |

**Python Implementation (`query.py:8-14`):**
```python
async def query(
    prompt: str,
    options: ClaudeAgentOptions | None = None,
) -> list[Message]:
```

**Rust Implementation (`query.rs:43-52`):**
```rust
pub async fn query(
    prompt: impl Into<String>,
    options: Option<ClaudeAgentOptions>,
) -> Result<Vec<Message>>
```

### 1.2 Streaming Query (Rust Extension)

| Feature | Python | Rust | Notes |
|---------|--------|------|-------|
| Streaming query | ❌ Not available | `query_stream()` | 🟡 Rust-only feature |

**Rust-only API (`query.rs:92-125`):**
```rust
pub async fn query_stream(
    prompt: impl Into<String>,
    options: Option<ClaudeAgentOptions>,
) -> Result<Pin<Box<dyn Stream<Item = Result<Message>> + Send>>>
```

### 1.3 Interactive Client

| Feature | Python | Rust | Notes |
|---------|--------|------|-------|
| Client struct | `ClaudeSDKClient` | `ClaudeClient` | ✅ Equivalent |
| Context manager | `async with client:` | `connect()/disconnect()` | ✅ Adapted for Rust |
| Query method | `client.query(prompt)` | `client.query(prompt)` | ✅ Identical |
| Session queries | `client.query(prompt, session_id)` | `client.query_with_session()` | ✅ Equivalent |
| Interrupt | `client.interrupt()` | `client.interrupt()` | ✅ Identical |
| Set permission mode | `client.set_permission_mode()` | `client.set_permission_mode()` | ✅ Identical |
| Set model | `client.set_model()` | `client.set_model()` | ✅ Identical |
| Get server info | `client.get_server_info()` | `client.get_server_info()` | ✅ Identical |
| New session | N/A | `client.new_session()` | 🟢 Rust convenience method |

**Python Implementation (`client.py:7-69`):**
```python
class ClaudeSDKClient:
    async def __aenter__(self) -> Self:
    async def __aexit__(self, *args: object) -> None:
    async def query(self, prompt: str, session_id: str = "default") -> AsyncIterator[Message]:
    async def interrupt(self) -> None:
    async def set_permission_mode(self, mode: PermissionMode) -> None:
    async def set_model(self, model: str | None) -> None:
    def get_server_info(self) -> dict | None:
```

**Rust Implementation (`client.rs:53-600`):**
```rust
impl ClaudeClient {
    pub fn new(options: ClaudeAgentOptions) -> Self
    pub async fn connect(&mut self) -> Result<()>
    pub async fn disconnect(&mut self) -> Result<()>
    pub async fn query(&mut self, prompt: impl Into<String>) -> Result<()>
    pub async fn query_with_session(&mut self, prompt, session_id) -> Result<()>
    pub fn receive_messages(&self) -> Pin<Box<dyn Stream<...>>>
    pub fn receive_response(&self) -> Pin<Box<dyn Stream<...>>>
    pub async fn interrupt(&self) -> Result<()>
    pub async fn set_permission_mode(&self, mode: PermissionMode) -> Result<()>
    pub async fn set_model(&self, model: Option<&str>) -> Result<()>
    pub async fn get_server_info(&self) -> Option<serde_json::Value>
    pub async fn new_session(&mut self, session_id, prompt) -> Result<()>
}
```

---

## 2. Configuration Options (ClaudeAgentOptions)

### 2.1 Full Options Comparison

| Option | Python | Rust | Status |
|--------|--------|------|--------|
| `prompt` | ✅ | ✅ (in query) | ✅ |
| `cwd` | ✅ | ✅ | ✅ |
| `permission_mode` | ✅ | ✅ | ✅ |
| `allowed_tools` | ✅ | ✅ | ✅ |
| `disallowed_tools` | ✅ | ✅ | ✅ |
| `mcp_servers` | ✅ | ✅ | ✅ |
| `can_use_tool` | ✅ | ✅ | ✅ |
| `model` | ✅ | ✅ | ✅ |
| `fallback_model` | ✅ | ✅ | ✅ |
| `max_thinking_tokens` | ✅ | ✅ | ✅ |
| `max_turns` | ✅ | ✅ | ✅ |
| `system_prompt` | ✅ | ✅ | ✅ |
| `hooks` | ✅ | ✅ | ✅ |
| `resume` | ✅ | ✅ | ✅ |
| `continue_conversation` | ✅ | ✅ | ✅ |
| `max_budget_usd` | ✅ | ✅ | ✅ |
| `env` | ✅ | ✅ | ✅ |
| `cli_path` | ✅ | ✅ | ✅ |
| `stderr_callback` | ✅ | ✅ | ✅ |
| `output_format` | ✅ | ✅ | ✅ |
| `fork_session` | ✅ | ✅ | ✅ |
| `plugins` | ✅ | ✅ | ✅ |
| `max_buffer_size` | ❌ | ✅ | 🟢 Rust extension |
| `extra_args` | ❌ | ✅ | 🟢 Rust extension |

**Python Type Definition (`types.py:180-219`):**
```python
@dataclass
class ClaudeAgentOptions:
    prompt: str | None = None
    cwd: str | None = None
    permission_mode: PermissionMode | None = None
    allowed_tools: list[str] = field(default_factory=list)
    disallowed_tools: list[str] = field(default_factory=list)
    mcp_servers: McpServers | None = None
    can_use_tool: CanUseToolCallback | None = None
    model: str | None = None
    fallback_model: str | None = None
    max_thinking_tokens: int | None = None
    max_turns: int | None = None
    system_prompt: str | SystemPromptPreset | None = None
    hooks: Hooks | None = None
    resume: str | None = None
    continue_conversation: bool = False
    max_budget_usd: float | None = None
    env: dict[str, str] = field(default_factory=dict)
    cli_path: str | None = None
    stderr_callback: Callable[[str], None] | None = None
    output_format: OutputFormat | None = None
    fork_session: bool = False
    plugins: list[SdkPluginConfig] = field(default_factory=list)
```

**Rust Type Definition (`config.rs:44-118`):**
```rust
pub struct ClaudeAgentOptions {
    pub cwd: Option<PathBuf>,
    pub permission_mode: Option<PermissionMode>,
    pub allowed_tools: Vec<String>,
    pub disallowed_tools: Vec<String>,
    pub mcp_servers: McpServers,
    pub can_use_tool: Option<CanUseToolCallback>,
    pub model: Option<String>,
    pub fallback_model: Option<String>,
    pub max_thinking_tokens: Option<u32>,
    pub max_turns: Option<u32>,
    pub system_prompt: Option<SystemPrompt>,
    pub hooks: Option<HashMap<HookEvent, Vec<HookMatcher>>>,
    pub resume: Option<String>,
    pub continue_conversation: bool,
    pub max_budget_usd: Option<f64>,
    pub env: HashMap<String, String>,
    pub cli_path: Option<PathBuf>,
    pub stderr_callback: Option<Arc<dyn Fn(String) + Send + Sync>>,
    pub output_format: Option<serde_json::Value>,
    pub fork_session: bool,
    pub plugins: Vec<SdkPluginConfig>,
    pub max_buffer_size: Option<usize>,
    pub extra_args: HashMap<String, Option<String>>,
}
```

### 2.2 Permission Modes

| Mode | Python | Rust | Status |
|------|--------|------|--------|
| `default` | ✅ | ✅ | ✅ |
| `acceptEdits` | ✅ | ✅ | ✅ |
| `plan` | ✅ | ✅ | ✅ |
| `bypassPermissions` | ✅ | ✅ | ✅ |

### 2.3 System Prompt Types

| Type | Python | Rust | Status |
|------|--------|------|--------|
| Plain text | ✅ | ✅ | ✅ |
| Preset with append | ✅ | ✅ | ✅ |

---

## 3. Message Types

### 3.1 Message Variants

| Message Type | Python | Rust | Status |
|--------------|--------|------|--------|
| `user` | ✅ | ✅ | ✅ |
| `assistant` | ✅ | ✅ | ✅ |
| `result` | ✅ | ✅ | ✅ |
| `system` | ✅ | ✅ | ✅ |

**Python Message Types (`types.py:61-84`):**
```python
@dataclass
class UserMessage: ...
@dataclass
class AssistantMessage: ...
@dataclass
class ResultMessage: ...
@dataclass
class SystemMessage: ...
```

**Rust Message Types (`messages.rs`):**
```rust
pub enum Message {
    User(UserMessage),
    Assistant(AssistantMessage),
    Result(ResultMessage),
    System(SystemMessage),
}
```

### 3.2 Content Blocks

| Block Type | Python | Rust | Status |
|------------|--------|------|--------|
| `text` | ✅ | ✅ | ✅ |
| `tool_use` | ✅ | ✅ | ✅ |
| `tool_result` | ✅ | ✅ | ✅ |
| `thinking` | ✅ | ✅ | ✅ |
| `redacted_thinking` | ✅ | ✅ | ✅ |
| `image` | ✅ | ✅ | ✅ |
| `document` | ✅ | ✅ | ✅ |

---

## 4. Hook System

### 4.1 Hook Events

| Event | Python | Rust | Status |
|-------|--------|------|--------|
| `PreToolUse` | ✅ | ✅ | ✅ |
| `PostToolUse` | ✅ | ✅ | ✅ |
| `UserPromptSubmit` | ✅ | ✅ | ✅ |
| `Stop` | ✅ | ✅ | ✅ |
| `SubagentStop` | ✅ | ✅ | ✅ |
| `PreCompact` | ✅ | ✅ | ✅ |

### 4.2 Hook Input Types

| Input Type | Python | Rust | Status |
|------------|--------|------|--------|
| `PreToolUseHookInput` | ✅ | ✅ | ✅ |
| `PostToolUseHookInput` | ✅ | ✅ | ✅ |
| `UserPromptSubmitHookInput` | ✅ | ✅ | ✅ |
| `StopHookInput` | ✅ | ✅ | ✅ |
| `SubagentStopHookInput` | ✅ | ✅ | ✅ |
| `PreCompactHookInput` | ✅ | ✅ | ✅ |

### 4.3 Hook Output Types

| Output Type | Python | Rust | Status |
|-------------|--------|------|--------|
| `SyncHookJsonOutput` | ✅ | ✅ | ✅ |
| `AsyncHookJsonOutput` | ✅ | ✅ | ✅ |
| `HookSpecificOutput` | ✅ | ✅ | ✅ |

### 4.4 Hook Registration API

**Python (`types.py:157-177`):**
```python
class Hooks:
    def add(self, event: HookEvent, callback: HookCallback, matcher: str | None = None) -> Self:
```

**Rust (`hooks.rs:961-1028`):**
```rust
impl Hooks {
    pub fn new() -> Self
    pub fn build(self) -> HashMap<HookEvent, Vec<HookMatcher>>
    pub fn add_pre_tool_use<F, Fut>(&mut self, hook_fn: F)
    pub fn add_pre_tool_use_with_matcher<F, Fut>(&mut self, matcher, hook_fn: F)
    pub fn add_post_tool_use<F, Fut>(&mut self, hook_fn: F)
    pub fn add_post_tool_use_with_matcher<F, Fut>(&mut self, matcher, hook_fn: F)
    pub fn add_user_prompt_submit<F, Fut>(&mut self, hook_fn: F)
    pub fn add_stop<F, Fut>(&mut self, hook_fn: F)
    pub fn add_subagent_stop<F, Fut>(&mut self, hook_fn: F)
    pub fn add_pre_compact<F, Fut>(&mut self, hook_fn: F)
}
```

---

## 5. MCP (Model Context Protocol) Support

### 5.1 Server Types

| Server Type | Python | Rust | Status |
|-------------|--------|------|--------|
| Stdio | ✅ | ✅ | ✅ |
| SSE | ✅ | ✅ | ✅ |
| HTTP | ❌ | ✅ | 🟢 Rust extension |
| SDK (in-process) | ✅ | ✅ | ✅ |

### 5.2 SDK MCP Server

**Python (`types.py:234-247`):**
```python
def create_sdk_mcp_server(
    name: str,
    version: str,
    tools: list[Tool],
) -> McpServer:
```

**Rust (`mcp.rs:133-148`):**
```rust
pub fn create_sdk_mcp_server(
    name: impl Into<String>,
    version: impl Into<String>,
    tools: Vec<SdkMcpTool>,
) -> McpSdkServerConfig
```

### 5.3 Tool Definition

| Feature | Python | Rust | Status |
|---------|--------|------|--------|
| Tool name | ✅ | ✅ | ✅ |
| Tool description | ✅ | ✅ | ✅ |
| Input schema | ✅ | ✅ | ✅ |
| Handler | ✅ | ✅ | ✅ |
| `tool!` macro | ❌ | ✅ | 🟢 Rust convenience |

---

## 6. Permission System

### 6.1 Permission Callback

| Feature | Python | Rust | Status |
|---------|--------|------|--------|
| `can_use_tool` callback | ✅ | ✅ | ✅ |
| `PermissionResult.Allow` | ✅ | ✅ | ✅ |
| `PermissionResult.Deny` | ✅ | ✅ | ✅ |
| Updated input | ✅ | ✅ | ✅ |
| Permission updates | ✅ | ✅ | ✅ |

### 6.2 Permission Update Types

| Type | Python | Rust | Status |
|------|--------|------|--------|
| `addRules` | ✅ | ✅ | ✅ |
| `replaceRules` | ✅ | ✅ | ✅ |
| `removeRules` | ✅ | ✅ | ✅ |
| `setMode` | ✅ | ✅ | ✅ |
| `addDirectories` | ✅ | ✅ | ✅ |
| `removeDirectories` | ✅ | ✅ | ✅ |

---

## 7. Error Handling

### 7.1 Error Types

| Error Type | Python | Rust | Status |
|------------|--------|------|--------|
| Connection error | ✅ | ✅ `ConnectionError` | ✅ |
| Process error | ✅ | ✅ `ProcessError` | ✅ |
| JSON decode error | ✅ | ✅ `JsonDecodeError` | ✅ |
| Message parse error | ✅ | ✅ `MessageParseError` | ✅ |
| CLI not found | ✅ | ✅ `CliNotFoundError` | ✅ |
| Transport error | ✅ | ✅ `Transport(String)` | ✅ |
| Control protocol error | ❌ | ✅ `ControlProtocol(String)` | 🟢 Rust extension |
| Invalid config error | ❌ | ✅ `InvalidConfig(String)` | 🟢 Rust extension |

**Python Errors (`_errors.py:1-50`):**
```python
class ClaudeSDKError(Exception): ...
class CLINotFoundError(ClaudeSDKError): ...
class CLIConnectionError(ClaudeSDKError): ...
class CLIJSONDecodeError(ClaudeSDKError): ...
class CLIProcessError(ClaudeSDKError): ...
```

**Rust Errors (`errors.rs:7-48`):**
```rust
pub enum ClaudeError {
    Connection(ConnectionError),
    Process(ProcessError),
    JsonDecode(JsonDecodeError),
    MessageParse(MessageParseError),
    Transport(String),
    ControlProtocol(String),
    InvalidConfig(String),
    CliNotFound(CliNotFoundError),
    Io(std::io::Error),
    Other(anyhow::Error),
}
```

---

## 8. Transport Layer

### 8.1 CLI Discovery

| Strategy | Python | Rust | Status |
|----------|--------|------|--------|
| Direct PATH execution | ✅ | ✅ | ✅ |
| `which`/`where` command | ✅ | ✅ | ✅ |
| Common paths fallback | ✅ | ✅ | ✅ |
| Environment variable | ✅ | ✅ | ✅ |
| Home directory expansion | ✅ | ✅ | ✅ |

### 8.2 Command Arguments

| Argument | Python | Rust | Status |
|----------|--------|------|--------|
| `--output-format stream-json` | ✅ | ✅ | ✅ |
| `--input-format stream-json` | ✅ | ✅ | ✅ |
| `--verbose` | ✅ | ✅ | ✅ |
| `--system-prompt` | ✅ | ✅ | ✅ |
| `--append-system-prompt` | ✅ | ✅ | ✅ |
| `--permission-mode` | ✅ | ✅ | ✅ |
| `--allowed-tools` | ✅ | ✅ | ✅ |
| `--disallowed-tools` | ✅ | ✅ | ✅ |
| `--model` | ✅ | ✅ | ✅ |
| `--fallback-model` | ✅ | ✅ | ✅ |
| `--max-budget-usd` | ✅ | ✅ | ✅ |
| `--max-thinking-tokens` | ✅ | ✅ | ✅ |
| `--json-schema` | ✅ | ✅ | ✅ |
| `--max-turns` | ✅ | ✅ | ✅ |
| `--resume` | ✅ | ✅ | ✅ |
| `--continue-conversation` | ✅ | ✅ | ✅ |
| `--fork-session` | ✅ | ✅ | ✅ |
| `--plugin-dir` | ✅ | ✅ | ✅ |

### 8.3 Version Checking

| Feature | Python | Rust | Status |
|---------|--------|------|--------|
| Min version check | ✅ | ✅ | ✅ |
| Skip via env var | ✅ | ✅ | ✅ |
| Warning on mismatch | ✅ | ✅ | ✅ |

---

## 9. Control Protocol

### 9.1 Request Types

| Request Type | Python | Rust | Status |
|--------------|--------|------|--------|
| `initialize` | ✅ | ✅ | ✅ |
| `interrupt` | ✅ | ✅ | ✅ |
| `set_permission_mode` | ✅ | ✅ | ✅ |
| `set_model` | ✅ | ✅ | ✅ |
| `hook_callback` | ✅ | ✅ | ✅ |
| `mcp_message` | ✅ | ✅ | ✅ |

### 9.2 Bidirectional Communication

| Feature | Python | Rust | Status |
|---------|--------|------|--------|
| Control request/response | ✅ | ✅ | ✅ |
| Hook callback invocation | ✅ | ✅ | ✅ |
| MCP message routing | ✅ | ✅ | ✅ |
| Async hook support | ✅ | ✅ | ✅ |

---

## 10. Plugin System

### 10.1 Plugin Types

| Type | Python | Rust | Status |
|------|--------|------|--------|
| Local filesystem | ✅ | ✅ | ✅ |

---

## 11. Rust-Specific Extensions

These features are available in Rust but not in Python:

### 11.1 Builder Pattern

```rust
ClaudeAgentOptions::builder()
    .model("claude-sonnet-4-20250514")
    .permission_mode(PermissionMode::AcceptEdits)
    .build()
```

### 11.2 Streaming Query Function

```rust
let stream = query_stream("prompt", None).await?;
while let Some(msg) = stream.next().await {
    // Process message as it arrives
}
```

### 11.3 Tool Macro

```rust
let tool = tool!(
    "my_tool",
    "Tool description",
    serde_json::json!({"type": "object"}),
    |args| async move { Ok(ToolResult { ... }) }
);
```

### 11.4 Extra Args Support

```rust
ClaudeAgentOptions::builder()
    .extra_args([("custom-flag".to_string(), Some("value".to_string()))])
    .build()
```

---

## 12. Summary

### Feature Parity Status: ✅ FULL PARITY

The Rust SDK now implements **all features** present in the Python SDK:

### What's Implemented ✅

**Core Features:**
1. **Core API**: Full parity with identical function signatures
2. **Hook System**: All 6 hook events supported with typed builders
3. **MCP Support**: All server types (Stdio, SSE, HTTP, SDK)
4. **Permission System**: Full callback and update support
5. **Transport**: Identical CLI argument handling
6. **Control Protocol**: Complete bidirectional communication

**File Checkpointing (NEW):**
- `enable_file_checkpointing` option
- `rewind_files()` client method
- `UserMessage.uuid` field for checkpoint tracking

**Streaming (NEW):**
- `StreamEvent` message type
- `include_partial_messages` option

**Configuration (NEW):**
- `tools` / `ToolsPreset` for tool configuration
- `betas` for beta features (e.g., `SdkBeta::Context1M`)
- `sandbox` for `SandboxSettings` configuration
- `agents` for custom `AgentDefinition` configurations
- `setting_sources` for `SettingSource` configuration
- `settings` path option
- `add_dirs` for additional directories
- `permission_prompt_tool_name`
- `user` option

### Rust Extensions

The Rust SDK provides these additional features not in Python:
- `query_stream()` for memory-efficient streaming
- `tool!` macro for ergonomic tool definition
- Builder pattern for all configuration types
- Additional error variants for better error handling

### API Differences (Idiomatic Rust)

| Python | Rust | Reason |
|--------|------|--------|
| `async with client:` | `connect()/disconnect()` | No context managers in Rust |
| `Optional[T]` | `Option<T>` | Rust idiom |
| `list[T]` | `Vec<T>` | Rust idiom |
| `dict[K,V]` | `HashMap<K,V>` | Rust idiom |
| Dataclasses | Structs with TypedBuilder | Rust idiom |
| `Hooks.add()` | `Hooks.add_*()` methods | Type safety |

---

## 13. Missing Features - Detailed Specifications

### 13.1 File Checkpointing & Rewind (HIGH PRIORITY)

**Python Implementation:**
```python
# types.py:680
enable_file_checkpointing: bool = False

# client.py:264-294
async def rewind_files(self, user_message_id: str) -> None:
    """Rewind tracked files to their state at a specific user message."""
    await self._query.rewind_files(user_message_id)

# types.py:721-724
class SDKControlRewindFilesRequest(TypedDict):
    subtype: Literal["rewind_files"]
    user_message_id: str
```

**Required Rust Implementation:**
```rust
// In ClaudeAgentOptions
pub enable_file_checkpointing: bool,

// In ClaudeClient
pub async fn rewind_files(&self, user_message_id: &str) -> Result<()>;

// Control protocol request
struct RewindFilesRequest {
    subtype: String,  // "rewind_files"
    user_message_id: String,
}
```

### 13.2 StreamEvent & Partial Messages (HIGH PRIORITY)

**Python Implementation:**
```python
# types.py:604-610
@dataclass
class StreamEvent:
    """Stream event for partial message updates during streaming."""
    uuid: str
    session_id: str
    event: dict[str, Any]  # The raw Anthropic API stream event
    parent_tool_use_id: str | None = None

# types.py:658
include_partial_messages: bool = False
```

**Required Rust Implementation:**
```rust
// New message variant
pub struct StreamEvent {
    pub uuid: String,
    pub session_id: String,
    pub event: serde_json::Value,
    pub parent_tool_use_id: Option<String>,
}

pub enum Message {
    // ... existing variants
    Stream(StreamEvent),
}

// In ClaudeAgentOptions
pub include_partial_messages: bool,
```

### 13.3 UserMessage.uuid Field (HIGH PRIORITY)

**Python Implementation:**
```python
# types.py:561-566
@dataclass
class UserMessage:
    content: str | list[ContentBlock]
    uuid: str | None = None  # <-- NEW FIELD
    parent_tool_use_id: str | None = None
```

**Required Rust Implementation:**
```rust
pub struct UserMessage {
    pub role: String,
    pub content: UserContent,
    pub uuid: Option<String>,  // ADD THIS
}
```

### 13.4 Beta Features (MEDIUM PRIORITY)

**Python Implementation:**
```python
# types.py:20-21
SdkBeta = Literal["context-1m-2025-08-07"]

# types.py:633
betas: list[SdkBeta] = field(default_factory=list)
```

**Required Rust Implementation:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SdkBeta {
    #[serde(rename = "context-1m-2025-08-07")]
    Context1M,
}

// In ClaudeAgentOptions
pub betas: Vec<SdkBeta>,
```

### 13.5 Agent Definitions (MEDIUM PRIORITY)

**Python Implementation:**
```python
# types.py:23-24
SettingSource = Literal["user", "project", "local"]

# types.py:42-49
@dataclass
class AgentDefinition:
    description: str
    prompt: str
    tools: list[str] | None = None
    model: Literal["sonnet", "opus", "haiku", "inherit"] | None = None

# types.py:663-665
agents: dict[str, AgentDefinition] | None = None
setting_sources: list[SettingSource] | None = None
```

**Required Rust Implementation:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SettingSource {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "project")]
    Project,
    #[serde(rename = "local")]
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentModel {
    #[serde(rename = "sonnet")]
    Sonnet,
    #[serde(rename = "opus")]
    Opus,
    #[serde(rename = "haiku")]
    Haiku,
    #[serde(rename = "inherit")]
    Inherit,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct AgentDefinition {
    pub description: String,
    pub prompt: String,
    #[builder(default)]
    pub tools: Option<Vec<String>>,
    #[builder(default)]
    pub model: Option<AgentModel>,
}

// In ClaudeAgentOptions
pub agents: Option<HashMap<String, AgentDefinition>>,
pub setting_sources: Option<Vec<SettingSource>>,
```

### 13.6 Sandbox Settings (MEDIUM PRIORITY)

**Python Implementation:**
```python
# types.py:436-509
class SandboxNetworkConfig(TypedDict, total=False):
    allowUnixSockets: list[str]
    allowAllUnixSockets: bool
    allowLocalBinding: bool
    httpProxyPort: int
    socksProxyPort: int

class SandboxIgnoreViolations(TypedDict, total=False):
    file: list[str]
    network: list[str]

class SandboxSettings(TypedDict, total=False):
    enabled: bool
    autoAllowBashIfSandboxed: bool
    excludedCommands: list[str]
    allowUnsandboxedCommands: bool
    network: SandboxNetworkConfig
    ignoreViolations: SandboxIgnoreViolations
    enableWeakerNestedSandbox: bool
```

**Required Rust Implementation:**
```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize, TypedBuilder)]
#[builder(doc)]
pub struct SandboxNetworkConfig {
    #[serde(skip_serializing_if = "Option::is_none", rename = "allowUnixSockets")]
    #[builder(default, setter(strip_option))]
    pub allow_unix_sockets: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "allowAllUnixSockets")]
    #[builder(default, setter(strip_option))]
    pub allow_all_unix_sockets: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "allowLocalBinding")]
    #[builder(default, setter(strip_option))]
    pub allow_local_binding: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "httpProxyPort")]
    #[builder(default, setter(strip_option))]
    pub http_proxy_port: Option<u16>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "socksProxyPort")]
    #[builder(default, setter(strip_option))]
    pub socks_proxy_port: Option<u16>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TypedBuilder)]
#[builder(doc)]
pub struct SandboxIgnoreViolations {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub file: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub network: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TypedBuilder)]
#[builder(doc)]
pub struct SandboxSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "autoAllowBashIfSandboxed")]
    #[builder(default, setter(strip_option))]
    pub auto_allow_bash_if_sandboxed: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "excludedCommands")]
    #[builder(default, setter(strip_option))]
    pub excluded_commands: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "allowUnsandboxedCommands")]
    #[builder(default, setter(strip_option))]
    pub allow_unsandboxed_commands: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub network: Option<SandboxNetworkConfig>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "ignoreViolations")]
    #[builder(default, setter(strip_option))]
    pub ignore_violations: Option<SandboxIgnoreViolations>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "enableWeakerNestedSandbox")]
    #[builder(default, setter(strip_option))]
    pub enable_weaker_nested_sandbox: Option<bool>,
}
```

### 13.7 Additional Options (LOW PRIORITY)

**Python Implementation:**
```python
# types.py
tools: list[str] | ToolsPreset | None = None
permission_prompt_tool_name: str | None = None
settings: str | None = None
add_dirs: list[str | Path] = field(default_factory=list)
user: str | None = None
```

**Required Rust Implementation:**
```rust
// ToolsPreset type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsPreset {
    #[serde(rename = "type")]
    pub type_: String,  // "preset"
    pub preset: String,  // "claude_code"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Tools {
    List(Vec<String>),
    Preset(ToolsPreset),
}

// In ClaudeAgentOptions
pub tools: Option<Tools>,
pub permission_prompt_tool_name: Option<String>,
pub settings: Option<String>,
pub add_dirs: Vec<PathBuf>,
pub user: Option<String>,
```

---

## Appendix A: File Structure Comparison

### Python SDK
```
src/claude_agent_sdk/
├── __init__.py
├── _errors.py
├── client.py
├── query.py
├── types.py
└── _internal/
    ├── client.py
    ├── query.py
    ├── message_parser.py
    └── transport/
        └── subprocess_cli.py
```

### Rust SDK
```
src/
├── lib.rs
├── client.rs
├── query.rs
├── errors.rs
├── version.rs
├── types/
│   ├── mod.rs
│   ├── config.rs
│   ├── messages.rs
│   ├── hooks.rs
│   ├── permissions.rs
│   ├── mcp.rs
│   └── plugin.rs
└── internal/
    ├── mod.rs
    ├── client.rs
    ├── query_full.rs
    ├── message_parser.rs
    └── transport/
        ├── mod.rs
        └── subprocess.rs
```

---

## Appendix B: Version Information

| Component | Python | Rust |
|-----------|--------|------|
| SDK Version | `0.0.1` | `0.2.3` |
| Min CLI Version | `1.0.41` | `1.0.41` |
| Entrypoint | `claude-agent-sdk` | `claude-agent-sdk-rs` |
