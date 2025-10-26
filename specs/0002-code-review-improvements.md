# Code Review & Improvement Opportunities

**Specification ID**: 0002
**Created**: 2024-10-26
**Status**: Proposed
**Priority**: Medium

## Executive Summary

This specification documents a comprehensive code review of the `claude-agent-sdk-rs` (v0.2.0) codebase, identifying opportunities for improvement across architecture, performance, robustness, developer experience, and maintainability. The codebase is already in excellent condition with 100% feature parity with the Python SDK, but several targeted improvements can enhance its production readiness and long-term maintainability.

### Overall Assessment

**Strengths:**
- ✅ Clean, idiomatic Rust code
- ✅ Zero clippy warnings with `-D warnings`
- ✅ Comprehensive test coverage (44 passing tests)
- ✅ Excellent documentation and examples (15 examples)
- ✅ Lock-free architecture preventing deadlocks
- ✅ Type-safe API with strong abstractions
- ✅ 100% feature parity with Python SDK

**Areas for Improvement:**
- Error handling could be more granular
- Some opportunities for better async resource management
- Testing could cover more integration scenarios
- Documentation could include more troubleshooting guidance
- Performance monitoring and observability could be enhanced

## Improvement Categories

### 1. Error Handling & Recovery

#### 1.1 Granular Error Types

**Current State:**
- `ClaudeError` enum has good coverage but some variants are too broad
- `Transport(String)` and `ControlProtocol(String)` lose type information
- Error context is sometimes limited for debugging

**Proposed Improvements:**

```rust
// In src/errors.rs

/// Transport error variants
#[derive(Debug, Error)]
pub enum TransportError {
    #[error("Failed to write to stdin: {0}")]
    StdinWrite(#[source] std::io::Error),

    #[error("Failed to read from stdout: {0}")]
    StdoutRead(#[source] std::io::Error),

    #[error("Connection closed unexpectedly")]
    ConnectionClosed,

    #[error("Buffer size exceeded: {current} > {max}")]
    BufferOverflow { current: usize, max: usize },

    #[error("Stdin not available")]
    StdinUnavailable,
}

/// Control protocol error variants
#[derive(Debug, Error)]
pub enum ControlProtocolError {
    #[error("Invalid request: missing {field}")]
    InvalidRequest { field: String },

    #[error("Request timeout after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    #[error("Unknown control request subtype: {subtype}")]
    UnknownSubtype { subtype: String },

    #[error("Hook callback not found: {callback_id}")]
    HookNotFound { callback_id: String },

    #[error("MCP server not found: {server_name}")]
    McpServerNotFound { server_name: String },
}
```

**Benefits:**
- Better error matching and handling in user code
- Richer error context for debugging
- Type-safe error discrimination
- Better integration with error reporting tools

**Implementation Complexity:** Low
**Impact:** Medium
**Breaking Change:** Yes (but minor - error enum variants)

---

#### 1.2 Timeout Handling for Control Requests

**Current State:**
- Control requests in `QueryFull::send_control_request()` wait indefinitely
- No timeout mechanism for unresponsive CLI
- Can hang if CLI encounters issues

**Proposed Improvements:**

```rust
// In src/internal/query_full.rs

pub struct QueryFull {
    // ... existing fields
    control_request_timeout: Duration,
}

async fn send_control_request(&self, request: serde_json::Value) -> Result<serde_json::Value> {
    // ... existing setup code

    // Wait for response with timeout
    let response = tokio::time::timeout(
        self.control_request_timeout,
        rx
    ).await.map_err(|_| {
        ClaudeError::ControlProtocol(ControlProtocolError::Timeout {
            timeout_ms: self.control_request_timeout.as_millis() as u64
        })
    })??;

    Ok(response)
}
```

**Benefits:**
- Prevents indefinite hangs
- Better error messages for timeout scenarios
- Configurable timeout per request or globally

**Implementation Complexity:** Low
**Impact:** High
**Breaking Change:** No (adds optional configuration)

---

### 2. Resource Management & Cleanup

#### 2.1 Graceful Shutdown with Cancellation Tokens

**Current State:**
- `ClaudeClient::disconnect()` has a hardcoded 100ms sleep
- Drop implementation warns but can't properly cleanup async resources
- Background tasks may continue briefly after disconnect

