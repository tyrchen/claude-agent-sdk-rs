# Specification: Concurrency Improvements

## Overview

This specification addresses concurrency patterns, lock contention, and architectural improvements in the claude-agent-sdk-rs codebase. The goal is to improve performance, eliminate potential deadlocks, add proper task management, and establish best practices for concurrent operations.

## Executive Summary

A comprehensive analysis of the codebase's concurrency patterns revealed several areas for improvement:

1. **Arc<Mutex<UnboundedReceiver>> antipattern** causing unnecessary lock contention
2. **Missing task management** infrastructure (no cancellation, no graceful shutdown)
3. **Nested lock acquisition** in disconnect flow creating potential deadlock risk
4. **Hardcoded sleep** instead of proper synchronization
5. **No error propagation** from background tasks
6. **Locks held across I/O** operations reducing throughput

The codebase demonstrates strong fundamentals (no blocking operations, proper use of tokio primitives, clever stdin bypass pattern), but needs architectural refinements for production-grade reliability and performance.

## Current State Analysis

### Positive Findings

✅ **No blocking operations in async code** - All I/O is async, no std::sync::Mutex usage
✅ **Stdin bypass pattern** - Clever solution to prevent transport lock deadlock
✅ **Proper use of oneshot channels** - Clean request/response pattern
✅ **Hook callbacks executed without holding locks** - Prevents callback delays from blocking system
✅ **Consistent tokio primitives** - All async primitives from tokio::sync

### Critical Issues

#### 1. Arc<Mutex<UnboundedReceiver>> Antipattern

**Location:** `src/internal/query_full.rs:64`, `src/client.rs:315-318`, `391-394`

**Problem:**
```rust
// In QueryFull
message_rx: Arc<Mutex<mpsc::UnboundedReceiver<serde_json::Value>>>,

// In receive_response()
let rx = Arc::clone(&query_guard.message_rx);
loop {
    let mut rx_guard = rx.lock().await;  // Lock on every message!
    let message = rx_guard.recv().await;
    // ...
}
```

**Issues:**
- UnboundedReceiver is already designed for single ownership
- Wrapping in Arc<Mutex> creates unnecessary contention
- Lock acquired and released on every single message
- Multiple consumers would fight for the lock
- Violates channel design principles

**Impact:** High - Degrades performance in high-throughput scenarios

#### 2. No Task Cancellation/Shutdown Mechanism

**Location:** All spawned tasks in `query_full.rs`, `subprocess.rs`

**Problem:**
```rust
tokio::spawn(async move {
    // Long-running background task
    while let Some(result) = stream.next().await {
        // No way to cancel this loop
    }
    // No cleanup, no error reporting
});
// JoinHandle dropped immediately - can't await or cancel
```

**Issues:**
- Background tasks run forever until transport closes
- No graceful shutdown capability
- No way to cancel operations mid-flight
- No task health monitoring
- Panics in tasks are silent
- Resource leaks possible if tasks don't complete

**Impact:** High - Prevents graceful shutdown, complicates testing, risks resource leaks

#### 3. Nested Lock Acquisition in Disconnect

**Location:** `src/client.rs:570-583`

**Problem:**
```rust
let query_guard = query.lock().await;       // Lock 1
if let Some(ref stdin_arc) = query_guard.stdin {
    let mut stdin_guard = stdin_arc.lock().await;  // Lock 2 (while holding 1!)
    // ...
}
```

**Issues:**
- Nested lock pattern creates deadlock risk
- If another code path acquires locks in different order, deadlock occurs
- Currently safe only because no other paths do this
- Fragile - easy to break in future changes

**Impact:** Medium - Low immediate risk, high future risk

#### 4. Hardcoded Sleep for Synchronization

**Location:** `src/client.rs:581`

**Problem:**
```rust
tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
// Comment: "Give the background task a moment to finish"
```

**Issues:**
- Race condition: 100ms may not be enough
- Inefficient: Always waits 100ms even if task finishes in 1ms
- No actual synchronization - just hoping task completes
- Brittle: Will break if system is slow or under load

**Impact:** Medium - Causes unnecessary delays and unreliable shutdown

#### 5. Locks Held Across Async I/O

**Location:** `src/client.rs:250-263`, `src/internal/query_full.rs:327-343`, `380-395`

**Problem:**
```rust
let mut stdin_guard = stdin_arc.lock().await;
if let Some(ref mut stdin_stream) = *stdin_guard {
    stdin_stream.write_all(message_str.as_bytes()).await  // I/O with lock!
        .map_err(...)?;
    stdin_stream.write_all(b"\n").await                   // I/O with lock!
        .map_err(...)?;
    stdin_stream.flush().await                            // I/O with lock!
        .map_err(...)?;
}
// Lock held for duration of 3 async I/O operations
```

