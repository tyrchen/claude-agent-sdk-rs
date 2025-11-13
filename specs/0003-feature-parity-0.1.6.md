# Feature Parity Plan: Rust SDK vs Python SDK v0.1.6

**Status:** Planning
**Target Version:** Rust SDK v0.3.0
**Python SDK Reference:** v0.1.6
**Date:** 2025-11-12

## Executive Summary

This document provides a comprehensive analysis of feature parity between the Rust Claude Agent SDK and the Python SDK v0.1.6. After thorough codebase exploration, the Rust SDK has achieved **~95% feature parity** with only 4 missing configuration options and 1 API design difference.

**Current Status:**
- ✅ Core APIs: 100% complete
- ✅ Message Types: 100% complete
- ✅ Hooks System: 100% complete
- ✅ Permission System: 100% complete
- ✅ MCP Integration: 100% complete
- ✅ Session Management: 100% complete
- ⚠️ Configuration Options: 4 missing (95% complete)
- ⚠️ Query API: Design difference (streaming vs collecting)

---

## 1. Feature Comparison Matrix

### 1.1 Core APIs

| Feature | Python SDK | Rust SDK | Status | Notes |
|---------|-----------|----------|--------|-------|
| Simple query API | ✅ `query()` | ✅ `query()` | ⚠️ PARTIAL | See Section 2.1 |
| Bidirectional client | ✅ `ClaudeSDKClient` | ✅ `ClaudeClient` | ✅ COMPLETE | - |
| Context manager | ✅ `async with` | ❌ Manual | ⚠️ DIFFERENT | Explicit connect/disconnect |
| Stream API | ✅ `AsyncIterator` | ✅ `Stream` | ✅ COMPLETE | Different types, same capability |

### 1.2 Configuration Options

| Option | Python SDK | Rust SDK | Priority | Implementation Effort |
|--------|-----------|----------|----------|---------------------|
| `allowed_tools` | ✅ | ✅ | - | - |
| `disallowed_tools` | ✅ | ✅ | - | - |
| `system_prompt` | ✅ | ✅ | - | - |
| `mcp_servers` | ✅ | ✅ | - | - |
| `permission_mode` | ✅ | ✅ | - | - |
| `continue_conversation` | ✅ | ✅ | - | - |
| `resume` | ✅ | ✅ | - | - |
| `fork_session` | ✅ | ✅ | - | - |
| `max_turns` | ✅ | ✅ | - | - |
| `model` | ✅ | ✅ | - | - |
| `permission_prompt_tool_name` | ✅ | ✅ | - | - |
| `cwd` | ✅ | ✅ | - | - |
| `cli_path` | ✅ | ✅ | - | - |
| `settings` | ✅ | ✅ | - | - |
| `add_dirs` | ✅ | ✅ | - | - |
| `env` | ✅ | ✅ | - | - |
| `extra_args` | ✅ | ✅ | - | - |
| `max_buffer_size` | ✅ | ✅ | - | - |
| `stderr_callback` | ✅ | ✅ | - | - |
| `can_use_tool` | ✅ | ✅ | - | - |
| `hooks` | ✅ | ✅ | - | - |
| `user` | ✅ | ✅ | - | - |
| `include_partial_messages` | ✅ | ✅ | - | - |
| `agents` | ✅ | ✅ | - | - |
| `setting_sources` | ✅ | ✅ | - | - |
| **`fallback_model`** | ✅ | ❌ | 🔴 HIGH | 2 hours |
| **`max_budget_usd`** | ✅ | ❌ | 🟡 MEDIUM | 2 hours |
| **`max_thinking_tokens`** | ✅ | ❌ | 🟡 MEDIUM | 2 hours |
| **`plugins`** | ✅ | ❌ | 🟢 LOW | 8 hours |

### 1.3 Message Types

