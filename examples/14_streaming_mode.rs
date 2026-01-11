//! Comprehensive examples of using ClaudeClient for streaming mode.
//!
//! This file demonstrates various patterns for building applications with
//! the ClaudeClient streaming interface.
//!
//! The queries are intentionally simplistic. In reality, a query can be a more
//! complex task that Claude SDK uses its agentic capabilities and tools (e.g. run
//! bash commands, edit files, search the web, fetch web content) to accomplish.
//!
//! Usage:
//! cargo run --example 14_streaming_mode - List the examples
//! cargo run --example 14_streaming_mode all - Run all examples
//! cargo run --example 14_streaming_mode basic_streaming - Run a specific example

use claude_agent_sdk_rs::{
    ClaudeAgentOptions, ClaudeClient, ContentBlock, Message, ToolResultContent, Tools,
};
use futures::StreamExt;
use std::env;
use std::time::Duration;

/// Standardized message display function.
///
/// - UserMessage: "User: <content>"
/// - AssistantMessage: "Claude: <content>"
/// - SystemMessage: ignored
/// - ResultMessage: "Result ended" + cost if available
fn display_message(msg: &Message) {
    match msg {
        Message::User(user_msg) => {
            if let Some(ref content) = user_msg.content {
                for block in content {
                    if let ContentBlock::Text(text) = block {
                        println!("User: {}", text.text);
                    }
                }
            } else if let Some(ref text) = user_msg.text {
                println!("User: {}", text);
            }
        }
        Message::Assistant(assistant_msg) => {
            for block in &assistant_msg.message.content {
                if let ContentBlock::Text(text) = block {
                    println!("Claude: {}", text.text);
                }
            }
        }
        Message::System(_) => {
            // Ignore system messages
        }
        Message::Result(_) => {
            println!("Result ended");
        }
        _ => {}
    }
}

/// Basic streaming with context manager.
async fn example_basic_streaming() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Basic Streaming Example ===");

    let mut client = ClaudeClient::new(ClaudeAgentOptions::default());
    client.connect().await?;

    println!("User: What is 2+2?");
    client.query("What is 2+2?").await?;

    // Receive complete response using the helper method
    let mut stream = client.receive_response();
    while let Some(message) = stream.next().await {
        display_message(&message?);
    }
    drop(stream);

    client.disconnect().await?;

    println!("\n");
    Ok(())
}

/// Multi-turn conversation using receive_response helper.
async fn example_multi_turn_conversation() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Multi-Turn Conversation Example ===");

    let mut client = ClaudeClient::new(ClaudeAgentOptions::default());
    client.connect().await?;

    // First turn
    println!("User: What's the capital of France?");
    client.query("What's the capital of France?").await?;

    // Extract and print response
    let mut stream = client.receive_response();
    while let Some(message) = stream.next().await {
        display_message(&message?);
    }
    drop(stream);

    // Second turn - follow-up
    println!("\nUser: What's the population of that city?");
    client.query("What's the population of that city?").await?;

    let mut stream = client.receive_response();
    while let Some(message) = stream.next().await {
        display_message(&message?);
    }
    drop(stream);

    client.disconnect().await?;

    println!("\n");
    Ok(())
}

/// Use ClaudeAgentOptions to configure the client.
async fn example_with_options() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Custom Options Example ===");

    // Configure options
    // Note: Use `tools` to restrict available tools (not `allowed_tools`)
    let options = ClaudeAgentOptions {
        tools: Some(Tools::List(vec!["Read".to_string(), "Write".to_string()])), // Allow file operations
        model: Some("sonnet".to_string()), // Use Sonnet for lower cost
        system_prompt: Some("You are a helpful coding assistant.".into()),
        ..Default::default()
    };

    let mut client = ClaudeClient::new(options);
    client.connect().await?;

    println!("User: Create a simple hello.txt file with a greeting message");
    client
        .query("Create a simple hello.txt file with a greeting message")
        .await?;

    let mut tool_uses = Vec::new();
    let mut stream = client.receive_response();
    while let Some(message) = stream.next().await {
        let msg = message?;
        if let Message::Assistant(ref assistant_msg) = msg {
            display_message(&msg);
            for block in &assistant_msg.message.content {
                if let ContentBlock::ToolUse(tool_use) = block {
                    tool_uses.push(tool_use.name.clone());
                }
            }
        } else {
            display_message(&msg);
        }
    }
    drop(stream);

    if !tool_uses.is_empty() {
        println!("Tools used: {}", tool_uses.join(", "));
    }

    client.disconnect().await?;

    println!("\n");
    Ok(())
}

