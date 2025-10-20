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

use claude_agent_sdk::{query, ClaudeAgentOptions, ContentBlock, Message};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Example 2: Limit Tool Use ===\n");

    // Create output directory
    std::fs::create_dir_all("./fixtures")?;

    println!("Test 1: With Write tool - should succeed\n");
    println!("--------------------------------------------------------");

    // Configure options to only allow Write tool
    let options = ClaudeAgentOptions {
        allowed_tools: vec!["Write".to_string()],
        permission_mode: Some(claude_agent_sdk::PermissionMode::AcceptEdits),
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

    // Check if file was created
    if std::path::Path::new("./fixtures/calculator.py").exists() {
        println!("✓ File created successfully with Write tool");
    } else {
        println!("✗ File was not created");
    }

    println!("\n\nTest 2: Without Edit tool - attempt to modify existing file\n");
    println!("--------------------------------------------------------");

    // Now try to edit the file without Edit tool
    let options2 = ClaudeAgentOptions {
        allowed_tools: vec!["Write".to_string(), "Read".to_string()],
        disallowed_tools: vec!["Edit".to_string()],
        permission_mode: Some(claude_agent_sdk::PermissionMode::AcceptEdits),
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

    if tool_uses2.contains(&"Edit".to_string()) {
        println!("✗ UNEXPECTED: Edit tool was used despite being disallowed!");
    } else {
        println!("✓ CORRECT: Edit tool was not used (as expected)");
        if claude_response.to_lowercase().contains("edit")
            || claude_response.to_lowercase().contains("modify")
            || claude_response.to_lowercase().contains("cannot")
        {
            println!("✓ Claude acknowledged the limitation");
        }
    }

    Ok(())
}
