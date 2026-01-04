# Implementation Plan: Support Image Input in User Prompts

**Version**: 1.0
**Date**: 2026-01-03
**Based on**: Technical Design v1.0
**Estimated Timeline**: 2-3 days
**Critical Path**: Phase 1 → Phase 2 → Phase 3 → Phase 4 → Phase 5 (sequential phases, parallel tasks within phases)
**Key Risks**:
1. JSON serialization format compatibility with Claude Code CLI
2. Correct handling of content blocks in both one-shot and streaming modes
3. Backward compatibility with existing text-only APIs

---

## Phase 1: Type Definitions and Core Data Structures

**Objective**: Define all new types required for multimodal image support
**Duration Estimate**: 4-6 hours
**Dependencies**: None (independent)
**Parallel Execution**: All tasks in this phase can be executed in parallel with testing

### Tasks

#### 1.1 Add ImageSource Enum to messages.rs
**Complexity**: Simple
**Description**: Create the `ImageSource` enum with Base64 and Url variants for representing image data sources.

- **Files/Components**: `src/types/messages.rs`
- **Technical Approach**:
  - Add enum with serde tagged representation (`#[serde(tag = "type", rename_all = "snake_case")]`)
  - Base64 variant: contains `media_type` and `data` fields
  - Url variant: contains single `url` field
  - Derive: Debug, Clone, Serialize, Deserialize, PartialEq
- **Acceptance Criteria**:
  - Enum compiles without errors
  - Serialization produces correct JSON structure with `"type"` discriminator
  - Both variants serialize/deserialize correctly
  - PartialEq allows equality comparison
- **Testing Requirements**:
  - Unit test for Base64 serialization format
  - Unit test for Url serialization format
  - Unit test for deserialization round-trip
- **Risks**: None - straightforward enum definition

---

#### 1.2 Add ImageBlock Struct to messages.rs
**Complexity**: Simple
**Description**: Create the `ImageBlock` struct containing an `ImageSource`.

- **Files/Components**: `src/types/messages.rs`
- **Technical Approach**:
  - Simple struct with single `source: ImageSource` field
  - Derive: Debug, Clone, Serialize, Deserialize, PartialEq
  - No additional serde attributes needed
- **Dependencies**: Task 1.1 (ImageSource must exist)
- **Acceptance Criteria**:
  - Struct compiles without errors
  - Serialization produces flat JSON object with source field
  - Can be constructed with both ImageSource variants
- **Testing Requirements**:
  - Unit test for serialization with base64 source
  - Unit test for serialization with url source
- **Risks**: None - straightforward struct definition

---

#### 1.3 Add UserContentBlock Enum to messages.rs
**Complexity**: Moderate
**Description**: Create the `UserContentBlock` enum for user prompt construction with Text and Image variants.

- **Files/Components**: `src/types/messages.rs`
- **Technical Approach**:
  - Tagged enum with serde attributes: `#[serde(tag = "type", rename_all = "snake_case")]`
  - Text variant: struct with `text: String` field
  - Image variant: struct with `source: ImageSource` field
  - Derive: Debug, Clone, Serialize, Deserialize, PartialEq
- **Dependencies**: Task 1.2 (ImageBlock/ImageSource must exist)
- **Acceptance Criteria**:
  - Enum compiles without errors
  - Text variant serializes to `{"type": "text", "text": "..."}`
  - Image variant serializes to `{"type": "image", "source": {...}}`
  - Both variants deserialize correctly
- **Testing Requirements**:
  - Unit test for Text variant serialization
  - Unit test for Image variant serialization (both base64 and url)
  - Unit test for mixed array of content blocks
- **Risks**: Low - ensure JSON structure matches Claude API expectations

---

#### 1.4 Implement Builder Methods for UserContentBlock
**Complexity**: Moderate
**Description**: Add convenience methods (`::text()`, `::image_base64()`, `::image_url()`) to UserContentBlock.

- **Files/Components**: `src/types/messages.rs`
- **Technical Approach**:
  - Implement methods in `impl UserContentBlock` block
  - `text(text: impl Into<String>)`: Creates Text variant
  - `image_base64(media_type: impl Into<String>, data: impl Into<String>)`: Creates Image with Base64 source
  - `image_url(url: impl Into<String>)`: Creates Image with Url source
  - Use generic `Into<String>` bounds for ergonomic API