| Feature | Python SDK | Rust SDK | Status |
|---------|-----------|----------|--------|
| `UserMessage` | ✅ | ✅ | ✅ COMPLETE |
| `AssistantMessage` | ✅ | ✅ | ✅ COMPLETE |
| `SystemMessage` | ✅ | ✅ | ✅ COMPLETE |
| `ResultMessage` | ✅ | ✅ | ✅ COMPLETE |
| `StreamEvent` | ✅ | ✅ | ✅ COMPLETE |
| `TextBlock` | ✅ | ✅ | ✅ COMPLETE |
| `ThinkingBlock` | ✅ | ✅ | ✅ COMPLETE |
| `ToolUseBlock` | ✅ | ✅ | ✅ COMPLETE |
| `ToolResultBlock` | ✅ | ✅ | ✅ COMPLETE |

### 1.4 Hooks System

| Feature | Python SDK | Rust SDK | Status |
|---------|-----------|----------|--------|
| `PreToolUse` hook | ✅ | ✅ | ✅ COMPLETE |
| `PostToolUse` hook | ✅ | ✅ | ✅ COMPLETE |
| `UserPromptSubmit` hook | ✅ | ✅ | ✅ COMPLETE |
| `Stop` hook | ✅ | ✅ | ✅ COMPLETE |
| `SubagentStop` hook | ✅ | ✅ | ✅ COMPLETE |
| `PreCompact` hook | ✅ | ✅ | ✅ COMPLETE |
| Hook matchers | ✅ | ✅ | ✅ COMPLETE |
| Async hooks | ✅ | ✅ | ✅ COMPLETE |
| Hook-specific outputs | ✅ | ✅ | ✅ COMPLETE |
| Hook builder API | ✅ | ✅ | ✅ COMPLETE |

### 1.5 Permission System

| Feature | Python SDK | Rust SDK | Status |
|---------|-----------|----------|--------|
| `can_use_tool` callback | ✅ | ✅ | ✅ COMPLETE |
| `PermissionResultAllow` | ✅ | ✅ | ✅ COMPLETE |
| `PermissionResultDeny` | ✅ | ✅ | ✅ COMPLETE |
| Tool input modification | ✅ | ✅ | ✅ COMPLETE |
| Permission updates | ✅ | ✅ | ✅ COMPLETE |
| Add/Remove rules | ✅ | ✅ | ✅ COMPLETE |
| Set mode | ✅ | ✅ | ✅ COMPLETE |
| Add/Remove directories | ✅ | ✅ | ✅ COMPLETE |
| Multiple destinations | ✅ | ✅ | ✅ COMPLETE |

### 1.6 MCP Integration

| Feature | Python SDK | Rust SDK | Status |
|---------|-----------|----------|--------|
| Stdio servers | ✅ | ✅ | ✅ COMPLETE |
| SSE servers | ✅ | ✅ | ✅ COMPLETE |
| HTTP servers | ✅ | ✅ | ✅ COMPLETE |
| SDK (in-process) servers | ✅ | ✅ | ✅ COMPLETE |
| Tool decorator/macro | ✅ `@tool` | ✅ `tool!()` | ✅ COMPLETE |
| `create_sdk_mcp_server()` | ✅ | ✅ | ✅ COMPLETE |
| Text tool results | ✅ | ✅ | ✅ COMPLETE |
| Image tool results | ✅ | ✅ | ✅ COMPLETE |

### 1.7 Session Management

| Feature | Python SDK | Rust SDK | Status |
|---------|-----------|----------|--------|
| Multiple sessions | ✅ | ✅ | ✅ COMPLETE |
| Session resume | ✅ | ✅ | ✅ COMPLETE |
| Session forking | ✅ | ✅ | ✅ COMPLETE |
| Continue conversation | ✅ | ✅ | ✅ COMPLETE |
| Session-specific queries | ✅ | ✅ | ✅ COMPLETE |

### 1.8 Dynamic Control

| Feature | Python SDK | Rust SDK | Status |
|---------|-----------|----------|--------|
| `interrupt()` | ✅ | ✅ | ✅ COMPLETE |
| `set_permission_mode()` | ✅ | ✅ | ✅ COMPLETE |
| `set_model()` | ✅ | ✅ | ✅ COMPLETE |
| `get_server_info()` | ✅ | ✅ | ✅ COMPLETE |

