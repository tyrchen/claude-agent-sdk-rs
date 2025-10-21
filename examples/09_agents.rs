//! Example 9: Custom Agents
//!
//! This example demonstrates how to define and use custom agents with specific
//! tools, prompts, and models.
//!
//! What it does:
//! 1. Defines custom agents (code-reviewer, doc-writer, analyzer)
//! 2. Shows how to configure agents with specific tools and prompts
//! 3. Demonstrates using multiple agents in a single session

use claude_agent_sdk::{
    query, AgentDefinition, AgentModel, ClaudeAgentOptions, ContentBlock, Message, SettingSource,
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Custom Agents Examples ===\n");

    code_reviewer_example().await?;
    documentation_writer_example().await?;
    multiple_agents_example().await?;

    Ok(())
}

async fn code_reviewer_example() -> anyhow::Result<()> {
    println!("=== Code Reviewer Agent Example ===");

    let mut agents = HashMap::new();
    agents.insert(
        "code-reviewer".to_string(),
        AgentDefinition {
            description: "Reviews code for best practices and potential issues".to_string(),
            prompt: "You are a code reviewer. Analyze code for bugs, performance issues, \
                     security vulnerabilities, and adherence to best practices. \
                     Provide constructive feedback."
                .to_string(),
            tools: Some(vec!["Read".to_string(), "Grep".to_string()]),
            model: Some(AgentModel::Sonnet),
        },
    );

    let options = ClaudeAgentOptions {
        agents: Some(agents),
        max_turns: Some(5),
        ..Default::default()
    };

    let messages = query(
        "Use the code-reviewer agent to review the code in src/lib.rs",
        Some(options),
    )
    .await?;

    display_messages(&messages);
    println!();

    Ok(())
}

async fn documentation_writer_example() -> anyhow::Result<()> {
    println!("=== Documentation Writer Agent Example ===");

    let mut agents = HashMap::new();
    agents.insert(
        "doc-writer".to_string(),
        AgentDefinition {
            description: "Writes comprehensive documentation".to_string(),
            prompt: "You are a technical documentation expert. Write clear, comprehensive \
                     documentation with examples. Focus on clarity and completeness."
                .to_string(),
            tools: Some(vec![
                "Read".to_string(),
                "Write".to_string(),
                "Edit".to_string(),
            ]),
            model: Some(AgentModel::Sonnet),
        },
    );

    let options = ClaudeAgentOptions {
        agents: Some(agents),
        max_turns: Some(5),
        ..Default::default()
    };

    let messages = query(
        "Use the doc-writer agent to explain what AgentDefinition is used for",
        Some(options),
    )
    .await?;

    display_messages(&messages);
    println!();

    Ok(())
}

async fn multiple_agents_example() -> anyhow::Result<()> {
    println!("=== Multiple Agents Example ===");

    let mut agents = HashMap::new();
    agents.insert(
        "analyzer".to_string(),
        AgentDefinition {
            description: "Analyzes code structure and patterns".to_string(),
            prompt: "You are a code analyzer. Examine code structure, patterns, and architecture."
                .to_string(),
            tools: Some(vec![
                "Read".to_string(),
                "Grep".to_string(),
                "Glob".to_string(),
            ]),
            model: None,
        },
    );
    agents.insert(
        "tester".to_string(),
        AgentDefinition {
            description: "Creates and runs tests".to_string(),
            prompt: "You are a testing expert. Write comprehensive tests and ensure code quality."
                .to_string(),
            tools: Some(vec![
                "Read".to_string(),
                "Write".to_string(),
                "Bash".to_string(),
            ]),
            model: Some(AgentModel::Sonnet),
        },
    );

    let options = ClaudeAgentOptions {
        agents: Some(agents),
        setting_sources: Some(vec![SettingSource::User, SettingSource::Project]),
        max_turns: Some(5),
        ..Default::default()
    };

    let messages = query(
        "Use the analyzer agent to find all Rust source files in the examples/ directory",
        Some(options),
    )
    .await?;

    display_messages(&messages);
    println!();

    Ok(())
}

fn display_messages(messages: &[Message]) {
    for message in messages {
        match message {
            Message::Assistant(msg) => {
                for block in &msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        println!("Claude: {}", text.text);
                    }
                }
            }
            Message::Result(result) => {
                if let Some(cost) = result.total_cost_usd {
                    if cost > 0.0 {
                        println!("\nCost: ${:.4}", cost);
                    }
                }
            }
            _ => {}
        }
    }
}