- **Dependencies**: Task 1.3 (UserContentBlock must exist)
- **Acceptance Criteria**:
  - All three methods compile and work correctly
  - Methods accept both String and &str arguments
  - Created instances serialize correctly
  - Methods are properly documented with rustdoc
- **Testing Requirements**:
  - Unit test for each builder method
  - Test that builder output matches direct construction
  - Test with different input types (&str, String)
- **Risks**: None - standard builder pattern

---

#### 1.5 Implement From Traits for UserContentBlock
**Complexity**: Simple
**Description**: Add `From<String>` and `From<&str>` implementations for backward compatibility.

- **Files/Components**: `src/types/messages.rs`
- **Technical Approach**:
  - `impl From<String>`: creates Text variant
  - `impl From<&str>`: creates Text variant with to_string()
  - Enables ergonomic conversion from strings
- **Dependencies**: Task 1.3 (UserContentBlock must exist)
- **Acceptance Criteria**:
  - Both From implementations compile
  - String and &str convert to Text variant correctly
  - Can use `.into()` syntax with UserContentBlock
- **Testing Requirements**:
  - Unit test for String conversion
  - Unit test for &str conversion
- **Risks**: None

---

#### 1.6 Update ContentBlock Enum with Image Variant
**Complexity**: Simple
**Description**: Add `Image(ImageBlock)` variant to existing `ContentBlock` enum for parsing assistant responses.

- **Files/Components**: `src/types/messages.rs`
- **Technical Approach**:
  - Add `Image(ImageBlock)` to existing ContentBlock enum
  - Maintains existing variants (Text, Thinking, ToolUse, ToolResult)
  - No changes to existing variants
- **Dependencies**: Task 1.2 (ImageBlock must exist)
- **Acceptance Criteria**:
  - Enum compiles with new variant
  - Existing code still works (backward compatible)
  - Image variant can deserialize from JSON responses
- **Testing Requirements**:
  - Unit test for Image variant deserialization
  - Regression test: existing variants still work
- **Risks**: Low - additive change, but ensure no breaking changes to existing code

---

#### 1.7 Write Comprehensive Unit Tests for Type Serialization
**Complexity**: Moderate
**Description**: Create complete test suite for all new types in messages.rs.

- **Files/Components**: `src/types/messages.rs` (in `#[cfg(test)]` module)
- **Technical Approach**:
  - Test serialization to JSON for all new types
  - Test deserialization from JSON
  - Test round-trip serialization/deserialization
  - Test edge cases (empty strings, special characters)
  - Use serde_json::to_value and from_value
- **Dependencies**: Tasks 1.1-1.6 (all types must be defined)
- **Acceptance Criteria**:
  - All tests pass
  - Test coverage for all public API surface of new types
  - Tests verify exact JSON structure matches design spec
  - Tests are well-documented
- **Testing Requirements**: N/A (this is the testing task)
- **Risks**: None - testing infrastructure already exists

---

## Phase 2: Transport Layer Extension

**Objective**: Extend transport layer to support content blocks in query prompts
**Duration Estimate**: 3-4 hours
**Dependencies**: Phase 1 must be complete (types must exist)
**Parallel Execution**: Tasks within this phase are sequential

### Tasks

#### 2.1 Extend QueryPrompt Enum
**Complexity**: Simple
**Description**: Add `Content(Vec<UserContentBlock>)` variant to QueryPrompt enum.

- **Files/Components**: `src/internal/transport/subprocess.rs`
- **Technical Approach**:
  - Add new variant to existing QueryPrompt enum
  - Maintain existing Text and Streaming variants
  - Keep existing Clone derive
- **Dependencies**: Phase 1 complete (UserContentBlock type exists)
- **Acceptance Criteria**:
  - Enum compiles with new variant
  - Existing code still compiles (backward compatible)
  - Content variant can hold Vec<UserContentBlock>
- **Testing Requirements**:
  - Compile-time verification (no unit test needed for enum variant)
- **Risks**: None - additive change

---

#### 2.2 Implement From<Vec<UserContentBlock>> for QueryPrompt
**Complexity**: Simple
**Description**: Add From trait implementation for ergonomic conversion.

- **Files/Components**: `src/internal/transport/subprocess.rs`
- **Technical Approach**:
  - `impl From<Vec<UserContentBlock>> for QueryPrompt`
  - Returns `QueryPrompt::Content(blocks)`
