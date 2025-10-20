//! Hook types for Claude Agent SDK

use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Hook events that can be intercepted
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookEvent {
    /// Before tool use
    PreToolUse,
    /// After tool use
    PostToolUse,
    /// When user prompt is submitted
    UserPromptSubmit,
    /// When execution stops
    Stop,
    /// When subagent stops
    SubagentStop,
    /// Before compacting conversation
    PreCompact,
}

/// Hook matcher for pattern-based hook registration
#[derive(Clone)]
pub struct HookMatcher {
    /// Optional matcher pattern (e.g., tool name)
    pub matcher: Option<String>,
    /// Hook callbacks to invoke
    pub hooks: Vec<HookCallback>,
}

/// Hook callback type
pub type HookCallback = Arc<
    dyn Fn(HookInput, Option<String>, HookContext) -> BoxFuture<'static, HookJsonOutput>
        + Send
        + Sync,
>;

/// Input to hook callbacks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "hook_event_name", rename_all = "PascalCase")]
pub enum HookInput {
    /// Pre-tool-use hook input
    PreToolUse(PreToolUseHookInput),
    /// Post-tool-use hook input
    PostToolUse(PostToolUseHookInput),
    /// User-prompt-submit hook input
    UserPromptSubmit(UserPromptSubmitHookInput),
    /// Stop hook input
    Stop(StopHookInput),
    /// Subagent-stop hook input
    SubagentStop(SubagentStopHookInput),
    /// Pre-compact hook input
    PreCompact(PreCompactHookInput),
}

/// Pre-tool-use hook input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreToolUseHookInput {
    /// Session ID
    pub session_id: String,
    /// Transcript path
    pub transcript_path: String,
    /// Current working directory
    pub cwd: String,
    /// Permission mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_mode: Option<String>,
    /// Tool name being used
    pub tool_name: String,
    /// Tool input parameters
    pub tool_input: serde_json::Value,
}

/// Post-tool-use hook input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostToolUseHookInput {
    /// Session ID
    pub session_id: String,
    /// Transcript path
    pub transcript_path: String,
    /// Current working directory
    pub cwd: String,
    /// Permission mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_mode: Option<String>,
    /// Tool name that was used
    pub tool_name: String,
    /// Tool input parameters
    pub tool_input: serde_json::Value,
    /// Tool output
    pub tool_output: serde_json::Value,
}

/// User-prompt-submit hook input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPromptSubmitHookInput {
    /// Session ID
    pub session_id: String,
    /// Transcript path
    pub transcript_path: String,
    /// Current working directory
    pub cwd: String,
    /// Permission mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_mode: Option<String>,
    /// User prompt
    pub prompt: String,
}

/// Stop hook input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopHookInput {
    /// Session ID
    pub session_id: String,
    /// Transcript path
    pub transcript_path: String,
    /// Current working directory
    pub cwd: String,
    /// Permission mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_mode: Option<String>,
    /// Stop reason
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
}

/// Subagent-stop hook input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentStopHookInput {
    /// Session ID
    pub session_id: String,
    /// Transcript path
    pub transcript_path: String,
    /// Current working directory
    pub cwd: String,
    /// Permission mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_mode: Option<String>,
    /// Subagent name
    pub subagent_name: String,
    /// Stop reason
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
}

/// Pre-compact hook input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreCompactHookInput {
    /// Session ID
    pub session_id: String,
    /// Transcript path
    pub transcript_path: String,
    /// Current working directory
    pub cwd: String,
    /// Permission mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_mode: Option<String>,
}

/// Hook context passed to callbacks
#[derive(Debug, Clone, Default)]
pub struct HookContext {
    /// Abort signal (future feature)
    pub signal: Option<()>,
}

/// Hook output (can be async or sync)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HookJsonOutput {
    /// Async hook output (returns immediately, hook continues in background)
    Async(AsyncHookJsonOutput),
    /// Sync hook output (blocks until hook completes)
    Sync(SyncHookJsonOutput),
}

/// Async hook output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncHookJsonOutput {
    /// Async timeout in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub async_timeout: Option<u64>,
}

/// Sync hook output
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncHookJsonOutput {
    /// Whether to continue execution
    #[serde(skip_serializing_if = "Option::is_none", rename = "continue")]
    pub continue_: Option<bool>,
    /// Whether to suppress output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppress_output: Option<bool>,
    /// Stop reason (if stopping)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    /// Permission decision
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<String>,
    /// System message to add
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_message: Option<String>,
    /// Reason for decision
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Hook-specific output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook_specific_output: Option<HookSpecificOutput>,
}

/// Hook-specific output for different hook types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HookSpecificOutput {
    /// Pre-tool-use specific output
    PreToolUse(PreToolUseHookSpecificOutput),
    /// Post-tool-use specific output
    PostToolUse(PostToolUseHookSpecificOutput),
    /// User-prompt-submit specific output
    UserPromptSubmit(UserPromptSubmitHookSpecificOutput),
}

/// Pre-tool-use hook specific output
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PreToolUseHookSpecificOutput {
    /// Permission decision (allow/deny)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_decision: Option<String>,
    /// Reason for permission decision
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_decision_reason: Option<String>,
    /// Updated tool input
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_input: Option<serde_json::Value>,
}

/// Post-tool-use hook specific output
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PostToolUseHookSpecificOutput {
    /// Updated tool output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_output: Option<serde_json::Value>,
}

/// User-prompt-submit hook specific output
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserPromptSubmitHookSpecificOutput {
    /// Updated prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_prompt: Option<String>,
}
