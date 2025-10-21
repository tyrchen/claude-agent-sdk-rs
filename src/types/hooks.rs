//! Hook types for Claude Agent SDK

use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Hook events that can be intercepted
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    /// Tool response (output from the tool)
    pub tool_response: serde_json::Value,
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
    /// Whether stop hook is active
    pub stop_hook_active: bool,
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
    /// Whether stop hook is active
    pub stop_hook_active: bool,
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
    /// Trigger type (manual or auto)
    pub trigger: String,
    /// Custom instructions for compaction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_instructions: Option<String>,
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
    /// Async field (always true to indicate async mode)
    /// Note: In Rust this field is named `async_` to avoid keyword conflict,
    /// but it serializes to "async" for the CLI
    #[serde(rename = "async")]
    pub async_: bool,
    /// Async timeout in milliseconds
    #[serde(skip_serializing_if = "Option::is_none", rename = "asyncTimeout")]
    pub async_timeout: Option<u64>,
}

impl Default for AsyncHookJsonOutput {
    fn default() -> Self {
        Self {
            async_: true, // Always true for async hooks
            async_timeout: None,
        }
    }
}

/// Sync hook output
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncHookJsonOutput {
    /// Whether to continue execution
    #[serde(skip_serializing_if = "Option::is_none", rename = "continue")]
    pub continue_: Option<bool>,
    /// Whether to suppress output
    #[serde(skip_serializing_if = "Option::is_none", rename = "suppressOutput")]
    pub suppress_output: Option<bool>,
    /// Stop reason (if stopping)
    #[serde(skip_serializing_if = "Option::is_none", rename = "stopReason")]
    pub stop_reason: Option<String>,
    /// Permission decision
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<String>,
    /// System message to add
    #[serde(skip_serializing_if = "Option::is_none", rename = "systemMessage")]
    pub system_message: Option<String>,
    /// Reason for decision
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Hook-specific output
    #[serde(skip_serializing_if = "Option::is_none", rename = "hookSpecificOutput")]
    pub hook_specific_output: Option<HookSpecificOutput>,
}

/// Hook-specific output for different hook types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "hookEventName")]
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
    /// Permission decision (allow/deny/ask)
    #[serde(skip_serializing_if = "Option::is_none", rename = "permissionDecision")]
    pub permission_decision: Option<String>,
    /// Reason for permission decision
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "permissionDecisionReason"
    )]
    pub permission_decision_reason: Option<String>,
    /// Updated tool input
    #[serde(skip_serializing_if = "Option::is_none", rename = "updatedInput")]
    pub updated_input: Option<serde_json::Value>,
}

/// Post-tool-use hook specific output
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PostToolUseHookSpecificOutput {
    /// Additional context to provide to Claude
    #[serde(skip_serializing_if = "Option::is_none", rename = "additionalContext")]
    pub additional_context: Option<String>,
}

