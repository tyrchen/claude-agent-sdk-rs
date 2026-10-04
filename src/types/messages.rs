//! Message types for Claude Agent SDK

use serde::{Deserialize, Serialize};

/// Supported image MIME types for Claude API
const SUPPORTED_IMAGE_MIME_TYPES: &[&str] = &["image/jpeg", "image/png", "image/gif", "image/webp"];

/// Maximum base64 data size (15MB results in ~20MB decoded, within Claude's limits)
const MAX_BASE64_SIZE: usize = 15_728_640;

/// Error types for assistant messages
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AssistantMessageError {
    /// Authentication failed
    AuthenticationFailed,
    /// Billing error
    BillingError,
    /// Rate limit exceeded
    RateLimit,
    /// Invalid request
    InvalidRequest,
    /// Server error
    ServerError,
    /// Unknown error
    Unknown,
}

/// Main message enum containing all message types from CLI
///
/// Deserialization routes `system` messages by their `subtype`: task
/// lifecycle events (`task_started`, `task_progress`, `task_notification`,
/// `task_updated`) and hook events (`hook_started`, `hook_response`) become
/// their own typed variants; everything else stays [`Message::System`].
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Message {
    /// Assistant message
    Assistant(AssistantMessage),
    /// System message
    System(SystemMessage),
    /// Result message
    Result(ResultMessage),
    /// Stream event
    StreamEvent(StreamEvent),
    /// User message (rarely used in stream output)
    User(UserMessage),
    /// Control cancel request (ignore this - it's internal control protocol)
    ControlCancelRequest(serde_json::Value),
    /// Rate limit state transition event
    RateLimit(RateLimitEvent),
    /// Conversation reset (`/clear`-style mid-session reset)
    ConversationReset(ConversationResetMessage),
    /// Background task started
    TaskStarted(TaskStartedMessage),
    /// Background task progress
    TaskProgress(TaskProgressMessage),
    /// Background task completed/failed/stopped
    TaskNotification(TaskNotificationMessage),
    /// Background task state change
    TaskUpdated(TaskUpdatedMessage),
    /// Hook lifecycle event (requires `include_hook_events`)
    HookEvent(HookEventMessage),
}

impl Message {
    /// Wire `type` tag used when serializing this message
    fn wire_type(&self) -> &'static str {
        match self {
            Message::Assistant(_) => "assistant",
            Message::System(_)
            | Message::TaskStarted(_)
            | Message::TaskProgress(_)
            | Message::TaskNotification(_)
            | Message::TaskUpdated(_)
            | Message::HookEvent(_) => "system",
            Message::Result(_) => "result",
            Message::StreamEvent(_) => "stream_event",
            Message::User(_) => "user",
            Message::ControlCancelRequest(_) => "control_cancel_request",
            Message::RateLimit(_) => "rate_limit_event",
            Message::ConversationReset(_) => "conversation_reset",
        }
    }
}