**Proposed Improvements:**

```rust
// In src/client.rs

use tokio_util::sync::CancellationToken;

pub struct ClaudeClient {
    // ... existing fields
    cancellation_token: CancellationToken,
}

impl ClaudeClient {
    pub async fn disconnect(&mut self) -> Result<()> {
        if !self.connected {
            return Ok(());
        }

        // Signal background tasks to stop
        self.cancellation_token.cancel();

        if let Some(query) = self.query.take() {
            let query_guard = query.lock().await;

            // Close stdin
            if let Some(ref stdin_arc) = query_guard.stdin {
                let mut stdin_guard = stdin_arc.lock().await;
                if let Some(mut stdin_stream) = stdin_guard.take() {
                    let _ = stdin_stream.shutdown().await;
                }
            }

            let transport = Arc::clone(&query_guard.transport);
            drop(query_guard);

            // Wait for background tasks to complete (with timeout)
            tokio::time::timeout(
                Duration::from_millis(500),
                self.wait_for_background_tasks()
            ).await.ok();

            let mut transport_guard = transport.lock().await;
            transport_guard.close().await?;
        }

        self.connected = false;
        Ok(())
    }
}
```

**Benefits:**
- Proper async resource cleanup
- No arbitrary sleep durations
- Clean shutdown even with running operations
- Drop implementation can safely cleanup

**Implementation Complexity:** Medium
**Impact:** High
**Breaking Change:** No

---

#### 2.2 Connection Pooling/Reuse

**Current State:**
- Each `ClaudeClient` spawns a new CLI subprocess
- No connection reuse or pooling
- Resource-intensive for multiple short interactions

**Proposed Improvements:**

```rust
// In src/client.rs (new file: src/client_pool.rs)

/// Connection pool for reusing Claude CLI processes
pub struct ClaudeClientPool {
    pool: Arc<Mutex<VecDeque<ClaudeClient>>>,
    max_size: usize,
    options: ClaudeAgentOptions,
}

impl ClaudeClientPool {
    pub async fn get(&self) -> Result<PooledClient> {
        // Try to get from pool, or create new if empty
        let client = {
            let mut pool = self.pool.lock().await;
            pool.pop_front()
        };

        match client {
            Some(mut c) => {
                // Verify connection still valid
                if c.is_healthy().await? {
                    Ok(PooledClient::new(c, Arc::clone(&self.pool)))
                } else {
                    // Reconnect
                    c.disconnect().await?;
                    c.connect().await?;
                    Ok(PooledClient::new(c, Arc::clone(&self.pool)))
                }
            }
            None => {
                let mut client = ClaudeClient::new(self.options.clone());
                client.connect().await?;
                Ok(PooledClient::new(client, Arc::clone(&self.pool)))
            }
        }
    }
}

/// RAII wrapper that returns client to pool on drop
pub struct PooledClient {
    client: Option<ClaudeClient>,
    pool: Arc<Mutex<VecDeque<ClaudeClient>>>,
}
```

**Benefits:**
- Reduced subprocess spawning overhead
- Better resource utilization for high-volume use cases
- Connection health checking
- Automatic connection recovery

**Implementation Complexity:** High
**Impact:** Medium
**Breaking Change:** No (additive feature)

---

### 3. Performance & Observability

#### 3.1 Tracing & Structured Logging

**Current State:**
- Uses `tracing` crate but minimal instrumentation
- No span tracking for async operations
- Hard to debug performance issues or track requests

**Proposed Improvements:**

```rust
// Throughout codebase, add structured tracing

// In src/client.rs
use tracing::{instrument, debug, warn, error, info_span};

#[instrument(skip(self), fields(session_id = ?self.session_id()))]
pub async fn query(&mut self, prompt: impl Into<String> + std::fmt::Debug) -> Result<()> {
    let prompt_str = prompt.into();
    debug!(prompt_len = prompt_str.len(), "Sending query to Claude");

    // ... existing code

    debug!("Query sent successfully");
    Ok(())
}

// In src/internal/query_full.rs
#[instrument(skip(self, request), fields(request_id = ?request["request_id"]))]
async fn send_control_request(&self, request: serde_json::Value) -> Result<serde_json::Value> {
    let start = std::time::Instant::now();

    // ... existing code

    let duration = start.elapsed();
    debug!(duration_ms = duration.as_millis(), "Control request completed");

    Ok(response)
}
```

