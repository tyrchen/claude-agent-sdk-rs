//! Example 3: Monitor Tool Use
//!
//! This example demonstrates how to monitor which tools Claude uses
//! by inspecting the message stream.
//!
//! What it does:
//! 1. Asks Claude to perform a multi-step task
//! 2. Tracks all tool uses
//! 3. Prints detailed information about each tool invocation

use claude_agent_sdk::{query, ClaudeAgentOptions, ContentBlock, Message};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Example 3: Monitor Tool Use ===\n");

    // Create output directory
    std::fs::create_dir_all("./fixtures")?;

    // Configure options
    let options = ClaudeAgentOptions {
        allowed_tools: vec!["Write".to_string(), "Read".to_string(), "Bash".to_string()],
        permission_mode: Some(claude_agent_sdk::PermissionMode::AcceptEdits),
        max_turns: Some(10),
        stderr_callback: Some(std::sync::Arc::new(|msg| {
            if !msg.trim().is_empty() && !msg.contains("STDERR:") {
                eprintln!("CLI: {}", msg.trim());
            }
        })),
        ..Default::default()
    };

    println!("Asking Claude to create and test a simple Python function...\n");
    println!("========================================================\n");

    // Query Claude
    let messages = query(
        "Create a Python file at ./fixtures/math_utils.py with a function that calculates factorial. Then create a test file and run it.",
        Some(options),
    )
    .await?;

    // Track tool usage
    let mut tool_usage: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
    let mut turn_number = 0;

    // Process messages
    for message in &messages {
        match message {
            Message::Assistant(msg) => {
                turn_number += 1;
                println!("--- Turn {} ---", turn_number);

                for block in &msg.message.content {
                    match block {
                        ContentBlock::Text(text) => {
                            println!("💬 Claude: {}", text.text);
                        }
                        ContentBlock::ToolUse(tool) => {
                            println!("🔧 Tool: {}", tool.name);
                            println!("   ID: {}", tool.id);
                            println!("   Input: {}", serde_json::to_string_pretty(&tool.input)?);

                            // Track usage
                            tool_usage
                                .entry(tool.name.clone())
                                .or_default()
                                .push(tool.input.clone());
                        }
                        ContentBlock::Thinking(thinking) => {
                            println!("🤔 Thinking: {}", thinking.thinking);
                        }
                        _ => {}
                    }
                }
                println!();
            }
            Message::System(sys) => {
                if sys.subtype == "init" {
                    println!("System initialized");
                    if let Some(ref session_id) = sys.session_id {
                        println!("Session ID: {}", session_id);
                    }
                    println!();
                }
            }
            Message::Result(result) => {
                println!("\n========================================================");
                println!("=== Final Result ===");
                println!(
                    "Duration: {}ms ({:.2}s)",
                    result.duration_ms,
                    result.duration_ms as f64 / 1000.0
                );
                println!(
                    "API Duration: {}ms ({:.2}s)",
                    result.duration_api_ms,
                    result.duration_api_ms as f64 / 1000.0
                );
                println!("Turns: {}", result.num_turns);
                println!("Error: {}", result.is_error);
                if let Some(cost) = result.total_cost_usd {
                    println!("Cost: ${:.4}", cost);
                }
                println!("Session ID: {}", result.session_id);

                if let Some(ref result_text) = result.result {
                    println!("Result: {}", result_text);
                }
            }
            _ => {}
        }
    }

    // Print summary
    println!("\n========================================================");
    println!("=== Tool Usage Summary ===\n");

    if tool_usage.is_empty() {
        println!("No tools were used.");
    } else {
        for (tool_name, invocations) in &tool_usage {
            println!("🔧 {} - used {} time(s)", tool_name, invocations.len());
            for (i, input) in invocations.iter().enumerate() {
                println!("   {}. {}", i + 1, serde_json::to_string(input)?);
            }
            println!();
        }
    }

    // Verify files were created
    println!("=== File Verification ===\n");

    let files_to_check = vec!["./fixtures/math_utils.py", "./fixtures/test_math_utils.py"];

    for file_path in &files_to_check {
        if std::path::Path::new(file_path).exists() {
            let size = std::fs::metadata(file_path)?.len();
            println!("✓ {} ({} bytes)", file_path, size);
        } else {
            println!("✗ {} (not found)", file_path);
        }
    }

    Ok(())
}
