//! Example 19: Maximum Thinking Tokens
//!
//! This example demonstrates the use of the `max_thinking_tokens` option, which controls
//! the maximum number of tokens Claude can use for "extended thinking" before responding.
//!
//! What it does:
//! 1. Configures a limit on thinking tokens (e.g., 1000 tokens)
//! 2. Asks Claude a question that might benefit from extended thinking
//! 3. Claude will use thinking blocks up to the specified limit
//!
//! Extended thinking allows Claude to reason through problems step-by-step before
//! providing a final answer. The thinking process is visible in ThinkingBlock content.
//!
//! Use cases:
//! - Complex problem solving
//! - Mathematical reasoning
//! - Multi-step analysis
//! - Cost control for thinking overhead

use claude_agent_sdk_rs::{query, ClaudeAgentOptions, ContentBlock, Message, PermissionMode};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Example 19: Maximum Thinking Tokens ===\n");

    // Configure with thinking token limit
    let max_thinking = 1000;
    let options = ClaudeAgentOptions::builder()
        .max_thinking_tokens(max_thinking)
        .permission_mode(PermissionMode::BypassPermissions)
        .max_turns(3)
        .build();

    println!("Configured with:");
    println!("  Max thinking tokens: {}", max_thinking);
    println!("  Max turns: 3\n");
    println!("Asking Claude a problem that benefits from thinking...\n");

    // Ask a question that might benefit from extended thinking
    let messages = query(
        "If a train travels 60 miles per hour and needs to cover 180 miles, \
         but stops for 15 minutes halfway through, how long will the total journey take?",
        Some(options),
    )
    .await?;

    // Process messages
    let mut found_thinking = false;
    let mut found_answer = false;

    for message in &messages {
        match message {
            Message::Assistant(msg) => {
                for block in &msg.message.content {
                    match block {
                        ContentBlock::Thinking(thinking) => {
                            found_thinking = true;
                            println!("=== Thinking Process ===");
                            println!("{}", thinking.thinking);
                            println!("Signature: {}", thinking.signature);
                            println!();
                        }
                        ContentBlock::Text(text) => {
                            found_answer = true;
                            println!("=== Final Answer ===");
                            println!("{}", text.text);
                            println!();
                        }
                        _ => {}
                    }
                }
            }
            Message::Result(result) => {
                println!("=== Result ===");
                println!("Duration: {}ms", result.duration_ms);
                println!("Turns: {}", result.num_turns);
                if let Some(cost) = result.total_cost_usd {
                    println!("Cost: ${:.4}", cost);
                }

                if let Some(ref usage) = result.usage {
                    println!("\nToken Usage:");
                    println!("{}", serde_json::to_string_pretty(usage)?);
                }
            }
            _ => {}
        }
    }

    println!("\n=== Summary ===");
    if found_thinking {
        println!("✓ Extended thinking was used");
        println!("  The thinking process shows Claude's reasoning steps");
    } else {
        println!("ℹ No thinking blocks observed");
        println!("  (Thinking may not be used for all models/queries)");
    }

    if found_answer {
        println!("✓ Final answer provided");
    }

    println!("\n=== About Extended Thinking ===");
    println!("- Allows Claude to reason through problems step-by-step");
    println!("- Thinking content is visible in ThinkingBlock messages");
    println!("- max_thinking_tokens controls the limit on thinking overhead");
    println!("- Useful for complex problems requiring multi-step reasoning");
    println!("- Thinking tokens are separate from regular response tokens");

    Ok(())
}