### 1.9 Error Handling

| Feature | Python SDK | Rust SDK | Status |
|---------|-----------|----------|--------|
| Base error type | ✅ `ClaudeSDKError` | ✅ `ClaudeError` | ✅ COMPLETE |
| Connection errors | ✅ | ✅ | ✅ COMPLETE |
| Process errors | ✅ | ✅ | ✅ COMPLETE |
| JSON decode errors | ✅ | ✅ | ✅ COMPLETE |
| Message parse errors | ✅ | ✅ | ✅ COMPLETE |
| CLI not found errors | ✅ | ✅ | ✅ COMPLETE |
| Detailed error context | ✅ | ✅ | ✅ COMPLETE |

---

## 2. Missing Features & Gaps

### 2.1 Query API Design Difference

**Issue:** Python's `query()` returns an `AsyncIterator[Message]` (streaming), while Rust's `query()` returns `Vec<Message>` (collected).

**Impact:**
- Memory efficiency: Python can process messages as they arrive; Rust collects everything
- User experience: Python provides real-time feedback; Rust waits until completion
- API consistency: Different programming models

**Analysis:**
- Rust SDK internally uses streaming (`ClaudeClient.receive_messages()`)
- The simple `query()` was designed for convenience, trading memory for simplicity
- For large conversations, this could be problematic

**Recommendation:** 🔴 **HIGH PRIORITY**

**Options:**
1. **Keep both APIs** (Recommended):
   - Keep current `query()` for simplicity (common case: few messages)
   - Add `query_stream()` that returns `impl Stream<Item = Result<Message>>`
   - Document trade-offs clearly

2. **Change to streaming only**:
   - Replace `query()` to return `impl Stream`
   - BREAKING CHANGE for existing users
   - Better memory efficiency

**Estimated Effort:** 4 hours (for Option 1)

**Implementation Plan:**
```rust
// Keep existing
pub async fn query(
    prompt: impl Into<String>,
    options: Option<ClaudeAgentOptions>,
) -> Result<Vec<Message>>

// Add new streaming variant
pub fn query_stream(
    prompt: impl Into<String>,
    options: Option<ClaudeAgentOptions>,
) -> impl Stream<Item = Result<Message>>
```

---

### 2.2 Missing Configuration Option: `fallback_model`

**Python SDK:**
```python
@dataclass
class ClaudeAgentOptions:
    model: str | None = None
    fallback_model: str | None = None  # ← MISSING IN RUST
```

**Description:** Backup model to use if the primary model fails or is unavailable.

**Use Case:** Reliability - automatic failover when primary model has issues.

**Priority:** 🔴 **HIGH** - Important for production resilience

**Estimated Effort:** 2 hours

**Implementation Plan:**
1. Add field to `ClaudeAgentOptions`:
   ```rust
   /// Fallback model to use if primary model fails
   #[builder(default, setter(into, strip_option))]
   pub fallback_model: Option<String>,
   ```

2. Update CLI argument builder in `subprocess.rs`:
   ```rust
   if let Some(fallback_model) = &options.fallback_model {
       args.push(format!("--fallback-model={}", fallback_model));
   }
   ```

3. Add test case in `tests/`
4. Update examples and documentation

**Files to Modify:**
- `src/types/config.rs` - Add field
- `src/internal/transport/subprocess.rs` - Pass to CLI
- `tests/integration_tests.rs` - Add test
- `examples/10-fallback-model.rs` - New example

---

### 2.3 Missing Configuration Option: `max_budget_usd`

**Python SDK:**
```python
@dataclass
class ClaudeAgentOptions:
    max_budget_usd: float | None = None  # ← MISSING IN RUST
```

**Description:** Maximum spending limit in USD for the conversation.

**Use Case:** Cost control - prevent runaway costs in production.

**Priority:** 🟡 **MEDIUM** - Important for cost management, but workarounds exist

**Estimated Effort:** 2 hours

**Implementation Plan:**
1. Add field to `ClaudeAgentOptions`:
   ```rust
   /// Maximum budget in USD
   #[builder(default, setter(strip_option))]
   pub max_budget_usd: Option<f64>,
   ```

