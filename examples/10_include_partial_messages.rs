//! Example 10: Include Partial Messages
//!
//! This example demonstrates using the "include_partial_messages" option to stream
//! partial messages from Claude Code SDK.
//!
//! This feature allows you to receive stream events that contain incremental
//! updates as Claude generates responses. This is useful for:
//! - Building real-time UIs that show text as it's being generated
//! - Monitoring tool use progress
//! - Getting early results before the full response is complete
//!
//! Note: Partial message streaming requires the CLI to support it, and the
//! messages will include StreamEvent messages interspersed with regular messages.

use claude_agent_sdk_rs::{ClaudeAgentOptions, ClaudeClient, Message};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Partial Message Streaming Example");
    println!("{}", "=".repeat(50));

    // Enable partial message streaming
    let mut env = HashMap::new();
    env.insert("MAX_THINKING_TOKENS".to_string(), "8000".to_string());

    let options = ClaudeAgentOptions {
        include_partial_messages: true,
        model: Some("claude-sonnet-4-5".to_string()),
        max_turns: Some(2),
        env,
        ..Default::default()
    };

    let mut client = ClaudeClient::new(options);

    // Connect to the CLI
    client.connect().await?;

    // Send a prompt that will generate a streaming response
    let prompt = "Think of three jokes, then tell one";
    println!("Prompt: {}\n", prompt);
    println!("{}", "=".repeat(50));

    client.query(prompt).await?;

    // Receive and display messages, including partial stream events
    use futures::StreamExt;
    let mut stream = client.receive_response();
    while let Some(message) = stream.next().await {
        match message? {
            Message::StreamEvent(event) => {
                println!("Stream Event: {:?}", event);
                // You can access partial content as it arrives
                // This is useful for real-time UI updates
            }
            Message::Assistant(msg) => {
                for block in &msg.message.content {
                    if let claude_agent_sdk_rs::ContentBlock::Text(text) = block {
                        println!("Claude: {}", text.text);
                    }
                }
            }
            Message::Result(result) => {
                println!("\n=== Result ===");
                println!("Duration: {}ms", result.duration_ms);
                println!("Turns: {}", result.num_turns);
                if let Some(cost) = result.total_cost_usd {
                    println!("Cost: ${:.4}", cost);
                }
                break;
            }
            _ => {
                // Other messages
            }
        }
    }
    drop(stream);

    // Disconnect
    client.disconnect().await?;

    Ok(())
}
