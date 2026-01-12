//! Example: Efficiency Hooks
//!
//! This example demonstrates how to enable built-in efficiency hooks that help
//! improve agent execution by:
//! - Injecting working directory reminders at prompt submission
//! - Tracking execution metrics (edits per file, build attempts, etc.)
//! - Providing efficiency feedback and warnings when execution stops
//!
//! Run with: cargo run --example 24_efficiency_hooks

use claude_agent_sdk_rs::{
    ClaudeAgentOptions, ClaudeClient, EfficiencyConfig, Message, PermissionMode,
};
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Example 1: Enable all efficiency features
    println!("=== Example 1: All efficiency features enabled ===\n");

    let options = ClaudeAgentOptions::builder()
        .permission_mode(PermissionMode::BypassPermissions)
        .max_turns(5)
        // Enable all efficiency features: CWD reminder, metrics tracking, stop tips
        .efficiency(EfficiencyConfig::enabled())
        .build();

    println!("Efficiency config: {:?}", options.efficiency);

    // Example 2: Only CWD reminder (lightweight)
    println!("\n=== Example 2: CWD reminder only ===\n");

    let _options_cwd_only = ClaudeAgentOptions::builder()
        .permission_mode(PermissionMode::BypassPermissions)
        .efficiency(EfficiencyConfig::cwd_reminder_only())
        .build();

    // Example 3: Metrics tracking with stop tips
    println!("=== Example 3: Metrics + stop tips ===\n");

    let _options_metrics = ClaudeAgentOptions::builder()
        .permission_mode(PermissionMode::BypassPermissions)
        .efficiency(EfficiencyConfig::with_metrics())
        .build();

    // Example 4: Custom CWD path
    println!("=== Example 4: Custom CWD path ===\n");

    let _options_custom_cwd = ClaudeAgentOptions::builder()
        .permission_mode(PermissionMode::BypassPermissions)
        .efficiency(EfficiencyConfig::enabled().with_cwd("/custom/project/path"))
        .build();

    // Example 5: Actually run with efficiency hooks
    println!("=== Example 5: Running with efficiency hooks ===\n");

    let mut client = ClaudeClient::new(options);
    client.connect().await?;

    // The UserPromptSubmit hook will inject CWD reminder
    client
        .query("What is the current working directory? Also, what is 2+2?")
        .await?;

    // Process responses - you'll see efficiency-related messages in the output
    let mut stream = client.receive_response();
    while let Some(result) = stream.next().await {
        match result? {
            Message::Assistant(_msg) => {
                println!("Assistant message received");
            }
            Message::Result(result) => {
                println!(
                    "\nExecution complete. Cost: ${:.4}, Turns: {}",
                    result.total_cost_usd.unwrap_or(0.0),
                    result.num_turns
                );
                // The Stop hook would have injected efficiency feedback here
                break;
            }
            _ => {}
        }
    }
    drop(stream);

    client.disconnect().await?;

    println!("\n=== Efficiency Hooks Summary ===");
    println!("The SDK automatically injected:");
    println!("1. UserPromptSubmit: CWD reminder and efficiency tips");
    println!("2. PostToolUse: Metrics collection (tracked edits, builds, etc.)");
    println!("3. Stop: Efficiency feedback based on collected metrics");

    Ok(())
}
