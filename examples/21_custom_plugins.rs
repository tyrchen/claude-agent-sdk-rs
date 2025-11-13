//! Example 21: Custom Plugins
//!
//! This example demonstrates how to load and use custom plugins with Claude Agent SDK.
//! Plugins allow you to extend Claude Code with custom functionality, tools, and integrations.
//!
//! What it does:
//! 1. Configures Claude with one or more custom plugins
//! 2. Shows how to load plugins from local paths
//! 3. Demonstrates plugin configuration and setup
//! 4. Shows how to use multiple plugins together
//!
//! Use cases for plugins:
//! - Add custom tools specific to your domain
//! - Integrate with third-party services
//! - Extend Claude's capabilities with organization-specific features
//! - Package reusable functionality for multiple projects

use claude_agent_sdk_rs::{
    query, ClaudeAgentOptions, ContentBlock, Message, PermissionMode, SdkPluginConfig,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Example 21: Custom Plugins ===\n");

    // Example 1: Single plugin configuration
    println!("--- Example 1: Single Plugin ---\n");

    let _single_plugin_options = ClaudeAgentOptions::builder()
        .plugins(vec![SdkPluginConfig::local("./my-plugin")])
        .permission_mode(PermissionMode::BypassPermissions)
        .max_turns(3)
        .build();

    println!("Configured with single plugin:");
    println!("  Plugin: ./my-plugin\n");

    // In a real scenario, you would query Claude here
    // For this example, we'll demonstrate the configuration only
    println!("✓ Single plugin configuration created\n");

    // Example 2: Multiple plugins configuration
    println!("--- Example 2: Multiple Plugins ---\n");

    let _multi_plugin_options = ClaudeAgentOptions::builder()
        .plugins(vec![
            SdkPluginConfig::local("./plugins/database-tools"),
            SdkPluginConfig::local("./plugins/api-integration"),
            SdkPluginConfig::local("~/.claude/plugins/company-tools"),
        ])
        .permission_mode(PermissionMode::BypassPermissions)
        .max_turns(5)
        .build();

    println!("Configured with multiple plugins:");
    println!("  1. ./plugins/database-tools");
    println!("  2. ./plugins/api-integration");
    println!("  3. ~/.claude/plugins/company-tools\n");
    println!("✓ Multiple plugin configuration created\n");

    // Example 3: Plugin with test fixture (if available)
    println!("--- Example 3: Using Test Plugin ---\n");

    let test_plugin_path = "./fixtures/test-plugin";

    if std::path::Path::new(test_plugin_path).exists() {
        println!("Test plugin found at: {}", test_plugin_path);

        let options = ClaudeAgentOptions::builder()
            .plugins(vec![SdkPluginConfig::local(test_plugin_path)])
            .permission_mode(PermissionMode::BypassPermissions)
            .max_turns(3)
            .build();

        println!("\nAttempting to use test plugin...");

        // Note: This will only work if Claude CLI has plugin support enabled
        // Note: Plugin feature uses --plugin-dir flag
        match query("Hello, check if plugins are loaded", Some(options)).await {
            Ok(messages) => {
                println!("\n✓ Successfully queried with plugin loaded!\n");

                let mut found_plugin_info = false;
                for message in &messages {
                    match message {
                        Message::System(sys) if sys.subtype == "init" => {
                            // Check for plugins in system message
                            if let Some(plugins) = sys.data.get("plugins") {
                                println!("Plugins loaded: {:?}", plugins);
                                found_plugin_info = true;
                            }
                        }
                        Message::Assistant(msg) => {
                            for block in &msg.message.content {
                                if let ContentBlock::Text(text) = block {
                                    println!("Claude: {}", text.text);
                                }
                            }
                        }
                        Message::Result(result) => {
                            println!("\nResult:");
                            println!("  Duration: {}ms", result.duration_ms);
                            println!("  Turns: {}", result.num_turns);
                            if let Some(cost) = result.total_cost_usd {
                                println!("  Cost: ${:.4}", cost);
                            }
                        }
                        _ => {}
                    }
                }

                if !found_plugin_info {
                    println!("Note: Plugin info not in system message (this is OK)");
                }
            }
            Err(e) => {
                println!("\n⚠ Plugin query failed: {}", e);
                println!("Note: The SDK correctly passes --plugin-dir to Claude CLI.");
                println!("Plugin support requires Claude CLI with plugin features enabled.");
            }
        }
    } else {
        println!("⚠ Test plugin not found at: {}", test_plugin_path);
        println!("This is expected in a fresh checkout.");
    }

    // Example 4: Plugin path patterns
    println!("\n--- Example 4: Plugin Path Patterns ---\n");

    println!("Supported plugin path patterns:");
    println!("  • Relative paths:     ./plugins/my-plugin");
    println!("  • Absolute paths:     /usr/local/plugins/my-plugin");
    println!("  • Home directory:     ~/.claude/plugins/my-plugin");
    println!("  • Project relative:   ./my-app-plugins/special-tools");

    let path_examples = [
        SdkPluginConfig::local("./plugins/local-plugin"),
        SdkPluginConfig::local("/opt/company-plugins/tool-suite"),
        SdkPluginConfig::local("~/.claude/plugins/personal-tools"),
    ];

    println!("\nExample configurations:");
    for (i, plugin) in path_examples.iter().enumerate() {
        println!("  {}. {:?}", i + 1, plugin.path().unwrap());
    }

    println!("\n=== Plugin Development Guide ===\n");
    println!("To create a custom plugin:");
    println!("1. Create a plugin directory with proper structure");
    println!("2. Define plugin metadata (name, version, description)");
    println!("3. Implement custom tools and functionality");
    println!("4. Test the plugin with Claude CLI");
    println!("5. Load it via SdkPluginConfig in your Rust code");

    println!("\n=== Plugin Configuration Tips ===\n");
    println!("• Plugins extend Claude's capabilities with custom tools");
    println!("• Multiple plugins can be loaded simultaneously");
    println!("• Plugin order may affect tool resolution");
    println!("• Use absolute paths for production deployments");
    println!("• Use relative paths for project-specific plugins");
    println!("• Home directory paths (~) work for user-specific plugins");

    println!("\n=== Security Considerations ===\n");
    println!("⚠ Important security notes:");
    println!("• Only load plugins from trusted sources");
    println!("• Review plugin code before deployment");
    println!("• Be cautious with plugins that access sensitive data");
    println!("• Use permission modes to control plugin capabilities");
    println!("• Test plugins in isolated environments first");

    Ok(())
}
