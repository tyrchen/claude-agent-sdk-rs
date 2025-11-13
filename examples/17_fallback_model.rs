//! Example 17: Fallback Model Configuration
//!
//! This example demonstrates the use of the `fallback_model` option, which provides
//! a backup model to use if the primary model fails or is unavailable.
//!
//! What it does:
//! 1. Configures a primary model (claude-opus-4) with a fallback (claude-sonnet-4)
//! 2. Sends a simple query to Claude
//! 3. Claude will attempt to use the primary model, falling back to the secondary if needed
//!
//! This feature is important for production applications that need reliability and
//! automatic failover when a model is experiencing issues.

use claude_agent_sdk_rs::{query, ClaudeAgentOptions, ContentBlock, Message, PermissionMode};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Example 17: Fallback Model Configuration ===\n");

    // Configure with both primary and fallback models
    let options = ClaudeAgentOptions::builder()
        .model("claude-opus-4")
        .fallback_model("claude-sonnet-4")
        .permission_mode(PermissionMode::BypassPermissions)
        .max_turns(3)
        .build();

    println!("Configured with:");
    println!("  Primary model: claude-opus-4");
    println!("  Fallback model: claude-sonnet-4\n");
    println!("Asking Claude a simple question...\n");

    // Query Claude
    let messages = query(
        "What is the capital of France? Please answer in one sentence.",
        Some(options),
    )
    .await?;

    // Process messages
    let mut found_text = false;
    for message in &messages {
        match message {
            Message::Assistant(msg) => {
                // Display which model was used
                if let Some(ref model) = msg.message.model {
                    println!("Model used: {}", model);
                }

                for block in &msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        println!("Claude: {}", text.text);
                        found_text = true;
                    }
                }
            }
            Message::Result(result) => {
                println!("\n=== Result ===");
                println!("Duration: {}ms", result.duration_ms);
                println!("Turns: {}", result.num_turns);
                println!("Error: {}", result.is_error);
                if let Some(cost) = result.total_cost_usd {
                    println!("Cost: ${:.4}", cost);
                }
            }
            _ => {}
        }
    }

    if !found_text {
        println!("✗ No text response received");
    } else {
        println!("\n✓ Successfully received response");
        println!("\nNote: The fallback model is used automatically if the primary model");
        println!("is unavailable or encounters an error. Check the 'Model used' field");
        println!("above to see which model actually handled the request.");
    }

    Ok(())
}