impl Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        fn tagged<T: Serialize>(
            tag: &str,
            inner: &T,
        ) -> Result<serde_json::Value, serde_json::Error> {
            let mut value = serde_json::to_value(inner)?;
            if let Some(obj) = value.as_object_mut() {
                obj.insert(
                    "type".to_string(),
                    serde_json::Value::String(tag.to_string()),
                );
            }
            Ok(value)
        }

        let tag = self.wire_type();
        let value = match self {
            Message::Assistant(m) => tagged(tag, m),
            Message::System(m) => tagged(tag, m),
            Message::Result(m) => tagged(tag, m),
            Message::StreamEvent(m) => tagged(tag, m),
            Message::User(m) => tagged(tag, m),
            Message::ControlCancelRequest(m) => tagged(tag, m),
            Message::RateLimit(m) => tagged(tag, m),
            Message::ConversationReset(m) => tagged(tag, m),
            Message::TaskStarted(m) => tagged(tag, m),
            Message::TaskProgress(m) => tagged(tag, m),
            Message::TaskNotification(m) => tagged(tag, m),
            Message::TaskUpdated(m) => tagged(tag, m),
            Message::HookEvent(m) => tagged(tag, m),
        }
        .map_err(serde::ser::Error::custom)?;
        value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error as DeError;
        let value = serde_json::Value::deserialize(deserializer)?;
        let msg_type = value.get("type").and_then(|v| v.as_str()).unwrap_or("");
        fn parse_as<T, E>(v: serde_json::Value) -> Result<T, E>
        where
            T: serde::de::DeserializeOwned,
            E: serde::de::Error,
        {
            serde_json::from_value(v).map_err(|e: serde_json::Error| E::custom(e.to_string()))
        }

        match msg_type {
            "assistant" => parse_as(value).map(Message::Assistant),
            "system" => {
                // Route typed system subtypes to their own variants
                let subtype = value.get("subtype").and_then(|v| v.as_str());
                match subtype {
                    Some("task_started") => parse_as(value).map(Message::TaskStarted),
                    Some("task_progress") => parse_as(value).map(Message::TaskProgress),
                    Some("task_notification") => parse_as(value).map(Message::TaskNotification),
                    Some("task_updated") => parse_as(value).map(Message::TaskUpdated),
                    Some("hook_started") | Some("hook_response") => {
                        parse_as(value).map(Message::HookEvent)
                    }
                    _ => parse_as(value).map(Message::System),
                }
            }
            "result" => parse_as(value).map(Message::Result),
            "stream_event" => parse_as(value).map(Message::StreamEvent),
            "user" => parse_as(value).map(Message::User),
            "control_cancel_request" => parse_as(value).map(Message::ControlCancelRequest),
            "rate_limit_event" => parse_as(value).map(Message::RateLimit),
            "conversation_reset" => parse_as(value).map(Message::ConversationReset),
            other => Err(DeError::unknown_variant(
                other,
                &[
                    "assistant",
                    "system",
                    "result",
                    "stream_event",
                    "user",
                    "control_cancel_request",
                    "rate_limit_event",
                    "conversation_reset",
                ],
            )),
        }
    }
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
    /// UUID for file checkpointing (used with enable_file_checkpointing and rewind_files)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
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
    /// Error type (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<AssistantMessageError>,
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
    /// Structured output (when output_format is specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structured_output: Option<serde_json::Value>,
    /// Stop reason for the turn
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    /// Per-model usage statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_usage: Option<std::collections::HashMap<String, serde_json::Value>>,
    /// Permission denials recorded during the turn
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_denials: Option<Vec<serde_json::Value>>,
    /// Error messages (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<String>>,
    /// Message UUID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    /// Why the query loop terminated (e.g. "completed", "max_turns",
    /// "aborted_streaming"). `None` when the CLI did not report one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal_reason: Option<String>,
    /// HTTP status code (e.g. 429, 500, 529) of the failing API call when
    /// `is_error` is true. Emitted by the CLI since v2.1.110.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_error_status: Option<i32>,
    /// Origin of the user message that triggered this turn
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<MessageOrigin>,
}

/// Origin of the user message that triggered a turn.
///
/// Lets a streaming-input consumer distinguish the result of its own prompt
/// from results of injected turns such as background-task notifications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageOrigin {
    /// Discriminator: "human", "channel", "peer", "task-notification",
    /// "coordinator", "unclassified", "observer", "auto-continuation",
    /// "observer-activity". Newer CLIs may emit unlisted kinds.
    pub kind: String,
    /// kind == "channel": name of the MCP server the message arrived on
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,
    /// kind == "peer" / "observer": sender address (sender-asserted)
    #[serde(skip_serializing_if = "Option::is_none", rename = "from")]
    pub from_: Option<String>,
    /// kind == "peer": sender display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// kind == "task-notification": "scheduled-trigger" or "peer-send-message"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subkind: Option<String>,
    /// Additional origin fields
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

/// Rate limit status values
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RateLimitStatus {
    /// Within the limit
    Allowed,
    /// Approaching the limit
    AllowedWarning,
    /// The limit has been hit
    Rejected,
}

/// Rate limit window types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RateLimitType {
    /// Five-hour window
    FiveHour,
    /// Seven-day window
    SevenDay,
    /// Seven-day window (Opus)
    SevenDayOpus,
    /// Seven-day window (Sonnet)
    SevenDaySonnet,
    /// Overage / pay-as-you-go window
    Overage,
    /// Unrecognized window type (forward compatibility)
    #[serde(other)]
    Other,
}