/// Manually handle message stream for custom logic.
async fn example_manual_message_handling() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Manual Message Handling Example ===");

    let mut client = ClaudeClient::new(ClaudeAgentOptions::default());
    client.connect().await?;

    client
        .query("List 5 programming languages and their main use cases")
        .await?;

    // Manually process messages with custom logic
    let mut languages_found = Vec::new();
    let language_list = vec!["Python", "JavaScript", "Java", "C++", "Go", "Rust", "Ruby"];

    let mut stream = client.receive_messages();
    while let Some(message) = stream.next().await {
        let msg = message?;
        match msg {
            Message::Assistant(assistant_msg) => {
                for block in assistant_msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        println!("Claude: {}", text.text);
                        // Custom logic: extract language names
                        for lang in &language_list {
                            if text.text.contains(lang)
                                && !languages_found.contains(&lang.to_string())
                            {
                                languages_found.push(lang.to_string());
                                println!("Found language: {}", lang);
                            }
                        }
                    }
                }
            }
            Message::Result(_) => {
                display_message(&msg);
                println!("Total languages mentioned: {}", languages_found.len());
                break;
            }
            _ => {}
        }
    }
    drop(stream);

    client.disconnect().await?;

    println!("\n");
    Ok(())
}

/// Example showing tool use blocks when running bash commands.
async fn example_bash_command() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Bash Command Example ===");

    let mut client = ClaudeClient::new(ClaudeAgentOptions::default());
    client.connect().await?;

    println!("User: Run a bash echo command");
    client
        .query("Run a bash echo command that says 'Hello from bash!'")
        .await?;

    // Track all message types received
    let mut message_types = Vec::new();

    let mut stream = client.receive_messages();
    while let Some(message) = stream.next().await {
        let msg = message?;
        message_types.push(
            format!("{:?}", msg)
                .split('(')
                .next()
                .unwrap_or("Unknown")
                .to_string(),
        );

        match msg {
            Message::User(user_msg) => {
                // User messages can contain tool results
                if let Some(ref content) = user_msg.content {
                    for block in content {
                        if let ContentBlock::Text(text) = block {
                            println!("User: {}", text.text);
                        } else if let ContentBlock::ToolResult(tool_result) = block {
                            let content_str = match &tool_result.content {
                                Some(ToolResultContent::Text(s)) => s.as_str(),
                                Some(ToolResultContent::Blocks(_)) => "[structured content]",
                                None => "None",
                            };
                            let preview = if content_str.len() > 100 {
                                &content_str[..100]
                            } else {
                                content_str
                            };
                            println!(
                                "Tool Result (id: {}): {}...",
                                tool_result.tool_use_id, preview
                            );
                        }
                    }
                }
            }
            Message::Assistant(assistant_msg) => {
                // Assistant messages can contain tool use blocks
                for block in assistant_msg.message.content {
                    match block {
                        ContentBlock::Text(text) => {
                            println!("Claude: {}", text.text);
                        }
                        ContentBlock::ToolUse(tool_use) => {
                            println!("Tool Use: {} (id: {})", tool_use.name, tool_use.id);
                            if tool_use.name == "Bash"
                                && let Some(command) = tool_use.input.get("command")
                                && let Some(cmd_str) = command.as_str()
                            {
                                println!("  Command: {}", cmd_str);
                            }
                        }
                        _ => {}
                    }
                }
            }
            Message::Result(result_msg) => {
                println!("Result ended");
                if let Some(cost) = result_msg.total_cost_usd {
                    println!("Cost: ${:.4}", cost);
                }
                break;
            }
            _ => {}
        }
    }
    drop(stream);

    // Get unique message types
    let mut unique_types: Vec<_> = message_types.into_iter().collect();
    unique_types.sort();
    unique_types.dedup();
    println!("\nMessage types received: {}", unique_types.join(", "));

    client.disconnect().await?;

    println!("\n");
    Ok(())
}

