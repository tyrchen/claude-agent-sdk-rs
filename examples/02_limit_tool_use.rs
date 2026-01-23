//! Example 2: Limit Tool Use
//!
//! This example demonstrates how to restrict which tools Claude can use.
//! By not allowing the Edit tool, Claude will be unable to modify code,
//! demonstrating the permission system.
//!
//! What it does:
//! 1. Asks Claude to write Python code
//! 2. Only allows the Write tool (not Edit)
//! 3. Shows that Claude can create files but cannot edit them

use claude_agent_sdk_rs::{ClaudeAgentOptions, ContentBlock, Message, query};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Example 2: Limit Tool Use ===\n");

    // Create output directory
    std::fs::create_dir_all("./fixtures")?;
    if std::path::Path::new("./fixtures/calculator.py").exists() {
        println!("Removing existing ./fixtures/calculator.py file\n");
        std::fs::remove_file("./fixtures/calculator.py")?;
    }

    println!("Test 1: With Write tool - should succeed\n");
    println!("--------------------------------------------------------");

    // Configure options to only allow Write tool
    // Note: Use `tools` to restrict available tools, not `allowed_tools`
    // - tools: Limits which tools Claude can use (maps to --tools CLI flag)
    // - allowed_tools: Adds extra tool permissions for MCP tools (maps to --allowedTools CLI flag)
    let options = ClaudeAgentOptions {
        tools: Some(["Write"].into()),
        model: Some("sonnet".to_string()), // Use Sonnet for lower cost
        permission_mode: Some(claude_agent_sdk_rs::PermissionMode::AcceptEdits),
        max_turns: Some(3),
        ..Default::default()
    };

    // Query Claude
    let messages = query(
        "Create a simple calculator.py file with add and subtract functions in ./fixtures/",
        Some(options),
    )
    .await?;

    // Process messages
    let mut tool_uses = Vec::new();
    for message in &messages {
        match message {
            Message::Assistant(msg) => {
                for block in &msg.message.content {
                    match block {
                        ContentBlock::Text(text) => {
                            println!("Claude: {}", text.text);
                        }
                        ContentBlock::ToolUse(tool) => {
                            println!("Tool used: {}", tool.name);
                            tool_uses.push(tool.name.clone());
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
            }
            _ => {}
        }
    }

    println!("\n--------------------------------------------------------");
    println!("Tools used: {:?}", tool_uses);

    // Check if file was created and record its modification time
    let file_path = std::path::Path::new("./fixtures/calculator.py");
    let original_modified_time = if file_path.exists() {
        println!("✓ File created successfully with Write tool");
        Some(std::fs::metadata(file_path)?.modified()?)
    } else {
        println!("✗ File was not created");
        None
    };

    println!("\n\nTest 2: Without Write tool - attempt to modify existing file\n");
    println!("--------------------------------------------------------");

    // Now try to edit the file without Edit or Write tool
    // Using `tools` to specify only Read
    let options2 = ClaudeAgentOptions {
        tools: Some(["Read"].into()),
        model: Some("sonnet".to_string()), // Use Sonnet for lower cost
        permission_mode: Some(claude_agent_sdk_rs::PermissionMode::AcceptEdits),
        max_turns: Some(3),
        ..Default::default()
    };

    let messages2 = query(
        "Read ./fixtures/calculator.py and add a multiply function to it",
        Some(options2),
    )
    .await?;

    let mut tool_uses2 = Vec::new();
    let mut claude_response = String::new();

    for message in &messages2 {
        if let Message::Assistant(msg) = message {
            for block in &msg.message.content {
                match block {
                    ContentBlock::Text(text) => {
                        claude_response.push_str(&text.text);
                        claude_response.push('\n');
                    }
                    ContentBlock::ToolUse(tool) => {
                        tool_uses2.push(tool.name.clone());
                    }
                    _ => {}
                }
            }
        }
    }

    println!("Claude's response:\n{}", claude_response);
    println!("\n--------------------------------------------------------");
    println!("Tools used: {:?}", tool_uses2);

    // Verify file content doesn't contain "multiply"
    let file_content = std::fs::read_to_string(file_path)?;
    if file_content.to_lowercase().contains("multiply") {
        println!("✗ UNEXPECTED: File contains 'multiply' - it was modified!");
    } else {
        println!("✓ CORRECT: File does not contain 'multiply' (unchanged)");
    }

    // Verify file was not modified by checking the last modified time
    if let Some(original_time) = original_modified_time {
        let current_modified_time = std::fs::metadata(file_path)?.modified()?;
        if current_modified_time == original_time {
            println!("✓ CORRECT: File modification time unchanged");
        } else {
            println!("✗ UNEXPECTED: File modification time changed!");
        }
    }

    Ok(())
}
