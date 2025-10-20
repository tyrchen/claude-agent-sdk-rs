//! Permission types for Claude Agent SDK

use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Callback for tool usage permission
pub type CanUseToolCallback = Arc<
    dyn Fn(String, serde_json::Value, ToolPermissionContext) -> BoxFuture<'static, PermissionResult>
        + Send
        + Sync,
>;

/// Context provided to permission callbacks
#[derive(Debug, Clone, Default)]
pub struct ToolPermissionContext {
    /// Abort signal (future feature)
    pub signal: Option<()>,
    /// Permission suggestions from Claude
    pub suggestions: Vec<PermissionUpdate>,
}

/// Result of a permission check
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "decision", rename_all = "lowercase")]
pub enum PermissionResult {
    /// Allow the tool use
    Allow(PermissionResultAllow),
    /// Deny the tool use
    Deny(PermissionResultDeny),
}

/// Permission result for allowing tool use
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PermissionResultAllow {
    /// Updated tool input (if modified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_input: Option<serde_json::Value>,
    /// Permission updates to apply
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_permissions: Option<Vec<PermissionUpdate>>,
}

/// Permission result for denying tool use
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionResultDeny {
    /// Denial message
    pub message: String,
    /// Whether to interrupt execution
    pub interrupt: bool,
}

impl Default for PermissionResultDeny {
    fn default() -> Self {
        Self {
            message: "Tool use denied".to_string(),
            interrupt: false,
        }
    }
}

/// Permission update to apply
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionUpdate {
    /// Type of update
    #[serde(rename = "type")]
    pub type_: PermissionUpdateType,
    /// Permission rules (for rule updates)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<PermissionRuleValue>>,
    /// Permission behavior (for rule updates)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub behavior: Option<PermissionBehavior>,
    /// Permission mode (for mode updates)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<super::config::PermissionMode>,
    /// Directories (for directory updates)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directories: Option<Vec<String>>,
    /// Destination for the update
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<PermissionUpdateDestination>,
}

/// Type of permission update
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PermissionUpdateType {
    /// Add permission rules
    AddRules,
    /// Replace permission rules
    ReplaceRules,
    /// Remove permission rules
    RemoveRules,
    /// Set permission mode
    SetMode,
    /// Add directories
    AddDirectories,
    /// Remove directories
    RemoveDirectories,
}

/// Permission rule value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRuleValue {
    /// Tool name for this rule
    pub tool_name: String,
    /// Rule content (optional pattern/constraint)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_content: Option<String>,
}

/// Permission behavior
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PermissionBehavior {
    /// Allow the action
    Allow,
    /// Deny the action
    Deny,
    /// Ask for permission
    Ask,
}

/// Destination for permission updates
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PermissionUpdateDestination {
    /// User settings
    UserSettings,
    /// Project settings
    ProjectSettings,
    /// Local settings
    LocalSettings,
    /// Current session only
    Session,
}
