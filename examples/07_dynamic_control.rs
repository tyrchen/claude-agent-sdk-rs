//! Example demonstrating dynamic control methods of ClaudeClient
//!
//! This example shows how to use the dynamic control methods:
//! - interrupt() - Stop Claude mid-execution
//! - set_permission_mode() - Change permission mode on the fly
//! - set_model() - Switch AI models during the session
//!
//! These methods are analogous to Python's ClaudeSDKClient methods.
//!
//! Run with:
//! ```bash
//! cargo run --example 07_dynamic_control
//! ```

use claude_agent_sdk::{ClaudeAgentOptions, ClaudeClient, ContentBlock, Message, PermissionMode};
use futures::StreamExt;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Dynamic Control Example ===\n");

    let options = ClaudeAgentOptions {
        permission_mode: Some(PermissionMode::Default),
        ..Default::default()
    };

    let mut client = ClaudeClient::new(options);
    client.connect().await?;
    println!("Connected!\n");

    // Example 1: Change permission mode dynamically
    println!("--- Example 1: Dynamic Permission Mode ---");
    println!("Current mode: Default");
    println!("Changing to AcceptEdits mode...");
    client
        .set_permission_mode(PermissionMode::AcceptEdits)
        .await?;
    println!("Permission mode changed!\n");

    client.query("What permission mode are you in?").await?;

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
                println!("\n[Result] Duration: {}ms\n", result.duration_ms);
            }
            _ => {}
        }
    }
    drop(stream); // Release borrow

    // Example 2: Change model dynamically (if available)
    println!("--- Example 2: Dynamic Model Switching ---");
    println!("Switching to a different model...");
    client.set_model(Some("claude-sonnet-4-20250514")).await?;
    println!("Model changed!\n");

    client.query("Tell me which model you are").await?;

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
                println!("\n[Result] Duration: {}ms\n", result.duration_ms);
            }
            _ => {}
        }
    }
    drop(stream); // Release borrow

    // Example 3: Interrupt (simulated - starts a long task then interrupts)
    println!("--- Example 3: Interrupt ---");
    println!("Starting a potentially long task...");

    client
        .query("Count from 1 to 100, showing each number")
        .await?;

    // Start a timer to interrupt after 1 second
    let mut interrupt_task = tokio::spawn(async {
        sleep(Duration::from_secs(1)).await;
        true
    });

    let mut stream = client.receive_response();
    let mut interrupted = false;

    // Process messages and check for interrupt
    loop {
        tokio::select! {
            message = stream.next() => {
                match message {
                    Some(Ok(msg)) => {
                        match msg {
                            Message::Assistant(msg) => {
                                for block in msg.message.content {
                                    if let ContentBlock::Text(text) = block {
                                        print!("{}", text.text);
                                    }
                                }
                            }
                            Message::Result(result) => {
                                println!(
                                    "\n[Result] Duration: {}ms, Interrupted: {}\n",
                                    result.duration_ms,
                                    interrupted
                                );
                                break;
                            }
                            _ => {}
                        }
                    }
                    Some(Err(e)) => {
                        eprintln!("Error: {}", e);
                        break;
                    }
                    None => break,
                }
            }
            _ = &mut interrupt_task, if !interrupted => {
                println!("\nSending interrupt signal...");
                if let Err(e) = client.interrupt().await {
                    eprintln!("Failed to interrupt: {}", e);
                } else {
                    println!("Interrupt sent!");
                    interrupted = true;
                }
            }
        }
    }
    drop(stream); // Release borrow before disconnect

    // Clean disconnect
    println!("Disconnecting...");
    client.disconnect().await?;
    println!("Done!");

    Ok(())
}
