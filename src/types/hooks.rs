//! Hook types for Claude Agent SDK

use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use typed_builder::TypedBuilder;

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
#[derive(Clone, TypedBuilder)]
#[builder(doc)]
pub struct HookMatcher {
    /// Optional matcher pattern (e.g., tool name)
    #[builder(default, setter(into, strip_option))]
    pub matcher: Option<String>,
    /// Hook callbacks to invoke
    #[builder(default)]
    pub hooks: Vec<HookCallback>,
    /// Timeout in seconds for all hooks in this matcher (default: 60)
    #[builder(default, setter(strip_option))]
    pub timeout: Option<f64>,
}

/// Hook callback type
pub type HookCallback = Arc<
    dyn Fn(HookInput, Option<String>, HookContext) -> BoxFuture<'static, HookJsonOutput>
        + Send
        + Sync,
>;

/// Hook function type that users implement
pub type HookFn = fn(HookInput, Option<String>, HookContext) -> BoxFuture<'static, HookJsonOutput>;

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
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
#[builder(doc)]
pub struct SyncHookJsonOutput {
    /// Whether to continue execution
    #[serde(skip_serializing_if = "Option::is_none", rename = "continue")]
    #[builder(default, setter(strip_option))]
    pub continue_: Option<bool>,
    /// Whether to suppress output
    #[serde(skip_serializing_if = "Option::is_none", rename = "suppressOutput")]
    #[builder(default, setter(strip_option))]
    pub suppress_output: Option<bool>,
    /// Stop reason (if stopping)
    #[serde(skip_serializing_if = "Option::is_none", rename = "stopReason")]
    #[builder(default, setter(into, strip_option))]
    pub stop_reason: Option<String>,
    /// Permission decision
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub decision: Option<String>,
    /// System message to add
    #[serde(skip_serializing_if = "Option::is_none", rename = "systemMessage")]
    #[builder(default, setter(into, strip_option))]
    pub system_message: Option<String>,
    /// Reason for decision
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub reason: Option<String>,
    /// Hook-specific output
    #[serde(skip_serializing_if = "Option::is_none", rename = "hookSpecificOutput")]
    #[builder(default, setter(strip_option))]
    pub hook_specific_output: Option<HookSpecificOutput>,
}

impl Default for SyncHookJsonOutput {
    fn default() -> Self {
        Self::builder().build()
    }
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
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
#[builder(doc)]
pub struct PreToolUseHookSpecificOutput {
    /// Permission decision (allow/deny/ask)
    #[serde(skip_serializing_if = "Option::is_none", rename = "permissionDecision")]
    #[builder(default, setter(into, strip_option))]
    pub permission_decision: Option<String>,
    /// Reason for permission decision
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "permissionDecisionReason"
    )]
    #[builder(default, setter(into, strip_option))]
    pub permission_decision_reason: Option<String>,
    /// Updated tool input
    #[serde(skip_serializing_if = "Option::is_none", rename = "updatedInput")]
    #[builder(default, setter(strip_option))]
    pub updated_input: Option<serde_json::Value>,
}

impl Default for PreToolUseHookSpecificOutput {
    fn default() -> Self {
        Self::builder().build()
    }
}

/// Post-tool-use hook specific output
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
#[builder(doc)]
pub struct PostToolUseHookSpecificOutput {
    /// Additional context to provide to Claude
    #[serde(skip_serializing_if = "Option::is_none", rename = "additionalContext")]
    #[builder(default, setter(into, strip_option))]
    pub additional_context: Option<String>,
}

impl Default for PostToolUseHookSpecificOutput {
    fn default() -> Self {
        Self::builder().build()
    }
}

/// User-prompt-submit hook specific output
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
#[builder(doc)]
pub struct UserPromptSubmitHookSpecificOutput {
    /// Additional context to provide to Claude
    #[serde(skip_serializing_if = "Option::is_none", rename = "additionalContext")]
    #[builder(default, setter(into, strip_option))]
    pub additional_context: Option<String>,
}

impl Default for UserPromptSubmitHookSpecificOutput {
    fn default() -> Self {
        Self::builder().build()
    }
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

