# Architecture Overview

This document describes the internal architecture of the Claude Agent SDK for Rust.

## Table of Contents

- [Component Diagram](#component-diagram)
- [Layer Architecture](#layer-architecture)
- [Data Flow](#data-flow)
- [Threading Model](#threading-model)
- [Key Design Decisions](#key-design-decisions)
- [Module Organization](#module-organization)

---

## Component Diagram

```
┌──────────────────────────────────────────────────────┐
│            User Application                          │
│  (Examples, tests, user code)                        │
└─────────────────┬────────────────────────────────────┘
                  │
         ┌────────▼───────────┐
         │   Public API       │
         │ ─────────────────  │
         │ • query()          │   One-shot queries
         │ • ClaudeClient     │   Bidirectional client
         │ • tool!()          │   Tool definition macro
         │ • create_sdk_*     │   MCP server helpers
         └────────┬───────────┘
                  │
    ┌─────────────▼──────────────────┐
    │   Control Protocol Layer       │
    │  (QueryFull)                   │
    │ ───────────────────────────────│
    │ • Bidirectional control        │
    │ • Hook management              │
    │ • MCP routing                  │
    │ • Message parsing              │
    │ • Control request/response     │
    └─────────────┬──────────────────┘
                  │
         ┌────────▼─────────┐
         │  Transport Layer │
         │  (Subprocess)    │
         │ ────────────────  │
         │ • Process spawn  │
         │ • Stdin/stdout   │
         │ • Message framing│
         └────────┬─────────┘
                  │
         ┌────────▼─────────┐
         │  Claude Code CLI │
         │  (External)      │
         └──────────────────┘
```

---

## Layer Architecture

The SDK follows a strict layered architecture with clear separation of concerns:

### Layer 1: Public API (`src/lib.rs`, `src/client.rs`, `src/query.rs`)

**Responsibility:** User-facing API surface

**Key Types:**
- `ClaudeClient` - Stateful bidirectional client
- `query()` - Stateless one-shot function
- `ClaudeAgentOptions` - Configuration

**Characteristics:**
- Ergonomic, idiomatic Rust API
- Async/await throughout
- Builder pattern for configuration
- Stream-based responses
- Rich documentation and examples

### Layer 2: Control Protocol (`src/internal/query_full.rs`)

**Responsibility:** Bidirectional control protocol implementation

**Key Types:**
- `QueryFull` - Core protocol handler
- `ControlRequest/ControlResponse` - Protocol messages

**Key Functions:**
- Hook callback management
- MCP message routing
- Control request/response handling
- Session management
- Background message reading

**Characteristics:**
- Lock-free architecture to prevent deadlocks
- Direct stdin access bypasses transport lock
- Atomic counters for request IDs
- Timeout handling for control requests

### Layer 3: Transport (`src/internal/transport/`)

**Responsibility:** Communication with Claude CLI process

**Key Types:**
- `SubprocessTransport` - Subprocess management
- `Transport` trait - Abstraction for future transports

**Key Functions:**
- Process spawning and lifecycle
- Stdin/stdout/stderr handling
- JSON message framing (newline-delimited)
- Buffer management
- Version checking

**Characteristics:**
- Async streams for stdout
- Buffered reading with configurable limits
- Clean shutdown handling
- Drop guard for process cleanup

### Layer 4: External (Claude Code CLI)

**Responsibility:** Actual AI interactions

The Claude Code CLI is an external process that:
- Communicates with Anthropic API
- Manages conversation state
- Handles tool execution
- Implements permission system

---

## Data Flow

### Query Lifecycle (Bidirectional Client)

```
User Code
  │
  ├─> ClaudeClient::connect()
  │     │
  │     ├─> SubprocessTransport::new()
  │     │     └─> Spawn claude CLI process
  │     │
  │     ├─> QueryFull::new()
  │     │
  │     ├─> QueryFull::start()
  │     │     └─> Background task: read messages from stdout
  │     │
  │     └─> QueryFull::initialize()
  │           └─> Send "initialize" control request
  │                 └─> Wait for response (with timeout)
  │
  ├─> ClaudeClient::query("prompt")
  │     │
  │     └─> Write JSON message to stdin
  │           {"type":"user","message":{"role":"user","content":"prompt"}}
  │
  ├─> ClaudeClient::receive_response()
  │     │
  │     └─> Return stream of messages
  │           │
  │           ├─> Message::Assistant (text, thinking, tool_use)
  │           ├─> Message::System (confirmation requests)
  │           └─> Message::Result (final metadata)
  │
  └─> ClaudeClient::disconnect()
        │
        └─> Close stdin, shutdown subprocess
```

### Control Request Flow

```
QueryFull::send_control_request()
  │
  ├─> Generate unique request_id
  │
  ├─> Create oneshot channel for response
  │
  ├─> Store channel in pending_responses map
  │
  ├─> Write control request to stdin
  │     {"type":"control_request","request_id":"...","request":{...}}
  │
  ├─> Wait on oneshot channel (with timeout)
  │
  └─> Background reader receives control_response
        │
        └─> Lookup pending_responses by request_id
              │
              └─> Send response through oneshot channel
```

### Hook Execution Flow

```
Claude CLI
  │
  └─> Sends control_request: hook_callback
        {"subtype":"hook_callback","callback_id":"...","input":{...}}
        │
        └─> QueryFull::handle_control_request()
              │
              ├─> Lookup hook by callback_id
              │
              ├─> Parse HookInput
              │
              ├─> Call user hook function (async)
              │
              ├─> Serialize HookOutput to JSON
              │
              └─> Send control_response back to CLI
                    {"type":"control_response","request_id":"...","response":{...}}
```

### MCP Message Flow

```
Claude CLI
  │
  └─> Wants to call SDK MCP tool
        │
        └─> Sends control_request: mcp_message
              {"subtype":"mcp_message","server_name":"...","message":{...}}
              │
              └─> QueryFull::handle_sdk_mcp_request()
                    │
                    ├─> Lookup SDK MCP server by name
                    │
                    ├─> Call server.handle_message()
                    │     │
                    │     ├─> "initialize" → return server info
                    │     ├─> "tools/list" → return tool list
                    │     └─> "tools/call" → execute tool handler
                    │
                    └─> Send MCP response back to CLI
```

---

## Threading Model

### Async Runtime

- **Tokio** is the required async runtime
- All I/O operations are async
- Single-threaded executor is sufficient (but multi-threaded works too)

### Concurrency Primitives

**Arc (Atomic Reference Counting):**
- Used for shared ownership across async tasks
- Examples: `Arc<Mutex<QueryFull>>`, `Arc<Mutex<HashMap<...>>>`

**Mutex (from tokio::sync):**
- Async-friendly mutex for shared mutable state
- Never held across `.await` points when possible
- Examples: stdin access, pending_responses map

**AtomicU64:**
- Lock-free counters
- Used for: request IDs, callback IDs
- No contention, no deadlocks

**Channels:**
- `mpsc::unbounded_channel` - Message passing from background reader
- `oneshot::channel` - Control request/response pairing

### Lock-Free Architecture

**Problem:** Deadlock risk when:
1. Transport holds lock on process
2. Background task reads from stdout (needs transport lock)
3. Foreground task tries to write to stdin (needs transport lock)

**Solution:**
1. Extract `stdin` as separate `Arc<Mutex<Option<ChildStdin>>>`
2. Background reader only reads stdout (no lock needed)
3. Foreground writer directly accesses stdin (separate lock)
4. No shared lock between reader and writer = no deadlock

**Code Example:**
```rust
pub struct QueryFull {
    transport: Arc<Mutex<Box<dyn Transport>>>,  // Background reader uses this
    stdin: Option<Arc<Mutex<Option<ChildStdin>>>>,  // Writers use this (separate!)
    // ...
}
```

### Background Tasks

**Message Reader Task:**
- Spawned in `QueryFull::start()`
- Continuously reads from stdout
- Routes messages:
  - Control responses → pending_responses map
  - Regular messages → message channel
  - Control requests → handle_control_request()
- Runs until stdout closes

---

## Key Design Decisions

### 1. Lock-Free stdin/stdout Access

**Rationale:** Prevents deadlocks in bidirectional communication

**Trade-off:** Slightly more complex state management, but eliminates entire class of bugs

### 2. Separate QueryFull and ClaudeClient

**Rationale:**
- `QueryFull` - Internal implementation, complex, handles protocol
- `ClaudeClient` - Public API, simple, ergonomic

**Benefit:** Can change internal implementation without breaking user code

### 3. Stream-Based Responses

**Rationale:** Matches async nature of Claude responses, enables real-time processing

**Benefit:** Users see partial results immediately, not after full response

### 4. Builder Pattern for Configuration

**Rationale:** Many optional configuration fields, want good defaults

**Benefit:** User code is concise, SDK is flexible

### 5. Subprocess Transport (not HTTP)

**Rationale:** Claude CLI handles authentication, sessions, complex tool execution

**Benefit:** SDK doesn't need to reimplement Claude Code logic

### 6. No Unsafe Code

**Rationale:** Memory safety is critical, SDK is not performance-bottleneck

**Benefit:** Guaranteed memory safety, easier to maintain

### 7. Timeout for Control Requests

**Rationale:** Prevent indefinite hangs if CLI becomes unresponsive

**Benefit:** Fail-fast with clear error message, configurable per-app needs

---

## Module Organization

```
src/
├── lib.rs                      # Public API re-exports
├── client.rs                   # ClaudeClient (bidirectional)
├── query.rs                    # query() function (one-shot)
├── errors.rs                   # Error types
├── version.rs                  # Version checking
├── internal/                   # Internal implementation
│   ├── query_full.rs           # Core protocol handler
│   ├── message_parser.rs       # JSON message parsing
│   └── transport/              # Transport layer
│       ├── mod.rs
│       ├── subprocess.rs       # Subprocess transport
│       └── traits.rs           # Transport trait
└── types/                      # Type definitions
    ├── mod.rs
    ├── config.rs               # Configuration types
    ├── messages.rs             # Message types
    ├── hooks.rs                # Hook system
    ├── permissions.rs          # Permissions
    └── mcp.rs                  # MCP server types
```

**Visibility Rules:**
- `lib.rs` re-exports public types
- `internal/` is private (not re-exported)
- `types/` is partially public (selected types re-exported)

**Import Patterns:**
- External crates grouped at top
- Blank line
- Crate-local imports using `crate::`
- Prefer specific imports over glob imports

---

## Performance Characteristics

### Latency

- **Subprocess spawn:** ~50-100ms (one-time cost at connect)
- **Query send:** <1ms (JSON serialize + write)
- **Control request:** ~10-50ms (round-trip with CLI)
- **Message receive:** <1ms per message (JSON parse)

### Memory

- **Base overhead:** ~10MB (mostly subprocess)
- **Per-query:** ~10KB (message buffers)
- **Streaming:** Constant memory (messages processed one at a time)

### Throughput

- **Concurrent queries:** Limited by Claude API rate limits, not SDK
- **Message rate:** 1000+ messages/sec (parsing)
- **Subprocess IPC:** 10-100 MB/sec (stdin/stdout)

### Bottlenecks

1. **Claude API latency** (external, not SDK)
2. **Subprocess spawn** (mitigated by keeping connection open)
3. **JSON parsing** (minimal, using serde)

**Not bottlenecks:**
- Lock contention (lock-free design)
- Memory allocation (streaming architecture)
- CPU (async I/O, not compute-bound)

---

## Future Extensibility

### Planned Enhancements

1. **Alternative Transports**
   - HTTP transport (direct API calls)
   - WebSocket transport (future CLI feature)
   - Mock transport (for testing)

2. **Connection Pooling**
   - Reuse CLI processes across queries
   - Configurable pool size
   - Health checking

3. **Metrics & Observability**
   - Built-in metrics collection
   - Integration with Prometheus, etc.
   - Performance profiling

4. **Retry Logic**
   - Automatic retry for transient failures
   - Exponential backoff
   - Configurable retry policy

### Design for Extension

**Transport abstraction:**
```rust
#[async_trait]
pub trait Transport: Send {
    async fn connect(&mut self) -> Result<()>;
    async fn write(&mut self, data: &str) -> Result<()>;
    fn read_messages(&mut self) -> MessageStream;
    async fn close(&mut self) -> Result<()>;
}
```

This allows adding new transports without changing protocol layer.

---

## Debugging & Troubleshooting

See [TROUBLESHOOTING.md](TROUBLESHOOTING.md) for:
- Common issues and solutions
- Debugging techniques
- Performance optimization
- Error message reference

---

## Contributing

When modifying the architecture:

1. **Maintain layering** - Don't let lower layers depend on higher layers
2. **Preserve lock-free properties** - Avoid holding locks across await
3. **Keep public API stable** - Changes to internal/ are OK, changes to public API need major version
4. **Document design decisions** - Update this doc when making significant changes
5. **Add tests** - Integration tests for protocol, unit tests for utilities
6. **Follow conventions** - See repository WORKFLOW.md for coding standards

---

## References

- [Python SDK](https://github.com/anthropics/claude-agent-sdk-python) - Reference implementation
- [Claude Code Documentation](https://docs.claude.com/claude-code) - CLI behavior
- [Tokio Documentation](https://docs.rs/tokio) - Async runtime
- [API Guidelines](https://rust-lang.github.io/api-guidelines/) - Rust API design