**Benefits:**
- Better debugging experience
- Performance profiling capabilities
- Request tracing across async boundaries
- Production monitoring integration

**Implementation Complexity:** Low
**Impact:** High (for debugging/observability)
**Breaking Change:** No

---

#### 3.2 Metrics & Performance Monitoring

**Current State:**
- No built-in metrics
- No way to track query latency, token usage, error rates
- Users must implement their own monitoring

**Proposed Improvements:**

```rust
// New file: src/metrics.rs

/// Metrics collector for Claude SDK operations
#[derive(Clone)]
pub struct SdkMetrics {
    queries_total: Arc<AtomicU64>,
    queries_failed: Arc<AtomicU64>,
    query_duration_ms: Arc<Mutex<Vec<u64>>>,
    tokens_used: Arc<AtomicU64>,
    control_requests_total: Arc<AtomicU64>,
}

impl SdkMetrics {
    pub fn record_query_start(&self) -> QueryMetric {
        self.queries_total.fetch_add(1, Ordering::Relaxed);
        QueryMetric {
            start: Instant::now(),
            metrics: self.clone(),
        }
    }

    pub fn record_tokens_used(&self, tokens: u64) {
        self.tokens_used.fetch_add(tokens, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        // Return current metrics
    }
}

// In ClaudeAgentOptions
pub struct ClaudeAgentOptions {
    // ... existing fields
    pub metrics: Option<Arc<SdkMetrics>>,
}
```

**Benefits:**
- Built-in performance monitoring
- Easy integration with metrics systems (Prometheus, etc.)
- Usage analytics for optimization
- Anomaly detection capabilities

**Implementation Complexity:** Medium
**Impact:** Medium
**Breaking Change:** No (opt-in feature)

---

### 4. Robustness & Reliability

#### 4.1 Automatic Retry with Exponential Backoff

**Current State:**
- No automatic retry for transient failures
- Users must implement retry logic themselves
- Network/process issues cause immediate failure

**Proposed Improvements:**

```rust
// New file: src/retry.rs

#[derive(Clone)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
    pub retryable_errors: fn(&ClaudeError) -> bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            retryable_errors: |e| matches!(
                e,
                ClaudeError::Transport(_) | ClaudeError::Connection(_)
            ),
        }
    }
}

pub async fn with_retry<F, T, Fut>(
    policy: &RetryPolicy,
    operation: F,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut attempt = 0;
    let mut delay = policy.initial_delay;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt < policy.max_attempts && (policy.retryable_errors)(&e) => {
                attempt += 1;
                warn!(attempt, ?delay, "Retrying after error: {}", e);
                tokio::time::sleep(delay).await;
                delay = (delay * policy.backoff_multiplier as u32).min(policy.max_delay);
            }
            Err(e) => return Err(e),
        }
    }
}

// In ClaudeAgentOptions
pub struct ClaudeAgentOptions {
    // ... existing fields
    pub retry_policy: Option<RetryPolicy>,
}
```

**Benefits:**
- Resilience to transient failures
- Better user experience with automatic recovery
- Configurable retry behavior
- Exponential backoff prevents thundering herd

**Implementation Complexity:** Medium
**Impact:** High
**Breaking Change:** No (opt-in feature)

---

#### 4.2 Health Checking & Connection Validation

**Current State:**
- No health check mechanism
- Can't detect if CLI process is unresponsive
- No connection validation before operations

**Proposed Improvements:**

```rust
// In src/client.rs

impl ClaudeClient {
    /// Check if the connection is healthy
    #[instrument(skip(self))]
    pub async fn is_healthy(&self) -> Result<bool> {
        if !self.connected {
            return Ok(false);
        }

        // Send a lightweight ping control request
        let request = json!({
            "subtype": "ping"
        });

        match tokio::time::timeout(
            Duration::from_secs(5),
            self.send_control_request(request)
        ).await {
            Ok(Ok(_)) => Ok(true),
            Ok(Err(_)) => Ok(false),
            Err(_) => Ok(false), // Timeout
        }
    }

    /// Ensure connection is healthy before operation
    async fn ensure_healthy(&mut self) -> Result<()> {
        if !self.is_healthy().await? {
            warn!("Connection unhealthy, attempting reconnection");
            self.disconnect().await.ok();
            self.connect().await?;
        }
        Ok(())
    }
}
```