- **Dependencies**: Task 2.1 (Content variant must exist)
- **Acceptance Criteria**:
  - From implementation compiles
  - Can use `.into()` to convert Vec<UserContentBlock> to QueryPrompt
- **Testing Requirements**:
  - Integration test verifying conversion works
- **Risks**: None

---

#### 2.3 Update SubprocessTransport::connect() Method
**Complexity**: Moderate
**Description**: Modify connect() to handle Content variant by serializing content blocks to JSON.

- **Files/Components**: `src/internal/transport/subprocess.rs`
- **Technical Approach**:
  - Locate the match statement handling `self.prompt` in connect()
  - Add new arm: `QueryPrompt::Content(blocks) => {...}`
  - Serialize blocks using `serde_json::to_string(blocks)`
  - Write serialized JSON to stdin
  - Call `end_input()` to close stdin
  - Map serialization errors to ClaudeError::Transport
- **Dependencies**: Task 2.1 (Content variant must exist)
- **Acceptance Criteria**:
  - Code compiles without errors
  - Content blocks serialize to JSON array
  - JSON is written to subprocess stdin correctly
  - Errors are properly mapped to ClaudeError
  - Stdin is closed after writing
- **Testing Requirements**:
  - Integration test with mock subprocess
  - Verify JSON format matches expected structure
  - Test error handling for serialization failures
- **Risks**: Medium - ensure JSON format matches CLI expectations exactly

---

#### 2.4 Add Transport Layer Tests
**Complexity**: Moderate
**Description**: Write tests verifying QueryPrompt::Content serialization and transport behavior.

- **Files/Components**: `src/internal/transport/subprocess.rs` (test module)
- **Technical Approach**:
  - Test QueryPrompt::Content creates correct JSON format
  - Verify JSON structure: `[{"type":"text","text":"..."},{"type":"image","source":{...}}]`
  - Test that serialization errors are handled gracefully
- **Dependencies**: Task 2.3 (connect() implementation complete)
- **Acceptance Criteria**:
  - All tests pass
  - JSON output format verified against design spec
  - Error cases tested
- **Testing Requirements**: N/A (this is the testing task)
- **Risks**: None

---

## Phase 3: Query Functions

**Objective**: Add new public API functions for multimodal queries
**Duration Estimate**: 3-4 hours
**Dependencies**: Phase 2 complete (transport layer ready)
**Parallel Execution**: Tasks 3.1 and 3.2 can be done in parallel, task 3.3 depends on both

### Tasks

#### 3.1 Implement query_with_content()
**Complexity**: Moderate
**Description**: Add one-shot multimodal query function to query.rs.

- **Files/Components**: `src/query.rs`
- **Technical Approach**:
  - Function signature: `pub async fn query_with_content(content: impl Into<Vec<UserContentBlock>>, options: Option<ClaudeAgentOptions>) -> Result<Vec<Message>>`
  - Convert content to Vec<UserContentBlock> using `.into()`
  - Create `QueryPrompt::Content(content.into())`
  - Use unwrap_or_default() for options
  - Create InternalClient with query_prompt and options
  - Call client.execute().await
  - Add comprehensive rustdoc with examples
- **Dependencies**: Phase 2 complete
- **Acceptance Criteria**:
  - Function compiles and has correct signature
  - Works with Vec<UserContentBlock> input
  - Returns Result<Vec<Message>>
  - Rustdoc includes working example code
  - Example shows base64 image usage
- **Testing Requirements**:
  - Integration test with mock CLI
  - Test text-only content (should work like regular query)
  - Test mixed text and image content
- **Risks**: Low - follows existing query() pattern

---

#### 3.2 Implement query_stream_with_content()
**Complexity**: Moderate
**Description**: Add streaming multimodal query function to query.rs.

- **Files/Components**: `src/query.rs`
- **Technical Approach**:
  - Function signature: `pub async fn query_stream_with_content(content: impl Into<Vec<UserContentBlock>>, options: Option<ClaudeAgentOptions>) -> Result<Pin<Box<dyn Stream<Item = Result<Message>> + Send>>>`
  - Create QueryPrompt::Content from input
  - Create SubprocessTransport and connect
  - Use async_stream::stream! macro to create message stream
  - Read messages using transport.read_messages()
  - Parse JSON using MessageParser::parse()
  - Yield results, break on error
  - Add comprehensive rustdoc with examples