2. Update CLI argument builder:
   ```rust
   if let Some(max_budget) = options.max_budget_usd {
       args.push(format!("--max-budget-usd={}", max_budget));
   }
   ```

3. Handle budget exceeded errors in error handling
4. Add test case and example

**Files to Modify:**
- `src/types/config.rs` - Add field
- `src/internal/transport/subprocess.rs` - Pass to CLI
- `src/errors.rs` - Add budget exceeded error variant (optional)
- `tests/integration_tests.rs` - Add test
- `examples/11-budget-control.rs` - New example

---

### 2.4 Missing Configuration Option: `max_thinking_tokens`

**Python SDK:**
```python
@dataclass
class ClaudeAgentOptions:
    max_thinking_tokens: int | None = None  # ← MISSING IN RUST
```

**Description:** Maximum tokens for thinking blocks (extended thinking feature).

**Use Case:** Control thinking resource usage for models with extended thinking.

**Priority:** 🟡 **MEDIUM** - Useful for advanced use cases

**Estimated Effort:** 2 hours

**Implementation Plan:**
1. Add field to `ClaudeAgentOptions`:
   ```rust
   /// Maximum tokens for thinking blocks
   #[builder(default, setter(strip_option))]
   pub max_thinking_tokens: Option<u32>,
   ```

2. Update CLI argument builder:
   ```rust
   if let Some(max_thinking) = options.max_thinking_tokens {
       args.push(format!("--max-thinking-tokens={}", max_thinking));
   }
   ```

3. Add test case and example

**Files to Modify:**
- `src/types/config.rs` - Add field
- `src/internal/transport/subprocess.rs` - Pass to CLI
- `tests/integration_tests.rs` - Add test
- `examples/12-thinking-tokens.rs` - New example

---

### 2.5 Missing Configuration Option: `plugins`

**Python SDK:**
```python
class SdkPluginConfig(TypedDict):
    """SDK plugin configuration."""
    type: Literal["local"]
    path: str

@dataclass
class ClaudeAgentOptions:
    plugins: list[SdkPluginConfig] = field(default_factory=list)  # ← MISSING IN RUST
```

**Description:** Support for loading custom plugins from local paths.

**Use Case:** Extensibility - load third-party or custom functionality.

**Priority:** 🟢 **LOW** - Advanced feature, limited current usage

**Estimated Effort:** 8 hours

**Implementation Plan:**

1. Define plugin types in new file `src/types/plugin.rs`:
   ```rust
   /// Plugin configuration
   #[derive(Debug, Clone, Serialize, Deserialize)]
   #[serde(tag = "type")]
   #[serde(rename_all = "lowercase")]
   pub enum SdkPluginConfig {
       /// Local filesystem plugin
       Local {
           /// Path to the plugin
           path: PathBuf,
       },
   }
   ```

2. Add to `ClaudeAgentOptions`:
   ```rust
   /// Plugin configurations
   #[builder(default, setter(into))]
   pub plugins: Vec<SdkPluginConfig>,
   ```

3. Implement plugin loading in transport layer:
   ```rust
   // Convert plugins to CLI args
   for plugin in &options.plugins {
       match plugin {
           SdkPluginConfig::Local { path } => {
               args.push(format!("--plugin={}", path.display()));
           }
       }
   }
   ```

4. Add comprehensive tests and examples

**Files to Modify:**
- `src/types/mod.rs` - Add plugin module
- `src/types/plugin.rs` - New file with plugin types
- `src/types/config.rs` - Add plugins field
- `src/internal/transport/subprocess.rs` - Pass to CLI
- `tests/integration_tests.rs` - Add plugin tests
- `examples/17-custom-plugins.rs` - New example

**Complexity Factors:**
- Plugin loading mechanism needs testing
- Error handling for missing/invalid plugins
- Documentation for plugin development
- May need to coordinate with CLI implementation

---

## 3. API Design Differences (Non-Breaking)

