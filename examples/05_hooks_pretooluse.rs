//! Example 5: PreToolUse Hooks with Callbacks
//!
//! This example demonstrates REAL PreToolUse hooks that:
//! 1. Are passed as callbacks to ClaudeAgentOptions
//! 2. Are invoked by the Claude CLI before each tool use
//! 3. Can print tool name and arguments
//! 4. Can allow or deny tool execution
//!
//! This example showcases the new user-friendly Hooks API.

use std::path::Path;

use claude_agent_sdk_rs::{
    ClaudeAgentOptions, ClaudeClient, ContentBlock, HookContext, HookInput, HookJsonOutput,
    HookSpecificOutput, Hooks, Message, PreToolUseHookSpecificOutput, SyncHookJsonOutput,
};
use futures::StreamExt;

/// PreToolUse hook callback that prints tool information
async fn print_tool_info(
    input: HookInput,
    tool_use_id: Option<String>,
    _context: HookContext,
) -> HookJsonOutput {
    // Print hook invocation
    println!("\n🔔 ===== PreToolUse Hook Fired =====");
    if let Some(ref id) = tool_use_id {
        println!("Tool Use ID: {}", id);
    }

    // Extract tool information based on hook input type
    match input {
        HookInput::PreToolUse(pre_tool) => {
            println!("Tool Name: {}", pre_tool.tool_name);
            println!("Tool Input:");
            println!(
                "{}",
                serde_json::to_string_pretty(&pre_tool.tool_input).unwrap()
            );

            // Additional context
            println!("\nContext:");
            println!("  Session ID: {}", pre_tool.session_id);
            println!("  CWD: {}", pre_tool.cwd);
            if let Some(ref mode) = pre_tool.permission_mode {
                println!("  Permission Mode: {}", mode);
            }

            // Allow the tool (hook passes through)
            println!("Decision: ALLOW");
            println!("=====================================\n");

            HookJsonOutput::Sync(SyncHookJsonOutput::default())
        }
        _ => {
            // Not a PreToolUse event
            HookJsonOutput::Sync(SyncHookJsonOutput::default())
        }
    }
}

/// PreToolUse hook that blocks dangerous Bash commands
async fn block_dangerous_bash(
    input: HookInput,
    _tool_use_id: Option<String>,
    _context: HookContext,
) -> HookJsonOutput {
    match input {
        HookInput::PreToolUse(pre_tool) if pre_tool.tool_name == "Bash" => {
            let command = pre_tool
                .tool_input
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            // Block dangerous commands
            let dangerous_patterns = vec!["rm -rf", "format", "delete"];

            for pattern in &dangerous_patterns {
                if command.contains(pattern) {
                    println!("\n🚫 ===== Hook BLOCKED Command =====");
                    println!("Tool: {}", pre_tool.tool_name);
                    println!("Command: {}", command);
                    println!("Reason: Contains dangerous pattern '{}'", pattern);
                    println!("====================================\n");

                    return HookJsonOutput::Sync(SyncHookJsonOutput {
                        hook_specific_output: Some(HookSpecificOutput::PreToolUse(
                            PreToolUseHookSpecificOutput {
                                permission_decision: Some("deny".to_string()),
                                permission_decision_reason: Some(format!(
                                    "Command contains dangerous pattern: {}",
                                    pattern
                                )),
                                ..Default::default()
                            },
                        )),
                        ..Default::default()
                    });
                }
            }

            // Allow safe commands
            HookJsonOutput::Sync(SyncHookJsonOutput::default())
        }
        _ => HookJsonOutput::Sync(SyncHookJsonOutput::default()),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Example 5: PreToolUse Hooks with New User-Friendly API ===\n");

    std::fs::create_dir_all("./fixtures")?;

    // Configure hooks using the new Hooks builder API
    let mut hooks = Hooks::new();

    // Add PreToolUse hook that prints info for all tools (no matcher)
    hooks.add_pre_tool_use(print_tool_info);

    // Add PreToolUse hook that blocks dangerous bash commands (only for Bash tool)
    hooks.add_pre_tool_use_with_matcher("Bash", block_dangerous_bash);

    let options = ClaudeAgentOptions::builder()
        .allowed_tools(vec!["Write".to_string(), "Bash".to_string()])
        .permission_mode(claude_agent_sdk_rs::PermissionMode::AcceptEdits)
        .cwd(Path::new("/tmp/todo"))
        .hooks(hooks.build())
        .build();

    println!("Creating ClaudeClient with PreToolUse hooks...\n");

    let mut client = ClaudeClient::new(options);

    println!("Connecting to Claude...\n");
    client.connect().await?;

    println!("Sending query: 'Make a plan for writing a simple Python script that greets the user, and execute the plan to ./fixtures/greet.py'...\n");
    client
        .query("Write a simple Python script to ./fixtures/greet.py that greets the user")
        .await?;

    println!("Receiving response with hooks active...\n");

    println!("\n========== Messages Received ==========\n");

    // Process messages as a stream
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

    // Drop the stream to release the borrow
    drop(stream);

    println!("\n========================================");
    println!("\n✅ Hook example completed!");
    println!("\nKey observations:");
    println!("- Used new Hooks::new() API for easy hook registration");
    println!("- PreToolUse hooks were called BEFORE each tool execution");
    println!("- Hooks received tool name and full input parameters");
    println!("- Hooks can allow or deny tool execution");
    println!("- Multiple hooks can be chained (print_info + block_dangerous)");
    println!("\nNew API benefits:");
    println!("- No need to manually create HashMap and Arc wrappers");
    println!("- Two methods: add_pre_tool_use() for all tools");
    println!("- add_pre_tool_use_with_matcher(\"ToolName\") for specific tools");
    println!("- Methods auto-generated using macros for all hook types");

    // Clean disconnect
    println!("\nDisconnecting...");
    client.disconnect().await?;
    println!("Disconnected!");

    Ok(())
}