    #[test]
    fn test_hooks_builder_new() {
        let hooks = Hooks::new();
        let built = hooks.build();
        assert!(built.is_empty());
    }

    #[test]
    fn test_hooks_builder_add_pre_tool_use() {
        async fn test_hook(
            _input: HookInput,
            _tool_use_id: Option<String>,
            _context: HookContext,
        ) -> HookJsonOutput {
            HookJsonOutput::Sync(SyncHookJsonOutput::default())
        }

        let mut hooks = Hooks::new();
        hooks.add_pre_tool_use(test_hook);

        let built = hooks.build();
        assert_eq!(built.len(), 1);
        assert!(built.contains_key(&HookEvent::PreToolUse));

        let matchers = &built[&HookEvent::PreToolUse];
        assert_eq!(matchers.len(), 1);
        assert_eq!(matchers[0].matcher, None);
        assert_eq!(matchers[0].hooks.len(), 1);
    }

    #[test]
    fn test_hooks_builder_add_pre_tool_use_with_matcher() {
        async fn test_hook(
            _input: HookInput,
            _tool_use_id: Option<String>,
            _context: HookContext,
        ) -> HookJsonOutput {
            HookJsonOutput::Sync(SyncHookJsonOutput::default())
        }

        let mut hooks = Hooks::new();
        hooks.add_pre_tool_use_with_matcher("Bash", test_hook);

        let built = hooks.build();
        assert_eq!(built.len(), 1);
        assert!(built.contains_key(&HookEvent::PreToolUse));

        let matchers = &built[&HookEvent::PreToolUse];
        assert_eq!(matchers.len(), 1);
        assert_eq!(matchers[0].matcher, Some("Bash".to_string()));
        assert_eq!(matchers[0].hooks.len(), 1);
    }

    #[test]
    fn test_hooks_builder_multiple_hooks_same_event() {
        async fn test_hook1(
            _input: HookInput,
            _tool_use_id: Option<String>,
            _context: HookContext,
        ) -> HookJsonOutput {
            HookJsonOutput::Sync(SyncHookJsonOutput::default())
        }

        async fn test_hook2(
            _input: HookInput,
            _tool_use_id: Option<String>,
            _context: HookContext,
        ) -> HookJsonOutput {
            HookJsonOutput::Sync(SyncHookJsonOutput::default())
        }

        let mut hooks = Hooks::new();
        hooks.add_pre_tool_use(test_hook1);
        hooks.add_pre_tool_use_with_matcher("Bash", test_hook2);

        let built = hooks.build();
        assert_eq!(built.len(), 1);
        assert!(built.contains_key(&HookEvent::PreToolUse));

        let matchers = &built[&HookEvent::PreToolUse];
        assert_eq!(matchers.len(), 2);
        assert_eq!(matchers[0].matcher, None);
        assert_eq!(matchers[1].matcher, Some("Bash".to_string()));
    }

    #[test]
    fn test_hooks_builder_add_post_tool_use() {
        async fn test_hook(
            _input: HookInput,
            _tool_use_id: Option<String>,
            _context: HookContext,
        ) -> HookJsonOutput {
            HookJsonOutput::Sync(SyncHookJsonOutput::default())
        }

        let mut hooks = Hooks::new();
        hooks.add_post_tool_use(test_hook);

        let built = hooks.build();
        assert!(built.contains_key(&HookEvent::PostToolUse));
        assert_eq!(built[&HookEvent::PostToolUse][0].matcher, None);
    }

    #[test]
    fn test_hooks_builder_add_post_tool_use_with_matcher() {
        async fn test_hook(
            _input: HookInput,
            _tool_use_id: Option<String>,
            _context: HookContext,
        ) -> HookJsonOutput {
            HookJsonOutput::Sync(SyncHookJsonOutput::default())
        }

        let mut hooks = Hooks::new();
        hooks.add_post_tool_use_with_matcher("Write", test_hook);

        let built = hooks.build();
        assert!(built.contains_key(&HookEvent::PostToolUse));
        assert_eq!(
            built[&HookEvent::PostToolUse][0].matcher,
            Some("Write".to_string())
        );
    }

