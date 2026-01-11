//! Example 15: Comprehensive Hooks Example
//!
//! This example demonstrates ALL hook types with callbacks:
//! 1. PreToolUse - Block/allow tool execution with permission decisions
//! 2. PostToolUse - Review tool output and add context
//! 3. UserPromptSubmit - Add custom instructions at prompt submission
//! 4. Decision fields - Use permissionDecision='allow'/'deny' with reason
//! 5. Continue control - Stop execution using continue=false
//!
//! Usage:
//!   cargo run --example 15_hooks_comprehensive               # List examples
//!   cargo run --example 15_hooks_comprehensive all           # Run all examples
//!   cargo run --example 15_hooks_comprehensive PreToolUse    # Run specific example

use claude_agent_sdk_rs::{
    ClaudeAgentOptions, ClaudeClient, ContentBlock, HookContext, HookEvent, HookInput,
    HookJsonOutput, HookMatcher, HookSpecificOutput, Message, PostToolUseHookSpecificOutput,
    PreToolUseHookSpecificOutput, SyncHookJsonOutput, Tools, UserPromptSubmitHookSpecificOutput,
};
use futures::StreamExt;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;

/// PreToolUse hook that blocks dangerous bash commands
async fn check_bash_command(
    input: HookInput,
    _tool_use_id: Option<String>,
    _context: HookContext,
) -> HookJsonOutput {
    match input {
        HookInput::PreToolUse(pre_tool) => {
            if pre_tool.tool_name != "Bash" {
                return HookJsonOutput::Sync(SyncHookJsonOutput::default());
            }

            let command = pre_tool
                .tool_input
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let block_patterns = vec!["foo.sh"];

            for pattern in &block_patterns {
                if command.contains(pattern) {
                    println!("\n🚫 ===== Hook BLOCKED Command =====");
                    println!("Tool: {}", pre_tool.tool_name);
                    println!("Command: {}", command);
                    println!("Reason: Contains invalid pattern '{}'", pattern);
                    println!("====================================\n");

                    return HookJsonOutput::Sync(SyncHookJsonOutput {
                        hook_specific_output: Some(HookSpecificOutput::PreToolUse(
                            PreToolUseHookSpecificOutput {
                                permission_decision: Some("deny".to_string()),
                                permission_decision_reason: Some(format!(
                                    "Command contains invalid pattern: {}",
                                    pattern
                                )),
                                ..Default::default()
                            },
                        )),
                        ..Default::default()
                    });
                }
            }

            HookJsonOutput::Sync(SyncHookJsonOutput::default())
        }
        _ => HookJsonOutput::Sync(SyncHookJsonOutput::default()),
    }
}

/// UserPromptSubmit hook that adds custom instructions
async fn add_custom_instructions(
    input: HookInput,
    _tool_use_id: Option<String>,
    _context: HookContext,
) -> HookJsonOutput {
    match input {
        HookInput::UserPromptSubmit(_) => {
            println!("\n📝 ===== UserPromptSubmit Hook Fired =====");
            println!("Adding custom context: 'My favorite color is hot pink'");
            println!("==========================================\n");

            HookJsonOutput::Sync(SyncHookJsonOutput {
                hook_specific_output: Some(HookSpecificOutput::UserPromptSubmit(
                    UserPromptSubmitHookSpecificOutput {
                        additional_context: Some("My favorite color is hot pink.".to_string()),
                    },
                )),
                ..Default::default()
            })
        }
        _ => HookJsonOutput::Sync(SyncHookJsonOutput::default()),
    }
}

/// PostToolUse hook that reviews tool output and provides feedback
async fn review_tool_output(
    input: HookInput,
    _tool_use_id: Option<String>,
    _context: HookContext,
) -> HookJsonOutput {
    match input {
        HookInput::PostToolUse(post_tool) => {
            let tool_output_str = post_tool.tool_response.to_string().to_lowercase();

            // If the tool produced an error, add helpful context
            if tool_output_str.contains("error") {
                println!("\n⚠️  ===== PostToolUse Hook Detected Error =====");
                println!("Tool: {}", post_tool.tool_name);
                println!("Adding context about error...");
                println!("==============================================\n");

                return HookJsonOutput::Sync(SyncHookJsonOutput {
                    system_message: Some("⚠️ The command produced an error".to_string()),
                    reason: Some(
                        "Tool execution failed - consider checking the command syntax".to_string(),
                    ),
                    hook_specific_output: Some(HookSpecificOutput::PostToolUse(
                        PostToolUseHookSpecificOutput {
                            additional_context: Some(
                                "The tool encountered an error during execution".to_string(),
                            ),
                        },
                    )),
                    ..Default::default()
                });
            }

            HookJsonOutput::Sync(SyncHookJsonOutput::default())
        }
        _ => HookJsonOutput::Sync(SyncHookJsonOutput::default()),
    }
}