- **Dependencies**: Phase 2 complete
- **Acceptance Criteria**:
  - Function compiles and has correct signature
  - Returns stream of Result<Message>
  - Stream processes messages in real-time
  - Rustdoc includes working example with image URL
- **Testing Requirements**:
  - Integration test verifying stream behavior
  - Test that messages arrive incrementally
- **Risks**: Low - mirrors existing query_stream() implementation

---

#### 3.3 Update lib.rs Exports
**Complexity**: Simple
**Description**: Export new types and functions from lib.rs.

- **Files/Components**: `src/lib.rs`
- **Technical Approach**:
  - Add to messages re-exports: `ImageSource`, `ImageBlock`, `UserContentBlock`
  - Add to query re-exports: `query_with_content`, `query_stream_with_content`
  - Maintain alphabetical ordering if present
  - Add documentation comments for new exports
- **Dependencies**: Tasks 3.1 and 3.2 (functions must exist)
- **Acceptance Criteria**:
  - All new types and functions are publicly exported
  - Exports compile without errors
  - Types are accessible from crate root
- **Testing Requirements**:
  - Compile-time verification
  - Example code using crate::* imports
- **Risks**: None

---

## Phase 4: Client Methods

**Objective**: Add multimodal query methods to ClaudeClient for bidirectional streaming
**Duration Estimate**: 3-4 hours
**Dependencies**: Phase 1 complete (types exist)
**Parallel Execution**: Tasks are sequential within this phase

### Tasks

#### 4.1 Implement ClaudeClient::query_with_content()
**Complexity**: Moderate
**Description**: Add query_with_content() method to ClaudeClient for sending multimodal queries.

- **Files/Components**: `src/client.rs`
- **Technical Approach**:
  - Method signature: `pub async fn query_with_content(&mut self, content: impl Into<Vec<UserContentBlock>>) -> Result<()>`
  - Call `self.query_with_content_and_session(content, "default").await`
  - Add comprehensive rustdoc with examples
  - Document bidirectional streaming usage pattern
- **Dependencies**: Phase 1 complete, Task 4.2 (calls session method)
- **Acceptance Criteria**:
  - Method compiles with correct signature
  - Delegates to session method with "default" session ID
  - Rustdoc includes complete usage example
- **Testing Requirements**:
  - Integration test verifying method works
  - Test with both text and image content
- **Risks**: Low - simple delegation pattern

---

#### 4.2 Implement ClaudeClient::query_with_content_and_session()
**Complexity**: Complex
**Description**: Add session-aware multimodal query method with proper stream-json formatting.

- **Files/Components**: `src/client.rs`
- **Technical Approach**:
  - Method signature: `pub async fn query_with_content_and_session(&mut self, content: impl Into<Vec<UserContentBlock>>, session_id: impl Into<String>) -> Result<()>`
  - Check that client is connected (query.is_some())
  - Convert content to Vec<UserContentBlock>
  - Build JSON message: `{"type":"user","message":{"role":"user","content":[...]},"session_id":"..."}`
  - Use serde_json::json! macro for structure
  - Serialize to string with serde_json::to_string
  - Get stdin from query lock
  - Write message + newline to stdin
  - Flush stdin
  - Handle all error cases with proper ClaudeError mapping
  - Add comprehensive rustdoc
- **Dependencies**: Phase 1 complete
- **Acceptance Criteria**:
  - Method compiles and has correct signature
  - JSON structure matches stream-json format exactly
  - Content array contains serialized UserContentBlock objects
  - Errors are properly mapped to ClaudeError
  - Works with custom session IDs
  - Stdin is flushed after writing
- **Testing Requirements**:
  - Integration test with connected client
  - Verify JSON format sent to CLI
  - Test error handling (not connected, serialization failure)
  - Test with multiple session IDs
- **Risks**: High - JSON format must match CLI expectations exactly, improper format will cause CLI errors

---

#### 4.3 Add Client Method Tests
**Complexity**: Moderate
**Description**: Write comprehensive tests for new client methods.

- **Files/Components**: `tests/integration_tests.rs` or `src/client.rs` test module
- **Technical Approach**:
  - Test query_with_content() with text-only content
  - Test query_with_content() with mixed content
  - Test query_with_content_and_session() with custom session
  - Test error case: calling before connect()
  - Mock subprocess to verify JSON output format