    #[test]
    fn test_hooks_builder_add_user_prompt_submit() {
        async fn test_hook(
            _input: HookInput,
            _tool_use_id: Option<String>,
            _context: HookContext,
        ) -> HookJsonOutput {
            HookJsonOutput::Sync(SyncHookJsonOutput::default())
        }

        let mut hooks = Hooks::new();
        hooks.add_user_prompt_submit(test_hook);

        let built = hooks.build();
        assert!(built.contains_key(&HookEvent::UserPromptSubmit));
        assert_eq!(built[&HookEvent::UserPromptSubmit][0].matcher, None);
    }

    #[test]
    fn test_hooks_builder_add_stop() {
        async fn test_hook(
            _input: HookInput,
            _tool_use_id: Option<String>,
            _context: HookContext,
        ) -> HookJsonOutput {
            HookJsonOutput::Sync(SyncHookJsonOutput::default())
        }

        let mut hooks = Hooks::new();
        hooks.add_stop(test_hook);

        let built = hooks.build();
        assert!(built.contains_key(&HookEvent::Stop));
        assert_eq!(built[&HookEvent::Stop][0].matcher, None);
    }

    #[test]
    fn test_hooks_builder_add_subagent_stop() {
        async fn test_hook(
            _input: HookInput,
            _tool_use_id: Option<String>,
            _context: HookContext,
        ) -> HookJsonOutput {
            HookJsonOutput::Sync(SyncHookJsonOutput::default())
        }

        let mut hooks = Hooks::new();
        hooks.add_subagent_stop(test_hook);

        let built = hooks.build();
        assert!(built.contains_key(&HookEvent::SubagentStop));
        assert_eq!(built[&HookEvent::SubagentStop][0].matcher, None);
    }

    #[test]
    fn test_hooks_builder_add_pre_compact() {
        async fn test_hook(
            _input: HookInput,
            _tool_use_id: Option<String>,
            _context: HookContext,
        ) -> HookJsonOutput {
            HookJsonOutput::Sync(SyncHookJsonOutput::default())
        }

        let mut hooks = Hooks::new();
        hooks.add_pre_compact(test_hook);

        let built = hooks.build();
        assert!(built.contains_key(&HookEvent::PreCompact));
        assert_eq!(built[&HookEvent::PreCompact][0].matcher, None);
    }

    #[test]
    fn test_hooks_builder_multiple_event_types() {
        async fn test_hook(
            _input: HookInput,
            _tool_use_id: Option<String>,
            _context: HookContext,
        ) -> HookJsonOutput {
            HookJsonOutput::Sync(SyncHookJsonOutput::default())
        }

        let mut hooks = Hooks::new();
        hooks.add_pre_tool_use(test_hook);
        hooks.add_post_tool_use(test_hook);
        hooks.add_user_prompt_submit(test_hook);
        hooks.add_stop(test_hook);

        let built = hooks.build();
        assert_eq!(built.len(), 4);
        assert!(built.contains_key(&HookEvent::PreToolUse));
        assert!(built.contains_key(&HookEvent::PostToolUse));
        assert!(built.contains_key(&HookEvent::UserPromptSubmit));
        assert!(built.contains_key(&HookEvent::Stop));
    }

    #[tokio::test]
    async fn test_hook_execution_returns_sync_output() {
        async fn test_hook(
            _input: HookInput,
            _tool_use_id: Option<String>,
            _context: HookContext,
        ) -> HookJsonOutput {
            HookJsonOutput::Sync(SyncHookJsonOutput {
                continue_: Some(true),
                ..Default::default()
            })
        }

        let mut hooks = Hooks::new();
        hooks.add_pre_tool_use(test_hook);

        let built = hooks.build();
        let hook_callback = &built[&HookEvent::PreToolUse][0].hooks[0];

        let input = HookInput::PreToolUse(PreToolUseHookInput {
            session_id: "test".to_string(),
            transcript_path: "/tmp/test".to_string(),
            cwd: "/tmp".to_string(),
            permission_mode: None,
            tool_name: "Bash".to_string(),
            tool_input: serde_json::json!({"command": "ls"}),
        });

        let result = hook_callback(input, None, HookContext::default()).await;
        match result {
            HookJsonOutput::Sync(output) => {
                assert_eq!(output.continue_, Some(true));
            }
            _ => panic!("Expected sync output"),
        }
    }