/// PreToolUse hook that demonstrates strict approval with permissionDecision
async fn strict_approval_hook(
    input: HookInput,
    _tool_use_id: Option<String>,
    _context: HookContext,
) -> HookJsonOutput {
    match input {
        HookInput::PreToolUse(pre_tool) => {
            // Block any Write operations to specific files
            if pre_tool.tool_name == "Write" {
                let file_path = pre_tool
                    .tool_input
                    .get("file_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if file_path.to_lowercase().contains("important") {
                    println!("\n🚫 ===== Strict Approval Hook BLOCKED Write =====");
                    println!("File: {}", file_path);
                    println!("Reason: Security policy blocks writes to important files");
                    println!("=================================================\n");

                    return HookJsonOutput::Sync(SyncHookJsonOutput {
                        reason: Some(
                            "Writes to files containing 'important' in the name are not allowed for safety"
                                .to_string(),
                        ),
                        system_message: Some(
                            "🚫 Write operation blocked by security policy".to_string(),
                        ),
                        hook_specific_output: Some(HookSpecificOutput::PreToolUse(
                            PreToolUseHookSpecificOutput {
                                permission_decision: Some("deny".to_string()),
                                permission_decision_reason: Some(
                                    "Security policy blocks writes to important files".to_string(),
                                ),
                                ..Default::default()
                            },
                        )),
                        ..Default::default()
                    });
                }
            }

            // Allow everything else explicitly
            HookJsonOutput::Sync(SyncHookJsonOutput {
                reason: Some("Tool use approved after security review".to_string()),
                hook_specific_output: Some(HookSpecificOutput::PreToolUse(
                    PreToolUseHookSpecificOutput {
                        permission_decision: Some("allow".to_string()),
                        permission_decision_reason: Some("Tool passed security checks".to_string()),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            })
        }
        _ => HookJsonOutput::Sync(SyncHookJsonOutput::default()),
    }
}

/// PostToolUse hook that stops execution on critical errors
async fn stop_on_error_hook(
    input: HookInput,
    _tool_use_id: Option<String>,
    _context: HookContext,
) -> HookJsonOutput {
    match input {
        HookInput::PostToolUse(post_tool) => {
            let tool_output_str = post_tool.tool_response.to_string().to_lowercase();

            // Stop execution if we see a critical error
            if tool_output_str.contains("critical") {
                println!("\n🛑 ===== Stop Hook Detected Critical Error =====");
                println!("Tool: {}", post_tool.tool_name);
                println!("Stopping execution for safety...");
                println!("================================================\n");

                return HookJsonOutput::Sync(SyncHookJsonOutput {
                    continue_: Some(false),
                    stop_reason: Some(
                        "Critical error detected in tool output - execution halted for safety"
                            .to_string(),
                    ),
                    system_message: Some("🛑 Execution stopped due to critical error".to_string()),
                    ..Default::default()
                });
            }

            HookJsonOutput::Sync(SyncHookJsonOutput {
                continue_: Some(true),
                ..Default::default()
            })
        }
        _ => HookJsonOutput::Sync(SyncHookJsonOutput::default()),
    }
}

/// Example 1: PreToolUse hook that blocks certain bash commands
async fn example_pretooluse() -> anyhow::Result<()> {
    println!("=== PreToolUse Example ===");
    println!(
        "This example demonstrates how PreToolUse can block some bash commands but not others.\n"
    );

    // Configure hooks using ClaudeAgentOptions
    let mut hooks: HashMap<HookEvent, Vec<HookMatcher>> = HashMap::new();

    hooks.insert(
        HookEvent::PreToolUse,
        vec![HookMatcher {
            matcher: Some("Bash".to_string()),
            hooks: vec![Arc::new(|input, tool_use_id, context| {
                Box::pin(check_bash_command(input, tool_use_id, context))
            })],
            timeout: None,
        }],
    );

    let options = ClaudeAgentOptions {
        tools: Some(Tools::List(vec!["Bash".to_string()])),
        model: Some("sonnet".to_string()),
        hooks: Some(hooks),
        max_turns: Some(5),
        ..Default::default()
    };

    let mut client = ClaudeClient::new(options);
    client.connect().await?;

    // Test 1: Command with forbidden pattern (will be blocked)
    println!("Test 1: Trying a command that our PreToolUse hook should block...");
    println!("User: Run the bash command: ./foo.sh --help\n");
    client
        .query("Run the bash command: ./foo.sh --help")
        .await?;

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
            Message::Result(_) => {
                println!("\nResult ended");
            }
            _ => {}
        }
    }
    drop(stream);

    println!("\n{}\n", "=".repeat(50));

    // Test 2: Safe command that should work
    println!("Test 2: Trying a command that our PreToolUse hook should allow...");
    println!("User: Run the bash command: echo 'Hello from hooks example!'\n");
    client
        .query("Run the bash command: echo 'Hello from hooks example!'")
        .await?;

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
            Message::Result(_) => {
                println!("\nResult ended");
            }
            _ => {}
        }
    }
    drop(stream);

    println!("\n{}\n", "=".repeat(50));

    client.disconnect().await?;
    Ok(())
}

