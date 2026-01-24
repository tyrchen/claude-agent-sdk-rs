//! Example 22: Plugin Integration - Real World Use Case
//!
//! This example demonstrates a realistic plugin integration scenario where you:
//! 1. Load domain-specific plugins for your application
//! 2. Combine plugins with custom MCP servers
//! 3. Use plugins with permission management
//! 4. Handle plugin errors gracefully
//!
//! Real-world scenario: A development tool that uses plugins for:
//! - Database schema tools
//! - API testing utilities
//! - Code generation helpers

use claude_agent_sdk_rs::{
    ClaudeAgentOptions, ClaudeClient, Message, PermissionMode, SdkPluginConfig,
};
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Example 22: Plugin Integration - Development Assistant ===\n");

    // Scenario: Building a development assistant with multiple specialized plugins
    println!("Building a development assistant with plugins:\n");

    // Define your plugin ecosystem
    let plugins = vec![
        // Database tools plugin (hypothetical)
        SdkPluginConfig::local("./plugins/database-tools"),
        // API testing plugin (hypothetical)
        SdkPluginConfig::local("./plugins/api-tester"),
        // Code generation plugin (hypothetical)
        SdkPluginConfig::local("./plugins/code-gen"),
    ];

    println!("Configured plugins:");
    for (i, plugin) in plugins.iter().enumerate() {
        println!("  {}. {:?}", i + 1, plugin.path().unwrap());
    }
    println!();

    // Build comprehensive configuration
    // Note: Use .tools() to restrict available tools (not .allowed_tools())
    let options = ClaudeAgentOptions::builder()
        .plugins(plugins)
        // Restrict Claude to specific tools plus plugin tools
        .tools(["Read", "Write", "Bash"])
        .model("sonnet".to_string()) // Use Sonnet for lower cost
        .permission_mode(PermissionMode::Default) // Ask for permission
        .max_turns(10)
        .max_budget_usd(5.0) // Budget control
        .build();

    println!("Configuration:");
    println!("  • Permission mode: Ask (for safety)");
    println!("  • Max turns: 10");
    println!("  • Budget limit: $5.00");
    println!("  • Standard tools: Read, Write, Bash");
    println!("  • Plugin count: 3\n");

    // Initialize client
    let mut client = ClaudeClient::new(options);

    println!("Connecting to Claude...");
    match client.connect().await {
        Ok(_) => println!("✓ Connected successfully\n"),
        Err(e) => {
            println!("✗ Connection failed: {}", e);
            println!("\nNote: This example requires:");
            println!("  • Claude CLI installed and configured");
            println!("  • Plugin support enabled (if available)");
            println!("  • Valid plugin paths");
            return Ok(());
        }
    }

    // Example task: Ask Claude to help with development
    println!("--- Example Task: Project Setup ---\n");

    client
        .query("What plugins and tools are currently available to you?")
        .await?;

    println!("Processing response...\n");

    let mut message_count = 0;
    {
        let mut stream = client.receive_response();

        while let Some(result) = stream.next().await {
            match result {
                Ok(Message::Assistant(msg)) => {
                    message_count += 1;
                    println!("Assistant message #{}:", message_count);

                    for block in &msg.message.content {
                        match block {
                            claude_agent_sdk_rs::ContentBlock::Text(text) => {
                                println!("{}", text.text);
                            }
                            claude_agent_sdk_rs::ContentBlock::ToolUse(tool) => {
                                println!("  [Tool: {}]", tool.name);
                            }
                            _ => {}
                        }
                    }
                    println!();
                }
                Ok(Message::System(sys)) => {
                    println!("System: {}", sys.subtype);
                }
                Ok(Message::Result(result)) => {
                    println!("--- Result ---");
                    println!("Duration: {}ms", result.duration_ms);
                    println!("Turns: {}", result.num_turns);
                    println!("Error: {}", result.is_error);
                    if let Some(cost) = result.total_cost_usd {
                        println!("Cost: ${:.4}", cost);
                    }
                    break;
                }
                Err(e) => {
                    println!("Error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    }

    client.disconnect().await?;
    println!("\n✓ Disconnected\n");

    // Best practices summary
    println!("=== Plugin Integration Best Practices ===\n");

    println!("1. Plugin Organization:");
    println!("   • Group related tools in single plugins");
    println!("   • Use descriptive plugin names");
    println!("   • Version your plugins properly");

    println!("\n2. Error Handling:");
    println!("   • Check plugin paths exist before loading");
    println!("   • Handle plugin load failures gracefully");
    println!("   • Provide fallback behavior when plugins fail");

    println!("\n3. Configuration:");
    println!("   • Use permission modes appropriately");
    println!("   • Set budget limits for safety");
    println!("   • Combine plugins with standard tools wisely");

    println!("\n4. Development Workflow:");
    println!("   • Test plugins individually first");
    println!("   • Verify plugin compatibility");
    println!("   • Monitor plugin performance impact");
    println!("   • Document plugin requirements");

    println!("\n5. Production Deployment:");
    println!("   • Use absolute paths for stability");
    println!("   • Implement plugin health checks");
    println!("   • Version lock plugin dependencies");
    println!("   • Have rollback plans ready");

    Ok(())
}