**Issues:**
- Lock held longer than necessary
- Serializes all writes (good for correctness, bad for performance)
- Could batch operations before acquiring lock
- Increases lock hold time linearly with write size

**Impact:** Medium - Reduces write throughput under concurrent load

## Requirements

### Functional Requirements

- **FR-1**: Replace Arc<Mutex<UnboundedReceiver>> with proper broadcast channel pattern
- **FR-2**: Implement structured concurrency with task cancellation support
- **FR-3**: Eliminate nested lock acquisition patterns
- **FR-4**: Replace sleep-based synchronization with proper signaling
- **FR-5**: Add error propagation from background tasks
- **FR-6**: Implement graceful shutdown with timeout
- **FR-7**: Store and manage JoinHandles for all spawned tasks
- **FR-8**: Add task health monitoring and restart capability
- **FR-9**: Optimize lock hold times across I/O operations
- **FR-10**: Document lock acquisition order and concurrency invariants

### Non-Functional Requirements

- **NFR-1**: Performance - Reduce lock contention by 80% in high-throughput scenarios
- **NFR-2**: Reliability - Enable 100% graceful shutdown success rate
- **NFR-3**: Maintainability - Clear concurrency patterns that prevent future bugs
- **NFR-4**: Testability - Allow tests to cancel operations and verify cleanup
- **NFR-5**: Latency - Reduce disconnect time from 100ms+ to <10ms typical case
- **NFR-6**: Backward Compatibility - Maintain existing public API surface

## Architecture

### High-Level Design

#### Current Architecture (Problematic)

```
┌─────────────────────────────────────────────────────────────┐
│                      ClaudeClient                           │
│  query: Arc<Mutex<QueryFull>>                              │
└─────────────────┬───────────────────────────────────────────┘
                  │
┌─────────────────┴───────────────────────────────────────────┐
│                      QueryFull                              │
│  message_rx: Arc<Mutex<UnboundedReceiver>> ⚠️              │
│  [No task management] ⚠️                                    │
│  [No cancellation] ⚠️                                       │
└─────────────────┬───────────────────────────────────────────┘
                  │
         ┌────────┴─────────┐
         │                  │
    [Spawned Task]    [Spawned Task]
    [No handle] ⚠️    [No handle] ⚠️
```

**Problems:**
- Shared receiver wrapped in Mutex (contention)
- No task lifecycle management
- No way to cancel or await tasks

#### Proposed Architecture (Improved)

```
┌─────────────────────────────────────────────────────────────┐
│                      ClaudeClient                           │
│  query: Arc<Mutex<QueryFull>>                              │
│  cancel_token: CancellationToken ✓                         │
└─────────────────┬───────────────────────────────────────────┘
                  │
┌─────────────────┴───────────────────────────────────────────┐
│                      QueryFull                              │
│  message_broadcast: broadcast::Sender<Message> ✓           │
│  task_manager: TaskManager ✓                               │
│  cancel_token: CancellationToken ✓                         │
└─────────────────┬───────────────────────────────────────────┘
                  │
         ┌────────┴─────────┐
         │                  │
    [Task Handle]      [Task Handle]
    [Cancellable]      [Cancellable]
    [Supervised]       [Supervised]
```

**Benefits:**
- Broadcast channel allows multiple subscribers without contention
- Task manager tracks and controls all spawned tasks
- Cancellation token enables graceful shutdown
- Task supervision allows health monitoring and restart

### Component Design

#### 1. Message Broadcasting System

**Replace:** Arc<Mutex<UnboundedReceiver>>
**With:** tokio::sync::broadcast channel

**Pattern:**

```rust
pub struct QueryFull {
    // Internal: single consumer receives from transport
    message_rx: mpsc::UnboundedReceiver<serde_json::Value>,

    // Public: broadcast to multiple stream consumers
    message_broadcast: broadcast::Sender<Message>,

    // ...
}

impl QueryFull {
    // Background task: Single consumer broadcasts to subscribers
    async fn message_distributor_task(
        mut message_rx: mpsc::UnboundedReceiver<serde_json::Value>,
        broadcast_tx: broadcast::Sender<Message>,
        cancel: CancellationToken,
    ) {
        loop {
            tokio::select! {
                _ = cancel.cancelled() => break,
                Some(json) = message_rx.recv() => {
                    match MessageParser::parse(json) {
                        Ok(msg) => {
                            let _ = broadcast_tx.send(msg);
                        }
                        Err(e) => {
                            // Log error
                        }
                    }
                }
            }
        }
    }
}
```

