//! Simple query function for one-shot interactions
//!
//! This module provides functions for simple, stateless queries to Claude Code CLI.
//! For bidirectional streaming with conversation management, use [`ClaudeClient`](crate::ClaudeClient).

use crate::errors::Result;
use crate::internal::client::InternalClient;
use crate::internal::message_parser::MessageParser;
use crate::internal::transport::subprocess::QueryPrompt;
use crate::internal::transport::{SubprocessTransport, Transport};
use crate::types::config::ClaudeAgentOptions;
use crate::types::messages::{Message, UserContentBlock};
use futures::stream::{Stream, StreamExt};
use std::pin::Pin;

// =============================================================================
// Internal helper functions (DRY principle)
// =============================================================================

/// Create a message stream from a connected transport.
///
/// This helper function extracts the common streaming logic used by both
/// `query_stream()` and `query_stream_with_content()`.
fn create_message_stream(
    transport: SubprocessTransport,
) -> Pin<Box<dyn Stream<Item = Result<Message>> + Send>> {
    let stream = async_stream::stream! {
        let mut message_stream = transport.read_messages();
        while let Some(json_result) = message_stream.next().await {
            match json_result {
                Ok(json) => {
                    match MessageParser::parse(json) {
                        Ok(message) => yield Ok(message),
                        Err(e) => {
                            yield Err(e);
                            break;
                        }
                    }
                }
                Err(e) => {
                    yield Err(e);
                    break;
                }
            }
        }
    };
    Box::pin(stream)
}

/// Setup and connect a transport for streaming queries.
///
/// This helper function extracts the common setup logic for streaming queries.
async fn setup_streaming_transport(
    query_prompt: QueryPrompt,
    options: Option<ClaudeAgentOptions>,
) -> Result<SubprocessTransport> {
    let opts = options.unwrap_or_default();
    let transport = SubprocessTransport::new(query_prompt, opts)?;
    transport.connect().await?;
    Ok(transport)
}

// =============================================================================
// Public API
// =============================================================================

/// Query Claude Code for one-shot interactions.
///
/// This function is ideal for simple, stateless queries where you don't need
/// bidirectional communication or conversation management.
///
/// # Examples
///
/// ```no_run
/// use claude_agent_sdk_rs::{query, Message, ContentBlock};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let messages = query("What is 2 + 2?", None).await?;
///
///     for message in messages {
///         match message {
///             Message::Assistant(msg) => {
///                 for block in &msg.message.content {
///                     if let ContentBlock::Text(text) = block {
///                         println!("Claude: {}", text.text);
///                     }
///                 }
///             }
///             _ => {}
///         }
///     }
///
///     Ok(())
/// }
/// ```
pub async fn query(
    prompt: impl Into<String>,
    options: Option<ClaudeAgentOptions>,
) -> Result<Vec<Message>> {
    let query_prompt = QueryPrompt::Text(prompt.into());
    let opts = options.unwrap_or_default();

    let client = InternalClient::new(query_prompt, opts)?;
    client.execute().await
}

/// Query Claude Code with streaming responses for memory-efficient processing.
///
/// Unlike `query()` which collects all messages in memory before returning,
/// this function returns a stream that yields messages as they arrive from Claude.
/// This is more memory-efficient for large conversations and provides real-time
/// message processing capabilities.
///
/// # Performance Comparison
///
/// - **`query()`**: O(n) memory usage, waits for all messages before returning
/// - **`query_stream()`**: O(1) memory per message, processes messages in real-time
///
/// # Examples
///
/// ```no_run
/// use claude_agent_sdk_rs::{query_stream, Message, ContentBlock};
/// use futures::stream::StreamExt;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let mut stream = query_stream("What is 2 + 2?", None).await?;
///
///     while let Some(result) = stream.next().await {
///         match result? {
///             Message::Assistant(msg) => {
///                 for block in &msg.message.content {
///                     if let ContentBlock::Text(text) = block {
///                         println!("Claude: {}", text.text);
///                     }
///                 }
///             }
///             _ => {}
///         }
///     }
///
///     Ok(())
/// }
/// ```
pub async fn query_stream(
    prompt: impl Into<String>,
    options: Option<ClaudeAgentOptions>,
) -> Result<Pin<Box<dyn Stream<Item = Result<Message>> + Send>>> {
    let query_prompt = QueryPrompt::Text(prompt.into());
    let transport = setup_streaming_transport(query_prompt, options).await?;
    Ok(create_message_stream(transport))
}

/// Query Claude Code with structured content blocks (supports images).
///
/// This function allows you to send mixed content including text and images
/// to Claude. Use [`UserContentBlock`] to construct the content array.
///
/// # Errors
///
/// Returns an error if:
/// - The content vector is empty (must include at least one text or image block)
/// - Claude CLI cannot be found or started
/// - The query execution fails
///
/// # Examples
///
/// ```no_run
/// use claude_agent_sdk_rs::{query_with_content, Message, ContentBlock, UserContentBlock};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Create content with text and image
///     let content = vec![
///         UserContentBlock::text("What's in this image?"),
///         UserContentBlock::image_url("https://example.com/image.png"),
///     ];
///
///     let messages = query_with_content(content, None).await?;
///
///     for message in messages {
///         if let Message::Assistant(msg) = message {
///             for block in &msg.message.content {
///                 if let ContentBlock::Text(text) = block {
///                     println!("Claude: {}", text.text);
///                 }
///             }
///         }
///     }
///
///     Ok(())
/// }
/// ```
pub async fn query_with_content(
    content: impl Into<Vec<UserContentBlock>>,
    options: Option<ClaudeAgentOptions>,
) -> Result<Vec<Message>> {
    let content_blocks = content.into();
    UserContentBlock::validate_content(&content_blocks)?;

    let query_prompt = QueryPrompt::Content(content_blocks);
    let opts = options.unwrap_or_default();

    let client = InternalClient::new(query_prompt, opts)?;
    client.execute().await
}

/// Query Claude Code with streaming and structured content blocks.
///
/// Combines the benefits of [`query_stream`] (memory efficiency, real-time processing)
/// with support for structured content blocks including images.
///
/// # Errors
///
/// Returns an error if:
/// - The content vector is empty (must include at least one text or image block)
/// - Claude CLI cannot be found or started
/// - The streaming connection fails
///
/// # Examples
///
/// ```no_run
/// use claude_agent_sdk_rs::{query_stream_with_content, Message, ContentBlock, UserContentBlock};
/// use futures::stream::StreamExt;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Create content with base64 image
///     let content = vec![
///         UserContentBlock::image_base64("image/png", "iVBORw0KGgo...")?,
///         UserContentBlock::text("Describe this diagram in detail"),
///     ];
///
///     let mut stream = query_stream_with_content(content, None).await?;
///
///     while let Some(result) = stream.next().await {
///         match result? {
///             Message::Assistant(msg) => {
///                 for block in &msg.message.content {
///                     if let ContentBlock::Text(text) = block {
///                         println!("Claude: {}", text.text);
///                     }
///                 }
///             }
///             _ => {}
///         }
///     }
///
///     Ok(())
/// }
/// ```
pub async fn query_stream_with_content(
    content: impl Into<Vec<UserContentBlock>>,
    options: Option<ClaudeAgentOptions>,
) -> Result<Pin<Box<dyn Stream<Item = Result<Message>> + Send>>> {
    let content_blocks = content.into();
    UserContentBlock::validate_content(&content_blocks)?;

    let query_prompt = QueryPrompt::Content(content_blocks);
    let transport = setup_streaming_transport(query_prompt, options).await?;
    Ok(create_message_stream(transport))
}
