//! Example 8: MCP Server Integration
//!
//! This example demonstrates how to create and use custom MCP (Model Context Protocol)
//! servers with the Claude Agent SDK. MCP servers allow you to extend Claude's
//! capabilities with custom tools that run in-process.
//!
//! What it demonstrates:
//! - Creating custom tools with the `tool!` macro
//! - Building an SDK MCP server with multiple tools
//! - Integrating MCP servers with ClaudeClient
//! - Configuring allowed_tools to enable custom tools
//! - Using custom tools in a bidirectional conversation
//!
//! The example creates a "math-tools" MCP server with calculator, statistics, and
//! random number generator tools.
//!
//! Run with:
//! ```bash
//! cargo run --example 08_mcp_server_integration
//! ```

use claude_agent_sdk::{
    create_sdk_mcp_server, tool, ClaudeAgentOptions, ClaudeClient, ContentBlock, McpServers,
    McpToolResultContent, Message, ToolResult,
};
use futures::StreamExt;
use serde_json::json;
use std::collections::HashMap;

/// Handler for the calculator tool
async fn calculator_handler(args: serde_json::Value) -> anyhow::Result<ToolResult> {
    let operation = args["operation"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing operation"))?;
    let a = args["a"]
        .as_f64()
        .ok_or_else(|| anyhow::anyhow!("Missing or invalid parameter 'a'"))?;
    let b = args["b"]
        .as_f64()
        .ok_or_else(|| anyhow::anyhow!("Missing or invalid parameter 'b'"))?;

    let result = match operation {
        "add" => a + b,
        "subtract" => a - b,
        "multiply" => a * b,
        "divide" => {
            if b == 0.0 {
                return Ok(ToolResult {
                    content: vec![McpToolResultContent::Text {
                        text: "Error: Division by zero".to_string(),
                    }],
                    is_error: true,
                });
            }
            a / b
        }
        _ => {
            return Ok(ToolResult {
                content: vec![McpToolResultContent::Text {
                    text: format!("Error: Unknown operation '{}'", operation),
                }],
                is_error: true,
            })
        }
    };

    Ok(ToolResult {
        content: vec![McpToolResultContent::Text {
            text: format!("Result: {} {} {} = {}", a, operation, b, result),
        }],
        is_error: false,
    })
}

/// Handler for the statistics tool
async fn statistics_handler(args: serde_json::Value) -> anyhow::Result<ToolResult> {
    let numbers = args["numbers"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'numbers' array"))?;

    let nums: Vec<f64> = numbers.iter().filter_map(|v| v.as_f64()).collect();

    if nums.is_empty() {
        return Ok(ToolResult {
            content: vec![McpToolResultContent::Text {
                text: "Error: No valid numbers provided".to_string(),
            }],
            is_error: true,
        });
    }

    let sum: f64 = nums.iter().sum();
    let mean = sum / nums.len() as f64;
    let max = nums.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let min = nums.iter().cloned().fold(f64::INFINITY, f64::min);

    // Calculate variance and standard deviation
    let variance = nums.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / nums.len() as f64;
    let std_dev = variance.sqrt();

    let stats = json!({
        "count": nums.len(),
        "sum": sum,
        "mean": mean,
        "min": min,
        "max": max,
        "variance": variance,
        "std_dev": std_dev
    });

    Ok(ToolResult {
        content: vec![McpToolResultContent::Text {
            text: format!("Statistics: {}", serde_json::to_string_pretty(&stats)?),
        }],
        is_error: false,
    })
}

/// Handler for the random number generator tool
async fn random_handler(args: serde_json::Value) -> anyhow::Result<ToolResult> {
    let min = args["min"].as_i64().unwrap_or(0);
    let max = args["max"].as_i64().unwrap_or(100);

    if min >= max {
        return Ok(ToolResult {
            content: vec![McpToolResultContent::Text {
                text: "Error: min must be less than max".to_string(),
            }],
            is_error: true,
        });
    }

    // Simple random number generation (not cryptographically secure)
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let random = (seed % (max - min) as u128) as i64 + min;

    Ok(ToolResult {
        content: vec![McpToolResultContent::Text {
            text: format!("Random number between {} and {}: {}", min, max, random),
        }],
        is_error: false,
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MCP Server Integration Example ===\n");

    // Step 1: Create custom tools using the tool! macro
    println!("Step 1: Creating custom MCP tools...\n");

    let calculator_tool = tool!(
        "calculator",
        "Perform basic arithmetic operations (add, subtract, multiply, divide)",
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["add", "subtract", "multiply", "divide"],
                    "description": "The arithmetic operation to perform"
                },
                "a": {
                    "type": "number",
                    "description": "First operand"
                },
                "b": {
                    "type": "number",
                    "description": "Second operand"
                }
            },
            "required": ["operation", "a", "b"]
        }),
        calculator_handler
    );

    let statistics_tool = tool!(
        "statistics",
        "Calculate statistical measures (mean, median, std dev, etc.) for a list of numbers",
        json!({
            "type": "object",
            "properties": {
                "numbers": {
                    "type": "array",
                    "items": { "type": "number" },
                    "description": "Array of numbers to analyze"
                }
            },
            "required": ["numbers"]
        }),
        statistics_handler
    );

    let random_tool = tool!(
        "random_number",
        "Generate a random number within a specified range",
        json!({
            "type": "object",
            "properties": {
                "min": {
                    "type": "integer",
                    "description": "Minimum value (inclusive)"
                },
                "max": {
                    "type": "integer",
                    "description": "Maximum value (exclusive)"
                }
            },
            "required": ["min", "max"]
        }),
        random_handler
    );

    println!("✓ Created 3 tools: calculator, statistics, random_number\n");

    // Step 2: Create an SDK MCP server with the tools
    println!("Step 2: Creating SDK MCP server...\n");

    let math_server = create_sdk_mcp_server(
        "math-tools",
        "1.0.0",
        vec![calculator_tool, statistics_tool, random_tool],
    );

    println!("✓ Created 'math-tools' MCP server v1.0.0\n");

    // Step 3: Configure ClaudeClient with the MCP server
    println!("Step 3: Configuring Claude client with MCP server...\n");

    let mut mcp_servers = HashMap::new();
    mcp_servers.insert(
        "math-tools".to_string(),
        claude_agent_sdk::McpServerConfig::Sdk(math_server),
    );

    let options = ClaudeAgentOptions {
        mcp_servers: McpServers::Dict(mcp_servers),
        // IMPORTANT: Tools must be explicitly allowed with format mcp__{server_name}__{tool_name}
        allowed_tools: vec![
            "mcp__math-tools__calculator".to_string(),
            "mcp__math-tools__statistics".to_string(),
            "mcp__math-tools__random_number".to_string(),
        ],
        max_turns: Some(10),
        permission_mode: Some(claude_agent_sdk::PermissionMode::AcceptEdits),
        ..Default::default()
    };

    let mut client = ClaudeClient::new(options);

    // Step 4: Connect and interact with Claude using the custom tools
    println!("Step 4: Connecting to Claude...\n");
    client.connect().await?;
    println!("✓ Connected!\n");

    // Query 1: Use the calculator tool
    println!("=== Query 1: Calculator Tool ===");
    println!("User: Calculate 42 multiplied by 7\n");
    client.query("Calculate 42 multiplied by 7").await?;

    let mut stream = client.receive_response();
    while let Some(message) = stream.next().await {
        match message? {
            Message::Assistant(msg) => {
                for block in msg.message.content {
                    match block {
                        ContentBlock::Text(text) => {
                            println!("Claude: {}", text.text);
                        }
                        ContentBlock::ToolUse(tool) => {
                            println!("🔧 Using tool: {} ({})", tool.name, tool.id);
                            println!("   Input: {}", serde_json::to_string_pretty(&tool.input)?);
                        }
                        _ => {}
                    }
                }
            }
            Message::User(user_msg) => {
                // Tool results come as User messages
                if let Some(content_blocks) = &user_msg.content {
                    for content in content_blocks {
                        if let ContentBlock::ToolResult(result) = content {
                            if let Some(claude_agent_sdk::ToolResultContent::Text(text)) =
                                &result.content
                            {
                                println!("📊 Tool result: {}", text);
                            }
                        }
                    }
                }
            }
            Message::Result(result) => {
                println!(
                    "\n[Result] Turns: {}, Duration: {}ms\n",
                    result.num_turns, result.duration_ms
                );
            }
            _ => {}
        }
    }
    drop(stream);

    // Query 2: Use the statistics tool
    println!("=== Query 2: Statistics Tool ===");
    println!("User: Calculate statistics for the numbers: 10, 20, 30, 40, 50\n");
    client
        .query("Calculate statistics for the numbers: 10, 20, 30, 40, 50")
        .await?;

    let mut stream = client.receive_response();
    while let Some(message) = stream.next().await {
        match message? {
            Message::Assistant(msg) => {
                for block in msg.message.content {
                    match block {
                        ContentBlock::Text(text) => {
                            println!("Claude: {}", text.text);
                        }
                        ContentBlock::ToolUse(tool) => {
                            println!("🔧 Using tool: {} ({})", tool.name, tool.id);
                            println!("   Input: {}", serde_json::to_string_pretty(&tool.input)?);
                        }
                        _ => {}
                    }
                }
            }
            Message::User(user_msg) => {
                // Tool results come as User messages
                if let Some(content_blocks) = &user_msg.content {
                    for content in content_blocks {
                        if let ContentBlock::ToolResult(result) = content {
                            if let Some(claude_agent_sdk::ToolResultContent::Text(text)) =
                                &result.content
                            {
                                println!("📊 Tool result: {}", text);
                            }
                        }
                    }
                }
            }
            Message::Result(result) => {
                println!(
                    "\n[Result] Turns: {}, Duration: {}ms\n",
                    result.num_turns, result.duration_ms
                );
            }
            _ => {}
        }
    }
    drop(stream);

    // Query 3: Use the random number generator
    println!("=== Query 3: Random Number Tool ===");
    println!("User: Generate a random number between 1 and 100\n");
    client
        .query("Generate a random number between 1 and 100")
        .await?;

    let mut stream = client.receive_response();
    while let Some(message) = stream.next().await {
        match message? {
            Message::Assistant(msg) => {
                for block in msg.message.content {
                    match block {
                        ContentBlock::Text(text) => {
                            println!("Claude: {}", text.text);
                        }
                        ContentBlock::ToolUse(tool) => {
                            println!("🔧 Using tool: {} ({})", tool.name, tool.id);
                            println!("   Input: {}", serde_json::to_string_pretty(&tool.input)?);
                        }
                        _ => {}
                    }
                }
            }
            Message::User(user_msg) => {
                // Tool results come as User messages
                if let Some(content_blocks) = &user_msg.content {
                    for content in content_blocks {
                        if let ContentBlock::ToolResult(result) = content {
                            if let Some(claude_agent_sdk::ToolResultContent::Text(text)) =
                                &result.content
                            {
                                println!("📊 Tool result: {}", text);
                            }
                        }
                    }
                }
            }
            Message::Result(result) => {
                println!(
                    "\n[Result] Turns: {}, Duration: {}ms\n",
                    result.num_turns, result.duration_ms
                );
            }
            _ => {}
        }
    }
    drop(stream);

    // Query 4: Complex task using multiple tools
    println!("=== Query 4: Multiple Tools ===");
    println!("User: Calculate the average of 15, 25, and 35, then multiply it by 3\n");
    client
        .query("Calculate the average of 15, 25, and 35, then multiply the result by 3")
        .await?;

    let mut stream = client.receive_response();
    while let Some(message) = stream.next().await {
        match message? {
            Message::Assistant(msg) => {
                for block in msg.message.content {
                    match block {
                        ContentBlock::Text(text) => {
                            println!("Claude: {}", text.text);
                        }
                        ContentBlock::ToolUse(tool) => {
                            println!("🔧 Using tool: {} ({})", tool.name, tool.id);
                            println!("   Input: {}", serde_json::to_string_pretty(&tool.input)?);
                        }
                        _ => {}
                    }
                }
            }
            Message::User(user_msg) => {
                // Tool results come as User messages
                if let Some(content_blocks) = &user_msg.content {
                    for content in content_blocks {
                        if let ContentBlock::ToolResult(result) = content {
                            if let Some(claude_agent_sdk::ToolResultContent::Text(text)) =
                                &result.content
                            {
                                println!("📊 Tool result: {}", text);
                            }
                        }
                    }
                }
            }
            Message::Result(result) => {
                println!(
                    "\n[Result] Turns: {}, Duration: {}ms\n",
                    result.num_turns, result.duration_ms
                );
            }
            _ => {}
        }
    }
    drop(stream);

    // Disconnect
    println!("Disconnecting...");
    client.disconnect().await?;
    println!("✓ Disconnected!\n");

    println!("=== Example Complete ===");
    println!("\nKey Takeaways:");
    println!("• Created custom tools with the tool! macro");
    println!("• Built an SDK MCP server with multiple tools");
    println!("• Integrated MCP server with ClaudeClient");
    println!("• Claude automatically selected and used appropriate tools");
    println!("• Tools can be composed to solve complex tasks");

    Ok(())
}
