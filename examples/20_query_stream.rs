//! Example 20: Streaming Query API
//!
//! This example demonstrates the `query_stream()` function, which provides a memory-efficient
//! streaming alternative to the standard `query()` function.
//!
//! Key differences:
//! - `query()`: Collects all messages in memory before returning (O(n) memory)
//! - `query_stream()`: Streams messages as they arrive (O(1) memory per message)
//!
//! When to use `query_stream()`:
//! - Large conversations that might consume significant memory
//! - Real-time processing of messages as they arrive
//! - Long-running operations where you want immediate feedback
//! - Applications with memory constraints
//!
//! When to use `query()`:
//! - Small to medium conversations
//! - When you need all messages collected for post-processing
//! - Simpler code when memory isn't a concern

use claude_agent_sdk_rs::{
    ClaudeAgentOptions, ContentBlock, Message, PermissionMode, query_stream,
};
use futures::stream::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Example 20: Streaming Query API ===\n");

    // Configure options
    let options = ClaudeAgentOptions::builder()
        .allowed_tools(vec!["Bash".to_string()])
        .permission_mode(PermissionMode::BypassPermissions)
        .max_turns(5)
        .build();

    println!("Using query_stream() for real-time message processing...\n");
    println!("Asking Claude to perform a task...\n");

    // Use query_stream instead of query
    let mut stream = query_stream(
        "Run the command: echo 'Streaming example' and tell me what you see",
        Some(options),
    )
    .await?;

    let mut message_count = 0;
    let start_time = std::time::Instant::now();

    // Process messages as they arrive (streaming)
    while let Some(result) = stream.next().await {
        message_count += 1;
        let message = result?;
        let elapsed = start_time.elapsed().as_millis();

        match message {
            Message::Assistant(msg) => {
                println!(
                    "[{}ms] Message #{}: Assistant response",
                    elapsed, message_count
                );

                for block in &msg.message.content {
                    match block {
                        ContentBlock::Text(text) => {
                            println!("  Text: {}", text.text);
                        }
                        ContentBlock::ToolUse(tool) => {
                            println!("  Tool: {} ({})", tool.name, tool.id);
                            println!("  Input: {}", serde_json::to_string(&tool.input)?);
                        }
                        ContentBlock::ToolResult(result) => {
                            println!("  Tool Result: {}", result.tool_use_id);
                            if let Some(ref content) = result.content {
                                match content {
                                    claude_agent_sdk_rs::ToolResultContent::Text(text) => {
                                        println!("    {}", text);
                                    }
                                    claude_agent_sdk_rs::ToolResultContent::Blocks(blocks) => {
                                        println!("    {} blocks", blocks.len());
                                    }
                                }
                            }
                        }
                        ContentBlock::Thinking(thinking) => {
                            println!("  Thinking: {} chars", thinking.thinking.len());
                        }
                    }
                }
                println!();
            }
            Message::System(sys) => {
                println!(
                    "[{}ms] Message #{}: System ({})",
                    elapsed, message_count, sys.subtype
                );
                println!();
            }
            Message::Result(result) => {
                println!("[{}ms] Message #{}: Result", elapsed, message_count);
                println!("  Duration: {}ms", result.duration_ms);
                println!("  Turns: {}", result.num_turns);
                println!("  Error: {}", result.is_error);
                if let Some(cost) = result.total_cost_usd {
                    println!("  Cost: ${:.4}", cost);
                }
                println!();
            }
            Message::StreamEvent(event) => {
                println!("[{}ms] Message #{}: Stream Event", elapsed, message_count);
                println!("  Session: {}", event.session_id);
                println!();
            }
            _ => {}
        }
    }

    println!("=== Streaming Complete ===");
    println!("Total messages processed: {}", message_count);
    println!("Total time: {}ms", start_time.elapsed().as_millis());

    println!("\n=== Performance Comparison ===");
    println!("query():          Waits for all messages, then returns Vec<Message>");
    println!("                  Memory: O(n) where n = number of messages");
    println!("                  Latency: Returns after conversation completes");
    println!();
    println!("query_stream():   Returns Stream<Item = Result<Message>>");
    println!("                  Memory: O(1) per message (constant)");
    println!("                  Latency: Processes messages as they arrive");
    println!();
    println!("✓ For this example, streaming allows real-time progress updates!");

    Ok(())
}