- **Dependencies**: Tasks 4.1 and 4.2 complete
- **Acceptance Criteria**:
  - All tests pass
  - JSON format is verified
  - Error cases are tested
  - Session management is verified
- **Testing Requirements**: N/A (this is the testing task)
- **Risks**: None

---

## Phase 5: Documentation, Examples, and Final Testing

**Objective**: Complete implementation with documentation, examples, and comprehensive testing
**Duration Estimate**: 4-6 hours
**Dependencies**: Phases 1-4 complete
**Parallel Execution**: Tasks 5.1 and 5.2 can be parallel, others are sequential

### Tasks

#### 5.1 Create Example File: 23_image_input.rs
**Complexity**: Moderate
**Description**: Write comprehensive example demonstrating all image input capabilities.

- **Files/Components**: `examples/23_image_input.rs`
- **Technical Approach**:
  - Example 1: query_with_content() with base64 image
    - Read image file with std::fs::read()
    - Encode with base64 crate
    - Create content blocks with text + image
    - Send query and print response
  - Example 2: query_stream_with_content() with URL image
    - Use image URL
    - Stream response
    - Process messages in real-time
  - Example 3: ClaudeClient with bidirectional streaming
    - Connect client
    - Send multiple queries with images
    - Handle responses
    - Disconnect cleanly
  - Include comprehensive comments explaining each approach
  - Add sample image file or instructions to use your own
- **Dependencies**: Phases 1-4 complete
- **Acceptance Criteria**:
  - Example compiles without errors
  - Example runs successfully (with valid API access)
  - Code demonstrates all three major use cases
  - Comments explain when to use each approach
  - Example follows existing example file patterns
- **Testing Requirements**:
  - Manual testing: run example and verify output
  - Ensure example is included in Cargo.toml [[example]] section
- **Risks**: Low - examples don't affect core functionality

---

#### 5.2 Add Rustdoc Documentation
**Complexity**: Simple
**Description**: Ensure all public APIs have comprehensive documentation.

- **Files/Components**: All modified files
- **Technical Approach**:
  - Review all new public types, functions, methods
  - Ensure rustdoc comments exist with:
    - Brief description
    - Detailed explanation of purpose
    - Examples section with working code
    - Parameters and return values documented
    - Notes about supported image formats
    - Security considerations for image sources
  - Run `cargo doc` to verify documentation builds
- **Dependencies**: Phases 1-4 complete
- **Acceptance Criteria**:
  - All public APIs have rustdoc comments
  - Documentation builds without warnings
  - Examples in docs are valid code (doc tests pass)
  - Documentation follows Rust conventions
- **Testing Requirements**:
  - Run `cargo doc --no-deps --open` and review
  - Run `cargo test --doc` to verify doc tests
- **Risks**: None

---

#### 5.3 Integration Tests
**Complexity**: Moderate
**Description**: Write end-to-end integration tests for all new functionality.

- **Files/Components**: `tests/integration_tests.rs`
- **Technical Approach**:
  - Test query_with_content() with text-only (backward compatibility)
  - Test query_with_content() with base64 image
  - Test query_with_content() with URL image
  - Test query_stream_with_content() with mixed content
  - Test ClaudeClient::query_with_content()
  - Test serialization format matches expected JSON structure
  - Mock subprocess or use conditional compilation for CI
- **Dependencies**: Phases 1-4 complete
- **Acceptance Criteria**:
  - All integration tests pass
  - Tests cover happy paths and error cases
  - Tests verify JSON format correctness
  - Tests can run in CI environment
- **Testing Requirements**: N/A (this is the testing task)
- **Risks**: Medium - may need to mock Claude CLI for CI

---

#### 5.4 Backward Compatibility Testing
**Complexity**: Simple
**Description**: Verify existing APIs continue to work unchanged.

- **Files/Components**: `tests/integration_tests.rs`
- **Technical Approach**:
  - Test existing query() function still works
  - Test existing query_stream() function still works
  - Test existing ClaudeClient::query() method still works
  - Verify no breaking changes to existing types
  - Run full test suite to catch regressions
- **Dependencies**: Phases 1-4 complete
- **Acceptance Criteria**:
  - All existing tests pass
  - No API changes to existing functions
  - Existing example code still compiles and runs
- **Testing Requirements**:
  - Run `cargo test` with all existing tests
  - Manually run a few existing examples
- **Risks**: Low - changes are additive

---

