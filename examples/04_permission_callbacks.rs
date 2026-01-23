//! Example 4: Dynamic Permission Callbacks
//!
//! This example demonstrates the `can_use_tool` callback feature which allows
//! fine-grained control over tool execution permissions. Unlike static permission
//! modes or hooks, callbacks can make dynamic decisions based on:
//! - Tool name
//! - Tool arguments
//! - Current context (session ID, working directory)
//! - External state (time, rate limits, user roles, etc.)
//!
//! This is particularly useful for:
//! - Implementing custom security policies
//! - Rate limiting tool usage
//! - Conditional tool access based on runtime state
//! - Auditing and logging tool requests
//!
//! Run with:
//! ```bash
//! cargo run --example 04_permission_callbacks
//! ```

use claude_agent_sdk_rs::{
    ClaudeAgentOptions, ClaudeClient, ContentBlock, Message, PermissionResult,
    PermissionResultDeny, ToolPermissionContext,
};
use futures::{FutureExt, StreamExt, future::BoxFuture};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Counter to track tool usage (simulating rate limiting)
#[derive(Default)]
struct ToolUsageTracker {
    counts: HashMap<String, usize>,
    max_per_tool: usize,
}

impl ToolUsageTracker {
    fn new(max_per_tool: usize) -> Self {
        Self {
            counts: HashMap::new(),
            max_per_tool,
        }
    }

    fn check_and_increment(&mut self, tool_name: &str) -> bool {
        let count = self.counts.entry(tool_name.to_string()).or_insert(0);
        if *count >= self.max_per_tool {
            false
        } else {
            *count += 1;
            true
        }
    }

    fn get_count(&self, tool_name: &str) -> usize {
        *self.counts.get(tool_name).unwrap_or(&0)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Example 4: Dynamic Permission Callbacks ===\n");

    std::fs::create_dir_all("./fixtures")?;

    // Create a shared tracker for rate limiting
    let tracker = Arc::new(Mutex::new(ToolUsageTracker::new(2)));

    // Clone for the callback
    let tracker_clone = Arc::clone(&tracker);

    // Define a permission callback with multiple policies
    let permission_callback = Arc::new(
        move |tool_name: String,
              tool_input: serde_json::Value,
              _context: ToolPermissionContext|
              -> BoxFuture<'static, PermissionResult> {
            let tracker = Arc::clone(&tracker_clone);

            async move {
                println!("\n🔍 Permission Check:");
                println!("   Tool: {}", tool_name);

                // Policy 1: Block dangerous file operations
                if tool_name == "Write"
                    && let Some(file_path) = tool_input.get("file_path").and_then(|v| v.as_str())
                {
                    let dangerous_paths = vec!["/etc/", "/sys/", "/boot/", "~/.ssh/"];
                    for path in &dangerous_paths {
                        if file_path.starts_with(path) {
                            println!("   ❌ DENIED: Dangerous file path");
                            return PermissionResult::Deny(PermissionResultDeny {
                                message: format!(
                                    "Writing to {} is not allowed for security reasons",
                                    path
                                ),
                                interrupt: false,
                            });
                        }
                    }
                }

                // Policy 2: Block destructive bash commands
                if tool_name == "Bash"
                    && let Some(command) = tool_input.get("command").and_then(|v| v.as_str())
                {
                    let dangerous_patterns = vec!["rm -rf /", "mkfs", "dd if=", "> /dev/"];
                    for pattern in &dangerous_patterns {
                        if command.contains(pattern) {
                            println!("   ❌ DENIED: Dangerous bash command");
                            return PermissionResult::Deny(PermissionResultDeny {
                                message: format!("Command contains dangerous pattern: {}", pattern),
                                interrupt: false,
                            });
                        }
                    }
                }

                // Policy 3: Rate limiting - max 2 uses per tool type
                let mut tracker_guard = tracker.lock().await;
                if !tracker_guard.check_and_increment(&tool_name) {
                    let count = tracker_guard.get_count(&tool_name);
                    println!("   ❌ DENIED: Rate limit exceeded ({} uses)", count);
                    drop(tracker_guard);
                    return PermissionResult::Deny(PermissionResultDeny {
                        message: format!(
                            "Rate limit: {} tool has been used {} times (max: 2)",
                            tool_name, count
                        ),
                        interrupt: false,
                    });
                }

                let count = tracker_guard.get_count(&tool_name);
                drop(tracker_guard);

                println!("   ✅ ALLOWED (use {}/2 for {})", count, tool_name);
                PermissionResult::Allow(Default::default())
            }
            .boxed()
        },
    );

    // Configure options with permission callback
    // Note: Use `tools` to restrict available tools (not `allowed_tools`)
    let options = ClaudeAgentOptions {
        tools: Some(["Write", "Read", "Bash"].into()),
        model: Some("sonnet".to_string()), // Use Sonnet for lower cost
        permission_mode: Some(claude_agent_sdk_rs::PermissionMode::AcceptEdits),
        can_use_tool: Some(permission_callback),
        max_turns: Some(10),
        ..Default::default()
    };

    let mut client = ClaudeClient::new(options);

    println!("Connecting to Claude...\n");
    client.connect().await?;

    println!("Sending query with permission policies active...\n");
    println!("Policies:");
    println!("  1. Block writes to system directories (/etc, /sys, /boot, ~/.ssh)");
    println!("  2. Block dangerous bash commands (rm -rf /, mkfs, dd, etc.)");
    println!("  3. Rate limit: Max 2 uses per tool type\n");
    println!("========================================\n");

    client
        .query(
            "Create three test files in ./fixtures: test1.txt, test2.txt, and test3.txt. \
             Each should contain a different message. Also run 'ls -la ./fixtures' to verify.",
        )
        .await?;

    // Process response
    let mut stream = client.receive_response();
    while let Some(message) = stream.next().await {
        match message? {
            Message::Assistant(msg) => {
                for block in msg.message.content {
                    if let ContentBlock::Text(text) = block
                        && !text.text.trim().is_empty()
                    {
                        println!("💬 Claude: {}", text.text);
                    }
                }
            }
            Message::Result(result) => {
                println!("\n========================================");
                println!("=== Execution Complete ===");
                println!("Duration: {}ms", result.duration_ms);
                println!("Turns: {}", result.num_turns);
                if let Some(cost) = result.total_cost_usd {
                    println!("Cost: ${:.4}", cost);
                }
            }
            _ => {}
        }
    }
    drop(stream);

    // Show final tool usage stats
    let tracker_guard = tracker.lock().await;
    println!("\n=== Tool Usage Summary ===");
    for (tool_name, count) in tracker_guard.counts.iter() {
        println!("  {}: {} uses", tool_name, count);
    }
    drop(tracker_guard);

    println!("\n========================================");
    println!("✅ Permission callback example completed!");
    println!("\nKey features demonstrated:");
    println!("- Dynamic permission decisions based on tool arguments");
    println!("- Path-based security policies (blocking system directories)");
    println!("- Command pattern matching (blocking dangerous operations)");
    println!("- Rate limiting with shared state across tool calls");
    println!("- Detailed logging and feedback for each permission check");

    // Clean disconnect
    println!("\nDisconnecting...");
    client.disconnect().await?;
    println!("Disconnected!");

    Ok(())
}