/// Rate limit status emitted by the CLI when rate limit state changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    /// Current rate limit status
    pub status: RateLimitStatus,
    /// Unix timestamp when the rate limit window resets
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resets_at: Option<i64>,
    /// Which rate limit window applies
    #[serde(skip_serializing_if = "Option::is_none", rename = "rateLimitType")]
    pub rate_limit_type: Option<RateLimitType>,
    /// Fraction of the rate limit consumed (0.0 - 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub utilization: Option<f64>,
    /// Status of overage/pay-as-you-go usage if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overage_status: Option<RateLimitStatus>,
    /// Unix timestamp when overage window resets
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overage_resets_at: Option<i64>,
    /// Why overage is unavailable if status is rejected
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overage_disabled_reason: Option<String>,
    /// Full raw payload from the CLI, including unmodeled fields
    #[serde(flatten)]
    pub raw: serde_json::Value,
}

/// Rate limit event emitted when rate limit info changes.
///
/// Use this to warn users before they hit a hard limit, or to gracefully
/// back off when `status == "rejected"`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitEvent {
    /// The rate limit status
    pub rate_limit_info: RateLimitInfo,
    /// Event UUID
    pub uuid: String,
    /// Session ID
    pub session_id: String,
}

/// Emitted when the session's conversation is replaced without ending the
/// connection — e.g. after `/clear`.
///
/// A reset zeroes the running totals reported on subsequent [`ResultMessage`]s
/// (e.g. `total_cost_usd`); snapshot any accumulated totals when this arrives.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationResetMessage {
    /// Opaque identifier for the fresh conversation (key a new empty
    /// transcript on this; it is *not* the session_id of later messages)
    pub new_conversation_id: String,
    /// Message UUID
    pub uuid: String,
    /// ID of the session that was reset
    pub session_id: String,
}

/// Usage statistics reported in task progress/notification messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskUsage {
    /// Total tokens used by the task
    pub total_tokens: u64,
    /// Number of tool uses
    pub tool_uses: u64,
    /// Task duration in milliseconds
    pub duration_ms: u64,
}

/// Terminal task statuses: the task has finished and should be cleared from
/// any active-task tracking. Spans both lifecycle vocabularies:
/// `task_notification` reports `stopped` (the CLI's mapped form of a killed
/// task) while `task_updated` reports the raw `killed`.
pub const TERMINAL_TASK_STATUSES: &[&str] = &["completed", "failed", "stopped", "killed"];

/// Returns true if the given task status is terminal
pub fn is_terminal_task_status(status: &str) -> bool {
    TERMINAL_TASK_STATUSES.contains(&status)
}

/// System message emitted when a background task starts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStartedMessage {
    /// Message subtype ("task_started")
    pub subtype: String,
    /// Task ID
    pub task_id: String,
    /// Task description
    pub description: String,
    /// Message UUID
    pub uuid: String,
    /// Session ID
    pub session_id: String,
    /// Tool use ID that spawned the task, if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use_id: Option<String>,
    /// Task type, if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_type: Option<String>,
    /// Full raw payload
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// System message emitted while a background task is in progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgressMessage {
    /// Message subtype ("task_progress")
    pub subtype: String,
    /// Task ID
    pub task_id: String,
    /// Task description
    pub description: String,
    /// Current usage statistics
    pub usage: TaskUsage,
    /// Message UUID
    pub uuid: String,
    /// Session ID
    pub session_id: String,
    /// Tool use ID that spawned the task, if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use_id: Option<String>,
    /// Name of the last tool used, if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_tool_name: Option<String>,
    /// Full raw payload
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// System message emitted when a background task completes, fails, or is stopped.
///
/// Note: not every terminal task emits this message. Background tasks may
/// instead report completion only via a [`TaskUpdatedMessage`] whose
/// `patch.status` is terminal. Clear tracked task IDs on a terminal status
/// from *either* message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskNotificationMessage {
    /// Message subtype ("task_notification")
    pub subtype: String,
    /// Task ID
    pub task_id: String,
    /// Terminal status: "completed", "failed", or "stopped"
    pub status: String,
    /// Path to the file holding the task's output
    pub output_file: String,
    /// Summary of the task result
    pub summary: String,
    /// Message UUID
    pub uuid: String,
    /// Session ID
    pub session_id: String,
    /// Tool use ID that spawned the task, if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use_id: Option<String>,
    /// Final usage statistics, if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<TaskUsage>,
    /// Full raw payload
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// System message emitted when a background task's state changes.
///
/// `patch` carries the changed fields (e.g. `status`, `end_time`). A
/// background task's terminal state can arrive *only* as this message with no
/// accompanying [`TaskNotificationMessage`] — e.g. a task stopped via
/// `stop_task` reports `status="killed"` here.
#[derive(Debug, Clone, Serialize)]
pub struct TaskUpdatedMessage {
    /// Message subtype ("task_updated")
    pub subtype: String,
    /// Task ID
    pub task_id: String,
    /// Changed fields
    pub patch: serde_json::Value,
    /// Terminal-ness derived from `patch.status`; `None` when the patch
    /// carries no status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Session ID, if present
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Message UUID, if present
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    /// Full raw payload
    #[serde(flatten)]
    pub data: serde_json::Value,
}