/// User-prompt-submit hook specific output
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserPromptSubmitHookSpecificOutput {
    /// Additional context to provide to Claude
    #[serde(skip_serializing_if = "Option::is_none", rename = "additionalContext")]
    pub additional_context: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_hook_event_serialization() {
        // HookEvent serializes to PascalCase to match Python SDK
        assert_eq!(
            serde_json::to_string(&HookEvent::PreToolUse).unwrap(),
            "\"PreToolUse\""
        );
        assert_eq!(
            serde_json::to_string(&HookEvent::PostToolUse).unwrap(),
            "\"PostToolUse\""
        );
        assert_eq!(
            serde_json::to_string(&HookEvent::UserPromptSubmit).unwrap(),
            "\"UserPromptSubmit\""
        );
        assert_eq!(serde_json::to_string(&HookEvent::Stop).unwrap(), "\"Stop\"");
        assert_eq!(
            serde_json::to_string(&HookEvent::SubagentStop).unwrap(),
            "\"SubagentStop\""
        );
        assert_eq!(
            serde_json::to_string(&HookEvent::PreCompact).unwrap(),
            "\"PreCompact\""
        );
    }

    #[test]
    fn test_pretooluse_hook_input_deserialization() {
        let json_str = r#"{
            "hook_event_name": "PreToolUse",
            "session_id": "test-session",
            "transcript_path": "/path/to/transcript",
            "cwd": "/working/dir",
            "permission_mode": "default",
            "tool_name": "Bash",
            "tool_input": {"command": "echo hello"}
        }"#;

        let input: HookInput = serde_json::from_str(json_str).unwrap();
        match input {
            HookInput::PreToolUse(pre_tool) => {
                assert_eq!(pre_tool.session_id, "test-session");
                assert_eq!(pre_tool.tool_name, "Bash");
                assert_eq!(pre_tool.tool_input["command"], "echo hello");
            }
            _ => panic!("Expected PreToolUse variant"),
        }
    }

    #[test]
    fn test_posttooluse_hook_input_deserialization() {
        let json_str = r#"{
            "hook_event_name": "PostToolUse",
            "session_id": "test-session",
            "transcript_path": "/path/to/transcript",
            "cwd": "/working/dir",
            "tool_name": "Bash",
            "tool_input": {"command": "echo hello"},
            "tool_response": "hello\n"
        }"#;

        let input: HookInput = serde_json::from_str(json_str).unwrap();
        match input {
            HookInput::PostToolUse(post_tool) => {
                assert_eq!(post_tool.session_id, "test-session");
                assert_eq!(post_tool.tool_name, "Bash");
                assert_eq!(post_tool.tool_response, "hello\n");
            }
            _ => panic!("Expected PostToolUse variant"),
        }
    }

    #[test]
    fn test_stop_hook_input_deserialization() {
        let json_str = r#"{
            "hook_event_name": "Stop",
            "session_id": "test-session",
            "transcript_path": "/path/to/transcript",
            "cwd": "/working/dir",
            "stop_hook_active": true
        }"#;

        let input: HookInput = serde_json::from_str(json_str).unwrap();
        match input {
            HookInput::Stop(stop) => {
                assert_eq!(stop.session_id, "test-session");
                assert!(stop.stop_hook_active);
            }
            _ => panic!("Expected Stop variant"),
        }
    }

    #[test]
    fn test_subagent_stop_hook_input_deserialization() {
        let json_str = r#"{
            "hook_event_name": "SubagentStop",
            "session_id": "test-session",
            "transcript_path": "/path/to/transcript",
            "cwd": "/working/dir",
            "stop_hook_active": false
        }"#;

        let input: HookInput = serde_json::from_str(json_str).unwrap();
        match input {
            HookInput::SubagentStop(subagent) => {
                assert_eq!(subagent.session_id, "test-session");
                assert!(!subagent.stop_hook_active);
            }
            _ => panic!("Expected SubagentStop variant"),
        }
    }

    #[test]
    fn test_precompact_hook_input_deserialization() {
        let json_str = r#"{
            "hook_event_name": "PreCompact",
            "session_id": "test-session",
            "transcript_path": "/path/to/transcript",
            "cwd": "/working/dir",
            "trigger": "manual",
            "custom_instructions": "Keep important details"
        }"#;

        let input: HookInput = serde_json::from_str(json_str).unwrap();
        match input {
            HookInput::PreCompact(precompact) => {
                assert_eq!(precompact.session_id, "test-session");
                assert_eq!(precompact.trigger, "manual");
                assert_eq!(
                    precompact.custom_instructions,
                    Some("Keep important details".to_string())
                );
            }
            _ => panic!("Expected PreCompact variant"),
        }
    }

    #[test]
    fn test_sync_hook_output_serialization() {
        let output = SyncHookJsonOutput {
            continue_: Some(false),
            stop_reason: Some("Test stop".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["continue"], false);
        assert_eq!(json["stopReason"], "Test stop");
    }

    #[test]
    fn test_hook_specific_output_pretooluse_serialization() {
        let output = HookSpecificOutput::PreToolUse(PreToolUseHookSpecificOutput {
            permission_decision: Some("deny".to_string()),
            permission_decision_reason: Some("Security policy".to_string()),
            updated_input: None,
        });

        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["hookEventName"], "PreToolUse");
        assert_eq!(json["permissionDecision"], "deny");
        assert_eq!(json["permissionDecisionReason"], "Security policy");
    }

    #[test]
    fn test_hook_specific_output_posttooluse_serialization() {
        let output = HookSpecificOutput::PostToolUse(PostToolUseHookSpecificOutput {
            additional_context: Some("Error occurred".to_string()),
        });

        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["hookEventName"], "PostToolUse");
        assert_eq!(json["additionalContext"], "Error occurred");
    }

    #[test]
    fn test_hook_specific_output_userpromptsubmit_serialization() {
        let output = HookSpecificOutput::UserPromptSubmit(UserPromptSubmitHookSpecificOutput {
            additional_context: Some("Custom context".to_string()),
        });

        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["hookEventName"], "UserPromptSubmit");
        assert_eq!(json["additionalContext"], "Custom context");
    }

    #[test]
    fn test_complete_hook_output_with_pretooluse() {
        let output = SyncHookJsonOutput {
            continue_: Some(true),
            hook_specific_output: Some(HookSpecificOutput::PreToolUse(
                PreToolUseHookSpecificOutput {
                    permission_decision: Some("allow".to_string()),
                    permission_decision_reason: Some("Approved".to_string()),
                    updated_input: Some(json!({"modified": true})),
                },
            )),
            ..Default::default()
        };

        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["continue"], true);
        assert_eq!(json["hookSpecificOutput"]["hookEventName"], "PreToolUse");
        assert_eq!(json["hookSpecificOutput"]["permissionDecision"], "allow");
    }

    #[test]
    fn test_optional_fields_omitted() {
        let output = SyncHookJsonOutput::default();
        let json = serde_json::to_value(&output).unwrap();

        // Default output should be an empty object
        assert!(json.as_object().unwrap().is_empty());
    }

    #[test]
    fn test_async_hook_output_serialization() {
        let output = AsyncHookJsonOutput::default();
        let json = serde_json::to_value(&output).unwrap();

        // Must have "async": true
        assert_eq!(json["async"], true);
        // asyncTimeout should not be present (None)
        assert!(json.get("asyncTimeout").is_none());
    }

    #[test]
    fn test_async_hook_output_with_timeout() {
        let output = AsyncHookJsonOutput {
            async_: true,
            async_timeout: Some(5000),
        };
        let json = serde_json::to_value(&output).unwrap();

        assert_eq!(json["async"], true);
        assert_eq!(json["asyncTimeout"], 5000);
    }
}
