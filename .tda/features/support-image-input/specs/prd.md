# Support Image Input in User Prompts

**Version**: 1.0
**Date**: 2026-01-03
**Author**: Specification Agent
**Status**: Draft

---

## 1. Executive Summary

This specification defines the requirements for adding image input support to user prompts in the Claude Agent SDK for Rust. Currently, the SDK only supports text-based prompts, but Claude's vision capabilities require the ability to send images alongside text. This feature will enable multimodal interactions where users can include images (screenshots, diagrams, photos) in their queries to Claude.

## 2. Problem Statement

### 2.1 Current State

The Rust SDK currently only supports text prompts in user messages:

1. **`query()` and `query_stream()` functions** accept `impl Into<String>`:
   ```rust
   pub async fn query(
       prompt: impl Into<String>,
       options: Option<ClaudeAgentOptions>,
   ) -> Result<Vec<Message>>
   ```

2. **`ClaudeClient::query()` and `ClaudeClient::query_with_session()`** also accept `impl Into<String>`:
   ```rust
   pub async fn query(&mut self, prompt: impl Into<String>) -> Result<()>
   ```

3. **Message serialization in `client.rs`** constructs a simple text content:
   ```rust
   let user_message = serde_json::json!({
       "type": "user",
       "message": {
           "role": "user",
           "content": prompt_str  // Only text supported
       },
       "session_id": session_id_str
   });
   ```

4. **Existing `ContentBlock` enum** lacks an `Image` variant:
   ```rust
   pub enum ContentBlock {
       Text(TextBlock),
       Thinking(ThinkingBlock),
       ToolUse(ToolUseBlock),
       ToolResult(ToolResultBlock),
       // No Image variant!
   }
   ```

### 2.2 Desired State

Users should be able to send images along with text in their prompts:

```rust
// Example: Query with image using content blocks
let messages = query_with_content(vec![
    UserContentBlock::Text { text: "What's in this image?".to_string() },
    UserContentBlock::Image {
        source: ImageSource::Base64 {
            media_type: "image/png".to_string(),
            data: base64_encoded_image,
        },
    },
], None).await?;

// Example: ClaudeClient with image
client.query_with_content(vec![
    UserContentBlock::text("Describe this diagram"),
    UserContentBlock::image_base64("image/png", base64_data),
]).await?;
```

### 2.3 Success Criteria

1. Users can send base64-encoded images in user prompts
2. API maintains backward compatibility (existing text-only code works unchanged)
3. Type safety is preserved with strongly-typed Rust structs
4. Feature parity with Python SDK for image input capabilities
5. Proper serialization to Claude Code CLI's expected format

## 3. Stakeholders

| Role | Concerns | Success Criteria |
|------|----------|------------------|
| SDK Users | Easy-to-use API for multimodal prompts | Ergonomic builder methods, clear documentation |
| SDK Maintainers | Backward compatibility, code consistency | No breaking changes, follows existing patterns |
| Claude Code CLI | Correct message format | Valid JSON structure matching CLI expectations |

## 4. Functional Requirements

### FR-001: Add ImageBlock to ContentBlock Enum
**Priority**: Must Have
**Description**: Add an `Image` variant to the `ContentBlock` enum in `src/types/messages.rs` for deserializing image blocks from responses.
**Rationale**: Enables the SDK to handle image content in assistant responses (e.g., when tools return images).
**Dependencies**: None

### FR-002: Create UserContentBlock Enum for User Prompts
**Priority**: Must Have
**Description**: Create a new `UserContentBlock` enum specifically for constructing user prompts with text and image content.
**Rationale**: User prompts have a different structure than assistant responses. Separating concerns allows for cleaner API design and proper serialization.
**Dependencies**: None

### FR-003: Create ImageSource and ImageBlock Structs
**Priority**: Must Have
**Description**: Create the `ImageSource` enum and `ImageBlock` struct to represent image data with proper serde attributes for serialization.
**Rationale**: Claude API expects a specific JSON structure for images with `source.type`, `source.media_type`, and `source.data` fields.
**Dependencies**: None

### FR-004: Add query_with_content() Function
**Priority**: Must Have
**Description**: Add a new `query_with_content()` function that accepts `Vec<UserContentBlock>` instead of a string prompt.
**Rationale**: Provides a clean API for multimodal queries while keeping the existing `query()` function for backward compatibility.
**Dependencies**: FR-002, FR-003

