//! Example 1: Basic Hello World
//!
//! This example demonstrates basic usage of the Claude Agent SDK to write
//! a simple Python "Hello, World!" program.
//!
//! What it does:
//! 1. Asks Claude to write a Python hello world script
//! 2. Saves it to ./fixtures/hello.py
//! 3. Runs the script to verify it works

use claude_agent_sdk_rs::{query, ClaudeAgentOptions, ContentBlock, Message};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Example 1: Hello World ===\n");

    // Create output directory
    std::fs::create_dir_all("./fixtures")?;

    // Configure options to allow Write tool
    let options = ClaudeAgentOptions {
        allowed_tools: vec!["Write".to_string()],
        permission_mode: Some(claude_agent_sdk_rs::PermissionMode::AcceptEdits),
        max_turns: Some(5),
        stderr_callback: Some(std::sync::Arc::new(|msg| {
            eprintln!("STDERR: {}", msg);
        })),
        ..Default::default()
    };

    println!("Asking Claude to write a Python hello world script...\n");

    // Query Claude
    let messages = query(
        "Write a simple Python hello world script to ./fixtures/hello.py",
        Some(options),
    )
    .await?;

    // Process messages
    for message in &messages {
        match message {
            Message::Assistant(msg) => {
                for block in &msg.message.content {
                    match block {
                        ContentBlock::Text(text) => {
                            println!("Claude: {}", text.text);
                        }
                        ContentBlock::ToolUse(tool) => {
                            println!("Tool use: {} ({})", tool.name, tool.id);
                        }
                        _ => {}
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
                println!("Session ID: {}", result.session_id);
            }
            _ => {}
        }
    }

    // Verify the file was created
    let filepath = "./fixtures/hello.py";
    if std::path::Path::new(filepath).exists() {
        println!("\n✓ File created: {}", filepath);

        // Read and display the file
        let content = std::fs::read_to_string(filepath)?;
        println!("\nFile contents:");
        println!("---");
        println!("{}", content);
        println!("---");

        // Try to run it
        println!("\nRunning the script...");
        let output = std::process::Command::new("python3")
            .arg(filepath)
            .output()?;

        if output.status.success() {
            println!("✓ Script executed successfully!");
            println!("Output: {}", String::from_utf8_lossy(&output.stdout));
        } else {
            println!("✗ Script failed to execute");
            println!("Error: {}", String::from_utf8_lossy(&output.stderr));
        }
    } else {
        println!("\n✗ File was not created");
    }

    Ok(())
}