    #[tokio::test]
    async fn test_hook_execution_returns_async_output() {
        async fn test_hook(
            _input: HookInput,
            _tool_use_id: Option<String>,
            _context: HookContext,
        ) -> HookJsonOutput {
            HookJsonOutput::Async(AsyncHookJsonOutput {
                async_: true,
                async_timeout: Some(5000),
            })
        }

        let mut hooks = Hooks::new();
        hooks.add_pre_tool_use(test_hook);

        let built = hooks.build();
        let hook_callback = &built[&HookEvent::PreToolUse][0].hooks[0];

        let input = HookInput::PreToolUse(PreToolUseHookInput {
            session_id: "test".to_string(),
            transcript_path: "/tmp/test".to_string(),
            cwd: "/tmp".to_string(),
            permission_mode: None,
            tool_name: "Bash".to_string(),
            tool_input: serde_json::json!({"command": "ls"}),
        });

        let result = hook_callback(input, None, HookContext::default()).await;
        match result {
            HookJsonOutput::Async(output) => {
                assert!(output.async_);
                assert_eq!(output.async_timeout, Some(5000));
            }
            _ => panic!("Expected async output"),
        }
    }

    #[test]
    fn test_hooks_builder_matcher_accepts_string_types() {
        async fn test_hook(
            _input: HookInput,
            _tool_use_id: Option<String>,
            _context: HookContext,
        ) -> HookJsonOutput {
            HookJsonOutput::Sync(SyncHookJsonOutput::default())
        }

        let mut hooks = Hooks::new();

        // Test with &str
        hooks.add_pre_tool_use_with_matcher("Bash", test_hook);

        // Test with String
        hooks.add_pre_tool_use_with_matcher("Write".to_string(), test_hook);

        let built = hooks.build();
        let matchers = &built[&HookEvent::PreToolUse];
        assert_eq!(matchers.len(), 2);
        assert_eq!(matchers[0].matcher, Some("Bash".to_string()));
        assert_eq!(matchers[1].matcher, Some("Write".to_string()));
    }
}

/// Macro to generate hook methods for the Hooks builder
///
/// This macro generates two methods for each hook event:
/// 1. `add_<event>(&mut self, hook_fn)` - For hooks without matcher
/// 2. `add_<event>_with_matcher(&mut self, matcher, hook_fn)` - For hooks with matcher
macro_rules! generate_hook_methods {
    // Entry point - separate with_matcher and without
    (
        with_matcher: {
            $($event_m:ident => $method_name_m:ident: $doc_m:expr),* $(,)?
        },
        without_matcher: {
            $($event:ident => $method_name:ident: $doc:expr),* $(,)?
        } $(,)?
    ) => {
        $(
            generate_hook_methods!(@with_matcher $event_m, $method_name_m, $doc_m);
        )*
        $(
            generate_hook_methods!(@no_matcher $event, $method_name, $doc);
        )*
    };

    // Generate method with matcher support
    (@with_matcher $event:ident, $method_name:ident, $doc:expr) => {
        #[doc = $doc]
        pub fn $method_name<F, Fut>(&mut self, hook_fn: F)
        where
            F: Fn(HookInput, Option<String>, HookContext) -> Fut + Send + Sync + 'static,
            Fut: std::future::Future<Output = HookJsonOutput> + Send + 'static,
        {
            let wrapper = move |input: HookInput, tool_use_id: Option<String>, context: HookContext| {
                Box::pin(hook_fn(input, tool_use_id, context)) as BoxFuture<'static, HookJsonOutput>
            };
            self.add_hook(HookEvent::$event, None::<String>, wrapper);
        }

        paste::paste! {
            #[doc = $doc]
            #[doc = " with a matcher pattern."]
            #[doc = ""]
            #[doc = "# Arguments"]
            #[doc = "* `matcher` - Tool name to match (e.g., \"Bash\", \"Write\")"]
            #[doc = "* `hook_fn` - The hook function to call"]
            pub fn [<$method_name _with_matcher>]<F, Fut>(&mut self, matcher: impl Into<String>, hook_fn: F)
            where
                F: Fn(HookInput, Option<String>, HookContext) -> Fut + Send + Sync + 'static,
                Fut: std::future::Future<Output = HookJsonOutput> + Send + 'static,
            {
                let wrapper = move |input: HookInput, tool_use_id: Option<String>, context: HookContext| {
                    Box::pin(hook_fn(input, tool_use_id, context)) as BoxFuture<'static, HookJsonOutput>
                };
                self.add_hook(HookEvent::$event, Some(matcher), wrapper);
            }
        }
    };

    // Generate method without matcher support
    (@no_matcher $event:ident, $method_name:ident, $doc:expr) => {
        #[doc = $doc]
        pub fn $method_name<F, Fut>(&mut self, hook_fn: F)
        where
            F: Fn(HookInput, Option<String>, HookContext) -> Fut + Send + Sync + 'static,
            Fut: std::future::Future<Output = HookJsonOutput> + Send + 'static,
        {
            let wrapper = move |input: HookInput, tool_use_id: Option<String>, context: HookContext| {
                Box::pin(hook_fn(input, tool_use_id, context)) as BoxFuture<'static, HookJsonOutput>
            };
            self.add_hook(HookEvent::$event, None::<String>, wrapper);
        }
    };
}

