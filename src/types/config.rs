//! Configuration types for Claude Agent SDK

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use typed_builder::TypedBuilder;

use super::hooks::{HookEvent, HookMatcher};
use super::mcp::McpServers;
use super::permissions::CanUseToolCallback;
use super::plugin::SdkPluginConfig;

/// Main configuration options for Claude Agent
#[derive(Clone, TypedBuilder)]
#[builder(doc)]
pub struct ClaudeAgentOptions {
    /// List of allowed tool names
    #[builder(default, setter(into))]
    pub allowed_tools: Vec<String>,
    /// System prompt configuration
    #[builder(default, setter(into, strip_option))]
    pub system_prompt: Option<SystemPrompt>,
    /// MCP server configuration
    #[builder(default)]
    pub mcp_servers: McpServers,
    /// Permission mode
    #[builder(default, setter(strip_option))]
    pub permission_mode: Option<PermissionMode>,
    /// Whether to continue the conversation
    #[builder(default = false)]
    pub continue_conversation: bool,
    /// Session ID to resume
    #[builder(default, setter(into, strip_option))]
    pub resume: Option<String>,
    /// Maximum number of turns
    #[builder(default, setter(strip_option))]
    pub max_turns: Option<u32>,
    /// List of disallowed tool names
    #[builder(default, setter(into))]
    pub disallowed_tools: Vec<String>,
    /// Model to use
    #[builder(default, setter(into, strip_option))]
    pub model: Option<String>,
    /// Fallback model to use if primary model fails
    #[builder(default, setter(into, strip_option))]
    pub fallback_model: Option<String>,
    /// Maximum budget in USD for the conversation
    #[builder(default, setter(strip_option))]
    pub max_budget_usd: Option<f64>,
    /// Maximum tokens for thinking blocks
    #[builder(default, setter(strip_option))]
    pub max_thinking_tokens: Option<u32>,
    /// Tool name for permission prompts
    #[builder(default, setter(into, strip_option))]
    pub permission_prompt_tool_name: Option<String>,
    /// Working directory
    #[builder(default, setter(into, strip_option))]
    pub cwd: Option<PathBuf>,
    /// Path to Claude CLI
    #[builder(default, setter(into, strip_option))]
    pub cli_path: Option<PathBuf>,
    /// Settings file path
    #[builder(default, setter(into, strip_option))]
    pub settings: Option<String>,
    /// Additional directories to include
    #[builder(default, setter(into))]
    pub add_dirs: Vec<PathBuf>,
    /// Environment variables
    #[builder(default)]
    pub env: HashMap<String, String>,
    /// Extra CLI arguments
    #[builder(default)]
    pub extra_args: HashMap<String, Option<String>>,
    /// Maximum buffer size for subprocess output
    #[builder(default, setter(strip_option))]
    pub max_buffer_size: Option<usize>,
    /// Callback for stderr output
    #[builder(default, setter(strip_option))]
    pub stderr_callback: Option<Arc<dyn Fn(String) + Send + Sync>>,
    /// Callback for tool usage permission
    #[builder(default, setter(strip_option))]
    pub can_use_tool: Option<CanUseToolCallback>,
    /// Hook callbacks
    #[builder(default, setter(strip_option))]
    pub hooks: Option<HashMap<HookEvent, Vec<HookMatcher>>>,
    /// User identifier
    #[builder(default, setter(into, strip_option))]
    pub user: Option<String>,
    /// Whether to include partial messages in stream
    #[builder(default = false)]
    pub include_partial_messages: bool,
    /// Whether to fork the session
    #[builder(default = false)]
    pub fork_session: bool,
    /// Custom agent definitions
    #[builder(default, setter(strip_option))]
    pub agents: Option<HashMap<String, AgentDefinition>>,
    /// Setting sources to use.
    ///
    /// When `None`, the SDK does **not** load any filesystem settings,
    /// providing isolation for SDK applications.
    ///
    /// Programmatic options (like `agents`, `allowed_tools`) always override filesystem settings.
    #[builder(default, setter(strip_option))]
    pub setting_sources: Option<Vec<SettingSource>>,
    /// Plugin configurations for custom plugins
    #[builder(default, setter(into))]
    pub plugins: Vec<SdkPluginConfig>,
    /// Output format for structured outputs (matches Messages API structure)
    /// Example: `json!({"type": "json_schema", "schema": {"type": "object", "properties": {...}}})`
    #[builder(default, setter(strip_option))]
    pub output_format: Option<serde_json::Value>,
}

impl Default for ClaudeAgentOptions {
    fn default() -> Self {
        Self::builder().build()
    }
}

/// System prompt configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SystemPrompt {
    /// Direct text prompt
    Text(String),
    /// Preset-based prompt
    Preset(SystemPromptPreset),
}

impl From<String> for SystemPrompt {
    fn from(text: String) -> Self {
        SystemPrompt::Text(text)
    }
}

impl From<&str> for SystemPrompt {
    fn from(text: &str) -> Self {
        SystemPrompt::Text(text.to_string())
    }
}

/// System prompt preset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPromptPreset {
    /// Type field (always "preset")
    #[serde(rename = "type")]
    pub type_: String,
    /// Preset name (e.g., "claude_code")
    pub preset: String,
    /// Text to append to the preset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub append: Option<String>,
}

impl SystemPromptPreset {
    /// Create a new preset with the given name
    pub fn new(preset: impl Into<String>) -> Self {
        Self {
            type_: "preset".to_string(),
            preset: preset.into(),
            append: None,
        }
    }

    /// Create a preset with appended text
    pub fn with_append(preset: impl Into<String>, append: impl Into<String>) -> Self {
        Self {
            type_: "preset".to_string(),
            preset: preset.into(),
            append: Some(append.into()),
        }
    }
}

/// Permission mode for tool execution
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PermissionMode {
    /// Default permission mode
    #[serde(rename = "default")]
    Default,
    /// Accept edits automatically
    AcceptEdits,
    /// Plan mode
    #[serde(rename = "plan")]
    Plan,
    /// Bypass all permissions
    BypassPermissions,
}

/// Controls which filesystem-based configuration sources the SDK loads settings from.
///
/// When multiple sources are loaded, settings are merged with this precedence (highest to lowest):
/// 1. `Local`
/// 2. `Project`
/// 3. `User`
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SettingSource {
    /// User settings from `~/.claude/settings.json`
    User,
    /// Project settings from `.claude/settings.json` (team-shared settings)
    Project,
    /// Local settings from `.claude/settings.local.json` (highest priority, git-ignored)
    Local,
}

/// Custom agent definition
#[derive(Debug, Clone, Serialize, Deserialize, TypedBuilder)]
#[builder(doc)]
pub struct AgentDefinition {
    /// Agent description
    #[builder(setter(into))]
    pub description: String,
    /// Agent prompt
    #[builder(setter(into))]
    pub prompt: String,
    /// Tools available to the agent
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(into, strip_option))]
    pub tools: Option<Vec<String>>,
    /// Model to use for the agent
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub model: Option<AgentModel>,
}

/// Model selection for agents
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AgentModel {
    /// Claude Sonnet
    Sonnet,
    /// Claude Opus
    Opus,
    /// Claude Haiku
    Haiku,
    /// Inherit from parent
    Inherit,
}
