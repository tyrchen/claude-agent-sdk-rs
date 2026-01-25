//! Builder for tool result control messages

/// Builder for tool result control messages
pub struct ToolResultBuilder {
    tool_use_id: String,
    content: serde_json::Value,
    is_error: bool,
}

impl ToolResultBuilder {
    /// Create a new builder for a tool use ID
    pub fn new(tool_use_id: impl Into<String>) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content: serde_json::Value::Null,
            is_error: false,
        }
    }

    /// Set success content
    pub fn success(mut self, content: serde_json::Value) -> Self {
        self.content = content;
        self.is_error = false;
        self
    }

    /// Set error content
    pub fn error(mut self, message: impl Into<String>) -> Self {
        self.content = serde_json::json!({ "error": message.into() });
        self.is_error = true;
        self
    }

    /// Build as control_response JSON (for transport simulation)
    pub fn build_control_response(self) -> serde_json::Value {
        serde_json::json!({
            "type": "control_response",
            "control_response": {
                "tool_use_id": self.tool_use_id,
                "content": self.content,
                "is_error": self.is_error,
            }
        })
    }

    /// Build as tool result content block
    pub fn build_content_block(self) -> serde_json::Value {
        serde_json::json!({
            "type": "tool_result",
            "tool_use_id": self.tool_use_id,
            "content": self.content,
            "is_error": self.is_error,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_result_success() {
        let result = ToolResultBuilder::new("tool_123")
            .success(serde_json::json!({"output": "file contents"}))
            .build_control_response();

        assert_eq!(result["type"], "control_response");
        assert_eq!(result["control_response"]["tool_use_id"], "tool_123");
        assert!(!result["control_response"]["is_error"].as_bool().unwrap());
    }

    #[test]
    fn test_tool_result_error() {
        let result = ToolResultBuilder::new("tool_456")
            .error("File not found")
            .build_control_response();

        assert!(result["control_response"]["is_error"].as_bool().unwrap());
        assert_eq!(
            result["control_response"]["content"]["error"],
            "File not found"
        );
    }
}