#### 5.5 Add Security and Usage Documentation
**Complexity**: Simple
**Description**: Document security considerations and best practices.

- **Files/Components**: Rustdoc comments in relevant types
- **Technical Approach**:
  - Add security notes to ImageSource documentation:
    - Validate image sources
    - Be aware of memory usage with large base64 images
    - Ensure URL sources are trustworthy
    - Claude performs content moderation
  - Document image size limits
  - Provide guidance on when to use base64 vs URL
  - Add examples of error handling
- **Dependencies**: Task 5.2 (documentation structure exists)
- **Acceptance Criteria**:
  - Security section added to relevant types
  - Best practices documented
  - Size limits mentioned
  - Users understand trade-offs
- **Testing Requirements**: Documentation review
- **Risks**: None

---

#### 5.6 Final Testing and Validation
**Complexity**: Moderate
**Description**: Run complete test suite and validate all functionality.

- **Files/Components**: All files
- **Technical Approach**:
  - Run `cargo test` - all tests must pass
  - Run `cargo clippy` - no warnings
  - Run `cargo fmt -- --check` - code formatted
  - Run `cargo doc` - documentation builds
  - Run examples manually with real Claude CLI
  - Test with various image formats (PNG, JPEG, GIF, WebP)
  - Test with various image sizes
  - Test error cases (invalid base64, bad URLs, etc.)
- **Dependencies**: All previous tasks complete
- **Acceptance Criteria**:
  - All tests pass
  - No clippy warnings
  - Code is properly formatted
  - Documentation builds without errors
  - Examples run successfully
  - All image formats work
- **Testing Requirements**: N/A (comprehensive validation)
- **Risks**: Low - final validation step

---

## Testing Strategy

### Unit Tests
- **Location**: `src/types/messages.rs`, `src/internal/transport/subprocess.rs`
- **Coverage**:
  - All new types (ImageSource, ImageBlock, UserContentBlock)
  - Serialization/deserialization for all variants
  - Builder methods
  - From trait implementations
  - QueryPrompt extension
- **Execution**: `cargo test --lib`

### Integration Tests
- **Location**: `tests/integration_tests.rs`
- **Coverage**:
  - query_with_content() end-to-end
  - query_stream_with_content() end-to-end
  - ClaudeClient methods
  - JSON format verification
  - Backward compatibility
- **Execution**: `cargo test --test integration_tests`

### Doc Tests
- **Location**: Inline in rustdoc comments
- **Coverage**: All public API examples
- **Execution**: `cargo test --doc`

### Manual Tests
- **Location**: `examples/23_image_input.rs`
- **Coverage**: Real-world usage scenarios
- **Execution**: `cargo run --example 23_image_input`

---

## Rollback Plan

### Feature Flags
- **Not required**: All changes are additive and backward compatible
- If issues arise, users can simply not use new APIs

### Database Rollback
- **Not applicable**: No database changes

### Safe Rollback Points
1. **After Phase 1**: Types defined but not used - no impact on existing code
2. **After Phase 2**: Transport extended but old paths still work
3. **After Phase 3**: New functions added but optional to use
4. **After Phase 4**: Client methods added but optional to use

### Reverting Changes
- All changes are in separate functions/methods
- Remove new exports from lib.rs
- Remove new functions from query.rs
- Remove new methods from client.rs
- Remove Content variant from QueryPrompt (if not used elsewhere)
- Remove new types from messages.rs
- Run tests to verify rollback success

---

## Risk Mitigation

### Risk 1: JSON Serialization Format Mismatch
- **Impact**: High
- **Probability**: Medium
- **Mitigation**:
  - Write tests verifying exact JSON structure
  - Test with actual Claude CLI early in development
  - Compare JSON output with working examples from CLI documentation
  - Add logging to show serialized JSON during development
- **Contingency**:
  - Adjust serde attributes to match expected format
  - Add custom serializer if needed

### Risk 2: Backward Compatibility Issues
- **Impact**: High
- **Probability**: Low
- **Mitigation**:
  - All changes are additive (no modifications to existing APIs)
  - Run full test suite after each change
  - Test existing examples regularly
  - Use separate types (UserContentBlock) instead of modifying existing ones
- **Contingency**:
  - Revert breaking changes
  - Find alternative implementation approach

### Risk 3: Large Image Memory Usage
- **Impact**: Medium
- **Probability**: Medium
- **Mitigation**:
  - Document size limits in rustdoc
  - Provide guidance on using URLs for large images
  - Mention Claude API limits (~20MB per request)