/// Demonstrate server info and interrupt capabilities.
async fn example_control_protocol() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Control Protocol Example ===");
    println!("Shows server info retrieval and interrupt capability\n");

    let mut client = ClaudeClient::new(ClaudeAgentOptions::default());
    client.connect().await?;

    // 1. Get server initialization info
    println!("1. Getting server info...");
    let server_info = client.get_server_info();

    if let Some(info) = server_info {
        println!("✓ Server info retrieved successfully!");
        if let Some(commands) = info.get("commands")
            && let Some(commands_array) = commands.as_array()
        {
            println!("  - Available commands: {}", commands_array.len());
        }
        if let Some(output_style) = info.get("output_style") {
            println!("  - Output style: {}", output_style);
        }

        // Show available output styles if present
        if let Some(styles) = info.get("available_output_styles")
            && let Some(styles_array) = styles.as_array()
        {
            let style_names: Vec<_> = styles_array.iter().filter_map(|s| s.as_str()).collect();
            println!("  - Available output styles: {}", style_names.join(", "));
        }

        // Show a few example commands
        if let Some(commands) = info.get("commands")
            && let Some(commands_array) = commands.as_array()
        {
            println!("  - Example commands:");
            for cmd in commands_array.iter().take(5) {
                if let Some(name) = cmd.get("name")
                    && let Some(name_str) = name.as_str()
                {
                    println!("    • {}", name_str);
                }
            }
        }
    } else {
        println!("✗ No server info available (may not be in streaming mode)");
    }

    println!("\n2. Testing interrupt capability...");

    // Start a long-running task
    println!("User: Count from 1 to 20 slowly");
    client
        .query("Count from 1 to 20 slowly, pausing between each number")
        .await?;

    // Start consuming messages in background to enable interrupt
    let mut messages = Vec::new();
    let mut stream = client.receive_response();

    // Read a few messages
    for _ in 0..3 {
        if let Some(message) = stream.next().await {
            let msg = message?;
            if let Message::Assistant(ref assistant_msg) = msg {
                for block in &assistant_msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        // Print first 50 chars to show progress
                        let preview = if text.text.len() > 50 {
                            &text.text[..50]
                        } else {
                            &text.text
                        };
                        println!("Claude: {}...", preview);
                        break;
                    }
                }
            }
            messages.push(msg);
        }
    }
    drop(stream);

    // Wait a moment then interrupt
    tokio::time::sleep(Duration::from_secs(2)).await;
    println!("\n[Sending interrupt after 2 seconds...]");

    match client.interrupt().await {
        Ok(_) => println!("✓ Interrupt sent successfully"),
        Err(e) => println!("✗ Interrupt failed: {}", e),
    }

    // Wait for interrupt to process
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Send new query after interrupt
    println!("\nUser: Just say 'Hello!'");
    client.query("Just say 'Hello!'").await?;

    let mut stream = client.receive_response();
    while let Some(message) = stream.next().await {
        let msg = message?;
        if let Message::Assistant(ref assistant_msg) = msg {
            for block in &assistant_msg.message.content {
                if let ContentBlock::Text(text) = block {
                    println!("Claude: {}", text.text);
                }
            }
        }
    }
    drop(stream);

    client.disconnect().await?;

    println!("\n");
    Ok(())
}

