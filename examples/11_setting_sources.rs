//! Example 11: Setting Sources Control
//!
//! This example demonstrates how to use the setting_sources option to control which
//! settings are loaded, including custom slash commands, agents, and other
//! configurations.
//!
//! Setting sources determine where Claude Code loads configurations from:
//! - "user": Global user settings (~/.claude/)
//! - "project": Project-level settings (.claude/ in project)
//! - "local": Local gitignored settings (.claude-local/)
//!
//! IMPORTANT: When setting_sources is not provided (None), NO settings are loaded
//! by default. This creates an isolated environment. To load settings, explicitly
//! specify which sources to use.
//!
//! Usage:
//! cargo run --example 11_setting_sources -- default
//! cargo run --example 11_setting_sources -- user_only
//! cargo run --example 11_setting_sources -- project_and_user
//! cargo run --example 11_setting_sources -- all

use claude_agent_sdk::{ClaudeAgentOptions, ClaudeClient, Message, SettingSource, SystemMessage};
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("Usage: cargo run --example 11_setting_sources -- <example_name>");
        println!("\nAvailable examples:");
        println!("  all - Run all examples");
        println!("  default - Default behavior (no settings)");
        println!("  user_only - Load only user settings");
        println!("  project_and_user - Load both project and user settings");
        return Ok(());
    }

    println!("Starting Claude SDK Setting Sources Examples...");
    println!("{}\n", "=".repeat(50));

    match args[1].as_str() {
        "all" => {
            example_default().await?;
            println!("{}\n", "-".repeat(50));
            example_user_only().await?;
            println!("{}\n", "-".repeat(50));
            example_project_and_user().await?;
        }
        "default" => example_default().await?,
        "user_only" => example_user_only().await?,
        "project_and_user" => example_project_and_user().await?,
        _ => {
            println!("Error: Unknown example '{}'", args[1]);
            println!("\nAvailable examples:");
            println!("  all - Run all examples");
            println!("  default");
            println!("  user_only");
            println!("  project_and_user");
            return Ok(());
        }
    }

    Ok(())
}

async fn example_default() -> anyhow::Result<()> {
    println!("=== Default Behavior Example ===");
    println!("Setting sources: None (default)");
    println!("Expected: No custom slash commands will be available\n");

    let options = ClaudeAgentOptions {
        cwd: Some(std::path::PathBuf::from(".")),
        max_turns: Some(1),
        ..Default::default()
    };

    let mut client = ClaudeClient::new(options);
    client.connect().await?;

    client.query("What is 2 + 2?").await?;

    let mut stream = client.receive_response();
    while let Some(message) = stream.next().await {
        match message? {
            Message::System(msg) => {
                if msg.subtype == "init" {
                    let commands = extract_slash_commands(&msg);
                    println!("Available slash commands: {:?}", commands);
                    if commands.contains(&"commit".to_string()) {
                        println!("❌ /commit is available (unexpected)");
                    } else {
                        println!("✓ /commit is NOT available (expected - no settings loaded)");
                    }
                    break;
                }
            }
            Message::Result(_) => break,
            _ => {}
        }
    }
    drop(stream);

    client.disconnect().await?;
    println!();

    Ok(())
}

async fn example_user_only() -> anyhow::Result<()> {
    println!("=== User Settings Only Example ===");
    println!("Setting sources: ['user']");
    println!("Expected: Project slash commands (like /commit) will NOT be available\n");

    let options = ClaudeAgentOptions {
        setting_sources: Some(vec![SettingSource::User]),
        cwd: Some(std::path::PathBuf::from(".")),
        max_turns: Some(1),
        ..Default::default()
    };

    let mut client = ClaudeClient::new(options);
    client.connect().await?;

    client.query("What is 2 + 2?").await?;

    let mut stream = client.receive_response();
    while let Some(message) = stream.next().await {
        match message? {
            Message::System(msg) => {
                if msg.subtype == "init" {
                    let commands = extract_slash_commands(&msg);
                    println!("Available slash commands: {:?}", commands);
                    if commands.contains(&"commit".to_string()) {
                        println!("❌ /commit is available (unexpected)");
                    } else {
                        println!("✓ /commit is NOT available (expected)");
                    }
                    break;
                }
            }
            Message::Result(_) => break,
            _ => {}
        }
    }
    drop(stream);

    client.disconnect().await?;
    println!();

    Ok(())
}

async fn example_project_and_user() -> anyhow::Result<()> {
    println!("=== Project + User Settings Example ===");
    println!("Setting sources: ['user', 'project']");
    println!("Expected: Project slash commands (like /commit) WILL be available\n");

    let options = ClaudeAgentOptions {
        setting_sources: Some(vec![SettingSource::User, SettingSource::Project]),
        cwd: Some(std::path::PathBuf::from(".")),
        max_turns: Some(1),
        ..Default::default()
    };

    let mut client = ClaudeClient::new(options);
    client.connect().await?;

    client.query("What is 2 + 2?").await?;

    let mut stream = client.receive_response();
    while let Some(message) = stream.next().await {
        match message? {
            Message::System(msg) => {
                if msg.subtype == "init" {
                    let commands = extract_slash_commands(&msg);
                    println!("Available slash commands: {:?}", commands);
                    if commands.contains(&"commit".to_string()) {
                        println!("✓ /commit is available (expected)");
                    } else {
                        println!("❌ /commit is NOT available (unexpected)");
                    }
                    break;
                }
            }
            Message::Result(_) => break,
            _ => {}
        }
    }
    drop(stream);

    client.disconnect().await?;
    println!();

    Ok(())
}

fn extract_slash_commands(msg: &SystemMessage) -> Vec<String> {
    if msg.subtype == "init" {
        if let Some(commands) = msg.data.get("slash_commands") {
            if let Some(arr) = commands.as_array() {
                return arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
            }
        }
    }
    Vec::new()
}