### FR-005: Add query_stream_with_content() Function
**Priority**: Must Have
**Description**: Add a streaming variant that accepts content blocks for memory-efficient processing of multimodal queries.
**Rationale**: Maintains feature parity between one-shot and streaming APIs.
**Dependencies**: FR-002, FR-003, FR-004

### FR-006: Add ClaudeClient::query_with_content() Method
**Priority**: Must Have
**Description**: Add a method to `ClaudeClient` for sending multimodal prompts in bidirectional streaming mode.
**Rationale**: Enables multimodal queries in the full client API.
**Dependencies**: FR-002, FR-003

### FR-007: Add ClaudeClient::query_with_content_and_session() Method
**Priority**: Must Have
**Description**: Add a session-aware variant of the multimodal query method.
**Rationale**: Maintains consistency with existing `query_with_session()` pattern.
**Dependencies**: FR-006

### FR-008: Support URL-Based Image Sources
**Priority**: Should Have
**Description**: Support URL-based image sources in addition to base64-encoded data.
**Rationale**: Some use cases prefer referencing images by URL rather than embedding data.
**Dependencies**: FR-003

### FR-009: Add Builder/Helper Methods for UserContentBlock
**Priority**: Should Have
**Description**: Add ergonomic builder methods like `UserContentBlock::text()`, `UserContentBlock::image_base64()`, and `UserContentBlock::image_url()`.
**Rationale**: Improves developer experience with idiomatic Rust patterns.
**Dependencies**: FR-002, FR-003

### FR-010: Add From Implementations for Backward Compatibility
**Priority**: Must Have
**Description**: Implement `From<String>` and `From<&str>` for content block types to allow seamless migration.
**Rationale**: Allows existing code patterns to work with new APIs through type coercion.
**Dependencies**: FR-002

### FR-011: Update QueryPrompt Enum
**Priority**: Must Have
**Description**: Extend `QueryPrompt` enum in `subprocess.rs` to support structured content blocks, not just text strings.
**Rationale**: The transport layer needs to serialize content blocks correctly for the CLI.
**Dependencies**: FR-002, FR-003

### FR-012: Export New Types in lib.rs
**Priority**: Must Have
**Description**: Export `UserContentBlock`, `ImageSource`, and `ImageBlock` from the public API in `lib.rs`.
**Rationale**: Users need access to these types to construct multimodal prompts.
**Dependencies**: FR-002, FR-003

## 5. Non-Functional Requirements

### NFR-001: Backward Compatibility
**Category**: Maintainability
**Description**: All existing code using `query()`, `query_stream()`, and `ClaudeClient::query()` must continue to work without modification.
**Measurement**: Existing tests pass without changes; no breaking API changes.

### NFR-002: Type Safety
**Category**: Reliability
**Description**: Image media types and content must be validated at compile time where possible.
**Measurement**: Invalid constructions (e.g., missing required fields) fail at compile time.

### NFR-003: Serialization Correctness
**Category**: Reliability
**Description**: Image content blocks must serialize to valid JSON matching Claude API expectations.
**Measurement**: Unit tests verify correct JSON output structure.

### NFR-004: Documentation
**Category**: Usability
**Description**: All new public types and functions must have rustdoc documentation with examples.
**Measurement**: `cargo doc` produces complete documentation; examples compile.

### NFR-005: Memory Efficiency
**Category**: Performance
**Description**: Image data should not be unnecessarily copied during serialization.
**Measurement**: Single allocation for base64 data in typical use cases.

## 6. Acceptance Criteria

### AC-001: Basic Image Query
**Given** a base64-encoded PNG image and text prompt
**When** the user calls `query_with_content()` with both
**Then** Claude receives and processes the multimodal content successfully

### AC-002: Streaming Image Query
**Given** a base64-encoded image
**When** the user calls `query_stream_with_content()`
**Then** responses stream correctly with proper message types

### AC-003: ClaudeClient Image Query
**Given** an active `ClaudeClient` connection
**When** the user calls `query_with_content()` on the client
**Then** the image is sent and responses are received via `receive_response()`