### 3.1 Context Manager vs Explicit Lifecycle

**Python SDK:**
```python
async with ClaudeSDKClient(options) as client:
    await client.query("Hello")
    async for msg in client.receive_messages():
        print(msg)
# Automatic cleanup on exit
```

**Rust SDK:**
```rust
let mut client = ClaudeClient::new(options);
client.connect().await?;
client.query("Hello").await?;
// Manual cleanup required
client.disconnect().await?;
```

**Analysis:**
- Python's context manager provides automatic cleanup
- Rust requires explicit lifecycle management
- Rust has `Drop` trait but async cleanup is not possible in `Drop`

**Recommendation:** 🟢 **LOW PRIORITY - Document Only**

This is an idiomatic difference between Python and Rust:
- Python: RAII via context managers
- Rust: RAII via `Drop`, but async not supported

**Possible Enhancement (Future):**
- Create a `ClaudeClientGuard` wrapper with custom drop behavior
- Spawn background task for cleanup
- Document best practices for cleanup

---

### 3.2 Runtime Independence

**Python SDK:**
- Uses `anyio` for runtime independence (asyncio, trio, etc.)
- Framework agnostic

**Rust SDK:**
- Uses `tokio` directly
- Tokio-specific

**Analysis:**
- Python ecosystem has multiple async runtimes
- Rust ecosystem is dominated by Tokio (~90% usage)
- Supporting multiple runtimes (async-std, smol) adds complexity

**Recommendation:** 🟢 **LOW PRIORITY - Document Only**

Keep Tokio-only for now:
- Tokio is the de facto standard
- Adding runtime abstraction adds complexity and maintenance burden
- Users can bridge runtimes if needed (rare)

**Possible Enhancement (Future):**
- Use `async-trait` with runtime abstraction layer
- Conditionally compile for different runtimes
- Only if user demand exists

---

## 4. Implementation Roadmap

### Phase 1: Critical Features (Week 1)
**Goal:** Achieve 98% feature parity on essential features

1. ✅ **Add `fallback_model` option** - 2 hours
   - High priority for production reliability
   - Simple pass-through to CLI

2. ✅ **Add `query_stream()` API** - 4 hours
   - Address memory efficiency concerns
   - Non-breaking addition

3. ✅ **Add `max_budget_usd` option** - 2 hours
   - Important for cost control
   - Simple pass-through to CLI

**Total Effort:** 8 hours (1 day)

### Phase 2: Enhancement Features (Week 2)
**Goal:** Achieve 99% feature parity

4. ✅ **Add `max_thinking_tokens` option** - 2 hours
   - Complete thinking feature support
   - Simple pass-through to CLI

5. ✅ **Update documentation** - 4 hours
   - Document all new features
   - Update examples
   - Add migration guide

**Total Effort:** 6 hours (0.75 days)

### Phase 3: Advanced Features (Week 3-4)
**Goal:** Achieve 100% feature parity

6. ✅ **Add `plugins` support** - 8 hours
   - Most complex feature
   - Requires thorough testing
   - Document plugin development

7. ✅ **Comprehensive testing** - 4 hours
   - Integration tests for all new features
   - Edge case handling
   - Error scenarios

**Total Effort:** 12 hours (1.5 days)

### Phase 4: Quality & Documentation (Week 4)
**Goal:** Production readiness

8. ✅ **Code review and refinement** - 4 hours
9. ✅ **Documentation polish** - 4 hours
10. ✅ **Examples for all features** - 4 hours
11. ✅ **Update CHANGELOG** - 1 hour

**Total Effort:** 13 hours (1.5 days)

---

## 5. Testing Strategy

### 5.1 Unit Tests

For each new configuration option:

```rust
#[test]
fn test_fallback_model_config() {
    let options = ClaudeAgentOptions::builder()
        .fallback_model("claude-sonnet-4")
        .build();

    assert_eq!(options.fallback_model, Some("claude-sonnet-4".to_string()));
}
```

### 5.2 Integration Tests

For each feature with CLI integration:

```rust
#[tokio::test]
async fn test_fallback_model_cli_arg() {
    let options = ClaudeAgentOptions::builder()
        .model("claude-opus-4")
        .fallback_model("claude-sonnet-4")
        .build();

    let result = query("test", Some(options)).await;
    assert!(result.is_ok());
}
```

### 5.3 E2E Tests

Test real-world scenarios:

1. Model failover with `fallback_model`
2. Budget exceeded with `max_budget_usd`
3. Thinking token limits with `max_thinking_tokens`
4. Plugin loading and execution
5. Streaming vs collected query comparison

---

## 6. Documentation Updates

### 6.1 README Updates

Add sections for:
- New configuration options table
- Streaming vs collecting trade-offs
- Plugin development guide
- Migration guide from Python SDK

### 6.2 API Documentation

Update rustdoc for:
- All new configuration options
- `query_stream()` function
- Plugin types and traits
- Examples for each feature

### 6.3 Examples

Create new examples:
- `examples/10-fallback-model.rs`
- `examples/11-budget-control.rs`
- `examples/12-thinking-tokens.rs`
- `examples/17-custom-plugins.rs`
- `examples/18-streaming-query.rs`

---

## 7. Breaking Changes Assessment

### 7.1 Breaking Changes: NONE

All proposed changes are **additive**:
- New configuration options with `Option<T>` (default: None)
- New function `query_stream()` alongside existing `query()`
- No modifications to existing APIs

### 7.2 SemVer Recommendation

- Current version: 0.2.1
- After Phase 1-2: **0.3.0** (minor bump for new features)
- After Phase 3-4: **0.3.0** (same minor version)

Since we're pre-1.0, we can include new features in minor bumps.

---

## 8. Risk Analysis

### 8.1 Low Risk Items
- ✅ Configuration options (pass-through to CLI)
- ✅ Query streaming API (well-tested pattern)
- ✅ Documentation updates

### 8.2 Medium Risk Items
- ⚠️ Plugin support
  - Risk: Complex plugin loading mechanism
  - Mitigation: Extensive testing, clear error messages
  - Fallback: Document limitations if issues arise

### 8.3 Dependencies
- No new external dependencies required
- All features use existing infrastructure
- CLI version compatibility needs verification

---

## 9. Success Criteria

### 9.1 Feature Completeness
- ✅ All Python SDK configuration options supported
- ✅ Query API provides both streaming and collecting variants
- ✅ Plugin system functional with at least one example plugin

### 9.2 Quality Metrics
- ✅ 100% of new code covered by tests (unit + integration)
- ✅ All examples run without errors
- ✅ Documentation complete for all new features
- ✅ Zero clippy warnings with default lints
- ✅ rustfmt clean

### 9.3 Performance
- ✅ No performance regression in existing APIs
- ✅ Streaming query more memory-efficient than collecting
- ✅ Plugin loading < 100ms overhead

---

## 10. Detailed Implementation Tasks

### Task 1: Add `fallback_model`

**Files:**
- `src/types/config.rs`
- `src/internal/transport/subprocess.rs`
- `tests/integration_tests.rs`
- `examples/10-fallback-model.rs`

**Steps:**
1. Add field with TypedBuilder annotation
2. Add CLI argument in subprocess transport
3. Write unit test for config
4. Write integration test for CLI interaction
5. Create example demonstrating failover
6. Update README

**Acceptance Criteria:**
- Config struct compiles with new field
- CLI receives correct argument
- Tests pass
- Example runs successfully

---

### Task 2: Add `query_stream()`

**Files:**
- `src/query.rs`
- `src/lib.rs`
- `tests/query_tests.rs`
- `examples/18-streaming-query.rs`

**Steps:**
1. Implement `query_stream()` returning `impl Stream`
2. Reuse internal streaming infrastructure
3. Add comprehensive documentation comparing both APIs
4. Write tests comparing streaming vs collecting
5. Create example showing real-time message processing
6. Update README with performance guidance

**Acceptance Criteria:**
- Function compiles with correct signature
- Stream yields messages in real-time
- Memory usage lower than `query()` for large responses
- Tests pass
- Example demonstrates streaming benefits