- **Contingency**:
  - Users can use URL-based images instead of base64
  - Document best practices

### Risk 4: Content Block Ordering
- **Impact**: Medium
- **Probability**: Low
- **Mitigation**:
  - Test that content blocks are serialized in order
  - Verify Vec maintains insertion order
- **Contingency**:
  - Add indices to JSON if needed (unlikely)

### Risk 5: Stream-JSON Format for Bidirectional Client
- **Impact**: High
- **Probability**: Medium
- **Mitigation**:
  - Study existing client.query() implementation
  - Match JSON structure exactly
  - Test with actual CLI in bidirectional mode
- **Contingency**:
  - Adjust JSON structure based on CLI feedback
  - Add logging for debugging

---

## Success Criteria

### Functional Requirements
- ✅ Users can send queries with base64-encoded images
- ✅ Users can send queries with URL-referenced images
- ✅ Users can mix text and images in a single query
- ✅ One-shot mode works with images
- ✅ Streaming mode works with images
- ✅ Bidirectional client works with images
- ✅ All existing APIs continue to work unchanged

### Non-Functional Requirements
- ✅ Code compiles without warnings
- ✅ All tests pass (unit, integration, doc tests)
- ✅ Documentation is comprehensive and accurate
- ✅ Examples demonstrate all use cases
- ✅ Code follows existing SDK patterns
- ✅ No performance regression for text-only queries

### Quality Requirements
- ✅ Code coverage for new functionality > 80%
- ✅ No clippy warnings
- ✅ Code formatted with rustfmt
- ✅ All public APIs documented
- ✅ Security considerations documented

---

## Dependencies and Prerequisites

### External Dependencies
- **serde**: Already in Cargo.toml (required for serialization)
- **serde_json**: Already in Cargo.toml (required for JSON handling)
- **base64**: Optional (only for examples, users provide their own encoding)

### Development Environment
- Rust toolchain (stable channel)
- Claude Code CLI installed and in PATH
- Access to test images for examples

### Knowledge Requirements
- Understanding of serde serialization/deserialization
- Familiarity with Claude API message format
- Understanding of async Rust and streams
- Knowledge of existing SDK architecture

---

## Monitoring and Validation

### Development Monitoring
- Run `cargo test` after each task
- Run `cargo clippy` regularly
- Check `cargo doc` builds without warnings
- Test examples manually with real CLI

### Error Handling
- All serialization errors mapped to ClaudeError::Transport
- Connection errors properly propagated
- Clear error messages for common issues

### Logging
- Add tracing::debug! logs for:
  - Content block serialization
  - Image data size (base64 character count)
  - JSON message structure (in debug mode)

### Performance Monitoring
- No performance impact expected for existing APIs
- Image queries may be slower (Claude API processing time, not SDK)
- Memory usage scales with image data size (as expected)

---

## Appendix

### File Change Summary

| File | Lines Changed (Est.) | Change Type | Risk Level |
|------|---------------------|-------------|------------|
| `src/types/messages.rs` | +150 | Add types, tests | Low |
| `src/internal/transport/subprocess.rs` | +30 | Extend enum, update method | Medium |
| `src/query.rs` | +70 | Add functions | Low |
| `src/client.rs` | +60 | Add methods | Medium |
| `src/lib.rs` | +5 | Add exports | Low |
| `examples/23_image_input.rs` | +150 | New file | Low |
| `tests/integration_tests.rs` | +100 | Add tests | Low |

**Total Estimated Changes**: ~565 lines of code

### Critical Path Tasks
The following tasks are on the critical path (longest dependent sequence):
1. Task 1.1 → 1.2 → 1.3 → 1.4
2. Task 2.1 → 2.2 → 2.3
3. Task 3.1 → 3.3
4. Task 4.2
5. Task 5.6

**Estimated Critical Path Duration**: 16-20 hours

### Parallel Execution Opportunities
- Phase 1 tasks 1.1-1.6 can mostly be done in parallel
- Phase 3 tasks 3.1 and 3.2 can be done in parallel
- Phase 5 tasks 5.1 and 5.2 can be done in parallel

### Reference Resources
- Technical Design Document v1.0
- Existing SDK code patterns (query.rs, client.rs)
- Claude API documentation for multimodal inputs
- Serde documentation for custom serialization