### AC-004: Backward Compatibility - query()
**Given** existing code using `query("text prompt", None)`
**When** the code is compiled with the new SDK version
**Then** the code compiles and runs without modification

### AC-005: Backward Compatibility - ClaudeClient::query()
**Given** existing code using `client.query("text prompt").await`
**When** the code is compiled with the new SDK version
**Then** the code compiles and runs without modification

### AC-006: Text-Only Content Blocks
**Given** a `Vec<UserContentBlock>` containing only text blocks
**When** serialized to JSON
**Then** the output matches the expected CLI format for text content

### AC-007: Mixed Content Serialization
**Given** a `Vec<UserContentBlock>` with text and image blocks
**When** serialized to JSON
**Then** the JSON structure matches Claude API's expected format:
```json
{
  "type": "user",
  "message": {
    "role": "user",
    "content": [
      {"type": "text", "text": "What is this?"},
      {"type": "image", "source": {"type": "base64", "media_type": "image/png", "data": "..."}}
    ]
  }
}
```

### AC-008: URL Image Source
**Given** a `UserContentBlock::Image` with URL source
**When** serialized to JSON
**Then** the output contains `{"type": "url", "url": "https://..."}` source structure

### AC-009: Builder Method Ergonomics
**Given** the helper methods `UserContentBlock::text()` and `UserContentBlock::image_base64()`
**When** used to construct content
**Then** the syntax is concise and readable

### AC-010: Error Handling
**Given** an invalid image format or empty data
**When** the message is processed by Claude
**Then** a clear error is returned (runtime validation deferred to Claude)

## 7. Technical Approach

### 7.1 Architecture Overview

The implementation adds new types for user-side content blocks while preserving existing types for parsing responses. The separation ensures:
1. User input types have serialization focus
2. Response types have deserialization focus
3. Clear ownership of concerns

### 7.2 Components Affected

| Component | Nature of Change |
|-----------|------------------|
| `src/types/messages.rs` | Add `ImageBlock` to `ContentBlock`, add new user content types |
| `src/query.rs` | Add `query_with_content()` and `query_stream_with_content()` functions |
| `src/client.rs` | Add `query_with_content()` and `query_with_content_and_session()` methods |
| `src/internal/transport/subprocess.rs` | Extend `QueryPrompt` to support content blocks |
| `src/lib.rs` | Export new types |

### 7.3 Data Model Changes

#### New Types in `src/types/messages.rs`:

```rust
/// Image source for user prompts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ImageSource {
    /// Base64-encoded image data
    Base64 {
        /// MIME type (e.g., "image/png", "image/jpeg", "image/gif", "image/webp")
        media_type: String,
        /// Base64-encoded image data
        data: String,
    },
    /// URL reference to an image
    Url {
        /// Image URL
        url: String,
    },
}

/// Image block for user prompts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageBlock {
    /// Image source
    pub source: ImageSource,
}

/// Content block for user prompts (input)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UserContentBlock {
    /// Text content
    Text {
        /// The text content
        text: String,
    },
    /// Image content
    Image {
        /// Image source
        source: ImageSource,
    },
}

impl UserContentBlock {
    /// Create a text content block
    pub fn text(text: impl Into<String>) -> Self {
        UserContentBlock::Text { text: text.into() }
    }

    /// Create an image content block from base64 data
    pub fn image_base64(media_type: impl Into<String>, data: impl Into<String>) -> Self {
        UserContentBlock::Image {
            source: ImageSource::Base64 {
                media_type: media_type.into(),
                data: data.into(),
            },
        }
    }

    /// Create an image content block from URL
    pub fn image_url(url: impl Into<String>) -> Self {
        UserContentBlock::Image {
            source: ImageSource::Url { url: url.into() },
        }
    }
}

impl From<String> for UserContentBlock {
    fn from(text: String) -> Self {
        UserContentBlock::Text { text }
    }
}

impl From<&str> for UserContentBlock {
    fn from(text: &str) -> Self {
        UserContentBlock::Text { text: text.to_string() }
    }
}
```

