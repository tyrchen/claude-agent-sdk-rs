//! Example 18: Maximum Budget Control
//!
//! This example demonstrates the use of the `max_budget_usd` option, which allows
//! you to set a spending limit for a conversation. This is crucial for production
//! applications to prevent runaway costs.
//!
//! What it does:
//! 1. Configures a budget limit (e.g., $1.00)
//! 2. Sends a query to Claude
//! 3. Claude will automatically stop if the conversation exceeds the budget
//!
//! This feature helps with:
//! - Cost control in production environments
//! - Testing with budget constraints
//! - Preventing unexpected API charges

use claude_agent_sdk_rs::{ClaudeAgentOptions, ContentBlock, Message, PermissionMode, query};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Example 18: Maximum Budget Control ===\n");

    // Configure with a budget limit
    let budget = 1.0; // $1.00 maximum
    let options = ClaudeAgentOptions::builder()
        .model("sonnet".to_string()) // Use Sonnet for lower cost
        .max_budget_usd(budget)
        .permission_mode(PermissionMode::BypassPermissions)
        .max_turns(5)
        .build();

    println!("Configured with:");
    println!("  Maximum budget: ${:.2}", budget);
    println!("  Max turns: 5\n");
    println!("Asking Claude a question...\n");

    // Query Claude with a simple task
    let messages = query(
        "Explain what recursion is in programming. Keep it brief.",
        Some(options),
    )
    .await?;

    // Process messages
    for message in &messages {
        match message {
            Message::Assistant(msg) => {
                for block in &msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        println!("Claude: {}", text.text);
                    }
                }
            }
            Message::Result(result) => {
                println!("\n=== Result ===");
                println!("Duration: {}ms", result.duration_ms);
                println!("Turns: {}", result.num_turns);
                println!("Error: {}", result.is_error);

                if let Some(cost) = result.total_cost_usd {
                    println!("Total Cost: ${:.4}", cost);
                    println!("Budget: ${:.2}", budget);
                    println!("Budget Used: {:.1}%", (cost / budget) * 100.0);

                    if cost < budget {
                        println!("✓ Stayed within budget!");
                    } else {
                        println!("⚠ Budget exceeded - conversation may have been stopped");
                    }
                }
            }
            _ => {}
        }
    }

    println!("\n=== Budget Control Benefits ===");
    println!("1. Prevents runaway costs in production");
    println!("2. Enables safe experimentation with known costs");
    println!("3. Helps with cost allocation across different tasks");
    println!("4. Automatically stops conversations that exceed limits");

    Ok(())
}
