//! Builder for AssistantMessage

use crate::types::messages::{
    AssistantMessage, AssistantMessageInner, ContentBlock, Message, TextBlock, ThinkingBlock,
    ToolUseBlock,
};

/// Builder for AssistantMessage
pub struct AssistantMessageBuilder {
    content: Vec<ContentBlock>,
    model: Option<String>,
    stop_reason: Option<String>,
    session_id: Option<String>,
}

impl AssistantMessageBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            content: Vec::new(),
            model: None,
            stop_reason: None,
            session_id: None,
        }
    }

    /// Add a text block
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.content
            .push(ContentBlock::Text(TextBlock { text: text.into() }));
        self
    }

    /// Add a tool use block
    pub fn tool_use(mut self, name: impl Into<String>, input: serde_json::Value) -> Self {
        self.content.push(ContentBlock::ToolUse(ToolUseBlock {
            id: format!("tool_{}", uuid::Uuid::new_v4()),
            name: name.into(),
            input,
        }));
        self
    }

    /// Add a tool use block with specific ID
    pub fn tool_use_with_id(
        mut self,
        id: impl Into<String>,
        name: impl Into<String>,
        input: serde_json::Value,
    ) -> Self {
        self.content.push(ContentBlock::ToolUse(ToolUseBlock {
            id: id.into(),
            name: name.into(),
            input,
        }));
        self
    }

    /// Add a thinking block (extended thinking)
    pub fn thinking(mut self, thinking: impl Into<String>) -> Self {
        self.content.push(ContentBlock::Thinking(ThinkingBlock {
            thinking: thinking.into(),
            signature: String::new(),
        }));
        self
    }

    /// Set the model
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set the stop reason
    pub fn stop_reason(mut self, reason: impl Into<String>) -> Self {
        self.stop_reason = Some(reason.into());
        self
    }

    /// Set the session ID
    pub fn session_id(mut self, id: impl Into<String>) -> Self {
        self.session_id = Some(id.into());
        self
    }

    /// Build the message
    pub fn build(self) -> Message {
        Message::Assistant(AssistantMessage {
            message: AssistantMessageInner {
                content: self.content,
                model: self.model,
                id: Some(format!("msg_{}", uuid::Uuid::new_v4())),
                stop_reason: self.stop_reason,
                usage: None,
                error: None,
            },
            parent_tool_use_id: None,
            session_id: self.session_id,
            uuid: Some(uuid::Uuid::new_v4().to_string()),
        })
    }

    /// Build as JSON value
    pub fn build_json(self) -> serde_json::Value {
        serde_json::to_value(self.build()).expect("Message serialization should not fail")
    }
}

impl Default for AssistantMessageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assistant_builder_text() {
        let msg = AssistantMessageBuilder::new().text("Hello, world!").build();

        if let Message::Assistant(assistant) = msg {
            assert_eq!(assistant.message.content.len(), 1);
            if let ContentBlock::Text(text) = &assistant.message.content[0] {
                assert_eq!(text.text, "Hello, world!");
            } else {
                panic!("Expected text block");
            }
        } else {
            panic!("Expected assistant message");
        }
    }

    #[test]
    fn test_assistant_builder_tool_use() {
        let msg = AssistantMessageBuilder::new()
            .tool_use("Read", serde_json::json!({"file_path": "/tmp/test.txt"}))
            .build();

        if let Message::Assistant(assistant) = msg {
            assert_eq!(assistant.message.content.len(), 1);
            if let ContentBlock::ToolUse(tool) = &assistant.message.content[0] {
                assert_eq!(tool.name, "Read");
            } else {
                panic!("Expected tool use block");
            }
        } else {
            panic!("Expected assistant message");
        }
    }

    #[test]
    fn test_assistant_builder_multiple_blocks() {
        let msg = AssistantMessageBuilder::new()
            .text("Let me read that file")
            .tool_use("Read", serde_json::json!({"file_path": "/tmp/test.txt"}))
            .text("Here's what I found")
            .build();

        if let Message::Assistant(assistant) = msg {
            assert_eq!(assistant.message.content.len(), 3);
        } else {
            panic!("Expected assistant message");
        }
    }
}