#### Update to `ContentBlock` enum:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Text block
    Text(TextBlock),
    /// Image block (for responses containing images)
    Image(ImageBlock),
    /// Thinking block (extended thinking)
    Thinking(ThinkingBlock),
    /// Tool use block
    ToolUse(ToolUseBlock),
    /// Tool result block
    ToolResult(ToolResultBlock),
}
```

### 7.4 API Specifications

#### New Functions in `src/query.rs`:

```rust
/// Query Claude with structured content blocks (supports images)
pub async fn query_with_content(
    content: impl Into<Vec<UserContentBlock>>,
    options: Option<ClaudeAgentOptions>,
) -> Result<Vec<Message>>

/// Query Claude with streaming and structured content blocks
pub async fn query_stream_with_content(
    content: impl Into<Vec<UserContentBlock>>,
    options: Option<ClaudeAgentOptions>,
) -> Result<Pin<Box<dyn Stream<Item = Result<Message>> + Send>>>
```

#### New Methods in `src/client.rs`:

```rust
impl ClaudeClient {
    /// Send a query with structured content blocks
    pub async fn query_with_content(
        &mut self,
        content: impl Into<Vec<UserContentBlock>>,
    ) -> Result<()>

    /// Send a query with structured content blocks and session ID
    pub async fn query_with_content_and_session(
        &mut self,
        content: impl Into<Vec<UserContentBlock>>,
        session_id: impl Into<String>,
    ) -> Result<()>
}
```

### 7.5 Integration Points

1. **Claude Code CLI**: Messages are serialized to JSON and sent via stdin to the CLI subprocess
2. **Transport Layer**: `QueryPrompt` enum extended to handle content blocks
3. **MCP Tool Results**: Already support images via `ToolResultContent::Image` - no changes needed

## 8. Constraints and Assumptions

### 8.1 Technical Constraints

- Base64 encoding increases data size by ~33%
- Large images may impact performance and memory usage
- Claude Code CLI must support the image content format (assumed supported)

### 8.2 Business Constraints

- Feature must maintain backward compatibility (no breaking changes)
- Must follow existing SDK patterns for consistency

### 8.3 Assumptions

- Claude Code CLI accepts the standard Claude API content block format
- Users are responsible for proper base64 encoding of images
- Supported image formats: JPEG, PNG, GIF, WebP (as per Claude API)

## 9. Risks and Mitigation

| Risk | Impact | Probability | Mitigation Strategy |
|------|--------|-------------|---------------------|
| CLI format mismatch | High | Low | Verify format against CLI documentation and test with real CLI |
| Large image memory issues | Medium | Medium | Document size limits; consider streaming for large images in future |
| Base64 encoding errors | Medium | Low | Provide helper utilities or clear documentation for encoding |
| Breaking existing code | High | Low | Extensive testing of existing patterns; use additive API changes only |

## 10. Out of Scope

1. **Automatic image resizing or compression** - Users handle preprocessing
2. **Image format conversion** - Users provide correct format
3. **Streaming image upload** - Base64 encoding is the initial approach
4. **File path-based image loading** - Users read files themselves
5. **Image validation** - Deferred to Claude API for format validation
6. **PDF or document support** - Text and image only in this iteration

## 11. Open Questions

1. **Q: Should we provide a helper for reading image files from disk?**
   - Recommendation: Out of scope for initial implementation; users can use `std::fs::read()` and `base64::encode()`

2. **Q: Should we validate image media types?**
   - Recommendation: No compile-time validation; let Claude API return errors for invalid types

3. **Q: Maximum image size limits?**
   - Recommendation: Document Claude API limits in rustdocs; no SDK-side enforcement

## 12. Appendices

### A. Glossary

| Term | Definition |
|------|------------|
| Content Block | A unit of content in a message (text, image, tool use, etc.) |
| Base64 | Binary-to-text encoding scheme for embedding binary data |
| Media Type | MIME type indicating the format of data (e.g., "image/png") |
| Multimodal | Involving multiple types of input (text + images) |

### B. References

1. [Claude API Documentation - Vision](https://docs.anthropic.com/en/docs/vision)
2. [Python SDK Implementation](vendors/claude-agent-sdk-python/src/claude_agent_sdk/types.py)
3. [Existing ToolResultContent::Image](src/types/mcp.rs:102-118)

### C. Change Log

| Version | Date | Changes | Author |
|---------|------|---------|--------|
| 1.0 | 2026-01-03 | Initial draft | Specification Agent |
