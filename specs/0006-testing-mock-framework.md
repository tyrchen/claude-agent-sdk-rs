# Testing Mock Framework Specification

**Version**: 1.0.0
**Status**: Proposed
**Target**: v0.7.0
**Feature Flag**: `testing`

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Problem Statement](#2-problem-statement)
3. [Design Goals](#3-design-goals)
4. [Architecture Overview](#4-architecture-overview)
5. [Component Specifications](#5-component-specifications)
6. [API Reference](#6-api-reference)
7. [Snapshot Testing](#7-snapshot-testing)
8. [Timing Simulation](#8-timing-simulation)
9. [Implementation Plan](#9-implementation-plan)
10. [Migration & Compatibility](#10-migration--compatibility)
11. [Verification Checklist](#11-verification-checklist)

---

## 1. Executive Summary

This specification defines a comprehensive testing mock framework for the Claude Agent SDK, enabling developers to write deterministic, fast, and reliable tests without requiring the Claude Code CLI or network access.

### Key Features

| Feature | Description |
|---------|-------------|
| `MockTransport` | Drop-in replacement for subprocess transport |
| Message Builders | Ergonomic builders for all message types |
| Scenario System | Linear message sequences with timing simulation |
| Snapshot Testing | Record real sessions, replay in tests |
| Hook/Permission Testing | Verify callbacks are invoked correctly |
| Deterministic Timing | Reproducible delays with seeded random jitters |

### Why This Matters

- **CI/CD**: Tests run without external dependencies
- **Speed**: No subprocess overhead, configurable timing
- **Reliability**: Deterministic behavior eliminates flaky tests
- **Coverage**: Test edge cases impossible to trigger with real CLI

---

## 2. Problem Statement

### Current State

The SDK's integration tests are marked `#[ignore]` because they require the Claude Code CLI:

```rust
#[tokio::test]
#[ignore] // Requires Claude CLI
async fn test_basic_client_connection() -> anyhow::Result<()> {
    let options = ClaudeAgentOptions { ... };
    let mut client = ClaudeClient::new(options);
    client.connect().await?;
    // ...
}
```

### Challenges

1. **External Dependency**: Tests fail without Claude CLI installed
2. **Non-Deterministic**: Real API responses vary
3. **Slow**: Each test spawns subprocess, waits for responses
4. **Cost**: Real API calls incur usage costs
5. **Edge Cases**: Cannot test error conditions, timeouts, malformed responses

### Solution

Provide a mock transport layer and supporting utilities that:
- Implement the `Transport` trait with controlled behavior
- Supply pre-defined message sequences
- Enable deterministic timing with configurable jitter
- Support recording/replaying real sessions

---

## 3. Design Goals

### 3.1 Primary Goals

1. **Zero External Dependencies**: Tests run offline, no CLI required
2. **Deterministic**: Same seed produces identical test runs
3. **Ergonomic API**: Easy to construct test scenarios
4. **Comprehensive**: Support all SDK features (hooks, MCP, permissions)
5. **Minimal Overhead**: Fast test execution

### 3.2 Non-Goals

- Mocking the Anthropic HTTP API directly (SDK uses CLI)
- Full CLI behavior emulation (we mock at transport layer)
- Property-based/fuzzing testing (may be added later)

### 3.3 Design Principles

1. **Simple Scenarios**: Linear message sequences only (no conditional branching)
2. **Composable**: Builders can be combined and reused
3. **Type-Safe**: Compile-time guarantees where possible
4. **Observable**: Easy to inspect what was sent/received

---

## 4. Architecture Overview

### 4.1 Layer Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                      User Test Code                         │
├─────────────────────────────────────────────────────────────┤
│  MockClient    │  ScenarioBuilder  │  Message Builders      │
├─────────────────────────────────────────────────────────────┤
│  HookRecorder  │  PermissionRecorder  │  SnapshotRecorder   │
├─────────────────────────────────────────────────────────────┤
│                      MockTransport                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │ Message Queue │  │ Write Capture │  │ Timing Simulator │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                   Transport Trait (existing)                │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 Module Structure

```
src/
├── lib.rs                    # Add: #[cfg(feature = "testing")] mod testing;
├── testing/                  # New module (feature-gated)
│   ├── mod.rs               # Public re-exports
│   ├── transport.rs         # MockTransport implementation
│   ├── client.rs            # MockClient wrapper
│   ├── scenario.rs          # Scenario and ScenarioBuilder
│   ├── timing.rs            # TimingSimulator with jitter
│   ├── snapshot.rs          # Recording and playback
│   ├── recorders/
│   │   ├── mod.rs
│   │   ├── hooks.rs         # HookRecorder
│   │   └── permissions.rs   # PermissionRecorder
│   └── builders/
│       ├── mod.rs
│       ├── assistant.rs     # AssistantMessageBuilder
│       ├── system.rs        # SystemMessageBuilder
│       ├── result.rs        # ResultMessageBuilder
│       ├── tool.rs          # ToolUseBuilder, ToolResultBuilder
│       └── stream_event.rs  # StreamEventBuilder
```

### 4.3 Dependency on Existing Code

The mock framework depends on:

| Existing Component | Usage |
|--------------------|-------|
| `Transport` trait | MockTransport implements this |
| `Message` enum | Builders produce these types |
| `ClaudeAgentOptions` | Configuration for MockClient |
| `HookEvent`, `HookMatcher` | Hook testing utilities |
| `CanUseToolCallback` | Permission testing utilities |

### 4.4 Required Modifications to Existing Code

To enable transport injection, `ClaudeClient` needs a new constructor:

```rust
// src/client.rs - New method
impl ClaudeClient {
    /// Create a client with a custom transport (for testing)
    #[cfg(feature = "testing")]
    pub fn with_transport(
        transport: Arc<dyn Transport>,
        options: ClaudeAgentOptions,
    ) -> Self {
        // Initialize with provided transport instead of SubprocessTransport
    }
}
```

---

## 5. Component Specifications

### 5.1 MockTransport

The core mock implementation of the `Transport` trait.

#### Definition

```rust
/// A mock transport for testing that simulates CLI communication
pub struct MockTransport {
    /// Pre-loaded messages to emit
    messages: Arc<Mutex<VecDeque<ScheduledMessage>>>,
    /// Dynamic message injection channel
    injector_tx: flume::Sender<serde_json::Value>,
    injector_rx: flume::Receiver<serde_json::Value>,
    /// Captured writes for assertions
    written: Arc<Mutex<Vec<WrittenMessage>>>,
    /// Connection state
    connected: AtomicBool,
    ready: AtomicBool,
    /// Timing configuration
    timing: TimingConfig,
    /// Random seed for deterministic jitter
    rng: Mutex<StdRng>,
}

/// A message scheduled for delivery
pub struct ScheduledMessage {
    pub value: serde_json::Value,
    pub timing: MessageTiming,
}

/// Timing configuration for a message
pub enum MessageTiming {
    /// Deliver immediately
    Immediate,
    /// Deliver after base delay + jitter
    Delayed { base_ms: u64, jitter_ms: u64 },
    /// Wait for a write matching pattern before delivering
    AfterWrite { pattern: String },
}

/// A captured write operation
pub struct WrittenMessage {
    pub data: String,
    pub parsed: Option<serde_json::Value>,
    pub timestamp: std::time::Instant,
}

/// Global timing configuration
pub struct TimingConfig {
    /// Seed for deterministic random jitter
    pub seed: u64,
    /// Global delay multiplier (0.0 = instant, 1.0 = normal, 2.0 = double)
    pub speed_factor: f64,
    /// Whether to use tokio::time::pause() for deterministic async timing
    pub use_paused_time: bool,
}
```

#### Implementation

```rust
#[async_trait]
impl Transport for MockTransport {
    async fn connect(&self) -> Result<()> {
        if self.connected.swap(true, Ordering::SeqCst) {
            return Err(ClaudeError::Connection(ConnectionError {
                message: "Already connected".to_string(),
                source: None,
            }));
        }
        self.ready.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn write(&self, data: &str) -> Result<()> {
        if !self.connected.load(Ordering::SeqCst) {
            return Err(ClaudeError::Transport("Not connected".to_string()));
        }

        let parsed = serde_json::from_str(data).ok();
        self.written.lock().await.push(WrittenMessage {
            data: data.to_string(),
            parsed,
            timestamp: std::time::Instant::now(),
        });

        // Check if any messages are waiting for this write
        self.check_write_triggers(data).await;

        Ok(())
    }

    fn read_messages(&self) -> Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send + '_>> {
        let messages = Arc::clone(&self.messages);
        let injector_rx = self.injector_rx.clone();
        let timing = self.timing.clone();
        let rng = Arc::new(Mutex::new(self.rng.lock().unwrap().clone()));

        Box::pin(async_stream::stream! {
            loop {
                // Check for injected messages first (non-blocking)
                if let Ok(msg) = injector_rx.try_recv() {
                    yield Ok(msg);
                    continue;
                }

                // Check pre-loaded messages
                let next = {
                    let mut guard = messages.lock().await;
                    guard.pop_front()
                };

                match next {
                    Some(scheduled) => {
                        // Apply timing
                        match scheduled.timing {
                            MessageTiming::Immediate => {},
                            MessageTiming::Delayed { base_ms, jitter_ms } => {
                                let jitter = {
                                    let mut rng = rng.lock().await;
                                    rng.gen_range(0..=jitter_ms)
                                };
                                let delay = Duration::from_millis(
                                    ((base_ms + jitter) as f64 * timing.speed_factor) as u64
                                );
                                tokio::time::sleep(delay).await;
                            },
                            MessageTiming::AfterWrite { .. } => {
                                // Already triggered, deliver now
                            },
                        }
                        yield Ok(scheduled.value);
                    }
                    None => {
                        // No more pre-loaded messages, wait for injection
                        match injector_rx.recv_async().await {
                            Ok(msg) => yield Ok(msg),
                            Err(_) => break, // Channel closed
                        }
                    }
                }
            }
        })
    }

    async fn close(&self) -> Result<()> {
        self.connected.store(false, Ordering::SeqCst);
        self.ready.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn is_ready(&self) -> bool {
        self.ready.load(Ordering::SeqCst)
    }

    async fn end_input(&self) -> Result<()> {
        Ok(())
    }
}
```

#### Builder Pattern

```rust
impl MockTransport {
    pub fn builder() -> MockTransportBuilder {
        MockTransportBuilder::default()
    }
}

#[derive(Default)]
pub struct MockTransportBuilder {
    messages: Vec<ScheduledMessage>,
    timing: Option<TimingConfig>,
}

impl MockTransportBuilder {
    /// Add a message to emit
    pub fn message(mut self, msg: impl Into<serde_json::Value>) -> Self {
        self.messages.push(ScheduledMessage {
            value: msg.into(),
            timing: MessageTiming::Immediate,
        });
        self
    }

    /// Add a message with delay
    pub fn message_delayed(
        mut self,
        msg: impl Into<serde_json::Value>,
        base_ms: u64,
        jitter_ms: u64,
    ) -> Self {
        self.messages.push(ScheduledMessage {
            value: msg.into(),
            timing: MessageTiming::Delayed { base_ms, jitter_ms },
        });
        self
    }

    /// Configure timing
    pub fn timing(mut self, config: TimingConfig) -> Self {
        self.timing = Some(config);
        self
    }

    /// Set random seed for deterministic jitter
    pub fn seed(mut self, seed: u64) -> Self {
        self.timing.get_or_insert_with(TimingConfig::default).seed = seed;
        self
    }

    /// Build the transport
    pub fn build(self) -> MockTransport {
        MockTransport::new(self.messages, self.timing.unwrap_or_default())
    }
}
```

---

### 5.2 Message Builders

Ergonomic builders for constructing test messages.

#### AssistantMessageBuilder

```rust
/// Builder for AssistantMessage
pub struct AssistantMessageBuilder {
    content: Vec<ContentBlock>,
    partial: bool,
}

impl AssistantMessageBuilder {
    pub fn new() -> Self {
        Self {
            content: Vec::new(),
            partial: false,
        }
    }

    /// Add a text block
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.content.push(ContentBlock::Text(TextBlock {
            text: text.into(),
        }));
        self
    }

    /// Add a tool use block
    pub fn tool_use(
        mut self,
        name: impl Into<String>,
        input: serde_json::Value,
    ) -> Self {
        self.content.push(ContentBlock::ToolUse(ToolUseBlock {
            id: format!("tool_{}", uuid::Uuid::new_v4()),
            name: name.into(),
            input,
        }));
        self
    }

    /// Add a tool use block with specific ID
    pub fn tool_use_with_id(
        mut self,
        id: impl Into<String>,
        name: impl Into<String>,
        input: serde_json::Value,
    ) -> Self {
        self.content.push(ContentBlock::ToolUse(ToolUseBlock {
            id: id.into(),
            name: name.into(),
            input,
        }));
        self
    }

    /// Add a thinking block (extended thinking)
    pub fn thinking(mut self, thinking: impl Into<String>) -> Self {
        self.content.push(ContentBlock::Thinking(ThinkingBlock {
            thinking: thinking.into(),
            signature: None,
        }));
        self
    }

    /// Mark as partial message (streaming)
    pub fn partial(mut self) -> Self {
        self.partial = true;
        self
    }

    /// Build the message
    pub fn build(self) -> Message {
        Message::Assistant(AssistantMessage {
            content: self.content,
            partial: self.partial,
        })
    }

    /// Build as JSON value
    pub fn build_json(self) -> serde_json::Value {
        serde_json::to_value(self.build()).expect("Message serialization should not fail")
    }
}
```

#### SystemMessageBuilder

```rust
/// Builder for SystemMessage (session initialization)
pub struct SystemMessageBuilder {
    session_id: String,
    model: String,
    tools: Vec<String>,
    mcp_servers: Vec<String>,
    cwd: Option<PathBuf>,
}

impl SystemMessageBuilder {
    pub fn new() -> Self {
        Self {
            session_id: format!("test-session-{}", uuid::Uuid::new_v4()),
            model: "claude-sonnet-4-20250514".to_string(),
            tools: vec!["Read".into(), "Write".into(), "Bash".into()],
            mcp_servers: Vec::new(),
            cwd: None,
        }
    }

    pub fn session_id(mut self, id: impl Into<String>) -> Self {
        self.session_id = id.into();
        self
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn tools(mut self, tools: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tools = tools.into_iter().map(Into::into).collect();
        self
    }

    pub fn mcp_servers(mut self, servers: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.mcp_servers = servers.into_iter().map(Into::into).collect();
        self
    }

    pub fn cwd(mut self, path: impl Into<PathBuf>) -> Self {
        self.cwd = Some(path.into());
        self
    }

    pub fn build(self) -> Message {
        Message::System(SystemMessage {
            session_id: self.session_id,
            model: self.model,
            tools: self.tools,
            mcp_servers: self.mcp_servers,
            cwd: self.cwd,
        })
    }

    pub fn build_json(self) -> serde_json::Value {
        serde_json::to_value(self.build()).expect("Message serialization should not fail")
    }
}

impl Default for SystemMessageBuilder {
    fn default() -> Self {
        Self::new()
    }
}
```

#### ResultMessageBuilder

```rust
/// Builder for ResultMessage (query completion)
pub struct ResultMessageBuilder {
    cost_usd: f64,
    duration_ms: u64,
    duration_api_ms: u64,
    turns: u32,
    is_error: bool,
    session_id: String,
}

impl ResultMessageBuilder {
    pub fn new() -> Self {
        Self {
            cost_usd: 0.01,
            duration_ms: 1000,
            duration_api_ms: 800,
            turns: 1,
            is_error: false,
            session_id: format!("test-session-{}", uuid::Uuid::new_v4()),
        }
    }

    pub fn cost_usd(mut self, cost: f64) -> Self {
        self.cost_usd = cost;
        self
    }

    pub fn duration_ms(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }

    pub fn duration_api_ms(mut self, ms: u64) -> Self {
        self.duration_api_ms = ms;
        self
    }

    pub fn turns(mut self, turns: u32) -> Self {
        self.turns = turns;
        self
    }

    pub fn error(mut self) -> Self {
        self.is_error = true;
        self
    }

    pub fn session_id(mut self, id: impl Into<String>) -> Self {
        self.session_id = id.into();
        self
    }

    pub fn build(self) -> Message {
        Message::Result(ResultMessage {
            cost_usd: self.cost_usd,
            duration_ms: self.duration_ms,
            duration_api_ms: self.duration_api_ms,
            num_turns: self.turns,
            is_error: self.is_error,
            session_id: self.session_id,
        })
    }

    pub fn build_json(self) -> serde_json::Value {
        serde_json::to_value(self.build()).expect("Message serialization should not fail")
    }
}

impl Default for ResultMessageBuilder {
    fn default() -> Self {
        Self::new()
    }
}
```

#### ToolResultBuilder (for control protocol simulation)

```rust
/// Builder for tool result control messages
pub struct ToolResultBuilder {
    tool_use_id: String,
    content: serde_json::Value,
    is_error: bool,
}

impl ToolResultBuilder {
    pub fn new(tool_use_id: impl Into<String>) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content: serde_json::Value::Null,
            is_error: false,
        }
    }

    pub fn success(mut self, content: serde_json::Value) -> Self {
        self.content = content;
        self.is_error = false;
        self
    }

    pub fn error(mut self, message: impl Into<String>) -> Self {
        self.content = serde_json::json!({ "error": message.into() });
        self.is_error = true;
        self
    }

    /// Build as control_response JSON (for transport simulation)
    pub fn build_control_response(self) -> serde_json::Value {
        serde_json::json!({
            "type": "control_response",
            "control_response": {
                "tool_use_id": self.tool_use_id,
                "content": self.content,
                "is_error": self.is_error,
            }
        })
    }
}
```

---

### 5.3 Scenario System

Linear scenarios for defining test conversations.

#### Definition

```rust
/// A test scenario defining a sequence of messages
pub struct Scenario {
    /// Initial messages sent on connect (e.g., SystemMessage)
    pub on_connect: Vec<ScheduledMessage>,
    /// Conversation exchanges
    pub exchanges: Vec<Exchange>,
    /// Name for debugging
    pub name: String,
}

/// A single request-response exchange
pub struct Exchange {
    /// Messages to emit for this exchange
    pub responses: Vec<ScheduledMessage>,
    /// Optional: only trigger after write matching this pattern
    pub trigger_pattern: Option<String>,
}

/// Builder for scenarios
pub struct ScenarioBuilder {
    name: String,
    on_connect: Vec<ScheduledMessage>,
    exchanges: Vec<Exchange>,
    current_exchange: Option<ExchangeBuilder>,
    timing_defaults: TimingDefaults,
}

/// Default timing for scenario messages
pub struct TimingDefaults {
    /// Base delay between messages in an exchange
    pub inter_message_delay_ms: u64,
    /// Jitter range
    pub jitter_ms: u64,
    /// Delay before first response after query
    pub initial_response_delay_ms: u64,
}

impl Default for TimingDefaults {
    fn default() -> Self {
        Self {
            inter_message_delay_ms: 50,
            jitter_ms: 20,
            initial_response_delay_ms: 100,
        }
    }
}
```

#### Builder Implementation

```rust
impl ScenarioBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            on_connect: Vec::new(),
            exchanges: Vec::new(),
            current_exchange: None,
            timing_defaults: TimingDefaults::default(),
        }
    }

    /// Configure default timing
    pub fn timing(mut self, defaults: TimingDefaults) -> Self {
        self.timing_defaults = defaults;
        self
    }

    /// Add system message on connect
    pub fn on_connect(mut self, msg: Message) -> Self {
        self.on_connect.push(ScheduledMessage {
            value: serde_json::to_value(msg).unwrap(),
            timing: MessageTiming::Immediate,
        });
        self
    }

    /// Start a new exchange (finishes previous if any)
    pub fn exchange(mut self) -> Self {
        if let Some(builder) = self.current_exchange.take() {
            self.exchanges.push(builder.build());
        }
        self.current_exchange = Some(ExchangeBuilder::new(&self.timing_defaults));
        self
    }

    /// Set trigger pattern for current exchange
    pub fn when_write_contains(mut self, pattern: impl Into<String>) -> Self {
        if let Some(ref mut exchange) = self.current_exchange {
            exchange.trigger_pattern = Some(pattern.into());
        }
        self
    }

    /// Add response message to current exchange
    pub fn respond(mut self, msg: Message) -> Self {
        if self.current_exchange.is_none() {
            self.current_exchange = Some(ExchangeBuilder::new(&self.timing_defaults));
        }
        if let Some(ref mut exchange) = self.current_exchange {
            exchange.add_response(msg);
        }
        self
    }

    /// Add response with custom delay
    pub fn respond_delayed(mut self, msg: Message, base_ms: u64, jitter_ms: u64) -> Self {
        if self.current_exchange.is_none() {
            self.current_exchange = Some(ExchangeBuilder::new(&self.timing_defaults));
        }
        if let Some(ref mut exchange) = self.current_exchange {
            exchange.add_response_delayed(msg, base_ms, jitter_ms);
        }
        self
    }

    /// Add final result message to current exchange
    pub fn then_result(self, result: Message) -> Self {
        self.respond(result)
    }

    /// Build the scenario
    pub fn build(mut self) -> Scenario {
        if let Some(builder) = self.current_exchange.take() {
            self.exchanges.push(builder.build());
        }
        Scenario {
            name: self.name,
            on_connect: self.on_connect,
            exchanges: self.exchanges,
        }
    }
}

struct ExchangeBuilder {
    responses: Vec<ScheduledMessage>,
    trigger_pattern: Option<String>,
    defaults: TimingDefaults,
    is_first: bool,
}

impl ExchangeBuilder {
    fn new(defaults: &TimingDefaults) -> Self {
        Self {
            responses: Vec::new(),
            trigger_pattern: None,
            defaults: defaults.clone(),
            is_first: true,
        }
    }

    fn add_response(&mut self, msg: Message) {
        let timing = if self.is_first {
            self.is_first = false;
            MessageTiming::Delayed {
                base_ms: self.defaults.initial_response_delay_ms,
                jitter_ms: self.defaults.jitter_ms,
            }
        } else {
            MessageTiming::Delayed {
                base_ms: self.defaults.inter_message_delay_ms,
                jitter_ms: self.defaults.jitter_ms,
            }
        };

        self.responses.push(ScheduledMessage {
            value: serde_json::to_value(msg).unwrap(),
            timing,
        });
    }

    fn add_response_delayed(&mut self, msg: Message, base_ms: u64, jitter_ms: u64) {
        self.is_first = false;
        self.responses.push(ScheduledMessage {
            value: serde_json::to_value(msg).unwrap(),
            timing: MessageTiming::Delayed { base_ms, jitter_ms },
        });
    }

    fn build(self) -> Exchange {
        Exchange {
            responses: self.responses,
            trigger_pattern: self.trigger_pattern,
        }
    }
}
```

#### Usage Example

```rust
let scenario = ScenarioBuilder::new("simple_greeting")
    .on_connect(SystemMessageBuilder::default().build())
    .exchange()
        .respond(AssistantMessageBuilder::new()
            .text("Hello! How can I help you today?")
            .build())
        .then_result(ResultMessageBuilder::default().build())
    .exchange()
        .when_write_contains("file")
        .respond(AssistantMessageBuilder::new()
            .tool_use("Read", json!({"file_path": "/tmp/test.txt"}))
            .build())
        .respond(AssistantMessageBuilder::new()
            .text("I found the file contents.")
            .build())
        .then_result(ResultMessageBuilder::default().build())
    .build();
```

---

### 5.4 MockClient

High-level wrapper combining MockTransport with ClaudeClient.

```rust
/// A test client wrapping ClaudeClient with MockTransport
pub struct MockClient {
    /// The underlying Claude client
    client: ClaudeClient,
    /// Reference to transport for assertions
    transport: Arc<MockTransport>,
    /// Options used to create the client
    options: ClaudeAgentOptions,
}

impl MockClient {
    /// Create from a scenario
    pub fn from_scenario(scenario: Scenario) -> Self {
        Self::from_scenario_with_options(scenario, ClaudeAgentOptions::default())
    }

    /// Create from scenario with custom options
    pub fn from_scenario_with_options(
        scenario: Scenario,
        options: ClaudeAgentOptions,
    ) -> Self {
        let transport = Arc::new(MockTransport::from_scenario(scenario));
        let client = ClaudeClient::with_transport(
            Arc::clone(&transport) as Arc<dyn Transport>,
            options.clone(),
        );

        Self {
            client,
            transport,
            options,
        }
    }

    /// Create from raw transport
    pub fn from_transport(transport: MockTransport, options: ClaudeAgentOptions) -> Self {
        let transport = Arc::new(transport);
        let client = ClaudeClient::with_transport(
            Arc::clone(&transport) as Arc<dyn Transport>,
            options.clone(),
        );

        Self {
            client,
            transport,
            options,
        }
    }

    /// Access the underlying client
    pub fn client(&mut self) -> &mut ClaudeClient {
        &mut self.client
    }

    /// Access the transport for assertions
    pub fn transport(&self) -> &MockTransport {
        &self.transport
    }

    /// Get options for use with query functions
    pub fn options(&self) -> ClaudeAgentOptions {
        self.options.clone()
    }

    // === Assertions ===

    /// Assert a string was written to the transport
    pub fn assert_wrote(&self, pattern: &str) {
        let written = self.transport.written_messages();
        assert!(
            written.iter().any(|w| w.data.contains(pattern)),
            "Expected to find '{}' in written messages, but got: {:?}",
            pattern,
            written.iter().map(|w| &w.data).collect::<Vec<_>>()
        );
    }

    /// Assert with JSON matcher
    pub fn assert_wrote_json<F>(&self, matcher: F)
    where
        F: Fn(&serde_json::Value) -> bool,
    {
        let written = self.transport.written_messages();
        assert!(
            written.iter().filter_map(|w| w.parsed.as_ref()).any(&matcher),
            "No written JSON matched the predicate"
        );
    }

    /// Get all written messages
    pub fn written_messages(&self) -> Vec<WrittenMessage> {
        self.transport.written_messages()
    }

    /// Inject a message dynamically during test
    pub fn inject_message(&self, msg: Message) {
        self.transport.inject(serde_json::to_value(msg).unwrap());
    }

    /// Inject an error
    pub fn inject_error(&self, err: &str) {
        self.transport.inject(serde_json::json!({
            "type": "error",
            "error": { "message": err }
        }));
    }
}

// Deref to ClaudeClient for convenience
impl std::ops::Deref for MockClient {
    type Target = ClaudeClient;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl std::ops::DerefMut for MockClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}
```

---

### 5.5 HookRecorder

Utility for testing hook invocations.

```rust
/// Records hook invocations for assertions
pub struct HookRecorder {
    invocations: Arc<Mutex<Vec<HookInvocation>>>,
}

/// A recorded hook invocation
#[derive(Debug, Clone)]
pub struct HookInvocation {
    pub event: HookEvent,
    pub tool_name: Option<String>,
    pub input: Option<serde_json::Value>,
    pub output: Option<serde_json::Value>,
    pub timestamp: std::time::Instant,
}

impl HookRecorder {
    pub fn new() -> Self {
        Self {
            invocations: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a pre_tool_use hook callback
    pub fn pre_tool_use_callback(&self) -> HookCallback {
        let invocations = Arc::clone(&self.invocations);
        Arc::new(move |tool_name, input| {
            let invocations = Arc::clone(&invocations);
            Box::pin(async move {
                invocations.lock().await.push(HookInvocation {
                    event: HookEvent::PreToolUse,
                    tool_name: Some(tool_name),
                    input: Some(input),
                    output: None,
                    timestamp: std::time::Instant::now(),
                });
                Ok(HookResult::Proceed)
            })
        })
    }

    /// Create a post_tool_use hook callback
    pub fn post_tool_use_callback(&self) -> HookCallback {
        let invocations = Arc::clone(&self.invocations);
        Arc::new(move |tool_name, output| {
            let invocations = Arc::clone(&invocations);
            Box::pin(async move {
                invocations.lock().await.push(HookInvocation {
                    event: HookEvent::PostToolUse,
                    tool_name: Some(tool_name),
                    input: None,
                    output: Some(output),
                    timestamp: std::time::Instant::now(),
                });
                Ok(HookResult::Proceed)
            })
        })
    }

    /// Get all invocations
    pub async fn invocations(&self) -> Vec<HookInvocation> {
        self.invocations.lock().await.clone()
    }

    /// Assert hook was called
    pub async fn assert_called(&self, event: HookEvent, times: usize) {
        let invocations = self.invocations.lock().await;
        let count = invocations.iter().filter(|i| i.event == event).count();
        assert_eq!(
            count, times,
            "Expected {:?} to be called {} times, but was called {} times",
            event, times, count
        );
    }

    /// Assert a specific tool was used
    pub async fn assert_tool_used(&self, tool_name: &str) {
        let invocations = self.invocations.lock().await;
        assert!(
            invocations.iter().any(|i| i.tool_name.as_deref() == Some(tool_name)),
            "Expected tool '{}' to be used, but it wasn't. Used tools: {:?}",
            tool_name,
            invocations.iter().filter_map(|i| i.tool_name.as_ref()).collect::<Vec<_>>()
        );
    }

    /// Clear recorded invocations
    pub async fn clear(&self) {
        self.invocations.lock().await.clear();
    }
}
```

---

### 5.6 PermissionRecorder

Utility for testing permission callbacks.

```rust
/// Records and controls permission decisions for testing
pub struct PermissionRecorder {
    decisions: Arc<Mutex<Vec<PermissionDecision>>>,
    default_response: PermissionResult,
    tool_responses: Arc<Mutex<HashMap<String, PermissionResult>>>,
}

/// A recorded permission decision
#[derive(Debug, Clone)]
pub struct PermissionDecision {
    pub tool_name: String,
    pub input: serde_json::Value,
    pub context: ToolPermissionContext,
    pub result: PermissionResult,
    pub timestamp: std::time::Instant,
}

impl PermissionRecorder {
    /// Create with default allow behavior
    pub fn allow_all() -> Self {
        Self {
            decisions: Arc::new(Mutex::new(Vec::new())),
            default_response: PermissionResult::Allow,
            tool_responses: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create with default deny behavior
    pub fn deny_all() -> Self {
        Self {
            decisions: Arc::new(Mutex::new(Vec::new())),
            default_response: PermissionResult::Deny {
                reason: "Denied by test".into(),
            },
            tool_responses: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Allow specific tools
    pub fn allow_tools(tools: &[&str]) -> Self {
        let mut tool_responses = HashMap::new();
        for tool in tools {
            tool_responses.insert(tool.to_string(), PermissionResult::Allow);
        }

        Self {
            decisions: Arc::new(Mutex::new(Vec::new())),
            default_response: PermissionResult::Deny {
                reason: "Not in allow list".into(),
            },
            tool_responses: Arc::new(Mutex::new(tool_responses)),
        }
    }

    /// Set response for a specific tool
    pub async fn set_response(&self, tool: &str, response: PermissionResult) {
        self.tool_responses.lock().await.insert(tool.to_string(), response);
    }

    /// Get as callback for ClaudeAgentOptions
    pub fn as_callback(&self) -> CanUseToolCallback {
        let decisions = Arc::clone(&self.decisions);
        let default_response = self.default_response.clone();
        let tool_responses = Arc::clone(&self.tool_responses);

        Arc::new(move |tool_name, input, context| {
            let decisions = Arc::clone(&decisions);
            let default_response = default_response.clone();
            let tool_responses = Arc::clone(&tool_responses);

            Box::pin(async move {
                let result = {
                    let responses = tool_responses.lock().await;
                    responses.get(&tool_name).cloned().unwrap_or(default_response)
                };

                decisions.lock().await.push(PermissionDecision {
                    tool_name,
                    input,
                    context,
                    result: result.clone(),
                    timestamp: std::time::Instant::now(),
                });

                result
            })
        })
    }

    /// Get all decisions
    pub async fn decisions(&self) -> Vec<PermissionDecision> {
        self.decisions.lock().await.clone()
    }

    /// Assert permission was asked for tool
    pub async fn assert_asked(&self, tool_name: &str) {
        let decisions = self.decisions.lock().await;
        assert!(
            decisions.iter().any(|d| d.tool_name == tool_name),
            "Expected permission to be asked for '{}', but it wasn't. Asked for: {:?}",
            tool_name,
            decisions.iter().map(|d| &d.tool_name).collect::<Vec<_>>()
        );
    }
}
```

---

## 6. API Reference

### 6.1 Public Exports

```rust
// src/testing/mod.rs

// Transport
pub use transport::{MockTransport, MockTransportBuilder, ScheduledMessage, MessageTiming, WrittenMessage, TimingConfig};

// Client
pub use client::MockClient;

// Scenario
pub use scenario::{Scenario, ScenarioBuilder, Exchange, TimingDefaults};

// Message Builders
pub use builders::{
    AssistantMessageBuilder,
    SystemMessageBuilder,
    ResultMessageBuilder,
    ToolResultBuilder,
    StreamEventBuilder,
};

// Recorders
pub use recorders::{
    HookRecorder, HookInvocation,
    PermissionRecorder, PermissionDecision,
};

// Snapshot Testing
pub use snapshot::{SnapshotRecorder, SnapshotPlayer, SessionSnapshot};

// Convenience re-exports
pub use crate::types::messages::*;
pub use crate::types::hooks::HookEvent;
```

### 6.2 Feature Flag Configuration

```toml
# Cargo.toml

[features]
default = []
testing = ["uuid"]

[dependencies]
uuid = { version = "1.0", features = ["v4"], optional = true }

[dev-dependencies]
claude-agent-sdk = { path = ".", features = ["testing"] }
```

---

## 7. Snapshot Testing

### 7.1 Overview

Snapshot testing allows recording real CLI sessions and replaying them in tests, ensuring tests reflect actual behavior.

### 7.2 SnapshotRecorder

```rust
/// Records a live session for later replay
pub struct SnapshotRecorder {
    messages: Arc<Mutex<Vec<RecordedMessage>>>,
    start_time: std::time::Instant,
}

/// A recorded message with timing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedMessage {
    /// Time offset from session start (milliseconds)
    pub offset_ms: u64,
    /// Message direction
    pub direction: MessageDirection,
    /// The message content
    pub content: serde_json::Value,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum MessageDirection {
    /// Message received from CLI (response)
    Received,
    /// Message sent to CLI (query)
    Sent,
}

/// A complete session snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    /// Snapshot format version
    pub version: u32,
    /// When the snapshot was recorded
    pub recorded_at: String,
    /// SDK version used
    pub sdk_version: String,
    /// CLI version used
    pub cli_version: Option<String>,
    /// Options used (sanitized)
    pub options: serde_json::Value,
    /// Recorded messages
    pub messages: Vec<RecordedMessage>,
}

impl SnapshotRecorder {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(Vec::new())),
            start_time: std::time::Instant::now(),
        }
    }

    /// Record a received message
    pub async fn record_received(&self, msg: serde_json::Value) {
        self.messages.lock().await.push(RecordedMessage {
            offset_ms: self.start_time.elapsed().as_millis() as u64,
            direction: MessageDirection::Received,
            content: msg,
        });
    }

    /// Record a sent message
    pub async fn record_sent(&self, msg: serde_json::Value) {
        self.messages.lock().await.push(RecordedMessage {
            offset_ms: self.start_time.elapsed().as_millis() as u64,
            direction: MessageDirection::Sent,
            content: msg,
        });
    }

    /// Export snapshot to file
    pub async fn save(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let snapshot = SessionSnapshot {
            version: 1,
            recorded_at: chrono::Utc::now().to_rfc3339(),
            sdk_version: env!("CARGO_PKG_VERSION").to_string(),
            cli_version: None, // Could be captured during session
            options: serde_json::Value::Null,
            messages: self.messages.lock().await.clone(),
        };

        let json = serde_json::to_string_pretty(&snapshot)?;
        std::fs::write(path, json)
    }

    /// Create a wrapping transport that records all traffic
    pub fn wrap_transport(&self, inner: Arc<dyn Transport>) -> RecordingTransport {
        RecordingTransport {
            inner,
            recorder: self.clone(),
        }
    }
}
```

### 7.3 SnapshotPlayer

```rust
/// Plays back a recorded session
pub struct SnapshotPlayer {
    snapshot: SessionSnapshot,
}

impl SnapshotPlayer {
    /// Load from file
    pub fn load(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let snapshot: SessionSnapshot = serde_json::from_str(&json)?;
        Ok(Self { snapshot })
    }

    /// Load from embedded string (for include_str! usage)
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        let snapshot: SessionSnapshot = serde_json::from_str(json)?;
        Ok(Self { snapshot })
    }

    /// Convert to a scenario
    pub fn to_scenario(&self) -> Scenario {
        let mut builder = ScenarioBuilder::new("snapshot_replay");

        // Group messages by exchanges (split on Sent messages)
        let mut current_responses = Vec::new();
        let mut last_offset = 0u64;

        for msg in &self.snapshot.messages {
            match msg.direction {
                MessageDirection::Sent => {
                    // Finish previous exchange if any
                    if !current_responses.is_empty() {
                        for response in current_responses.drain(..) {
                            builder = builder.respond_delayed(
                                serde_json::from_value(response.content).unwrap(),
                                response.offset_ms.saturating_sub(last_offset),
                                10, // Small jitter
                            );
                        }
                        builder = builder.exchange();
                    }
                    last_offset = msg.offset_ms;
                }
                MessageDirection::Received => {
                    current_responses.push(msg.clone());
                }
            }
        }

        // Add remaining responses
        for response in current_responses {
            builder = builder.respond_delayed(
                serde_json::from_value(response.content).unwrap(),
                response.offset_ms.saturating_sub(last_offset),
                10,
            );
        }

        builder.build()
    }

    /// Create a MockTransport from this snapshot
    pub fn to_mock_transport(&self) -> MockTransport {
        MockTransport::from_scenario(self.to_scenario())
    }
}
```

### 7.4 RecordingTransport

```rust
/// A transport that records all traffic while delegating to inner transport
pub struct RecordingTransport {
    inner: Arc<dyn Transport>,
    recorder: SnapshotRecorder,
}

#[async_trait]
impl Transport for RecordingTransport {
    async fn connect(&self) -> Result<()> {
        self.inner.connect().await
    }

    async fn write(&self, data: &str) -> Result<()> {
        if let Ok(json) = serde_json::from_str(data) {
            self.recorder.record_sent(json).await;
        }
        self.inner.write(data).await
    }

    fn read_messages(&self) -> Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send + '_>> {
        let inner_stream = self.inner.read_messages();
        let recorder = self.recorder.clone();

        Box::pin(async_stream::stream! {
            tokio::pin!(inner_stream);
            while let Some(result) = inner_stream.next().await {
                if let Ok(ref msg) = result {
                    recorder.record_received(msg.clone()).await;
                }
                yield result;
            }
        })
    }

    async fn close(&self) -> Result<()> {
        self.inner.close().await
    }

    fn is_ready(&self) -> bool {
        self.inner.is_ready()
    }

    async fn end_input(&self) -> Result<()> {
        self.inner.end_input().await
    }
}
```

### 7.5 Usage Example

```rust
// Recording a session
#[tokio::test]
#[ignore] // Run manually to record
async fn record_session() {
    let recorder = SnapshotRecorder::new();

    let options = ClaudeAgentOptions::default();
    let mut client = ClaudeClient::new(options);

    // Wrap transport with recorder
    // (This requires internal modification to ClaudeClient)

    client.connect().await.unwrap();
    client.query("Hello, Claude!").await.unwrap();

    let mut stream = client.receive_response();
    while let Some(_msg) = stream.next().await {}

    client.disconnect().await.unwrap();

    recorder.save("tests/snapshots/hello_session.json").await.unwrap();
}

// Replaying a session
#[tokio::test]
async fn replay_hello_session() {
    let snapshot = SnapshotPlayer::load("tests/snapshots/hello_session.json").unwrap();
    let mut client = MockClient::from_scenario(snapshot.to_scenario());

    client.connect().await.unwrap();
    client.query("Hello, Claude!").await.unwrap();

    let messages: Vec<_> = client.receive_response().collect().await;

    // Assertions on replayed messages
    assert!(messages.iter().any(|m| matches!(m, Ok(Message::Assistant(_)))));
    assert!(messages.last().map(|m| matches!(m, Ok(Message::Result(_)))).unwrap_or(false));
}
```

---

## 8. Timing Simulation

### 8.1 Deterministic Timing with Jitter

The framework uses seeded random number generation for deterministic but realistic timing.

```rust
/// Timing simulator with deterministic jitter
pub struct TimingSimulator {
    rng: StdRng,
    speed_factor: f64,
    use_paused_time: bool,
}

impl TimingSimulator {
    /// Create with specific seed for reproducibility
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
            speed_factor: 1.0,
            use_paused_time: true,
        }
    }

    /// Create with instant timing (no delays)
    pub fn instant() -> Self {
        Self {
            rng: StdRng::seed_from_u64(0),
            speed_factor: 0.0,
            use_paused_time: true,
        }
    }

    /// Set speed factor (0.0 = instant, 1.0 = normal, 2.0 = 2x slower)
    pub fn with_speed_factor(mut self, factor: f64) -> Self {
        self.speed_factor = factor;
        self
    }

    /// Calculate delay with jitter
    pub fn delay(&mut self, base_ms: u64, jitter_ms: u64) -> Duration {
        let jitter = if jitter_ms > 0 {
            self.rng.gen_range(0..=jitter_ms)
        } else {
            0
        };

        let total_ms = (base_ms + jitter) as f64 * self.speed_factor;
        Duration::from_millis(total_ms as u64)
    }

    /// Apply delay asynchronously
    pub async fn apply_delay(&mut self, base_ms: u64, jitter_ms: u64) {
        let delay = self.delay(base_ms, jitter_ms);
        if delay > Duration::ZERO {
            tokio::time::sleep(delay).await;
        }
    }
}
```

### 8.2 Test Setup for Paused Time

```rust
/// Initialize test with paused time for determinism
pub fn setup_deterministic_test() {
    tokio::time::pause();
}

/// Test helper macro
#[macro_export]
macro_rules! deterministic_test {
    ($name:ident, $body:expr) => {
        #[tokio::test(start_paused = true)]
        async fn $name() {
            $body
        }
    };
}

// Usage
deterministic_test!(test_with_timing, {
    let scenario = ScenarioBuilder::new("test")
        .timing(TimingDefaults {
            inter_message_delay_ms: 100,
            jitter_ms: 20,
            initial_response_delay_ms: 200,
        })
        .seed(12345) // Fixed seed for reproducibility
        // ...
        .build();

    let mut client = MockClient::from_scenario(scenario);
    // Test runs with deterministic timing
});
```

### 8.3 Timing Profiles

```rust
/// Pre-defined timing profiles
pub mod timing_profiles {
    use super::TimingDefaults;

    /// Instant responses (for fast tests)
    pub fn instant() -> TimingDefaults {
        TimingDefaults {
            inter_message_delay_ms: 0,
            jitter_ms: 0,
            initial_response_delay_ms: 0,
        }
    }

    /// Fast but realistic
    pub fn fast() -> TimingDefaults {
        TimingDefaults {
            inter_message_delay_ms: 10,
            jitter_ms: 5,
            initial_response_delay_ms: 20,
        }
    }

    /// Simulates typical network latency
    pub fn realistic() -> TimingDefaults {
        TimingDefaults {
            inter_message_delay_ms: 50,
            jitter_ms: 30,
            initial_response_delay_ms: 150,
        }
    }

    /// Simulates slow network
    pub fn slow() -> TimingDefaults {
        TimingDefaults {
            inter_message_delay_ms: 200,
            jitter_ms: 100,
            initial_response_delay_ms: 500,
        }
    }
}
```

---

## 9. Implementation Plan

### 9.1 Phase 1: Core Infrastructure

| Task | Files | Priority |
|------|-------|----------|
| Add `testing` feature flag | `Cargo.toml` | High |
| Create module structure | `src/testing/mod.rs` | High |
| Implement `MockTransport` | `src/testing/transport.rs` | High |
| Modify `ClaudeClient::with_transport()` | `src/client.rs` | High |

### 9.2 Phase 2: Message Builders

| Task | Files | Priority |
|------|-------|----------|
| `AssistantMessageBuilder` | `src/testing/builders/assistant.rs` | High |
| `SystemMessageBuilder` | `src/testing/builders/system.rs` | High |
| `ResultMessageBuilder` | `src/testing/builders/result.rs` | High |
| `ToolResultBuilder` | `src/testing/builders/tool.rs` | Medium |
| `StreamEventBuilder` | `src/testing/builders/stream_event.rs` | Low |

### 9.3 Phase 3: Scenario System

| Task | Files | Priority |
|------|-------|----------|
| `Scenario` and `ScenarioBuilder` | `src/testing/scenario.rs` | High |
| `MockClient` wrapper | `src/testing/client.rs` | High |
| `TimingSimulator` | `src/testing/timing.rs` | Medium |

### 9.4 Phase 4: Recorders

| Task | Files | Priority |
|------|-------|----------|
| `HookRecorder` | `src/testing/recorders/hooks.rs` | Medium |
| `PermissionRecorder` | `src/testing/recorders/permissions.rs` | Medium |

### 9.5 Phase 5: Snapshot Testing

| Task | Files | Priority |
|------|-------|----------|
| `SnapshotRecorder` | `src/testing/snapshot.rs` | Medium |
| `SnapshotPlayer` | `src/testing/snapshot.rs` | Medium |
| `RecordingTransport` | `src/testing/snapshot.rs` | Medium |
| Snapshot file format | `tests/snapshots/` | Low |

### 9.6 Dependency Graph

```
Phase 1 (Core)
    │
    ├──► Phase 2 (Builders) ──► Phase 3 (Scenario)
    │                                  │
    │                                  ├──► Phase 4 (Recorders)
    │                                  │
    │                                  └──► Phase 5 (Snapshot)
    │
    └──► (Can run in parallel with Phase 2)
```

---

## 10. Migration & Compatibility

### 10.1 Breaking Changes

**None**. All additions are behind the `testing` feature flag.

### 10.2 New Public API

When `testing` feature is enabled:

```rust
use claude_agent_sdk::testing::{
    MockClient, MockTransport, ScenarioBuilder,
    AssistantMessageBuilder, SystemMessageBuilder, ResultMessageBuilder,
    HookRecorder, PermissionRecorder,
    SnapshotRecorder, SnapshotPlayer,
};
```

### 10.3 Minimum Rust Version

No change. Compatible with existing MSRV.

### 10.4 New Dependencies

| Dependency | Purpose | Conditional |
|------------|---------|-------------|
| `uuid` | Generate unique tool use IDs | `testing` feature |

---

## 11. Verification Checklist

### 11.1 Unit Tests

- [ ] `MockTransport` correctly implements `Transport` trait
- [ ] Message builders produce valid JSON
- [ ] Scenario builder creates correct message sequences
- [ ] Timing simulator produces deterministic results with same seed
- [ ] `HookRecorder` captures all invocations
- [ ] `PermissionRecorder` controls and records decisions

### 11.2 Integration Tests

- [ ] `MockClient` works with `query()` function
- [ ] `MockClient` works with streaming `query_stream()`
- [ ] Multi-turn conversations work correctly
- [ ] Hook callbacks are invoked in mock scenarios
- [ ] Permission callbacks control tool execution
- [ ] Snapshot recording captures all messages
- [ ] Snapshot playback reproduces behavior

### 11.3 Edge Cases

- [ ] Empty scenario (no messages)
- [ ] Scenario with only errors
- [ ] Very long message sequences
- [ ] Concurrent access to recorders
- [ ] Transport closed mid-stream
- [ ] Malformed JSON in snapshot files

### 11.4 Documentation

- [ ] API documentation for all public items
- [ ] Example tests in `examples/` directory
- [ ] Update README with testing section

### 11.5 Build Verification

```bash
# All must pass
cargo build --features testing
cargo test --features testing
cargo clippy --features testing -- -D warnings
cargo doc --features testing --no-deps
```

---

## Appendix A: Complete Usage Examples

### A.1 Simple Query Test

```rust
use claude_agent_sdk::testing::*;

#[tokio::test]
async fn test_simple_query() {
    let scenario = ScenarioBuilder::new("simple_query")
        .on_connect(SystemMessageBuilder::default().build())
        .exchange()
            .respond(AssistantMessageBuilder::new()
                .text("Hello! I'm Claude.")
                .build())
            .then_result(ResultMessageBuilder::default().build())
        .build();

    let mut client = MockClient::from_scenario(scenario);
    client.connect().await.unwrap();
    client.query("Hi there!").await.unwrap();

    let messages: Vec<_> = client.receive_response()
        .filter_map(|r| async { r.ok() })
        .collect()
        .await;

    assert_eq!(messages.len(), 2);
    assert!(matches!(&messages[0], Message::Assistant(_)));
    assert!(matches!(&messages[1], Message::Result(_)));

    client.assert_wrote("Hi there!");
}
```

### A.2 Tool Use Test

```rust
#[tokio::test]
async fn test_tool_use_flow() {
    let scenario = ScenarioBuilder::new("tool_use")
        .on_connect(SystemMessageBuilder::default().build())
        .exchange()
            .respond(AssistantMessageBuilder::new()
                .text("I'll read that file for you.")
                .tool_use("Read", json!({"file_path": "/tmp/test.txt"}))
                .build())
            .respond(AssistantMessageBuilder::new()
                .text("The file contains: Hello World")
                .build())
            .then_result(ResultMessageBuilder::default().build())
        .build();

    let hooks = HookRecorder::new();
    let permissions = PermissionRecorder::allow_all();

    let options = ClaudeAgentOptions::builder()
        .can_use_tool(permissions.as_callback())
        .build();

    let mut client = MockClient::from_scenario_with_options(scenario, options);
    client.connect().await.unwrap();
    client.query("Read /tmp/test.txt").await.unwrap();

    let _: Vec<_> = client.receive_response().collect().await;

    permissions.assert_asked("Read").await;
}
```

### A.3 Snapshot Test

```rust
#[tokio::test]
async fn test_from_snapshot() {
    // Load recorded session
    let snapshot = SnapshotPlayer::from_json(include_str!("../snapshots/hello.json")).unwrap();

    let mut client = MockClient::from_scenario(snapshot.to_scenario());
    client.connect().await.unwrap();
    client.query("Hello!").await.unwrap();

    let messages: Vec<_> = client.receive_response()
        .filter_map(|r| async { r.ok() })
        .collect()
        .await;

    // Verify behavior matches recording
    assert!(messages.iter().any(|m| {
        if let Message::Assistant(a) = m {
            a.content.iter().any(|c| matches!(c, ContentBlock::Text(_)))
        } else {
            false
        }
    }));
}
```

### A.4 Deterministic Timing Test

```rust
#[tokio::test(start_paused = true)]
async fn test_with_deterministic_timing() {
    let scenario = ScenarioBuilder::new("timed")
        .timing(timing_profiles::realistic())
        .seed(42) // Fixed seed
        .on_connect(SystemMessageBuilder::default().build())
        .exchange()
            .respond_delayed(
                AssistantMessageBuilder::new().text("Thinking...").partial().build(),
                100, 20,
            )
            .respond_delayed(
                AssistantMessageBuilder::new().text("Here's my answer.").build(),
                200, 30,
            )
            .then_result(ResultMessageBuilder::default().build())
        .build();

    let mut client = MockClient::from_scenario(scenario);

    let start = tokio::time::Instant::now();
    client.connect().await.unwrap();
    client.query("Question").await.unwrap();

    let _: Vec<_> = client.receive_response().collect().await;

    // With seed 42 and paused time, elapsed time is deterministic
    let elapsed = start.elapsed();
    // Exact value depends on jitter with seed 42
    assert!(elapsed >= Duration::from_millis(300));
}
```

---

## Appendix B: Snapshot File Format

```json
{
  "version": 1,
  "recorded_at": "2024-01-15T10:30:00Z",
  "sdk_version": "0.6.0",
  "cli_version": "1.0.0",
  "options": {
    "model": "claude-sonnet-4-20250514",
    "max_turns": 10
  },
  "messages": [
    {
      "offset_ms": 0,
      "direction": "Received",
      "content": {
        "type": "system",
        "session_id": "abc123",
        "model": "claude-sonnet-4-20250514",
        "tools": ["Read", "Write", "Bash"]
      }
    },
    {
      "offset_ms": 50,
      "direction": "Sent",
      "content": {
        "type": "user",
        "content": "Hello, Claude!"
      }
    },
    {
      "offset_ms": 1250,
      "direction": "Received",
      "content": {
        "type": "assistant",
        "content": [
          {"type": "text", "text": "Hello! How can I help you today?"}
        ]
      }
    },
    {
      "offset_ms": 1300,
      "direction": "Received",
      "content": {
        "type": "result",
        "cost_usd": 0.002,
        "duration_ms": 1250,
        "num_turns": 1
      }
    }
  ]
}
```

---

## Appendix C: Future Enhancements (Out of Scope)

The following may be added in future versions:

1. **Property-based testing**: Integration with proptest for fuzzing
2. **Mocking MCP servers**: Full MCP protocol simulation
3. **Network failure simulation**: Timeouts, disconnects, partial messages
4. **Benchmark utilities**: Performance testing helpers
5. **Visual snapshot diffing**: Tools for comparing recorded sessions
