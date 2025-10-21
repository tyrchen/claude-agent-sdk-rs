//! Example demonstrating bidirectional communication with ClaudeClient
//!
//! This example shows how to use ClaudeClient (analogous to Python's ClaudeClient)
//! for bidirectional, streaming communication with Claude Code.
//!
//! Key features demonstrated:
//! - Connecting to Claude
//! - Sending multiple queries in the same session
//! - Streaming responses with receive_response()
//! - Claude remembering context across queries
//! - Clean disconnection
//!
//! Run with:
//! ```bash
//! cargo run --example 06_bidirectional_client
//! ```

use claude_agent_sdk_rs::{ClaudeAgentOptions, ClaudeClient, ContentBlock, Message};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Bidirectional ClaudeClient Example ===\n");

    // Create client with options
    let options = ClaudeAgentOptions {
        max_turns: Some(5),
        ..Default::default()
    };

    let mut client = ClaudeClient::new(options);

    // Connect to Claude (analogous to Python's `async with ClaudeClient()`)
    println!("Connecting to Claude...");
    client.connect().await?;
    println!("Connected!\n");

    // First query
    println!("Query 1: What is your name?");
    client.query("What is your name?").await?;

    // Receive response as a stream (continues until ResultMessage)
    let mut stream = client.receive_response();
    while let Some(message) = stream.next().await {
        match message? {
            Message::Assistant(msg) => {
                for block in msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        println!("Claude: {}", text.text);
                    }
                }
            }
            Message::Result(result) => {
                println!(
                    "\n[Result] Duration: {}ms, Cost: ${:.4}\n",
                    result.duration_ms,
                    result.total_cost_usd.unwrap_or(0.0)
                );
            }
            _ => {}
        }
    }
    drop(stream); // Release borrow before next query

    // Second query - Claude remembers context!
    println!("Query 2: Can you remember what I just asked you?");
    client
        .query("Can you remember what I just asked you?")
        .await?;

    let mut stream = client.receive_response();
    while let Some(message) = stream.next().await {
        match message? {
            Message::Assistant(msg) => {
                for block in msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        println!("Claude: {}", text.text);
                    }
                }
            }
            Message::Result(result) => {
                println!(
                    "\n[Result] Duration: {}ms, Cost: ${:.4}\n",
                    result.duration_ms,
                    result.total_cost_usd.unwrap_or(0.0)
                );
            }
            _ => {}
        }
    }
    drop(stream); // Release borrow before next query

    // Third query
    println!("Query 3: Tell me a short joke");
    client.query("Tell me a short joke").await?;

    let mut stream = client.receive_response();
    while let Some(message) = stream.next().await {
        match message? {
            Message::Assistant(msg) => {
                for block in msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        println!("Claude: {}", text.text);
                    }
                }
            }
            Message::Result(result) => {
                println!(
                    "\n[Result] Duration: {}ms, Cost: ${:.4}\n",
                    result.duration_ms,
                    result.total_cost_usd.unwrap_or(0.0)
                );
            }
            _ => {}
        }
    }
    drop(stream); // Release borrow before disconnect

    // Clean disconnect (analogous to Python's `async with` exit)
    println!("Disconnecting...");
    client.disconnect().await?;
    println!("Disconnected!");

    Ok(())
}
