//! Message types for Claude Agent SDK

use serde::{Deserialize, Serialize};

/// Main message enum containing all message types from CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Message {
    /// Assistant message
    #[serde(rename = "assistant")]
    Assistant(AssistantMessage),
    /// System message
    #[serde(rename = "system")]
    System(SystemMessage),
    /// Result message
    #[serde(rename = "result")]
    Result(ResultMessage),
    /// Stream event
    #[serde(rename = "stream_event")]
    StreamEvent(StreamEvent),
    /// User message (rarely used in stream output)
    #[serde(rename = "user")]
    User(UserMessage),
    /// Control cancel request (ignore this - it's internal control protocol)
    #[serde(rename = "control_cancel_request")]
    ControlCancelRequest(serde_json::Value),
}

/// User message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessage {
    /// Message text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Message content blocks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<ContentBlock>>,
    /// Parent tool use ID (if this is a tool result)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_tool_use_id: Option<String>,
    /// Additional fields
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

/// Message content can be text or blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// Simple text content
    Text { text: String },
    /// Structured content blocks
    Blocks { content: Vec<ContentBlock> },
}

impl From<String> for MessageContent {
    fn from(text: String) -> Self {
        MessageContent::Text { text }
    }
}

impl From<&str> for MessageContent {
    fn from(text: &str) -> Self {
        MessageContent::Text {
            text: text.to_string(),
        }
    }
}

impl From<Vec<ContentBlock>> for MessageContent {
    fn from(blocks: Vec<ContentBlock>) -> Self {
        MessageContent::Blocks { content: blocks }
    }
}

/// Assistant message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessage {
    /// The actual message content (wrapped)
    pub message: AssistantMessageInner,
    /// Parent tool use ID (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_tool_use_id: Option<String>,
    /// Session ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// UUID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
}

/// Inner assistant message content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessageInner {
    /// Message content blocks
    #[serde(default)]
    pub content: Vec<ContentBlock>,
    /// Model used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Message ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Stop reason
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    /// Usage statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<serde_json::Value>,
}

/// System message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMessage {
    /// Message subtype
    pub subtype: String,
    /// Current working directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    /// Session ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Available tools
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,
    /// MCP servers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<Vec<serde_json::Value>>,
    /// Model being used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Permission mode
    #[serde(skip_serializing_if = "Option::is_none", rename = "permissionMode")]
    pub permission_mode: Option<String>,
    /// UUID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    /// Additional data
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// Result message indicating query completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultMessage {
    /// Result subtype
    pub subtype: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// API duration in milliseconds
    pub duration_api_ms: u64,
    /// Whether this is an error result
    pub is_error: bool,
    /// Number of turns in conversation
    pub num_turns: u32,
    /// Session ID
    pub session_id: String,
    /// Total cost in USD
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_cost_usd: Option<f64>,
    /// Usage statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<serde_json::Value>,
    /// Result text (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
}

/// Stream event message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEvent {
    /// Event UUID
    pub uuid: String,
    /// Session ID
    pub session_id: String,
    /// Event data
    pub event: serde_json::Value,
    /// Parent tool use ID (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_tool_use_id: Option<String>,
}

/// Content block types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Text block
    Text(TextBlock),
    /// Thinking block (extended thinking)
    Thinking(ThinkingBlock),
    /// Tool use block
    ToolUse(ToolUseBlock),
    /// Tool result block
    ToolResult(ToolResultBlock),
}

/// Text content block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBlock {
    /// Text content
    pub text: String,
}

/// Thinking block (extended thinking)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingBlock {
    /// Thinking content
    pub thinking: String,
    /// Signature
    pub signature: String,
}

/// Tool use block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUseBlock {
    /// Tool use ID
    pub id: String,
    /// Tool name
    pub name: String,
    /// Tool input parameters
    pub input: serde_json::Value,
}

/// Tool result block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultBlock {
    /// Tool use ID this result corresponds to
    pub tool_use_id: String,
    /// Result content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<ToolResultContent>,
    /// Whether this is an error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

/// Tool result content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolResultContent {
    /// Text result
    Text(String),
    /// Structured blocks
    Blocks(Vec<serde_json::Value>),
}