/// Demonstrate proper error handling.
async fn example_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Error Handling Example ===");

    let mut client = ClaudeClient::new(ClaudeAgentOptions::default());

    match client.connect().await {
        Ok(_) => {
            // Send a message that will take time to process
            println!("User: Run a bash sleep command for 60 seconds not in the background");
            client
                .query("Run a bash sleep command for 60 seconds not in the background")
                .await?;

            // Try to receive response with a short timeout
            let mut messages = Vec::new();
            let timeout_duration = Duration::from_secs(10);

            match tokio::time::timeout(timeout_duration, async {
                let mut stream = client.receive_response();
                while let Some(message) = stream.next().await {
                    let msg = message?;
                    if let Message::Assistant(ref assistant_msg) = msg {
                        for block in &assistant_msg.message.content {
                            if let ContentBlock::Text(text) = block {
                                let preview = if text.text.len() > 50 {
                                    &text.text[..50]
                                } else {
                                    &text.text
                                };
                                println!("Claude: {}...", preview);
                            }
                        }
                    } else if let Message::Result(_) = msg {
                        display_message(&msg);
                    }
                    messages.push(msg);
                }
                Ok::<(), Box<dyn std::error::Error>>(())
            })
            .await
            {
                Ok(result) => {
                    result?;
                }
                Err(_) => {
                    println!(
                        "\nResponse timeout after 10 seconds - demonstrating graceful handling"
                    );
                    println!("Received {} messages before timeout", messages.len());
                }
            }
        }
        Err(e) => {
            println!("Connection error: {}", e);
        }
    }

    // Always disconnect
    client.disconnect().await?;

    println!("\n");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        // List available examples
        println!("Usage: cargo run --example 14_streaming_mode <example_name>");
        println!("\nAvailable examples:");
        println!("  all - Run all examples");
        println!("  basic_streaming");
        println!("  multi_turn_conversation");
        println!("  with_options");
        println!("  manual_message_handling");
        println!("  bash_command");
        println!("  control_protocol");
        println!("  error_handling");
        return Ok(());
    }

    let example_name = &args[1];

    match example_name.as_str() {
        "all" => {
            // Run all examples
            println!("\n=== Running: basic_streaming ===\n");
            example_basic_streaming().await?;
            println!("{}\n", "-".repeat(50));

            println!("\n=== Running: multi_turn_conversation ===\n");
            example_multi_turn_conversation().await?;
            println!("{}\n", "-".repeat(50));

            println!("\n=== Running: with_options ===\n");
            example_with_options().await?;
            println!("{}\n", "-".repeat(50));

            println!("\n=== Running: manual_message_handling ===\n");
            example_manual_message_handling().await?;
            println!("{}\n", "-".repeat(50));

            println!("\n=== Running: bash_command ===\n");
            example_bash_command().await?;
            println!("{}\n", "-".repeat(50));

            println!("\n=== Running: control_protocol ===\n");
            example_control_protocol().await?;
            println!("{}\n", "-".repeat(50));

            println!("\n=== Running: error_handling ===\n");
            example_error_handling().await?;
            println!("{}\n", "-".repeat(50));
        }
        "basic_streaming" => example_basic_streaming().await?,
        "multi_turn_conversation" => example_multi_turn_conversation().await?,
        "with_options" => example_with_options().await?,
        "manual_message_handling" => example_manual_message_handling().await?,
        "bash_command" => example_bash_command().await?,
        "control_protocol" => example_control_protocol().await?,
        "error_handling" => example_error_handling().await?,
        _ => {
            println!("Error: Unknown example '{}'", example_name);
            println!("\nAvailable examples:");
            println!("  all - Run all examples");
            println!("  basic_streaming");
            println!("  multi_turn_conversation");
            println!("  with_options");
            println!("  manual_message_handling");
            println!("  bash_command");
            println!("  control_protocol");
            println!("  error_handling");
            std::process::exit(1);
        }
    }

    Ok(())
}