impl<'de> serde::Deserialize<'de> for TaskUpdatedMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct Raw {
            subtype: String,
            #[serde(default)]
            task_id: String,
            #[serde(default)]
            patch: serde_json::Value,
            #[serde(default)]
            session_id: Option<String>,
            #[serde(default)]
            uuid: Option<String>,
            #[serde(flatten)]
            data: serde_json::Value,
        }
        let raw = Raw::deserialize(deserializer)?;
        // Terminal-ness is derived from patch.status, mirroring the Python SDK
        let status = raw
            .patch
            .get("status")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        Ok(TaskUpdatedMessage {
            subtype: raw.subtype,
            task_id: raw.task_id,
            patch: raw.patch,
            status,
            session_id: raw.session_id,
            uuid: raw.uuid,
            data: raw.data,
        })
    }
}

/// Hook lifecycle event emitted by the CLI when `include_hook_events` is enabled.
///
/// Arrives on the wire as `{"type": "system", "subtype": "hook_started" |
/// "hook_response", ...}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookEventMessage {
    /// Lifecycle phase: "hook_started" or "hook_response"
    pub subtype: String,
    /// Name of the hook event (e.g. "PreToolUse", "PostToolUse", "Stop").
    /// The CLI may send this as `hook_event`, `hook_name`, or `hook_event_name`.
    #[serde(default, alias = "hook_event", alias = "hook_name")]
    pub hook_event_name: String,
    /// Full raw event payload
    #[serde(flatten)]
    pub data: serde_json::Value,
    /// Session ID the event belongs to, if present
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Unique ID of the event, if present
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
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
    /// Image block
    Image(ImageBlock),
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

/// Image source for user prompts
///
/// Represents the source of image data that can be included in user messages.
/// Claude supports both base64-encoded images and URL references.
///
/// # Supported Formats
///
/// - JPEG (`image/jpeg`)
/// - PNG (`image/png`)
/// - GIF (`image/gif`)
/// - WebP (`image/webp`)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ImageSource {
    /// Base64-encoded image data
    Base64 {
        /// MIME type (e.g., "image/png", "image/jpeg", "image/gif", "image/webp")
        media_type: String,
        /// Base64-encoded image data (without data URI prefix)
        data: String,
    },
    /// URL reference to an image
    Url {
        /// Publicly accessible image URL
        url: String,
    },
}

/// Image block for user prompts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageBlock {
    /// Image source (base64 or URL)
    pub source: ImageSource,
}

/// Content block for user prompts (input)
///
/// Represents content that can be included in user messages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UserContentBlock {
    /// Text content
    Text {
        /// Text content string
        text: String,
    },
    /// Image content
    Image {
        /// Image source (base64 or URL)
        source: ImageSource,
    },
}

impl UserContentBlock {
    /// Create a text content block
    pub fn text(text: impl Into<String>) -> Self {
        UserContentBlock::Text { text: text.into() }
    }