---

### Task 3: Add `max_budget_usd`

**Files:**
- `src/types/config.rs`
- `src/internal/transport/subprocess.rs`
- `src/errors.rs` (optional)
- `tests/integration_tests.rs`
- `examples/11-budget-control.rs`

**Steps:**
1. Add field as `Option<f64>`
2. Add CLI argument
3. Optionally add `BudgetExceeded` error variant
4. Write tests with mocked budget exceeded scenarios
5. Create example with budget limits
6. Update README

**Acceptance Criteria:**
- Config accepts f64 values
- CLI receives budget parameter
- Error handling for budget exceeded (if applicable)
- Tests pass
- Example demonstrates cost control

---

### Task 4: Add `max_thinking_tokens`

**Files:**
- `src/types/config.rs`
- `src/internal/transport/subprocess.rs`
- `tests/integration_tests.rs`
- `examples/12-thinking-tokens.rs`

**Steps:**
1. Add field as `Option<u32>`
2. Add CLI argument
3. Write tests for thinking token configuration
4. Create example with thinking block inspection
5. Update README

**Acceptance Criteria:**
- Config accepts u32 values
- CLI receives thinking token limit
- Tests pass
- Example demonstrates thinking control

---

### Task 5: Add Plugin Support

**Files:**
- `src/types/mod.rs`
- `src/types/plugin.rs` (new)
- `src/types/config.rs`
- `src/internal/transport/subprocess.rs`
- `tests/plugin_tests.rs`
- `examples/17-custom-plugins.rs`
- `PLUGIN_GUIDE.md` (new documentation)

**Steps:**
1. Define `SdkPluginConfig` enum
2. Add `plugins` field to config
3. Implement plugin path serialization for CLI
4. Create test plugin for integration tests
5. Write comprehensive plugin tests
6. Create example with custom plugin
7. Write plugin development guide
8. Update README

**Acceptance Criteria:**
- Plugin types compile and serialize correctly
- CLI receives plugin paths
- Test plugin loads and executes
- Tests cover success and failure scenarios
- Example demonstrates plugin usage
- Documentation guides plugin development

---

## 11. Post-Implementation Checklist

### Code Quality
- [ ] All files pass `cargo fmt`
- [ ] All files pass `cargo clippy`
- [ ] All tests pass with `cargo test`
- [ ] Integration tests pass
- [ ] Examples run without errors
- [ ] Documentation builds without warnings

### Documentation
- [ ] README updated with new features
- [ ] API documentation complete (rustdoc)
- [ ] Examples documented with comments
- [ ] CHANGELOG updated
- [ ] Migration guide written (if needed)

### Release
- [ ] Version bumped in Cargo.toml
- [ ] Git tags created
- [ ] Release notes written
- [ ] Crates.io publish ready

---

## 12. Compatibility Matrix

| Python SDK Feature | Rust SDK v0.2.1 | Rust SDK v0.3.0 (Target) |
|-------------------|-----------------|--------------------------|
| Core APIs | ✅ | ✅ |
| Message Types | ✅ | ✅ |
| Hooks (6 types) | ✅ | ✅ |
| Permission Callbacks | ✅ | ✅ |
| MCP Servers | ✅ | ✅ |
| Session Management | ✅ | ✅ |
| Dynamic Control | ✅ | ✅ |
| `fallback_model` | ❌ | ✅ |
| `max_budget_usd` | ❌ | ✅ |
| `max_thinking_tokens` | ❌ | ✅ |
| `plugins` | ❌ | ✅ |
| Streaming Query | ⚠️ | ✅ |

**Legend:**
- ✅ Fully supported
- ⚠️ Partially supported / Different design
- ❌ Not supported

---

## 13. Performance Considerations

### 13.1 Memory Usage

**Current `query()` (collecting):**
- Collects all messages in memory
- O(n) memory where n = number of messages
- Suitable for: Small to medium conversations

**Proposed `query_stream()` (streaming):**
- Processes messages as they arrive
- O(1) memory per message
- Suitable for: Large conversations, real-time processing

