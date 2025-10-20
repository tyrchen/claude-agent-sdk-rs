//! Configuration types for Claude Agent SDK

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use super::hooks::{HookEvent, HookMatcher};
use super::mcp::McpServers;
use super::permissions::CanUseToolCallback;

/// Main configuration options for Claude Agent
#[derive(Clone, Default)]
pub struct ClaudeAgentOptions {
    /// List of allowed tool names
    pub allowed_tools: Vec<String>,
    /// System prompt configuration
    pub system_prompt: Option<SystemPrompt>,
    /// MCP server configuration
    pub mcp_servers: McpServers,
    /// Permission mode
    pub permission_mode: Option<PermissionMode>,
    /// Whether to continue the conversation
    pub continue_conversation: bool,
    /// Session ID to resume
    pub resume: Option<String>,
    /// Maximum number of turns
    pub max_turns: Option<u32>,
    /// List of disallowed tool names
    pub disallowed_tools: Vec<String>,
    /// Model to use
    pub model: Option<String>,
    /// Tool name for permission prompts
    pub permission_prompt_tool_name: Option<String>,
    /// Working directory
    pub cwd: Option<PathBuf>,
    /// Path to Claude CLI
    pub cli_path: Option<PathBuf>,
    /// Settings file path
    pub settings: Option<String>,
    /// Additional directories to include
    pub add_dirs: Vec<PathBuf>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Extra CLI arguments
    pub extra_args: HashMap<String, Option<String>>,
    /// Maximum buffer size for subprocess output
    pub max_buffer_size: Option<usize>,
    /// Callback for stderr output
    pub stderr_callback: Option<Arc<dyn Fn(String) + Send + Sync>>,
    /// Callback for tool usage permission
    pub can_use_tool: Option<CanUseToolCallback>,
    /// Hook callbacks
    pub hooks: Option<HashMap<HookEvent, Vec<HookMatcher>>>,
    /// User identifier
    pub user: Option<String>,
    /// Whether to include partial messages in stream
    pub include_partial_messages: bool,
    /// Whether to fork the session
    pub fork_session: bool,
    /// Custom agent definitions
    pub agents: Option<HashMap<String, AgentDefinition>>,
    /// Setting sources to use
    pub setting_sources: Option<Vec<SettingSource>>,
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
    /// Preset name (e.g., "claude_code")
    pub preset: String,
    /// Text to append to the preset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub append: Option<String>,
}

/// Permission mode for tool execution
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PermissionMode {
    /// Default permission mode
    Default,
    /// Accept edits automatically
    AcceptEdits,
    /// Plan mode
    Plan,
    /// Bypass all permissions
    BypassPermissions,
}

/// Setting source location
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SettingSource {
    /// User settings
    User,
    /// Project settings
    Project,
    /// Local settings
    Local,
}

/// Custom agent definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    /// Agent description
    pub description: String,
    /// Agent prompt
    pub prompt: String,
    /// Tools available to the agent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,
    /// Model to use for the agent
    #[serde(skip_serializing_if = "Option::is_none")]
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

impl ClaudeAgentOptions {
    /// Create a new builder for ClaudeAgentOptions
    pub fn builder() -> ClaudeAgentOptionsBuilder {
        ClaudeAgentOptionsBuilder::default()
    }
}

/// Builder for ClaudeAgentOptions
#[derive(Default)]
pub struct ClaudeAgentOptionsBuilder {
    options: ClaudeAgentOptions,
}

impl ClaudeAgentOptionsBuilder {
    /// Set allowed tools
    pub fn allowed_tools(mut self, tools: Vec<String>) -> Self {
        self.options.allowed_tools = tools;
        self
    }

    /// Set system prompt
    pub fn system_prompt(mut self, prompt: impl Into<SystemPrompt>) -> Self {
        self.options.system_prompt = Some(prompt.into());
        self
    }

    /// Set MCP servers
    pub fn mcp_servers(mut self, servers: McpServers) -> Self {
        self.options.mcp_servers = servers;
        self
    }

    /// Set permission mode
    pub fn permission_mode(mut self, mode: PermissionMode) -> Self {
        self.options.permission_mode = Some(mode);
        self
    }

    /// Set whether to continue conversation
    pub fn continue_conversation(mut self, continue_: bool) -> Self {
        self.options.continue_conversation = continue_;
        self
    }

    /// Set session ID to resume
    pub fn resume(mut self, session_id: String) -> Self {
        self.options.resume = Some(session_id);
        self
    }

    /// Set maximum number of turns
    pub fn max_turns(mut self, max: u32) -> Self {
        self.options.max_turns = Some(max);
        self
    }

    /// Set model
    pub fn model(mut self, model: String) -> Self {
        self.options.model = Some(model);
        self
    }

    /// Set working directory
    pub fn cwd(mut self, cwd: PathBuf) -> Self {
        self.options.cwd = Some(cwd);
        self
    }

    /// Set CLI path
    pub fn cli_path(mut self, path: PathBuf) -> Self {
        self.options.cli_path = Some(path);
        self
    }

    /// Set can_use_tool callback
    pub fn can_use_tool(mut self, callback: CanUseToolCallback) -> Self {
        self.options.can_use_tool = Some(callback);
        self
    }

    /// Set hooks
    pub fn hooks(mut self, hooks: HashMap<HookEvent, Vec<HookMatcher>>) -> Self {
        self.options.hooks = Some(hooks);
        self
    }

    /// Build the options
    pub fn build(self) -> ClaudeAgentOptions {
        self.options
    }
}