**Benefits:**
- Detect connection issues proactively
- Automatic recovery from transient failures
- Better error messages
- Connection pooling support

**Implementation Complexity:** Low
**Impact:** Medium
**Breaking Change:** No

---

### 5. Developer Experience

#### 5.1 Builder Pattern for Complex Queries

**Current State:**
- `ClaudeClient::query()` only accepts a string prompt
- No way to specify query-specific options
- Must recreate client for different configurations

**Proposed Improvements:**

```rust
// New file: src/query_builder.rs

pub struct QueryBuilder {
    prompt: String,
    model: Option<String>,
    max_turns: Option<u32>,
    permission_mode: Option<PermissionMode>,
    timeout: Option<Duration>,
    metadata: HashMap<String, serde_json::Value>,
}

impl QueryBuilder {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            model: None,
            max_turns: None,
            permission_mode: None,
            timeout: None,
            metadata: HashMap::new(),
        }
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn max_turns(mut self, turns: u32) -> Self {
        self.max_turns = Some(turns);
        self
    }

    pub fn permission_mode(mut self, mode: PermissionMode) -> Self {
        self.permission_mode = Some(mode);
        self
    }

    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    pub fn metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

// In ClaudeClient
impl ClaudeClient {
    pub async fn query_with(&mut self, builder: QueryBuilder) -> Result<()> {
        // Apply temporary settings from builder
        // Send query
        // Restore original settings
    }
}
```

**Benefits:**
- More flexible API
- Query-specific configurations
- Better discoverability via builder pattern
- Metadata for tracking/analytics

**Implementation Complexity:** Medium
**Impact:** Medium
**Breaking Change:** No (additive)

---

#### 5.2 Response Stream Helpers

**Current State:**
- Users must manually iterate streams and match on message types
- Repetitive boilerplate in examples
- No helper methods for common patterns

**Proposed Improvements:**

```rust
// In src/client.rs

impl ClaudeClient {
    /// Collect all text from assistant messages
    pub async fn query_for_text(&mut self, prompt: impl Into<String>) -> Result<String> {
        self.query(prompt).await?;

        let mut text = String::new();
        let mut stream = self.receive_response();

        while let Some(message) = stream.next().await {
            if let Message::Assistant(msg) = message? {
                for block in msg.message.content {
                    if let ContentBlock::Text(t) = block {
                        text.push_str(&t.text);
                    }
                }
            }
        }

        Ok(text)
    }

    /// Collect all messages until result
    pub async fn query_and_collect(&mut self, prompt: impl Into<String>) -> Result<Vec<Message>> {
        self.query(prompt).await?;

        let mut messages = Vec::new();
        let mut stream = self.receive_response();

        while let Some(message) = stream.next().await {
            messages.push(message?);
        }

        Ok(messages)
    }

    /// Get result metadata (cost, duration, etc.)
    pub async fn query_for_result(&mut self, prompt: impl Into<String>) -> Result<ResultMessage> {
        self.query(prompt).await?;

        let mut stream = self.receive_response();
        while let Some(message) = stream.next().await {
            if let Message::Result(result) = message? {
                return Ok(result);
            }
        }

        Err(ClaudeError::InvalidConfig("No result message received".to_string()))
    }
}
```

**Benefits:**
- Less boilerplate for common use cases
- More ergonomic API
- Consistent patterns across codebase
- Easier learning curve

**Implementation Complexity:** Low
**Impact:** Medium
**Breaking Change:** No

---

#### 5.3 Async Drop Support (Future Enhancement)

**Current State:**
- `Drop` implementation cannot run async code
- Warning printed if not properly disconnected
- No automatic cleanup on panic

**Proposed Improvements:**