/// User-friendly hooks builder
///
/// This provides a convenient API for registering hooks.
///
/// # Example
/// ```no_run
/// use claude_agent_sdk_rs::{Hooks, HookInput, HookContext, HookJsonOutput};
///
/// async fn my_hook(input: HookInput, tool_use_id: Option<String>, context: HookContext) -> HookJsonOutput {
///     HookJsonOutput::Sync(Default::default())
/// }
///
/// let mut hooks = Hooks::new();
/// hooks.add_pre_tool_use(my_hook); // Matches all tools
/// hooks.add_pre_tool_use_with_matcher("Bash", my_hook); // Only Bash tool
/// ```
#[derive(Default)]
pub struct Hooks {
    hooks: HashMap<HookEvent, Vec<HookMatcher>>,
}

impl Hooks {
    /// Create a new empty hooks builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert to the internal HashMap format used by ClaudeAgentOptions
    pub fn build(self) -> HashMap<HookEvent, Vec<HookMatcher>> {
        self.hooks
    }

    /// Add a hook for a specific event and optional matcher (internal method)
    ///
    /// # Arguments
    /// * `event` - The hook event type
    /// * `matcher` - Optional matcher (None for all tools, Some("ToolName") for specific tool)
    /// * `hook_fn` - The hook function to call
    fn add_hook<F>(&mut self, event: HookEvent, matcher: Option<impl Into<String>>, hook_fn: F)
    where
        F: Fn(HookInput, Option<String>, HookContext) -> BoxFuture<'static, HookJsonOutput>
            + Send
            + Sync
            + 'static,
    {
        let matcher_string = matcher.map(|m| m.into());
        let hook_callback = Arc::new(hook_fn);

        self.hooks.entry(event).or_default().push(HookMatcher {
            matcher: matcher_string,
            hooks: vec![hook_callback],
            timeout: None,
        });
    }

    // Generate all hook methods
    generate_hook_methods! {
        with_matcher: {
            PreToolUse => add_pre_tool_use: "Add a PreToolUse hook that fires before tool execution.",
            PostToolUse => add_post_tool_use: "Add a PostToolUse hook that fires after tool execution.",
        },
        without_matcher: {
            UserPromptSubmit => add_user_prompt_submit: "Add a UserPromptSubmit hook that fires when user submits a prompt.",
            Stop => add_stop: "Add a Stop hook that fires when execution stops.",
            SubagentStop => add_subagent_stop: "Add a SubagentStop hook that fires when a subagent stops.",
            PreCompact => add_pre_compact: "Add a PreCompact hook that fires before conversation compaction.",
        },
    }
}