**Benefits:**
- No lock contention - each subscriber has own receiver
- Multiple streams can coexist without interfering
- Follows proper channel patterns
- Better performance under load

**Tradeoffs:**
- Broadcast has bounded buffer (configurable, e.g., 100)
- Slow consumers may miss messages if buffer overflows
- Slightly higher memory usage per subscriber

**Mitigation:**
- Document buffer size and lagging behavior
- Configure buffer large enough for typical usage
- Provide backpressure mechanism if needed

#### 2. Structured Concurrency with Task Management

**Add:** TaskManager component

```rust
use tokio_util::sync::CancellationToken;
use tokio::task::JoinHandle;

pub struct TaskManager {
    tasks: Vec<JoinHandle<Result<()>>>,
    cancel_token: CancellationToken,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            cancel_token: CancellationToken::new(),
        }
    }

    pub fn spawn<F>(&mut self, name: &str, future: F) -> &mut Self
    where
        F: Future<Output = Result<()>> + Send + 'static,
    {
        let cancel = self.cancel_token.child_token();
        let task_name = name.to_string();

        let handle = tokio::spawn(async move {
            tokio::select! {
                _ = cancel.cancelled() => {
                    tracing::debug!("Task '{}' cancelled", task_name);
                    Ok(())
                }
                result = future => {
                    if let Err(ref e) = result {
                        tracing::error!("Task '{}' failed: {}", task_name, e);
                    }
                    result
                }
            }
        });

        self.tasks.push(handle);
        self
    }

    pub async fn shutdown(self, timeout: Duration) -> Result<()> {
        // Signal all tasks to stop
        self.cancel_token.cancel();

        // Wait for all tasks with timeout
        match tokio::time::timeout(timeout, async {
            for handle in self.tasks {
                let _ = handle.await;
            }
        })
        .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(ClaudeError::Transport(
                "Task shutdown timeout".to_string()
            )),
        }
    }

    pub fn cancel_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }
}
```

**Integration:**

```rust
pub struct QueryFull {
    task_manager: TaskManager,
    // ... existing fields
}

impl QueryFull {
    pub async fn start(&mut self) -> Result<()> {
        // Spawn background tasks with management
        self.task_manager.spawn(
            "message_reader",
            Self::message_reader_task(/* ... */),
        );

        self.task_manager.spawn(
            "message_distributor",
            Self::message_distributor_task(/* ... */),
        );

        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        // Replace hardcoded sleep with proper shutdown
        self.task_manager
            .shutdown(Duration::from_secs(5))
            .await
    }
}
```

**Benefits:**
- Graceful shutdown with timeout
- All tasks properly awaited
- Error propagation from background tasks
- Cancellation token hierarchy for nested tasks
- Testable - can inject cancel token

#### 3. Lock Ordering and Deadlock Prevention

**Add:** Documentation and helper methods

```rust
// In QueryFull or new concurrency.rs module:

/// Lock Acquisition Order (MUST follow):
/// 1. query (ClaudeClient level)
/// 2. stdin
/// 3. transport
///
/// NEVER acquire in different order!
///
/// Example CORRECT:
/// ```
/// let query = self.query.lock().await;
/// let stdin = query.stdin.clone();
/// drop(query);  // Release before next lock
/// let stdin = stdin.lock().await;
/// ```
///
/// Example INCORRECT (will deadlock):
/// ```
/// let stdin = self.stdin.lock().await;
/// let query = self.query.lock().await;  // ❌ Wrong order!
/// ```
mod lock_order {
    // Type system enforcement (future work)
}

/// Helper to extract stdin without nested lock
impl QueryFull {
    pub(crate) async fn get_stdin(&self) -> Option<Arc<Mutex<Option<ChildStdin>>>> {
        // Lock query, clone stdin, release immediately
        let guard = self.query.lock().await;
        let stdin = guard.stdin.clone();
        drop(guard);
        stdin
    }
}
```

**Refactor disconnect() to eliminate nesting:**

```rust
// Before (nested locks):
pub async fn disconnect(&mut self) -> Result<()> {
    let query_guard = query.lock().await;
    if let Some(ref stdin_arc) = query_guard.stdin {
        let mut stdin_guard = stdin_arc.lock().await;  // ❌ Nested!
        // ...
    }
}