```rust
// Wait for Rust language support for async Drop
// Or implement AsyncDrop trait pattern

#[async_trait]
pub trait AsyncDrop {
    async fn async_drop(&mut self);
}

#[async_trait]
impl AsyncDrop for ClaudeClient {
    async fn async_drop(&mut self) {
        if self.connected {
            let _ = self.disconnect().await;
        }
    }
}

// Helper for RAII with async cleanup
pub struct AsyncDropGuard<T: AsyncDrop> {
    inner: Option<T>,
}

impl<T: AsyncDrop> Drop for AsyncDropGuard<T> {
    fn drop(&mut self) {
        if let Some(mut inner) = self.inner.take() {
            // Spawn blocking cleanup
            tokio::runtime::Handle::current().spawn(async move {
                inner.async_drop().await;
            });
        }
    }
}
```

**Note:** This is a workaround until Rust supports async Drop natively.

**Benefits:**
- Automatic cleanup even on panic
- Better RAII semantics
- Fewer resource leaks

**Implementation Complexity:** High
**Impact:** Low (current approach is acceptable)
**Breaking Change:** No

---

### 6. Testing & Quality Assurance

#### 6.1 Integration Test Suite Expansion

**Current State:**
- Good unit test coverage (44 tests)
- Limited integration tests
- No end-to-end tests with real CLI
- No mock transport for testing

**Proposed Improvements:**

```rust
// New file: tests/mock_transport.rs

/// Mock transport for testing without Claude CLI
pub struct MockTransport {
    responses: Arc<Mutex<VecDeque<serde_json::Value>>>,
    sent_messages: Arc<Mutex<Vec<String>>>,
}

impl MockTransport {
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(VecDeque::new())),
            sent_messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn push_response(&self, response: serde_json::Value) {
        // Add canned response
    }

    pub fn get_sent_messages(&self) -> Vec<String> {
        // Return messages sent to transport
    }
}

// New tests:
// - tests/integration_bidirectional.rs - Full bidirectional flow
// - tests/integration_hooks.rs - Hook system testing
// - tests/integration_mcp.rs - MCP server testing
// - tests/integration_error_handling.rs - Error scenarios
// - tests/stress_test.rs - Concurrent operations, memory leaks
```

**Benefits:**
- Better test isolation
- Faster test execution
- More comprehensive coverage
- Easier to test edge cases

**Implementation Complexity:** Medium
**Impact:** High (for quality assurance)
**Breaking Change:** No

---

#### 6.2 Property-Based Testing

**Current State:**
- Only example-based unit tests
- No property testing for protocol correctness
- No fuzzing for message parsing

**Proposed Improvements:**

```rust
// Add proptest to dev-dependencies

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_message_roundtrip(message in any::<Message>()) {
            // Serialize and deserialize should be identity
            let json = serde_json::to_value(&message)?;
            let parsed: Message = serde_json::from_value(json)?;
            prop_assert_eq!(message, parsed);
        }

        #[test]
        fn test_control_request_parsing(
            request_id in "[a-z0-9]{10}",
            subtype in prop::sample::select(vec!["initialize", "interrupt", "set_model"])
        ) {
            // Any valid control request should parse without panic
            let request = json!({
                "type": "control_request",
                "request_id": request_id,
                "request": {
                    "subtype": subtype
                }
            });

            // Should not panic
            let _ = serde_json::from_value::<ControlRequest>(request);
        }
    }
}
```

**Benefits:**
- Find edge cases automatically
- Ensure protocol correctness
- Better confidence in parsing logic
- Discover unexpected behaviors

**Implementation Complexity:** Medium
**Impact:** Medium
**Breaking Change:** No

---

### 7. Documentation & Examples

#### 7.1 Troubleshooting Guide

**Current State:**
- Good API documentation
- Excellent examples (15 examples)
- No troubleshooting guide
- No common pitfalls documented

**Proposed Improvements:**

Create `docs/TROUBLESHOOTING.md`:

```markdown
# Troubleshooting Guide

## Common Issues

### Connection Errors

**Problem:** `CLI connection error: Failed to spawn Claude CLI process`

**Solutions:**
1. Verify Claude CLI is installed: `which claude`
2. Check CLI version: `claude --version` (need >= 2.0.0)
3. Set explicit path: `ClaudeAgentOptions { cli_path: Some(...), ... }`

### Timeout Issues

**Problem:** Operations hang indefinitely

**Solutions:**
1. Check Claude CLI is not waiting for input
2. Verify permission_mode is set correctly
3. Enable debug logging: `RUST_LOG=debug`

### Memory Leaks

**Problem:** Memory grows over time

**Solutions:**
1. Always call `disconnect()` on ClaudeClient
2. Drop stream before next query
3. Check for circular Arc references

## Performance Issues

### Slow Response Times

**Diagnostics:**
- Enable tracing: `RUST_LOG=claude_agent_sdk_rs=trace`
- Check network latency to Anthropic API
- Monitor subprocess CPU/memory usage

## Debugging Tips

### Enable Detailed Logging

```rust
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::TRACE)
    .init();