/// Example 2: UserPromptSubmit hook that adds context
async fn example_userpromptsubmit() -> anyhow::Result<()> {
    println!("=== UserPromptSubmit Example ===");
    println!("This example shows how a UserPromptSubmit hook can add context.\n");

    let mut hooks: HashMap<HookEvent, Vec<HookMatcher>> = HashMap::new();

    hooks.insert(
        HookEvent::UserPromptSubmit,
        vec![HookMatcher {
            matcher: None, // Match all prompts
            hooks: vec![Arc::new(|input, tool_use_id, context| {
                Box::pin(add_custom_instructions(input, tool_use_id, context))
            })],
            timeout: None,
        }],
    );

    let options = ClaudeAgentOptions {
        hooks: Some(hooks),
        max_turns: Some(3),
        ..Default::default()
    };

    let mut client = ClaudeClient::new(options);
    client.connect().await?;

    println!("User: What's my favorite color?\n");
    client.query("What's my favorite color?").await?;

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
            Message::Result(_) => {
                println!("\nResult ended");
            }
            _ => {}
        }
    }
    drop(stream);

    println!("\n");

    client.disconnect().await?;
    Ok(())
}

/// Example 3: PostToolUse hook with reason and systemMessage
async fn example_posttooluse() -> anyhow::Result<()> {
    println!("=== PostToolUse Example ===");
    println!(
        "This example shows how PostToolUse can provide feedback with reason and systemMessage.\n"
    );

    let mut hooks: HashMap<HookEvent, Vec<HookMatcher>> = HashMap::new();

    hooks.insert(
        HookEvent::PostToolUse,
        vec![HookMatcher {
            matcher: Some("Bash".to_string()),
            hooks: vec![Arc::new(|input, tool_use_id, context| {
                Box::pin(review_tool_output(input, tool_use_id, context))
            })],
            timeout: None,
        }],
    );

    let options = ClaudeAgentOptions {
        tools: Some(Tools::List(vec!["Bash".to_string()])),
        model: Some("sonnet".to_string()),
        hooks: Some(hooks),
        max_turns: Some(5),
        ..Default::default()
    };

    let mut client = ClaudeClient::new(options);
    client.connect().await?;

    println!("User: Run a command that will produce an error: ls /nonexistent_directory\n");
    client
        .query("Run this command: ls /nonexistent_directory")
        .await?;

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
            Message::Result(_) => {
                println!("\nResult ended");
            }
            _ => {}
        }
    }
    drop(stream);

    println!("\n");

    client.disconnect().await?;
    Ok(())
}

