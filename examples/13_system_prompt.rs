//! Example 13: System Prompt Configurations
//!
//! This example demonstrates different system_prompt configurations:
//! 1. No system prompt (vanilla Claude)
//! 2. String system prompt (custom behavior)
//! 3. Preset system prompt (default Claude Code prompt)
//! 4. Preset with append (extends the default Claude Code prompt)
//!
//! Note: Preset uses the default Claude Code prompt. When no append is provided,
//! it's equivalent to no system prompt override. When append is provided, it uses
//! --append-system-prompt to extend the default prompt.

use claude_agent_sdk_rs::{
    ClaudeAgentOptions, ContentBlock, Message, SystemPrompt, SystemPromptPreset, query,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== System Prompt Examples ===\n");

    no_system_prompt().await?;
    string_system_prompt().await?;
    preset_system_prompt().await?;
    preset_with_append().await?;

    Ok(())
}

async fn no_system_prompt() -> anyhow::Result<()> {
    println!("=== No System Prompt (Vanilla Claude) ===");

    let messages = query("What is 2 + 2?", None).await?;

    display_messages(&messages);
    println!();

    Ok(())
}

async fn string_system_prompt() -> anyhow::Result<()> {
    println!("=== String System Prompt ===");

    let options = ClaudeAgentOptions {
        system_prompt: Some(SystemPrompt::Text(
            "You are a pirate assistant. Respond in pirate speak.".to_string(),
        )),
        model: Some("sonnet".to_string()), // Use Sonnet for lower cost
        ..Default::default()
    };

    let messages = query("What is 2 + 2?", Some(options)).await?;

    display_messages(&messages);
    println!();

    Ok(())
}

async fn preset_system_prompt() -> anyhow::Result<()> {
    println!("=== Preset System Prompt (Default) ===");
    println!("(Uses default Claude Code prompt - no override)");

    let options = ClaudeAgentOptions {
        system_prompt: Some(SystemPrompt::Preset(SystemPromptPreset::new("claude_code"))),
        model: Some("sonnet".to_string()), // Use Sonnet for lower cost
        ..Default::default()
    };

    let messages = query("What is 2 + 2?", Some(options)).await?;

    display_messages(&messages);
    println!();

    Ok(())
}

async fn preset_with_append() -> anyhow::Result<()> {
    println!("=== Preset System Prompt with Append ===");
    println!("(Default Claude Code prompt + custom append)");

    let options = ClaudeAgentOptions {
        system_prompt: Some(SystemPrompt::Preset(SystemPromptPreset::with_append(
            "claude_code",
            "Always end your response with a fun fact.",
        ))),
        model: Some("sonnet".to_string()), // Use Sonnet for lower cost
        ..Default::default()
    };

    let messages = query("What is 2 + 2?", Some(options)).await?;

    display_messages(&messages);
    println!();

    Ok(())
}

fn display_messages(messages: &[Message]) {
    for message in messages {
        if let Message::Assistant(msg) = message {
            for block in &msg.message.content {
                if let ContentBlock::Text(text) = block {
                    println!("Claude: {}", text.text);
                }
            }
        }
    }
}