```

### Inspect Raw Messages

```rust
let options = ClaudeAgentOptions {
    stderr_callback: Some(Arc::new(|line| {
        eprintln!("CLI stderr: {}", line);
    })),
    ..Default::default()
};
```
```

**Benefits:**
- Faster issue resolution
- Better user experience
- Reduced support burden
- Community self-service

**Implementation Complexity:** Low
**Impact:** High (for UX)
**Breaking Change:** No

---

#### 7.2 Architecture Documentation

**Current State:**
- Code is well-structured but architecture not explicitly documented
- No diagrams showing component interactions
- Hard to onboard contributors

**Proposed Improvements:**

Create `docs/ARCHITECTURE.md`:

```markdown
# Architecture Overview

## Component Diagram

```
┌─────────────────────────────────────────────┐
│           User Application                  │
└─────────────────┬───────────────────────────┘
                  │
         ┌────────▼──────────┐
         │   Public API      │
         │ ─────────────────  │
         │ • query()         │
         │ • ClaudeClient    │
         │ • tool!()         │
         └────────┬──────────┘
                  │
    ┌─────────────▼──────────────┐
    │   Control Protocol Layer   │
    │  (QueryFull)               │
    │ ───────────────────────────│
    │ • Bidirectional control    │
    │ • Hook management          │
    │ • MCP routing              │
    └─────────────┬──────────────┘
                  │
         ┌────────▼──────────┐
         │  Transport Layer  │
         │  (Subprocess)     │
         └────────┬──────────┘
                  │
         ┌────────▼──────────┐
         │  Claude Code CLI  │
         └───────────────────┘
```

## Data Flow

[Sequence diagrams for common operations]

## Threading Model

[Explanation of async runtime, Arc/Mutex usage, lock-free design]
```

**Benefits:**
- Easier contributor onboarding
- Better design decisions
- Documentation of design rationale
- System understanding for debugging

**Implementation Complexity:** Low
**Impact:** Medium
**Breaking Change:** No

---

### 8. Security Enhancements

#### 8.1 Input Sanitization

**Current State:**
- User input passed directly to CLI
- No sanitization of prompts or tool inputs
- Potential for injection attacks via tool parameters

**Proposed Improvements:**

```rust
// New file: src/security.rs

/// Sanitize user input for safe CLI passage
pub fn sanitize_prompt(input: &str) -> String {
    // Remove null bytes
    let mut sanitized = input.replace('\0', "");

    // Limit length
    if sanitized.len() > MAX_PROMPT_LENGTH {
        sanitized.truncate(MAX_PROMPT_LENGTH);
    }

    sanitized
}

/// Validate tool input schema
pub fn validate_tool_input(
    input: &serde_json::Value,
    schema: &serde_json::Value,
) -> Result<()> {
    // Use jsonschema crate to validate
    let schema = jsonschema::JSONSchema::compile(schema)
        .map_err(|e| ClaudeError::InvalidConfig(format!("Invalid schema: {}", e)))?;

    schema.validate(input)
        .map_err(|e| ClaudeError::InvalidConfig(format!("Invalid tool input: {:?}", e)))?;

    Ok(())
}

// In ClaudeClient::query
pub async fn query(&mut self, prompt: impl Into<String>) -> Result<()> {
    let prompt_str = sanitize_prompt(&prompt.into());
    // ... rest of implementation
}
```

**Benefits:**
- Protection against injection attacks
- Input validation at SDK boundary
- Better error messages for invalid input
- Defense in depth

**Implementation Complexity:** Medium
**Impact:** High (for security)
**Breaking Change:** Potentially (if current invalid inputs work)

---

#### 8.2 Secure MCP Server Validation

**Current State:**
- No validation that MCP servers are safe
- Tool handlers can execute arbitrary code
- No sandboxing or resource limits