// After (sequential locks):
pub async fn disconnect(&mut self) -> Result<()> {
    // Step 1: Extract stdin reference
    let stdin_arc = {
        let query_guard = query.lock().await;
        query_guard.stdin.clone()
    }; // query_guard dropped here

    // Step 2: Lock stdin separately
    if let Some(stdin_arc) = stdin_arc {
        let mut stdin_guard = stdin_arc.lock().await;  // ✓ No nesting
        // ...
    }

    // Step 3: Lock transport separately
    let transport = {
        let query_guard = query.lock().await;
        Arc::clone(&query_guard.transport)
    }; // query_guard dropped here

    let mut transport_guard = transport.lock().await;  // ✓ No nesting
    // ...
}
```

#### 4. Optimize Lock Hold Times

**Pattern:** Prepare data before lock, minimize critical section

```rust
// Before (lock held across I/O):
let mut stdin_guard = stdin_arc.lock().await;
if let Some(ref mut stdin_stream) = *stdin_guard {
    stdin_stream.write_all(message.as_bytes()).await?;
    stdin_stream.write_all(b"\n").await?;
    stdin_stream.flush().await?;
}

// After (prepare outside lock):
// Batch all data first
let mut buffer = Vec::with_capacity(message.len() + 1);
buffer.extend_from_slice(message.as_bytes());
buffer.push(b'\n');

// Lock only for write
let mut stdin_guard = stdin_arc.lock().await;
if let Some(ref mut stdin_stream) = *stdin_guard {
    stdin_stream.write_all(&buffer).await?;
    stdin_stream.flush().await?;  // Still needed for correctness
}
```

**Alternative:** Use channel for write serialization

```rust
// Instead of locking stdin, use mpsc channel
pub struct QueryFull {
    write_tx: mpsc::UnboundedSender<Vec<u8>>,
    // ...
}

// Writer task (single consumer)
async fn writer_task(
    mut write_rx: mpsc::UnboundedReceiver<Vec<u8>>,
    stdin: ChildStdin,
    cancel: CancellationToken,
) {
    loop {
        tokio::select! {
            _ = cancel.cancelled() => break,
            Some(data) = write_rx.recv() => {
                if let Err(e) = stdin.write_all(&data).await {
                    // Handle error
                    break;
                }
                let _ = stdin.flush().await;
            }
        }
    }
}

// To send: just submit to channel
self.write_tx.send(data)?;
```

**Tradeoffs:**
- Channel approach: No lock contention, but adds latency
- Batching approach: Reduces lock time, minimal complexity
- Recommendation: Start with batching, add channel if profiling shows contention

#### 5. Proper Synchronization for Shutdown

**Replace hardcoded sleep with oneshot channel:**

```rust
// Before:
tokio::time::sleep(Duration::from_millis(100)).await;

// After:
pub struct QueryFull {
    shutdown_tx: Option<oneshot::Sender<()>>,
    shutdown_rx: Option<oneshot::Receiver<()>>,
}

// In background task:
async fn background_task(
    // ...
    shutdown_tx: oneshot::Sender<()>,
) {
    // ... do work ...

    // When shutting down:
    let _ = shutdown_tx.send(());  // Signal completion
}

// In disconnect():
if let Some(rx) = self.shutdown_rx.take() {
    match tokio::time::timeout(Duration::from_secs(5), rx).await {
        Ok(Ok(())) => { /* Clean shutdown */ }
        Ok(Err(_)) => { /* Sender dropped */ }
        Err(_) => { /* Timeout */ }
    }
}
```

**Better: Use TaskManager (already includes this pattern)**

### Interfaces

#### Public API Changes

**No breaking changes to public API.** All improvements are internal.

**Behavioral changes:**
- `disconnect()` becomes faster (no 100ms sleep)
- `disconnect()` becomes more reliable (proper synchronization)
- Better error messages if shutdown fails

#### Internal Interface Changes

**QueryFull:**
```rust
pub struct QueryFull {
    // REMOVED:
    // message_rx: Arc<Mutex<mpsc::UnboundedReceiver<serde_json::Value>>>,

    // ADDED:
    message_broadcast: broadcast::Sender<Message>,
    task_manager: TaskManager,
    cancel_token: CancellationToken,

    // ... existing fields unchanged
}

impl QueryFull {
    // NEW METHODS:

    /// Get a subscription to message stream
    /// Multiple subscribers can coexist without contention
    pub fn subscribe_messages(&self) -> broadcast::Receiver<Message> {
        self.message_broadcast.subscribe()
    }

    /// Gracefully shutdown all background tasks
    pub async fn shutdown(&mut self, timeout: Duration) -> Result<()> {
        self.task_manager.shutdown(timeout).await
    }