    /// Create an image content block from base64 data
    ///
    /// # Arguments
    ///
    /// * `media_type` - MIME type of the image (e.g., "image/png", "image/jpeg")
    /// * `data` - Base64-encoded image data (without data URI prefix)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The MIME type is not supported (valid types: image/jpeg, image/png, image/gif, image/webp)
    /// - The base64 data exceeds the maximum size limit (15MB)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use claude_agent_sdk_rs::UserContentBlock;
    /// let block = UserContentBlock::image_base64("image/png", "iVBORw0KGgo=")?;
    /// # Ok::<(), claude_agent_sdk_rs::ClaudeError>(())
    /// ```
    pub fn image_base64(
        media_type: impl Into<String>,
        data: impl Into<String>,
    ) -> crate::errors::Result<Self> {
        let media_type_str = media_type.into();
        let data_str = data.into();

        // Validate MIME type
        if !SUPPORTED_IMAGE_MIME_TYPES.contains(&media_type_str.as_str()) {
            return Err(crate::errors::ImageValidationError::new(format!(
                "Unsupported media type '{}'. Supported types: {:?}",
                media_type_str, SUPPORTED_IMAGE_MIME_TYPES
            ))
            .into());
        }

        // Validate base64 size
        if data_str.len() > MAX_BASE64_SIZE {
            return Err(crate::errors::ImageValidationError::new(format!(
                "Base64 data exceeds maximum size of {} bytes (got {} bytes)",
                MAX_BASE64_SIZE,
                data_str.len()
            ))
            .into());
        }

        Ok(UserContentBlock::Image {
            source: ImageSource::Base64 {
                media_type: media_type_str,
                data: data_str,
            },
        })
    }

    /// Create an image content block from URL
    pub fn image_url(url: impl Into<String>) -> Self {
        UserContentBlock::Image {
            source: ImageSource::Url { url: url.into() },
        }
    }

    /// Validate a collection of content blocks
    ///
    /// Ensures the content is non-empty. This is used internally by query functions
    /// to provide consistent validation.
    ///
    /// # Errors
    ///
    /// Returns an error if the content blocks slice is empty.
    pub fn validate_content(blocks: &[UserContentBlock]) -> crate::Result<()> {
        if blocks.is_empty() {
            return Err(crate::errors::ClaudeError::InvalidConfig(
                "Content must include at least one block (text or image)".to_string(),
            ));
        }
        Ok(())
    }
}

impl From<String> for UserContentBlock {
    fn from(text: String) -> Self {
        UserContentBlock::Text { text }
    }
}

impl From<&str> for UserContentBlock {
    fn from(text: &str) -> Self {
        UserContentBlock::Text {
            text: text.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_content_block_text_serialization() {
        let block = ContentBlock::Text(TextBlock {
            text: "Hello".to_string(),
        });

        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(json["type"], "text");
        assert_eq!(json["text"], "Hello");
    }

    #[test]
    fn test_content_block_tool_use_serialization() {
        let block = ContentBlock::ToolUse(ToolUseBlock {
            id: "tool_123".to_string(),
            name: "Bash".to_string(),
            input: json!({"command": "echo hello"}),
        });

        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(json["type"], "tool_use");
        assert_eq!(json["id"], "tool_123");
        assert_eq!(json["name"], "Bash");
        assert_eq!(json["input"]["command"], "echo hello");
    }

    #[test]
    fn test_message_assistant_deserialization() {
        let json_str = r#"{
            "type": "assistant",
            "message": {
                "content": [{"type": "text", "text": "Hello"}],
                "model": "claude-sonnet-4"
            },
            "session_id": "test-session"
        }"#;

        let msg: Message = serde_json::from_str(json_str).unwrap();
        match msg {
            Message::Assistant(assistant) => {
                assert_eq!(assistant.session_id, Some("test-session".to_string()));
                assert_eq!(assistant.message.model, Some("claude-sonnet-4".to_string()));
            }
            _ => panic!("Expected Assistant variant"),
        }
    }

    #[test]
    fn test_message_result_deserialization() {
        let json_str = r#"{
            "type": "result",
            "subtype": "query_complete",
            "duration_ms": 1500,
            "duration_api_ms": 1200,
            "is_error": false,
            "num_turns": 3,
            "session_id": "test-session",
            "total_cost_usd": 0.0042
        }"#;

        let msg: Message = serde_json::from_str(json_str).unwrap();
        match msg {
            Message::Result(result) => {
                assert_eq!(result.subtype, "query_complete");
                assert_eq!(result.duration_ms, 1500);
                assert_eq!(result.num_turns, 3);
                assert_eq!(result.total_cost_usd, Some(0.0042));
            }
            _ => panic!("Expected Result variant"),
        }
    }

    #[test]
    fn test_message_system_deserialization() {
        let json_str = r#"{
            "type": "system",
            "subtype": "session_start",
            "cwd": "/home/user",
            "session_id": "test-session",
            "tools": ["Bash", "Read", "Write"]
        }"#;