**Proposed Improvements:**

```rust
// In src/types/mcp.rs

pub struct McpServerSecurity {
    /// Maximum execution time for tool handlers
    pub max_execution_time: Duration,
    /// Maximum memory usage (future)
    pub max_memory_bytes: Option<usize>,
    /// Allowed filesystem paths (future)
    pub allowed_paths: Option<Vec<PathBuf>>,
}

// Wrap tool handlers with security boundaries
pub async fn execute_tool_with_limits(
    handler: &dyn ToolHandler,
    args: serde_json::Value,
    security: &McpServerSecurity,
) -> Result<ToolResult> {
    tokio::time::timeout(
        security.max_execution_time,
        handler.handle(args)
    )
    .await
    .map_err(|_| ClaudeError::Transport("Tool execution timeout".to_string()))?
}
```

**Benefits:**
- Prevent runaway tool executions
- Resource exhaustion protection
- Better control over custom tools
- Safety guarantees

**Implementation Complexity:** Medium
**Impact:** Medium
**Breaking Change:** No (opt-in)

---

## Implementation Roadmap

### Phase 1: Foundation (High Impact, Low Complexity)
**Priority: High**
**Estimated Effort: 1-2 weeks**

- [ ] Error handling improvements (1.1)
- [ ] Timeout handling for control requests (1.2)
- [ ] Tracing & structured logging (3.1)
- [ ] Response stream helpers (5.2)
- [ ] Troubleshooting guide (7.1)
- [ ] Architecture documentation (7.2)

**Deliverables:**
- More robust error handling
- Better observability
- Improved developer experience
- Documentation updates

---

### Phase 2: Reliability (High Impact, Medium Complexity)
**Priority: High**
**Estimated Effort: 2-3 weeks**

- [ ] Graceful shutdown improvements (2.1)
- [ ] Health checking (4.2)
- [ ] Automatic retry logic (4.1)
- [ ] Integration test expansion (6.1)
- [ ] Input sanitization (8.1)

**Deliverables:**
- Improved reliability
- Automatic error recovery
- Better test coverage
- Security hardening

---

### Phase 3: Advanced Features (Medium Impact, Medium-High Complexity)
**Priority: Medium**
**Estimated Effort: 3-4 weeks**

- [ ] Metrics & monitoring (3.2)
- [ ] Query builder pattern (5.1)
- [ ] Connection pooling (2.2)
- [ ] Property-based testing (6.2)
- [ ] MCP server security (8.2)

**Deliverables:**
- Performance monitoring
- Advanced API features
- Better resource utilization
- Comprehensive testing

---

### Phase 4: Polish (Lower Impact, Varies)
**Priority: Low**
**Estimated Effort: 1-2 weeks**

- [ ] Async drop support (5.3)
- [ ] Additional examples
- [ ] Performance benchmarks
- [ ] Contribution guidelines

**Deliverables:**
- Final polish
- Community enablement
- Performance baseline

---

## Breaking Changes Assessment

Most improvements are **non-breaking** and can be implemented additively. The few potentially breaking changes:

### Minor Breaking Changes (Acceptable for 0.x versions)

1. **Error enum refinement (1.1)**
   - Impact: Pattern matching on errors
   - Mitigation: Keep old variants as deprecated, add new ones
   - Upgrade path: Update match arms

2. **Input sanitization (8.1)**
   - Impact: Previously "working" invalid input may be rejected
   - Mitigation: Add `allow_unsanitized` option
   - Upgrade path: Fix invalid inputs or opt-out

### Non-Breaking Additions (Preferred)

All other improvements can be implemented as:
- Optional configuration fields
- New methods alongside existing ones
- Opt-in features via cargo features
- Internal implementation changes

---

## Success Metrics

### Code Quality
- [ ] Zero clippy warnings (already achieved ✅)
- [ ] Test coverage > 80%
- [ ] All public APIs documented (already achieved ✅)
- [ ] No unsafe code (already achieved ✅)

### Reliability
- [ ] < 0.1% error rate in integration tests
- [ ] Automatic recovery from transient failures
- [ ] No memory leaks under stress testing
- [ ] Graceful degradation on CLI failures

