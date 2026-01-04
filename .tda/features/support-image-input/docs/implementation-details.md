# Implementation Details: Support Image Input in User Prompts

**Version**: 1.0
**Date**: 2026-01-03
**Status**: Completed

---

## Overview

This document details the implementation of multimodal image input support for the Claude Agent SDK for Rust. The feature enables users to include images alongside text in their prompts, supporting both base64-encoded images and URL references.

---

## Implementation Summary

### Commits

| Commit | Phase | Description |
|--------|-------|-------------|
| `3fcb6e2` | Phase 1 | Type definitions: ImageSource, ImageBlock, UserContentBlock |
| `8393cb9` | Phase 2 | Transport layer: QueryPrompt Content variant |
| `516495a` | Phase 3 | Query functions: query_with_content, query_stream_with_content |
| `7250bb8` | Phase 4 | Client methods: ClaudeClient::query_with_content* |

### Files Changed

| File | Lines Added | Lines Removed | Description |
|------|-------------|---------------|-------------|
| `src/types/messages.rs` | +296 | 0 | New types and unit tests |
| `src/internal/transport/subprocess.rs` | +30 | -3 | QueryPrompt Content variant |
| `src/query.rs` | +117 | -1 | New query functions |
| `src/client.rs` | +126 | -2 | New client methods |
| `src/lib.rs` | +2 | -1 | Export new functions |
| `examples/20_query_stream.rs` | +8 | 0 | Handle Image variant |

**Total**: ~571 lines added

---

## Detailed Implementation

### Phase 1: Type Definitions

**File**: `src/types/messages.rs`

#### New Types

1. **ImageSource** (enum)
   - `Base64 { media_type: String, data: String }` - For base64-encoded image data
   - `Url { url: String }` - For URL-referenced images
   - Uses `#[serde(tag = "type", rename_all = "snake_case")]` for proper JSON serialization

2. **ImageBlock** (struct)
   - Simple wrapper containing `source: ImageSource`
   - Used within ContentBlock for response parsing

3. **UserContentBlock** (enum)
   - `Text { text: String }` - Text content block
   - `Image { source: ImageSource }` - Image content block
   - Builder methods: `text()`, `image_base64()`, `image_url()`
   - `From<String>` and `From<&str>` implementations for ergonomic conversion

4. **ContentBlock Update**
   - Added `Image(ImageBlock)` variant for parsing assistant responses containing images

#### Serialization Format

```json
// UserContentBlock::Text
{"type": "text", "text": "What's in this image?"}

// UserContentBlock::Image with Base64
{"type": "image", "source": {"type": "base64", "media_type": "image/png", "data": "iVBORw0KGgo..."}}

// UserContentBlock::Image with URL
{"type": "image", "source": {"type": "url", "url": "https://example.com/image.png"}}
```

#### Unit Tests Added

- `test_image_source_base64_serialization`
- `test_image_source_url_serialization`
- `test_image_source_base64_deserialization`
- `test_image_source_url_deserialization`
- `test_user_content_block_text_serialization`
- `test_user_content_block_image_base64_serialization`
- `test_user_content_block_image_url_serialization`
- `test_user_content_block_deserialization`
- `test_user_content_block_image_deserialization`
- `test_user_content_block_from_string`
- `test_user_content_block_from_owned_string`
- `test_image_block_serialization`
- `test_content_block_image_serialization`
- `test_content_block_image_deserialization`

---

### Phase 2: Transport Layer

**File**: `src/internal/transport/subprocess.rs`

#### QueryPrompt Extension

```rust
pub enum QueryPrompt {
    Text(String),
    Content(Vec<UserContentBlock>),  // NEW
    Streaming,
}
```

#### Changes Made

1. Added import for `UserContentBlock` from `crate::types::messages`
2. Added `Content(Vec<UserContentBlock>)` variant to `QueryPrompt` enum
3. Implemented `From<Vec<UserContentBlock>> for QueryPrompt`
4. Updated `build_command()` to enable `stream-json` input format for Content mode
5. Updated `connect()` to serialize Content blocks as JSON user messages:

```rust
QueryPrompt::Content(blocks) => {
    let user_message = serde_json::json!({
        "type": "user",
        "message": {
            "role": "user",
            "content": blocks
        },
        "session_id": "default"
    });
    let message_str = serde_json::to_string(&user_message)?;
    self.write(&message_str).await?;
    self.end_input().await?;
}
```

---

### Phase 3: Query Functions

**File**: `src/query.rs`

#### New Functions

1. **query_with_content()**
   ```rust
   pub async fn query_with_content(
       content: impl Into<Vec<UserContentBlock>>,
       options: Option<ClaudeAgentOptions>,
   ) -> Result<Vec<Message>>
   ```
   - One-shot query with structured content blocks
   - Collects all messages and returns them as a vector