    /// Get cancellation token for this query
    pub fn cancel_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }
}
```

**ClaudeClient:**
```rust
impl ClaudeClient {
    // MODIFIED (internal implementation only):
    pub async fn disconnect(&mut self) -> Result<()> {
        // Now uses proper shutdown instead of sleep
    }
}
```

## Implementation Steps

### Phase 1: Foundation (Add Infrastructure)

**Goal:** Add new components without breaking existing code

**Steps:**

1. **Add dependencies to Cargo.toml**
   ```toml
   tokio-util = { version = "0.7", features = ["sync"] }
   ```

2. **Create TaskManager module**
   - File: `src/internal/task_manager.rs`
   - Implement TaskManager struct and methods
   - Add tests for cancellation and shutdown

3. **Add TaskManager to QueryFull**
   - Add fields: `task_manager: TaskManager`
   - Add fields: `cancel_token: CancellationToken`
   - Initialize in `new()`
   - No functional changes yet

4. **Add broadcast channel alongside existing receiver**
   - Add field: `message_broadcast: broadcast::Sender<Message>`
   - Create broadcast channel in `new()`
   - Don't remove old receiver yet (parallel implementation)

### Phase 2: Migrate to Broadcast Channel

**Goal:** Replace Arc<Mutex<UnboundedReceiver>> with broadcast

**Steps:**

1. **Create message distributor task**
   - New method: `message_distributor_task()`
   - Reads from `message_rx`, parses, broadcasts to `message_broadcast`
   - Spawn via `task_manager`

2. **Update receive_messages() to use broadcast**
   - Replace: `Arc::clone(&query_guard.message_rx)`
   - With: `query_guard.subscribe_messages()`
   - Test thoroughly

3. **Update receive_response() to use broadcast**
   - Same change as receive_messages()
   - Test both can coexist

4. **Remove old Arc<Mutex<UnboundedReceiver>>**
   - Remove field: `message_rx: Arc<Mutex<...>>`
   - Clean up all references
   - Verify tests pass

### Phase 3: Implement Structured Concurrency

**Goal:** All background tasks managed via TaskManager

**Steps:**

1. **Update start() to use TaskManager**
   - Replace: `tokio::spawn(async move { ... })`
   - With: `self.task_manager.spawn("name", async move { ... })`
   - Add cancellation support to all task bodies

2. **Add cancellation to background tasks**
   - Wrap main loops with `tokio::select! { _ = cancel.cancelled() => break, ... }`
   - Ensure clean exit path

3. **Update disconnect() to use proper shutdown**
   - Replace: `tokio::time::sleep(Duration::from_millis(100)).await`
   - With: `self.task_manager.shutdown(Duration::from_secs(5)).await?`
   - Update error handling

### Phase 4: Fix Lock Ordering Issues

**Goal:** Eliminate nested lock acquisition

**Steps:**

1. **Add lock ordering documentation**
   - Document in module-level comments
   - Add comments at each lock site
   - Create developer guide

2. **Refactor disconnect() to avoid nesting**
   - Extract stdin before locking
   - Extract transport before locking
   - Sequential lock acquisition only

3. **Audit all lock sites**
   - Search for nested lock patterns
   - Refactor any found
   - Add tests for deadlock scenarios (using tokio::time::timeout)

### Phase 5: Optimize Lock Hold Times

**Goal:** Minimize time locks are held across I/O

**Steps:**

1. **Batch writes before locking stdin**
   - Prepare complete buffer
   - Acquire lock
   - Single write_all() call
   - Flush
   - Release lock

2. **Profile lock contention**
   - Add tracing instrumentation
   - Measure lock hold times
   - Identify bottlenecks

3. **Consider channel-based write serialization**
   - If profiling shows contention, implement writer task
   - Otherwise keep batching approach for simplicity

### Phase 6: Testing and Validation

**Goal:** Ensure all improvements work correctly

**Steps:**

1. **Add concurrency tests**
   - Test multiple concurrent streams
   - Test cancellation
   - Test shutdown timeout
   - Test graceful vs forceful shutdown

2. **Add stress tests**
   - High message throughput
   - Multiple simultaneous clients
   - Rapid connect/disconnect cycles

3. **Add deadlock detection tests**
   - Use tokio::time::timeout to detect hangs
   - Test various lock acquisition orders
   - Test error paths

4. **Update examples**
   - Add example demonstrating cancellation
   - Add example demonstrating multiple streams
   - Add example showing graceful shutdown

### Phase 7: Documentation

**Goal:** Document new patterns and best practices

**Steps:**

1. **Update module documentation**
   - Document lock ordering requirements
   - Document task management patterns
   - Document broadcast channel semantics

2. **Create concurrency guide**
   - File: `docs/CONCURRENCY.md`
   - Explain architecture decisions
   - Provide examples of correct patterns
   - Show common pitfalls to avoid

3. **Update CHANGELOG**
   - Document all improvements
   - Note any behavioral changes
   - Emphasize backward compatibility

## Testing Strategy

### Unit Tests

**TaskManager:**
```rust
#[tokio::test]
async fn test_task_manager_shutdown() {
    let mut tm = TaskManager::new();
    let cancel = tm.cancel_token();

    tm.spawn("test", async move {
        loop {
            tokio::select! {
                _ = cancel.cancelled() => break,
                _ = tokio::time::sleep(Duration::from_millis(10)) => {}
            }
        }
        Ok(())
    });

    let result = tm.shutdown(Duration::from_secs(1)).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_task_manager_timeout() {
    let mut tm = TaskManager::new();

    tm.spawn("stuck", async {
        tokio::time::sleep(Duration::from_secs(10)).await;  // Ignores cancellation
        Ok(())
    });

    let result = tm.shutdown(Duration::from_millis(100)).await;
    assert!(result.is_err());
}
```

**Broadcast Channel:**
```rust
#[tokio::test]
async fn test_multiple_subscribers() {
    let (tx, _) = broadcast::channel(10);

    let mut sub1 = tx.subscribe();
    let mut sub2 = tx.subscribe();

    tx.send("message".to_string()).unwrap();

    assert_eq!(sub1.recv().await.unwrap(), "message");
    assert_eq!(sub2.recv().await.unwrap(), "message");
}

#[tokio::test]
async fn test_lagging_subscriber() {
    let (tx, _) = broadcast::channel(2);  // Small buffer
    let mut sub = tx.subscribe();

    // Overfill buffer
    for i in 0..5 {
        tx.send(i).unwrap();
    }

    // First recv should return Lagged error
    match sub.recv().await {
        Err(broadcast::error::RecvError::Lagged(n)) => {
            assert!(n > 0);
        }
        _ => panic!("Expected Lagged error"),
    }
}
```

### Integration Tests

**Lock Ordering:**
```rust
#[tokio::test]
async fn test_no_deadlock_on_disconnect() {
    let mut client = ClaudeClient::new(ClaudeAgentOptions::default());
    client.connect().await.unwrap();

    // Should complete in reasonable time (no deadlock)
    let result = tokio::time::timeout(
        Duration::from_secs(2),
        client.disconnect()
    ).await;

    assert!(result.is_ok());
}
```

**Concurrent Streams:**
```rust
#[tokio::test]
async fn test_multiple_concurrent_streams() {
    let mut client = ClaudeClient::new(ClaudeAgentOptions::default());
    client.connect().await.unwrap();

    client.query("test").await.unwrap();

    // Create multiple streams simultaneously
    let mut stream1 = client.receive_response();
    let mut stream2 = client.receive_response();

    // Both should receive messages independently
    tokio::select! {
        msg1 = stream1.next() => assert!(msg1.is_some()),
        msg2 = stream2.next() => assert!(msg2.is_some()),
    }
}
```

### Stress Tests

**High Throughput:**
```rust
#[tokio::test]
#[ignore]  // Run with --ignored for stress testing
async fn stress_test_high_message_rate() {
    // Simulate receiving 10,000 messages/second
    // Verify no lock contention causes timeouts
}
```

**Rapid Reconnect:**
```rust
#[tokio::test]
#[ignore]
async fn stress_test_rapid_reconnect() {
    // Connect/disconnect 1000 times rapidly
    // Verify no resource leaks, all tasks cleaned up
}
```

## Acceptance Criteria

### Functional Criteria

- [ ] All existing tests pass without modification
- [ ] Can create multiple message streams from single client without contention
- [ ] Disconnect completes in <10ms in typical case (vs 100ms+ currently)
- [ ] Disconnect can be cancelled mid-flight
- [ ] Background tasks cleanly shut down on disconnect
- [ ] Errors from background tasks are propagated to client
- [ ] No nested lock acquisition patterns remain in codebase
- [ ] Lock hold times reduced by 50% for write operations

### Non-Functional Criteria

- [ ] No performance regression in single-threaded usage
- [ ] 80% reduction in lock contention under concurrent load
- [ ] Zero deadlocks in stress testing
- [ ] 100% graceful shutdown success in tests
- [ ] All public APIs remain backward compatible
- [ ] Documentation covers all new concurrency patterns
- [ ] Examples demonstrate new capabilities

### Code Quality Criteria

- [ ] All new code has >80% test coverage
- [ ] Clippy passes with zero warnings
- [ ] rustfmt applied to all changed files
- [ ] Lock ordering documented at all lock sites
- [ ] Task spawning uses TaskManager exclusively
- [ ] No `tokio::spawn` calls outside TaskManager
- [ ] All background tasks support cancellation

## Performance Considerations

### Expected Improvements

**Lock Contention:**
- Current: Every message locks Arc<Mutex<Receiver>>
- After: Zero lock contention for message receiving
- Impact: 80-90% reduction in lock overhead

**Disconnect Latency:**
- Current: Minimum 100ms (hardcoded sleep)
- After: <10ms typical, <100ms with timeout
- Impact: 10x improvement in shutdown time

**Throughput:**
- Current: Locks serialize message processing
- After: Multiple consumers can process independently
- Impact: Scales with number of CPU cores

### Memory Impact

**Broadcast Channel:**
- Buffer size: 100 messages (configurable)
- Per subscriber: ~8KB overhead
- Impact: Minimal for typical usage (2-3 subscribers)

**TaskManager:**
- Per task: JoinHandle + CancellationToken (~100 bytes)
- Impact: Negligible (typically <10 tasks)

### CPU Impact

**Message Distribution:**
- New task distributes messages to broadcast channel
- Cost: One additional message parse + broadcast send per message
- Impact: <1% overhead (broadcasting is fast)

**Cancellation Checking:**
- tokio::select! adds ~100ns per iteration
- Impact: Negligible compared to I/O operations

## Migration Path

### For Library Users

**No action required.** All changes are internal and backward compatible.

**Optional:** Update code to use new patterns:
```rust
// Old (still works):
client.disconnect().await?;

// New (same behavior, but faster):
client.disconnect().await?;  // Just faster now!
```

### For Contributors

**Must update internal code to use:**
1. TaskManager for all task spawning
2. Broadcast channel for message distribution
3. Sequential lock acquisition (no nesting)
4. Batching before lock acquisition for writes

**Guidelines documented in:**
- Module-level comments
- `docs/CONCURRENCY.md`
- Code review checklist

## Risks and Mitigations

### Risk 1: Broadcast Channel Buffer Overflow

**Risk:** Slow consumers may miss messages if broadcast buffer fills up

**Likelihood:** Low - Buffer size 100, typical usage has <10 pending messages

**Impact:** Medium - Missed messages break protocol

**Mitigation:**
- Configure buffer size conservatively (100 messages)
- Document lagging behavior in API docs
- Add warning logs when lagging occurs
- Tests verify behavior under various buffer sizes

### Risk 2: Performance Regression

**Risk:** New architecture adds overhead in single-threaded case

**Likelihood:** Low - Broadcast is highly optimized

**Impact:** High - Would violate NFR-1

**Mitigation:**
- Benchmark before/after in single-threaded scenario
- If regression >5%, optimize or reconsider approach
- Profile to identify bottlenecks

### Risk 3: Subtle Deadlock Bugs

**Risk:** Refactoring lock acquisition may introduce new deadlocks

**Likelihood:** Medium - Lock refactoring is error-prone

**Impact:** High - Deadlocks are critical bugs

**Mitigation:**
- Document lock ordering clearly
- Add timeout-based deadlock tests
- Code review focuses on lock acquisition order
- Use lock ordering lints (future: type system enforcement)

### Risk 4: Task Cancellation Incompleteness

**Risk:** Some background tasks may not respect cancellation

**Likelihood:** Medium - Easy to forget cancellation in some paths

**Impact:** Medium - Tasks don't shut down cleanly

**Mitigation:**
- Review all spawned tasks for cancellation support
- Add tests that verify cancellation
- Use tokio-console to monitor task completion

### Risk 5: Breaking Internal Consumers

**Risk:** Internal API changes break other parts of codebase

**Likelihood:** Low - Small codebase, limited internal coupling

**Impact:** Medium - Would require additional fixes

**Mitigation:**
- Run full test suite after each phase
- Use compiler to find all call sites
- Update incrementally, not all at once

## Dependencies

### New Dependencies

```toml
[dependencies]
tokio-util = { version = "0.7", features = ["sync"] }  # CancellationToken
```

**Justification:**
- CancellationToken is standard pattern in tokio ecosystem
- Well-maintained, widely used
- No alternative in tokio core

**License:** MIT (compatible with project)

### Updated Dependencies

None required. All other needed primitives (broadcast, mpsc) already in tokio.

## Alternatives Considered

### Alternative 1: Use async-channel Instead of broadcast

**Pros:**
- Multi-producer, multi-consumer by design
- May be slightly faster in some scenarios

**Cons:**
- Additional dependency
- tokio::sync::broadcast is standard in tokio ecosystem
- Less familiar to tokio users

**Decision:** Use tokio::sync::broadcast (standard, no new deps)

### Alternative 2: Use flume Instead of tokio Channels

**Pros:**
- Faster than tokio channels in some benchmarks
- Multi-producer, multi-consumer

**Cons:**
- Additional dependency
- Not integrated with tokio runtime
- Would mix channel implementations (confusing)

**Decision:** Stick with tokio channels (consistency)

### Alternative 3: Keep Arc<Mutex<Receiver>> with Better Docs

**Pros:**
- No code changes needed
- Simple to understand

**Cons:**
- Fundamentally wrong pattern
- Poor performance
- Misleading to users

**Decision:** Rejected - architectural improvement needed

### Alternative 4: Use RwLock for Read-Heavy Data

**Pros:**
- Better performance for read-heavy workloads (hooks, MCP servers)
- Allows concurrent reads

**Cons:**
- More complex than Mutex
- Writes are slower
- Need to analyze read/write patterns carefully

**Decision:** Future optimization - not in this spec. Profile first.

### Alternative 5: Lock-Free Data Structures

**Pros:**
- Maximum performance
- No deadlock risk

**Cons:**
- Very complex to implement correctly
- Hard to debug
- Overkill for this use case

**Decision:** Rejected - not worth complexity

## Future Enhancements

**Not in scope for this spec, but potential future work:**

1. **Type-System Lock Ordering Enforcement**
   - Use phantom types to enforce lock order at compile time
   - Prevents lock ordering bugs completely
   - Complex to implement, high value

2. **RwLock for Read-Heavy Data**
   - Profile hook_callbacks and sdk_mcp_servers access patterns
   - If read-dominated, replace Mutex with RwLock
   - Measure before/after performance

3. **Lock-Free Message Queue**
   - Replace mpsc with lock-free queue (e.g., crossbeam)
   - May improve throughput under high load
   - Need benchmarks to justify

4. **Task Health Monitoring**
   - Add metrics for task health
   - Automatic restart on panic
   - Alerting for stuck tasks

5. **Adaptive Broadcast Buffer**
   - Dynamically adjust buffer size based on load
   - Prevents lagging under burst load
   - Reduces memory in steady state

6. **Dedicated Thread Pool for Blocking Operations**
   - If any blocking operations added in future
   - Use dedicated thread pool via spawn_blocking
   - Prevents blocking tokio runtime

## References

### Internal Documentation

- `src/client.rs` - ClaudeClient implementation
- `src/internal/query_full.rs` - Core protocol handler
- `src/internal/transport/subprocess.rs` - Transport layer
- `examples/06_bidirectional_client.rs` - Example usage

### External Resources

- [Tokio Broadcast Channel](https://docs.rs/tokio/latest/tokio/sync/broadcast/index.html)
- [Tokio CancellationToken](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html)
- [Async Rust: Cooperative Scheduling](https://rust-lang.github.io/async-book/08_ecosystem/00_chapter.html)
- [Rust API Guidelines: Concurrency](https://rust-lang.github.io/api-guidelines/predictability.html)

### Related Specifications

- N/A - First concurrency-focused spec

## Appendix A: Lock Contention Benchmarks (Proposed)

To validate improvements, add benchmarks:

```rust
// benchmarks/lock_contention.rs (to be created)

#[tokio::main]
async fn bench_message_reception() {
    // Measure time to receive 10,000 messages
    // Compare Arc<Mutex<Receiver>> vs broadcast
}

#[tokio::main]
async fn bench_concurrent_streams() {
    // Measure throughput with 1, 2, 4, 8 concurrent streams
    // Should scale linearly with broadcast, not with Mutex
}

#[tokio::main]
async fn bench_disconnect() {
    // Measure time from disconnect() call to completion
    // Should be <10ms with proper shutdown
}
```

## Appendix B: Deadlock Detection Test Template

```rust
#[tokio::test]
async fn test_deadlock_detection_template() {
    // Pattern for testing potential deadlocks

    let result = tokio::time::timeout(
        Duration::from_secs(2),  // Should complete in <2s
        async {
            // Code path that might deadlock
        }
    ).await;

    match result {
        Ok(_) => { /* Success */ }
        Err(_) => panic!("Deadlock detected: operation timed out"),
    }
}
```

## Appendix C: Task Naming Convention

**Convention for task names:**
- `message_reader` - Reads from transport
- `message_distributor` - Distributes to broadcast channel
- `control_handler_{id}` - Handles specific control request
- `stderr_monitor` - Monitors stderr output
- `writer` - Serializes writes to stdin

**Benefits:**
- Clear purpose from name
- Easy to debug with logging
- Consistent across codebase

---

## Version History

- **v1.0.0** (2025-01-XX) - Initial specification