### Performance
- [ ] < 50ms overhead over raw CLI
- [ ] Constant memory usage over time
- [ ] Efficient streaming (no buffering beyond limits)
- [ ] Concurrent operations without contention

### Developer Experience
- [ ] < 5 minutes to first successful query
- [ ] Common tasks achievable in < 10 LOC
- [ ] Clear error messages for all failure modes
- [ ] Examples cover 90% of use cases

---

## Risks & Mitigations

### Risk: Breaking Changes
**Likelihood:** Medium
**Impact:** High
**Mitigation:**
- Maintain backwards compatibility where possible
- Use deprecation warnings before removal
- Version according to semver strictly
- Provide migration guides

### Risk: Performance Regression
**Likelihood:** Low
**Impact:** Medium
**Mitigation:**
- Benchmark before/after changes
- Profile critical paths
- Load testing for pooling/retry logic
- Make heavy features opt-in

### Risk: Increased Complexity
**Likelihood:** Medium
**Impact:** Medium
**Mitigation:**
- Keep core simple, advanced features optional
- Maintain clear separation of concerns
- Document complexity tradeoffs
- Provide sensible defaults

### Risk: Python SDK Divergence
**Likelihood:** Low
**Impact:** Medium
**Mitigation:**
- Monitor Python SDK changes
- Maintain feature parity checklist
- Contribute improvements back to Python SDK
- Clear documentation of Rust-specific features

---

## Alternatives Considered

### Alternative 1: Keep Current Implementation
**Pros:**
- No development effort
- No risk of regressions
- Stable API

**Cons:**
- Missing production-ready features
- Limited observability
- Harder to debug issues
- Less competitive with other SDKs

**Decision:** Rejected - improvements provide significant value

---

### Alternative 2: Major Refactor
**Pros:**
- Clean slate for ideal architecture
- Fix all design issues at once
- Modern patterns throughout

**Cons:**
- Large breaking change
- High development cost
- High risk
- Delays value delivery

**Decision:** Rejected - current architecture is sound, incremental improvements preferred

---

### Alternative 3: Fork Architecture from Another SDK
**Pros:**
- Proven architecture
- Faster development
- Existing best practices

**Cons:**
- May not fit Rust idioms
- Loses current advantages (lock-free, etc.)
- Not NIH, but not optimal

**Decision:** Rejected - current architecture is excellent, just needs refinement

---

## Open Questions

1. **Metrics backend integration:**
   - Should we integrate with specific metrics libraries (prometheus-rs)?
   - Or provide a generic trait for users to implement?
   - **Recommendation:** Generic trait + optional feature for popular backends

2. **Connection pool configuration:**
   - Max pool size?
   - Idle timeout?
   - Health check frequency?
   - **Recommendation:** Research common values from database pools

3. **Async drop:**
   - Wait for language feature?
   - Implement workaround now?
   - **Recommendation:** Document current limitation, implement workaround if users request

4. **Feature flags:**
   - Should metrics, tracing, retry be behind feature flags?
   - Balance between flexibility and compilation time?
   - **Recommendation:** Core features included, experimental ones behind flags

---

## Conclusion

The `claude-agent-sdk-rs` codebase is in excellent condition with clean architecture, comprehensive testing, and good documentation. The proposed improvements focus on:

1. **Production readiness:** Error handling, timeouts, retry logic
2. **Observability:** Tracing, metrics, health checks
3. **Developer experience:** Better APIs, helpers, documentation
4. **Security:** Input validation, resource limits

The phased approach allows incremental delivery of value while maintaining backwards compatibility and minimizing risk. Phase 1 improvements alone will significantly enhance the production readiness and developer experience of the SDK.

### Recommended Next Steps

1. Review and prioritize improvements based on user feedback
2. Start with Phase 1 (foundation) improvements
3. Gather metrics from early adopters to validate Phase 2 priorities
4. Iterate based on real-world usage patterns

### Estimated Total Effort

- **Phase 1:** 1-2 weeks (high priority)
- **Phase 2:** 2-3 weeks (high priority)
- **Phase 3:** 3-4 weeks (medium priority)
- **Phase 4:** 1-2 weeks (polish)

**Total:** 7-11 weeks for complete implementation

With the excellent foundation already in place, these improvements will elevate the SDK to production-ready status while maintaining its current strengths of type safety, performance, and idiomatic Rust design.