**Recommendation:** Document trade-offs in API docs

### 13.2 CPU Usage

**Plugin Loading:**
- Overhead: ~50-100ms per plugin (estimated)
- Impact: Minimal for typical 1-3 plugins
- Mitigation: Lazy loading, caching

### 13.3 Network/IPC

**No impact expected** - all features use existing transport layer

---

## 14. Future Enhancements (Out of Scope)

Items not included in this plan but worth considering:

1. **Context Manager Pattern for Rust**
   - Async drop workaround
   - Custom guard types
   - Effort: 8 hours

2. **Runtime Independence**
   - Support async-std, smol
   - Runtime abstraction layer
   - Effort: 16 hours

3. **Advanced Plugin Features**
   - Hot reloading
   - Plugin sandboxing
   - Plugin marketplace
   - Effort: 40+ hours

4. **Performance Optimizations**
   - Zero-copy JSON parsing
   - Message pooling
   - Parallel processing
   - Effort: 20 hours

5. **Additional Safety Features**
   - Rate limiting
   - Retry logic with backoff
   - Circuit breaker pattern
   - Effort: 12 hours

---

## 15. Conclusion

The Rust Claude Agent SDK is in excellent shape with **95% feature parity** with the Python SDK v0.1.6. The remaining gaps are small and well-defined:

**Missing Features:**
1. `fallback_model` - 2 hours
2. `max_budget_usd` - 2 hours
3. `max_thinking_tokens` - 2 hours
4. `plugins` - 8 hours
5. Streaming query API - 4 hours

**Total Effort:** ~18 hours (~2.5 days)

The implementation plan is low-risk with no breaking changes. All additions are backward-compatible and follow existing patterns.

**Recommendation:** Proceed with implementation in 4 phases over 2-3 weeks, targeting v0.3.0 release.

---

## Appendix A: Python SDK v0.1.6 Complete Feature List

*This section provides the comprehensive exploration results for reference.*

### Configuration Options (30 total)
1. allowed_tools
2. system_prompt
3. mcp_servers
4. permission_mode
5. continue_conversation
6. resume
7. max_turns
8. max_budget_usd ⭐
9. disallowed_tools
10. model
11. fallback_model ⭐
12. permission_prompt_tool_name
13. cwd
14. cli_path
15. settings
16. add_dirs
17. env
18. extra_args
19. max_buffer_size
20. debug_stderr (deprecated)
21. stderr
22. can_use_tool
23. hooks
24. user
25. include_partial_messages
26. fork_session
27. agents
28. setting_sources
29. plugins ⭐
30. max_thinking_tokens ⭐

⭐ = Missing in Rust SDK v0.2.1

### Message Types (9 total)
- UserMessage
- AssistantMessage
- SystemMessage
- ResultMessage
- StreamEvent
- TextBlock
- ThinkingBlock
- ToolUseBlock
- ToolResultBlock

### Hook Events (6 total)
- PreToolUse
- PostToolUse
- UserPromptSubmit
- Stop
- SubagentStop
- PreCompact

### MCP Server Types (4 total)
- Stdio
- SSE
- HTTP
- SDK (in-process)

---

## Appendix B: Rust SDK v0.2.1 Complete Feature List

*This section provides the comprehensive exploration results for reference.*

### Current Capabilities
- All core APIs (query, ClaudeClient)
- All message types
- All hook types
- Complete permission system
- Complete MCP integration
- Session management
- Dynamic control
- Error handling
- 16 comprehensive examples

### Architecture Highlights
- Type-safe throughout
- Zero-copy streaming
- Lock-free design for stdin/stdout
- Async-first with Tokio
- Compile-time correctness
- Ergonomic builders

---

## Document Metadata

**Version:** 1.0
**Author:** Claude Code
**Date:** 2025-11-12
**Review Status:** Draft
**Target Audience:** Rust SDK developers, maintainers
**Related Documents:**
- Python SDK v0.1.6 source code
- Rust SDK v0.2.1 source code
- Claude Code CLI documentation

---

**END OF DOCUMENT**