        let msg: Message = serde_json::from_str(json_str).unwrap();
        match msg {
            Message::System(system) => {
                assert_eq!(system.subtype, "session_start");
                assert_eq!(system.cwd, Some("/home/user".to_string()));
                assert_eq!(system.tools.as_ref().unwrap().len(), 3);
            }
            _ => panic!("Expected System variant"),
        }
    }

    #[test]
    fn test_tool_result_content_text() {
        let content = ToolResultContent::Text("Command output".to_string());
        let json = serde_json::to_value(&content).unwrap();
        assert_eq!(json, "Command output");
    }

    #[test]
    fn test_tool_result_content_blocks() {
        let content = ToolResultContent::Blocks(vec![json!({"type": "text", "text": "Result"})]);
        let json = serde_json::to_value(&content).unwrap();
        assert!(json.is_array());
        assert_eq!(json[0]["type"], "text");
    }

    #[test]
    fn test_image_source_base64_serialization() {
        let source = ImageSource::Base64 {
            media_type: "image/png".to_string(),
            data: "iVBORw0KGgo=".to_string(),
        };

        let json = serde_json::to_value(&source).unwrap();
        assert_eq!(json["type"], "base64");
        assert_eq!(json["media_type"], "image/png");
        assert_eq!(json["data"], "iVBORw0KGgo=");
    }

    #[test]
    fn test_image_source_url_serialization() {
        let source = ImageSource::Url {
            url: "https://example.com/image.png".to_string(),
        };

        let json = serde_json::to_value(&source).unwrap();
        assert_eq!(json["type"], "url");
        assert_eq!(json["url"], "https://example.com/image.png");
    }

    #[test]
    fn test_image_source_base64_deserialization() {
        let json_str = r#"{
            "type": "base64",
            "media_type": "image/jpeg",
            "data": "base64data=="
        }"#;

        let source: ImageSource = serde_json::from_str(json_str).unwrap();
        match source {
            ImageSource::Base64 { media_type, data } => {
                assert_eq!(media_type, "image/jpeg");
                assert_eq!(data, "base64data==");
            }
            _ => panic!("Expected Base64 variant"),
        }
    }

    #[test]
    fn test_image_source_url_deserialization() {
        let json_str = r#"{
            "type": "url",
            "url": "https://example.com/test.gif"
        }"#;

        let source: ImageSource = serde_json::from_str(json_str).unwrap();
        match source {
            ImageSource::Url { url } => {
                assert_eq!(url, "https://example.com/test.gif");
            }
            _ => panic!("Expected Url variant"),
        }
    }

    #[test]
    fn test_user_content_block_text_serialization() {
        let block = UserContentBlock::text("Hello world");

        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(json["type"], "text");
        assert_eq!(json["text"], "Hello world");
    }

    #[test]
    fn test_user_content_block_image_base64_serialization() {
        let block = UserContentBlock::image_base64("image/png", "iVBORw0KGgo=").unwrap();

        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(json["type"], "image");
        assert_eq!(json["source"]["type"], "base64");
        assert_eq!(json["source"]["media_type"], "image/png");
        assert_eq!(json["source"]["data"], "iVBORw0KGgo=");
    }

    #[test]
    fn test_user_content_block_image_url_serialization() {
        let block = UserContentBlock::image_url("https://example.com/image.webp");

        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(json["type"], "image");
        assert_eq!(json["source"]["type"], "url");
        assert_eq!(json["source"]["url"], "https://example.com/image.webp");
    }

    #[test]
    fn test_user_content_block_from_string() {
        let block: UserContentBlock = "Test message".into();

        match block {
            UserContentBlock::Text { text } => {
                assert_eq!(text, "Test message");
            }
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_user_content_block_from_owned_string() {
        let block: UserContentBlock = String::from("Owned message").into();

        match block {
            UserContentBlock::Text { text } => {
                assert_eq!(text, "Owned message");
            }
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_image_block_serialization() {
        let block = ImageBlock {
            source: ImageSource::Base64 {
                media_type: "image/gif".to_string(),
                data: "R0lGODlh".to_string(),
            },
        };

        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(json["source"]["type"], "base64");
        assert_eq!(json["source"]["media_type"], "image/gif");
        assert_eq!(json["source"]["data"], "R0lGODlh");
    }

    #[test]
    fn test_content_block_image_serialization() {
        let block = ContentBlock::Image(ImageBlock {
            source: ImageSource::Url {
                url: "https://example.com/photo.jpg".to_string(),
            },
        });

        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(json["type"], "image");
        assert_eq!(json["source"]["type"], "url");
        assert_eq!(json["source"]["url"], "https://example.com/photo.jpg");
    }

    #[test]
    fn test_content_block_image_deserialization() {
        let json_str = r#"{
            "type": "image",
            "source": {
                "type": "base64",
                "media_type": "image/webp",
                "data": "UklGR"
            }
        }"#;

        let block: ContentBlock = serde_json::from_str(json_str).unwrap();
        match block {
            ContentBlock::Image(image) => match image.source {
                ImageSource::Base64 { media_type, data } => {
                    assert_eq!(media_type, "image/webp");
                    assert_eq!(data, "UklGR");
                }
                _ => panic!("Expected Base64 source"),
            },
            _ => panic!("Expected Image variant"),
        }
    }

    #[test]
    fn test_user_content_block_deserialization() {
        let json_str = r#"{
            "type": "text",
            "text": "Describe this image"
        }"#;

        let block: UserContentBlock = serde_json::from_str(json_str).unwrap();
        match block {
            UserContentBlock::Text { text } => {
                assert_eq!(text, "Describe this image");
            }
            _ => panic!("Expected Text variant"),
        }
    }

    #[test]
    fn test_user_content_block_image_deserialization() {
        let json_str = r#"{
            "type": "image",
            "source": {
                "type": "url",
                "url": "https://example.com/diagram.png"
            }
        }"#;

        let block: UserContentBlock = serde_json::from_str(json_str).unwrap();
        match block {
            UserContentBlock::Image { source } => match source {
                ImageSource::Url { url } => {
                    assert_eq!(url, "https://example.com/diagram.png");
                }
                _ => panic!("Expected Url source"),
            },
            _ => panic!("Expected Image variant"),
        }
    }

    #[test]
    fn test_image_base64_valid() {
        let block = UserContentBlock::image_base64("image/png", "iVBORw0KGgo=");
        assert!(block.is_ok());
    }

    #[test]
    fn test_image_base64_invalid_mime_type() {
        let block = UserContentBlock::image_base64("image/bmp", "data");
        assert!(block.is_err());
        let err = block.unwrap_err().to_string();
        assert!(err.contains("Unsupported media type"));
        assert!(err.contains("image/bmp"));
    }

    #[test]
    fn test_image_base64_exceeds_size_limit() {
        let large_data = "a".repeat(MAX_BASE64_SIZE + 1);
        let block = UserContentBlock::image_base64("image/png", large_data);
        assert!(block.is_err());
        let err = block.unwrap_err().to_string();
        assert!(err.contains("exceeds maximum size"));
    }
}