/// Example 4: Permission decision with allow/deny
async fn example_decision_fields() -> anyhow::Result<()> {
    println!("=== Permission Decision Example ===");
    println!(
        "This example shows how to use permissionDecision='allow'/'deny' with reason and systemMessage.\n"
    );

    let mut hooks: HashMap<HookEvent, Vec<HookMatcher>> = HashMap::new();

    hooks.insert(
        HookEvent::PreToolUse,
        vec![HookMatcher {
            matcher: Some("Write".to_string()),
            hooks: vec![Arc::new(|input, tool_use_id, context| {
                Box::pin(strict_approval_hook(input, tool_use_id, context))
            })],
            timeout: None,
        }],
    );

    let options = ClaudeAgentOptions {
        tools: Some(Tools::List(vec!["Write".to_string(), "Bash".to_string()])),
        model: Some("sonnet".to_string()),
        hooks: Some(hooks),
        max_turns: Some(5),
        ..Default::default()
    };

    let mut client = ClaudeClient::new(options);
    client.connect().await?;

    // Test 1: Try to write to a file with "important" in the name (should be blocked)
    println!("Test 1: Trying to write to important_config.txt (should be blocked)...");
    println!("User: Write 'test' to important_config.txt\n");
    client
        .query("Write the text 'test data' to a file called important_config.txt")
        .await?;

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
            Message::Result(_) => {
                println!("\nResult ended");
            }
            _ => {}
        }
    }
    drop(stream);

    println!("\n{}\n", "=".repeat(50));

    // Test 2: Write to a regular file (should be approved)
    println!("Test 2: Trying to write to regular_file.txt (should be approved)...");
    println!("User: Write 'test' to regular_file.txt\n");
    client
        .query("Write the text 'test data' to a file called regular_file.txt")
        .await?;

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
            Message::Result(_) => {
                println!("\nResult ended");
            }
            _ => {}
        }
    }
    drop(stream);

    println!("\n");

    client.disconnect().await?;
    Ok(())
}

/// Example 5: Continue/stop control with continue=false
async fn example_continue_control() -> anyhow::Result<()> {
    println!("=== Continue/Stop Control Example ===");
    println!("This example shows how to use continue_=False with stopReason to halt execution.\n");

    let mut hooks: HashMap<HookEvent, Vec<HookMatcher>> = HashMap::new();

    hooks.insert(
        HookEvent::PostToolUse,
        vec![HookMatcher {
            matcher: Some("Bash".to_string()),
            hooks: vec![Arc::new(|input, tool_use_id, context| {
                Box::pin(stop_on_error_hook(input, tool_use_id, context))
            })],
            timeout: None,
        }],
    );

    let options = ClaudeAgentOptions {
        tools: Some(Tools::List(vec!["Bash".to_string()])),
        model: Some("sonnet".to_string()),
        hooks: Some(hooks),
        max_turns: Some(5),
        ..Default::default()
    };

    let mut client = ClaudeClient::new(options);
    client.connect().await?;

    println!("User: Run a command that outputs 'CRITICAL ERROR'\n");
    client
        .query("Run this bash command: echo 'CRITICAL ERROR: system failure'")
        .await?;

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
            Message::Result(_) => {
                println!("\nResult ended");
            }
            _ => {}
        }
    }
    drop(stream);

    println!("\n");

    client.disconnect().await?;
    Ok(())
}

fn print_usage() {
    println!("Usage: cargo run --example 15_hooks_comprehensive <example_name>");
    println!("\nAvailable examples:");
    println!("  all              - Run all examples");
    println!("  PreToolUse       - Block commands using PreToolUse hook");
    println!("  UserPromptSubmit - Add context at prompt submission");
    println!("  PostToolUse      - Review tool output with reason and systemMessage");
    println!("  DecisionFields   - Use permissionDecision='allow'/'deny' with reason");
    println!("  ContinueControl  - Control execution with continue_ and stopReason");
    println!("\nExample descriptions:");
    println!("  PreToolUse       - Block commands using PreToolUse hook");
    println!("  UserPromptSubmit - Add context at prompt submission");
    println!("  PostToolUse      - Review tool output with reason and systemMessage");
    println!("  DecisionFields   - Use permissionDecision='allow'/'deny' with reason");
    println!("  ContinueControl  - Control execution with continue_ and stopReason");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Starting Claude SDK Hooks Examples...");
    println!("{}\n", "=".repeat(50));

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    let example_name = &args[1];

    match example_name.as_str() {
        "all" => {
            // Run all examples
            example_pretooluse().await?;
            println!("{}\n", "-".repeat(50));

            example_userpromptsubmit().await?;
            println!("{}\n", "-".repeat(50));

            example_posttooluse().await?;
            println!("{}\n", "-".repeat(50));

            example_decision_fields().await?;
            println!("{}\n", "-".repeat(50));

            example_continue_control().await?;
            println!("{}\n", "-".repeat(50));
        }
        "PreToolUse" => {
            example_pretooluse().await?;
        }
        "UserPromptSubmit" => {
            example_userpromptsubmit().await?;
        }
        "PostToolUse" => {
            example_posttooluse().await?;
        }
        "DecisionFields" => {
            example_decision_fields().await?;
        }
        "ContinueControl" => {
            example_continue_control().await?;
        }
        _ => {
            println!("Error: Unknown example '{}'", example_name);
            println!();
            print_usage();
            return Ok(());
        }
    }

    Ok(())
}
