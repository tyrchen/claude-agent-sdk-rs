//! Example 23: Multimodal Image Input
//!
//! This example demonstrates how to send images alongside text to Claude using
//! the `query_with_content()` and `query_stream_with_content()` functions.
//!
//! The SDK supports two types of image sources:
//! - Base64-encoded image data (useful for local files)
//! - URL references (useful for publicly accessible images)
//!
//! Supported image formats:
//! - JPEG (`image/jpeg`)
//! - PNG (`image/png`)
//! - GIF (`image/gif`)
//! - WebP (`image/webp`)
//!
//! Size limits:
//! - Maximum base64 data: 15MB (results in ~20MB decoded)
//! - Large images may timeout - consider resizing before encoding

use claude_agent_sdk_rs::{
    ClaudeAgentOptions, ContentBlock, Message, PermissionMode, UserContentBlock,
    query_stream_with_content, query_with_content,
};
use futures::stream::StreamExt;

/// A minimal 1x1 red PNG pixel for demonstration
/// In real usage, you would load and encode actual image files
const SAMPLE_RED_PNG_BASE64: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==";

/// A minimal 1x1 blue PNG pixel
const SAMPLE_BLUE_PNG_BASE64: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPj/HwADBwIAMCbHYQAAAABJRU5ErkJggg==";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Example 23: Multimodal Image Input ===\n");

    // Configure options for the examples
    let options = ClaudeAgentOptions::builder()
        .model("sonnet".to_string()) // Use Sonnet for lower cost
        .permission_mode(PermissionMode::BypassPermissions)
        .max_turns(1)
        .build();

    // Example 1: Simple query with base64 image
    println!("--- Example 1: Query with Base64 Image ---\n");
    example_base64_image(options.clone()).await?;

    // Example 2: Query with multiple images
    println!("\n--- Example 2: Query with Multiple Images ---\n");
    example_multiple_images(options.clone()).await?;

    // Example 3: Streaming with image content
    println!("\n--- Example 3: Streaming with Image Content ---\n");
    example_streaming_with_image(options.clone()).await?;

    // Example 4: Image URL reference
    println!("\n--- Example 4: Image URL Reference ---\n");
    example_image_url().await?;

    // Example 5: Error handling for invalid MIME types
    println!("\n--- Example 5: Validation Error Handling ---\n");
    example_validation_errors();

    println!("\n=== All Examples Complete ===");
    Ok(())
}

/// Example 1: Basic query with a base64-encoded image
async fn example_base64_image(options: ClaudeAgentOptions) -> anyhow::Result<()> {
    println!("Creating content with text and image...");

    // Build content blocks
    let content = vec![
        UserContentBlock::text(
            "What color is this 1x1 pixel image? Reply with just the color name.",
        ),
        UserContentBlock::image_base64("image/png", SAMPLE_RED_PNG_BASE64)?,
    ];

    println!("Sending query with image to Claude...");
    let messages = query_with_content(content, Some(options)).await?;

    // Process response
    for message in messages {
        if let Message::Assistant(msg) = message {
            for block in &msg.message.content {
                if let ContentBlock::Text(text) = block {
                    println!("Claude's response: {}", text.text);
                }
            }
        }
    }

    Ok(())
}

/// Example 2: Query with multiple images for comparison
async fn example_multiple_images(options: ClaudeAgentOptions) -> anyhow::Result<()> {
    println!("Creating content with multiple images for comparison...");

    let content = vec![
        UserContentBlock::text(
            "I'm showing you two 1x1 pixel images. What colors are they? Reply briefly.",
        ),
        UserContentBlock::image_base64("image/png", SAMPLE_RED_PNG_BASE64)?,
        UserContentBlock::image_base64("image/png", SAMPLE_BLUE_PNG_BASE64)?,
    ];

    println!("Sending query with multiple images...");
    let messages = query_with_content(content, Some(options)).await?;

    for message in messages {
        if let Message::Assistant(msg) = message {
            for block in &msg.message.content {
                if let ContentBlock::Text(text) = block {
                    println!("Claude's response: {}", text.text);
                }
            }
        }
    }

    Ok(())
}

/// Example 3: Using streaming API with image content
async fn example_streaming_with_image(options: ClaudeAgentOptions) -> anyhow::Result<()> {
    println!("Creating streaming query with image...");

    let content = vec![
        UserContentBlock::image_base64("image/png", SAMPLE_RED_PNG_BASE64)?,
        UserContentBlock::text("Describe what you see in this image. Keep it brief."),
    ];

    let mut stream = query_stream_with_content(content, Some(options)).await?;

    print!("Claude's response: ");
    while let Some(result) = stream.next().await {
        let message = result?;
        if let Message::Assistant(msg) = message {
            for block in &msg.message.content {
                if let ContentBlock::Text(text) = block {
                    print!("{}", text.text);
                }
            }
        }
    }
    println!();

    Ok(())
}

/// Example 4: Using an image URL (does not make actual request)
async fn example_image_url() -> anyhow::Result<()> {
    println!("Creating content with image URL...");

    // Note: This example just shows how to construct the content
    // In real usage, this would be sent to Claude
    let content = [
        UserContentBlock::text("Describe this diagram"),
        UserContentBlock::image_url("https://example.com/architecture-diagram.png"),
    ];

    println!("Content blocks created:");
    for (i, block) in content.iter().enumerate() {
        let json = serde_json::to_string_pretty(block)?;
        println!("  Block {}: {}", i + 1, json);
    }

    println!("\nNote: URL images require publicly accessible URLs.");
    println!("The URL is passed to Claude who fetches the image directly.");

    Ok(())
}

/// Example 5: Demonstration of validation errors
fn example_validation_errors() {
    println!("Testing validation for unsupported MIME types...\n");

    // Test invalid MIME type
    let result = UserContentBlock::image_base64("image/bmp", "somedata");
    match result {
        Err(e) => println!("Expected error for image/bmp: {}", e),
        Ok(_) => println!("Unexpected: image/bmp should have failed"),
    }

    let result = UserContentBlock::image_base64("image/tiff", "somedata");
    match result {
        Err(e) => println!("Expected error for image/tiff: {}", e),
        Ok(_) => println!("Unexpected: image/tiff should have failed"),
    }

    // Show supported types
    println!("\nSupported MIME types:");
    for mime in &["image/jpeg", "image/png", "image/gif", "image/webp"] {
        let result = UserContentBlock::image_base64(*mime, "data");
        match result {
            Ok(_) => println!("  {} - OK", mime),
            Err(_) => println!("  {} - FAILED (unexpected)", mime),
        }
    }
}
