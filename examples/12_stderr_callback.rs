//! Example 12: Stderr Callback
//!
//! Simple example demonstrating stderr callback for capturing CLI debug output.
//!
//! This example shows how to:
//! 1. Set up a callback to capture stderr output from the CLI
//! 2. Enable debug mode to see CLI internal messages
//! 3. Filter and process stderr messages

use claude_agent_sdk_rs::{query, ClaudeAgentOptions, ContentBlock, Message};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Stderr Callback Example ===\n");

    // Collect stderr messages
    let stderr_messages = Arc::new(Mutex::new(Vec::new()));
    let stderr_messages_clone = stderr_messages.clone();

    // Create stderr callback
    let stderr_callback = move |message: String| {
        stderr_messages_clone.lock().unwrap().push(message.clone());

        // Optionally print specific messages
        if message.contains("[ERROR]") {
            println!("Error detected: {}", message);
        }
    };

    // Create extra args to enable debug output
    let mut extra_args = HashMap::new();
    extra_args.insert("debug-to-stderr".to_string(), None);

    // Create options with stderr callback and enable debug mode
    let options = ClaudeAgentOptions {
        stderr_callback: Some(Arc::new(stderr_callback)),
        extra_args,
        max_turns: Some(3),
        ..Default::default()
    };

    // Run a query
    println!("Running query with stderr capture...\n");

    let messages = query("What is 2+2?", Some(options)).await?;

    // Display response
    for message in &messages {
        if let Message::Assistant(msg) = message {
            for block in &msg.message.content {
                if let ContentBlock::Text(text) = block {
                    println!("Response: {}", text.text);
                }
            }
        }
    }

    // Show what we captured
    let captured = stderr_messages.lock().unwrap();
    println!("\nCaptured {} stderr lines", captured.len());
    if !captured.is_empty() {
        println!(
            "First stderr line: {}",
            &captured[0][..100.min(captured[0].len())]
        );
    }

    Ok(())
}