2. **query_stream_with_content()**
   ```rust
   pub async fn query_stream_with_content(
       content: impl Into<Vec<UserContentBlock>>,
       options: Option<ClaudeAgentOptions>,
   ) -> Result<Pin<Box<dyn Stream<Item = Result<Message>> + Send>>>
   ```
   - Streaming query with structured content blocks
   - Returns messages as they arrive for memory-efficient processing

---

### Phase 4: Client Methods

**File**: `src/client.rs`

#### New Methods

1. **ClaudeClient::query_with_content()**
   ```rust
   pub async fn query_with_content(
       &mut self,
       content: impl Into<Vec<UserContentBlock>>,
   ) -> Result<()>
   ```
   - Sends content blocks using the default session ID

2. **ClaudeClient::query_with_content_and_session()**
   ```rust
   pub async fn query_with_content_and_session(
       &mut self,
       content: impl Into<Vec<UserContentBlock>>,
       session_id: impl Into<String>,
   ) -> Result<()>
   ```
   - Sends content blocks with a specific session ID
   - Enables maintaining separate conversation contexts

---

### Phase 5: Public Exports

**File**: `src/lib.rs`

Updated exports to include:
```rust
pub use query::{query, query_stream, query_stream_with_content, query_with_content};
```

Types `ImageSource`, `ImageBlock`, and `UserContentBlock` are automatically exported via `pub use types::messages::*`.

---

## API Usage Examples

### One-Shot Query with Image

```rust
use claude_agent_sdk_rs::{query_with_content, UserContentBlock, Message, ContentBlock};
use std::fs;
use base64::Engine;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Read and encode image
    let image_bytes = fs::read("screenshot.png")?;
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&image_bytes);

    // Query with text and image
    let messages = query_with_content(vec![
        UserContentBlock::text("What's shown in this screenshot?"),
        UserContentBlock::image_base64("image/png", base64_data),
    ], None).await?;

    for message in messages {
        if let Message::Assistant(msg) = message {
            for block in &msg.message.content {
                if let ContentBlock::Text(text) = block {
                    println!("Claude: {}", text.text);
                }
            }
        }
    }

    Ok(())
}
```

### Streaming Query with URL Image

```rust
use claude_agent_sdk_rs::{query_stream_with_content, UserContentBlock, Message, ContentBlock};
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut stream = query_stream_with_content(vec![
        UserContentBlock::text("Describe this image"),
        UserContentBlock::image_url("https://example.com/photo.jpg"),
    ], None).await?;

    while let Some(result) = stream.next().await {
        match result? {
            Message::Assistant(msg) => {
                for block in &msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        print!("{}", text.text);
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}
```

### Bidirectional Client with Images

```rust
use claude_agent_sdk_rs::{ClaudeClient, ClaudeAgentOptions, UserContentBlock, Message};
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = ClaudeClient::new(ClaudeAgentOptions::default());
    client.connect().await?;

    // Send query with image
    client.query_with_content(vec![
        UserContentBlock::text("What's in this image?"),
        UserContentBlock::image_base64("image/png", base64_data),
    ]).await?;

    // Receive response
    {
        let mut stream = client.receive_response();
        while let Some(result) = stream.next().await {
            match result? {
                Message::Assistant(msg) => { /* handle response */ }
                Message::Result(_) => break,
                _ => {}
            }
        }
    }

    client.disconnect().await?;
    Ok(())
}
```

---

## Testing

### Test Results

- **Unit Tests**: 65 passed
- **Integration Tests**: 27 passed (12 run, 15 ignored)
- **Fixture Tests**: 28 passed
- **Doc Tests**: 24 passed

All tests pass with `cargo test`.

### Linting

- `cargo clippy -- -D warnings` passes with no warnings
- `cargo fmt -- --check` passes with correct formatting

---

## Backward Compatibility

All changes are additive and maintain full backward compatibility:

1. Existing `query()` and `query_stream()` functions work unchanged
2. Existing `ClaudeClient::query()` and `ClaudeClient::query_with_session()` work unchanged
3. New functions are additions, not modifications
4. `UserContentBlock` implements `From<String>` and `From<&str>` for easy conversion

---

## Security Considerations

1. **No Image Content Validation**: SDK does not validate image content; Claude API performs content moderation
2. **Base64 Size**: Large images increase memory usage; users should be aware of Claude API limits (~20MB per request)
3. **URL Security**: SDK does not fetch or validate URLs; users must ensure URLs are trustworthy

---

## Dependencies

No new dependencies added. Uses existing:
- `serde` for serialization
- `serde_json` for JSON handling
- `tokio` for async runtime
- `futures` for stream handling

---

## Future Enhancements

Potential improvements for future versions:
1. Helper method to create image block from file path (with base64 encoding)
2. Image size validation with configurable limits
3. Support for additional image formats as Claude API evolves
4. Metrics/logging for multimodal query usage
