//! Simple query function for one-shot interactions

use crate::errors::Result;
use crate::internal::client::InternalClient;
use crate::internal::transport::subprocess::QueryPrompt;
use crate::types::config::ClaudeAgentOptions;
use crate::types::messages::Message;

/// Query Claude Code for one-shot interactions.
///
/// This function is ideal for simple, stateless queries where you don't need
/// bidirectional communication or conversation management.
///
/// # Examples
///
/// ```no_run
/// use claude_agent_sdk::{query, Message, ContentBlock};
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
