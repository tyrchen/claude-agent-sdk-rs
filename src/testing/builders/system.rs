//! Builder for SystemMessage

use crate::types::messages::{Message, SystemMessage};
use std::path::PathBuf;

/// Builder for SystemMessage (session initialization)
pub struct SystemMessageBuilder {
    subtype: String,
    session_id: String,
    model: String,
    tools: Vec<String>,
    mcp_servers: Vec<serde_json::Value>,
    cwd: Option<String>,
    permission_mode: Option<String>,
}

impl SystemMessageBuilder {
    /// Create a new builder with defaults
    pub fn new() -> Self {
        Self {
            subtype: "init".to_string(),
            session_id: format!("test-session-{}", uuid::Uuid::new_v4()),
            model: "claude-sonnet-4-20250514".to_string(),
            tools: vec!["Read".into(), "Write".into(), "Bash".into()],
            mcp_servers: Vec::new(),
            cwd: None,
            permission_mode: None,
        }
    }

    /// Set the subtype
    pub fn subtype(mut self, subtype: impl Into<String>) -> Self {
        self.subtype = subtype.into();
        self
    }

    /// Set the session ID
    pub fn session_id(mut self, id: impl Into<String>) -> Self {
        self.session_id = id.into();
        self
    }

    /// Set the model
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Set the available tools
    pub fn tools(mut self, tools: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tools = tools.into_iter().map(Into::into).collect();
        self
    }

    /// Add a tool
    pub fn add_tool(mut self, tool: impl Into<String>) -> Self {
        self.tools.push(tool.into());
        self
    }

    /// Set MCP servers
    pub fn mcp_servers(mut self, servers: impl IntoIterator<Item = serde_json::Value>) -> Self {
        self.mcp_servers = servers.into_iter().collect();
        self
    }

    /// Set current working directory
    pub fn cwd(mut self, path: impl Into<PathBuf>) -> Self {
        self.cwd = Some(path.into().to_string_lossy().to_string());
        self
    }

    /// Set permission mode
    pub fn permission_mode(mut self, mode: impl Into<String>) -> Self {
        self.permission_mode = Some(mode.into());
        self
    }

    /// Build the message
    pub fn build(self) -> Message {
        Message::System(SystemMessage {
            subtype: self.subtype,
            session_id: Some(self.session_id),
            model: Some(self.model),
            tools: Some(self.tools),
            mcp_servers: if self.mcp_servers.is_empty() {
                None
            } else {
                Some(self.mcp_servers)
            },
            cwd: self.cwd,
            permission_mode: self.permission_mode,
            uuid: Some(uuid::Uuid::new_v4().to_string()),
            data: serde_json::Value::Null,
        })
    }

    /// Build as JSON value
    pub fn build_json(self) -> serde_json::Value {
        serde_json::to_value(self.build()).expect("Message serialization should not fail")
    }
}

impl Default for SystemMessageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_builder_defaults() {
        let msg = SystemMessageBuilder::default().build();

        if let Message::System(system) = msg {
            assert_eq!(system.subtype, "init");
            assert!(system.session_id.is_some());
            assert_eq!(system.model, Some("claude-sonnet-4-20250514".to_string()));
            assert!(system.tools.is_some());
        } else {
            panic!("Expected system message");
        }
    }

    #[test]
    fn test_system_builder_custom() {
        let msg = SystemMessageBuilder::new()
            .model("claude-opus-4")
            .tools(["Read", "Write"])
            .cwd("/tmp")
            .build();

        if let Message::System(system) = msg {
            assert_eq!(system.model, Some("claude-opus-4".to_string()));
            assert_eq!(system.cwd, Some("/tmp".to_string()));
            let tools = system.tools.unwrap();
            assert_eq!(tools.len(), 2);
        } else {
            panic!("Expected system message");
        }
    }
}