#[cfg(test)]
mod new_message_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_rate_limit_event() {
        let msg: Message = serde_json::from_value(json!({
            "type": "rate_limit_event",
            "rate_limit_info": {
                "status": "allowed_warning",
                "resets_at": 1234567890,
                "utilization": 0.85
            },
            "uuid": "u1",
            "session_id": "s1"
        }))
        .unwrap();
        match msg {
            Message::RateLimit(e) => {
                assert_eq!(e.uuid, "u1");
                assert_eq!(e.rate_limit_info.status, RateLimitStatus::AllowedWarning);
                assert_eq!(e.rate_limit_info.utilization, Some(0.85));
            }
            other => panic!("expected RateLimit, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_conversation_reset() {
        let msg: Message = serde_json::from_value(json!({
            "type": "conversation_reset",
            "new_conversation_id": "conv-2",
            "uuid": "u2",
            "session_id": "s1"
        }))
        .unwrap();
        match msg {
            Message::ConversationReset(m) => {
                assert_eq!(m.new_conversation_id, "conv-2");
                assert_eq!(m.session_id, "s1");
            }
            other => panic!("expected ConversationReset, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_task_started() {
        let msg: Message = serde_json::from_value(json!({
            "type": "system",
            "subtype": "task_started",
            "task_id": "t1",
            "description": "build",
            "uuid": "u3",
            "session_id": "s1"
        }))
        .unwrap();
        match msg {
            Message::TaskStarted(m) => {
                assert_eq!(m.task_id, "t1");
                assert_eq!(m.description, "build");
            }
            other => panic!("expected TaskStarted, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_task_progress() {
        let msg: Message = serde_json::from_value(json!({
            "type": "system",
            "subtype": "task_progress",
            "task_id": "t1",
            "description": "build",
            "usage": {"total_tokens": 100, "tool_uses": 3, "duration_ms": 500},
            "uuid": "u4",
            "session_id": "s1"
        }))
        .unwrap();
        match msg {
            Message::TaskProgress(m) => {
                assert_eq!(m.usage.total_tokens, 100);
                assert_eq!(m.usage.tool_uses, 3);
            }
            other => panic!("expected TaskProgress, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_task_notification() {
        let msg: Message = serde_json::from_value(json!({
            "type": "system",
            "subtype": "task_notification",
            "task_id": "t1",
            "status": "completed",
            "output_file": "/tmp/out",
            "summary": "done",
            "uuid": "u5",
            "session_id": "s1"
        }))
        .unwrap();
        match msg {
            Message::TaskNotification(m) => {
                assert_eq!(m.status, "completed");
                assert!(is_terminal_task_status(&m.status));
            }
            other => panic!("expected TaskNotification, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_task_updated_killed_is_terminal() {
        let msg: Message = serde_json::from_value(json!({
            "type": "system",
            "subtype": "task_updated",
            "task_id": "t1",
            "patch": {"status": "killed", "end_time": "2026-10-04T00:00:00Z"}
        }))
        .unwrap();
        match msg {
            Message::TaskUpdated(m) => {
                assert_eq!(m.status.as_deref(), Some("killed"));
                assert!(is_terminal_task_status(m.status.as_deref().unwrap()));
            }
            other => panic!("expected TaskUpdated, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_hook_event_message() {
        for subtype in ["hook_started", "hook_response"] {
            let msg: Message = serde_json::from_value(json!({
                "type": "system",
                "subtype": subtype,
                "hook_event": "PreToolUse",
                "uuid": "u6",
                "session_id": "s1"
            }))
            .unwrap();
            match msg {
                Message::HookEvent(m) => {
                    assert_eq!(m.hook_event_name, "PreToolUse");
                    assert_eq!(m.subtype, subtype);
                }
                other => panic!("expected HookEvent, got {:?}", other),
            }
        }
    }

    #[test]
    fn test_plain_system_message_still_parses_as_system() {
        let msg: Message = serde_json::from_value(json!({
            "type": "system",
            "subtype": "init",
            "session_id": "s1"
        }))
        .unwrap();
        assert!(matches!(msg, Message::System(_)));
    }

    #[test]
    fn test_result_message_new_fields() {
        let msg: Message = serde_json::from_value(json!({
            "type": "result",
            "subtype": "success",
            "duration_ms": 100,
            "duration_api_ms": 90,
            "is_error": false,
            "num_turns": 2,
            "session_id": "s1",
            "stop_reason": "end_turn",
            "terminal_reason": "completed",
            "uuid": "u7",
            "errors": ["e1"],
            "origin": {"kind": "human"}
        }))
        .unwrap();
        match msg {
            Message::Result(r) => {
                assert_eq!(r.stop_reason.as_deref(), Some("end_turn"));
                assert_eq!(r.terminal_reason.as_deref(), Some("completed"));
                assert_eq!(r.uuid.as_deref(), Some("u7"));
                assert_eq!(r.errors.as_deref(), Some(&["e1".to_string()][..]));
                assert_eq!(r.origin.as_ref().unwrap().kind, "human");
            }
            other => panic!("expected Result, got {:?}", other),
        }
    }

    #[test]
    fn test_message_serialization_round_trip() {
        let msg = Message::RateLimit(RateLimitEvent {
            rate_limit_info: RateLimitInfo {
                status: RateLimitStatus::Rejected,
                resets_at: None,
                rate_limit_type: None,
                utilization: None,
                overage_status: None,
                overage_resets_at: None,
                overage_disabled_reason: None,
                raw: serde_json::Value::Null,
            },
            uuid: "u8".to_string(),
            session_id: "s1".to_string(),
        });
        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["type"], serde_json::json!("rate_limit_event"));
        let back: Message = serde_json::from_value(json).unwrap();
        assert!(matches!(back, Message::RateLimit(_)));
    }
}
