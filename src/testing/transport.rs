//! Mock transport implementation for testing

use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use futures::stream::Stream;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use tokio::sync::Mutex;

use crate::errors::{ClaudeError, ConnectionError, Result};
use crate::internal::transport::Transport;

/// A message scheduled for delivery
#[derive(Debug, Clone)]
pub struct ScheduledMessage {
    /// The JSON value to emit
    pub value: serde_json::Value,
    /// Timing configuration for this message
    pub timing: MessageTiming,
}

/// Timing configuration for a message
#[derive(Debug, Clone, Default)]
pub enum MessageTiming {
    /// Deliver immediately
    #[default]
    Immediate,
    /// Deliver after base delay + jitter
    Delayed {
        /// Base delay in milliseconds
        base_ms: u64,
        /// Random jitter range in milliseconds
        jitter_ms: u64,
    },
    /// Wait for a write matching pattern before delivering
    AfterWrite {
        /// Pattern to match in written data
        pattern: String,
    },
}

/// A captured write operation
#[derive(Debug, Clone)]
pub struct WrittenMessage {
    /// Raw data written
    pub data: String,
    /// Parsed JSON if valid
    pub parsed: Option<serde_json::Value>,
    /// Timestamp of write
    pub timestamp: std::time::Instant,
}

/// Global timing configuration
#[derive(Debug, Clone)]
pub struct TimingConfig {
    /// Seed for deterministic random jitter
    pub seed: u64,
    /// Global delay multiplier (0.0 = instant, 1.0 = normal, 2.0 = double)
    pub speed_factor: f64,
    /// Whether to use tokio::time::pause() for deterministic async timing
    pub use_paused_time: bool,
}

impl Default for TimingConfig {
    fn default() -> Self {
        Self {
            seed: 0,
            speed_factor: 1.0,
            use_paused_time: false,
        }
    }
}

/// A mock transport for testing that simulates CLI communication
pub struct MockTransport {
    /// Pre-loaded messages to emit
    messages: Arc<Mutex<VecDeque<ScheduledMessage>>>,
    /// Messages waiting for write triggers
    pending_triggers: Arc<Mutex<Vec<ScheduledMessage>>>,
    /// Triggered messages ready for delivery
    triggered_messages: Arc<Mutex<VecDeque<serde_json::Value>>>,
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
    /// Random number generator for deterministic jitter (reserved for future use)
    #[allow(dead_code)]
    rng: Mutex<StdRng>,
}

impl MockTransport {
    /// Create a new MockTransport with the given messages and timing configuration
    pub fn new(messages: Vec<ScheduledMessage>, timing: TimingConfig) -> Self {
        let (injector_tx, injector_rx) = flume::unbounded();

        // Separate messages by timing type
        let mut immediate_messages = VecDeque::new();
        let mut pending_triggers = Vec::new();

        for msg in messages {
            match &msg.timing {
                MessageTiming::AfterWrite { .. } => {
                    pending_triggers.push(msg);
                }
                _ => {
                    immediate_messages.push_back(msg);
                }
            }
        }

        Self {
            messages: Arc::new(Mutex::new(immediate_messages)),
            pending_triggers: Arc::new(Mutex::new(pending_triggers)),
            triggered_messages: Arc::new(Mutex::new(VecDeque::new())),
            injector_tx,
            injector_rx,
            written: Arc::new(Mutex::new(Vec::new())),
            connected: AtomicBool::new(false),
            ready: AtomicBool::new(false),
            timing: timing.clone(),
            rng: Mutex::new(StdRng::seed_from_u64(timing.seed)),
        }
    }

    /// Create a builder for MockTransport
    pub fn builder() -> MockTransportBuilder {
        MockTransportBuilder::default()
    }

    /// Inject a message dynamically during test
    pub fn inject(&self, msg: serde_json::Value) {
        let _ = self.injector_tx.send(msg);
    }

    /// Get all written messages (blocking)
    ///
    /// Note: Prefer `written_messages_async()` in async contexts to avoid potential
    /// issues with mixed runtime executors.
    pub fn written_messages(&self) -> Vec<WrittenMessage> {
        // Use futures executor for synchronous access
        // Safe in test contexts but may have issues if called from within tokio runtime
        futures::executor::block_on(async { self.written.lock().await.clone() })
    }

    /// Get written messages asynchronously (preferred)
    pub async fn written_messages_async(&self) -> Vec<WrittenMessage> {
        self.written.lock().await.clone()
    }

    /// Check if any messages are waiting for a write pattern
    async fn check_write_triggers(&self, data: &str) {
        let mut pending = self.pending_triggers.lock().await;
        let mut triggered = self.triggered_messages.lock().await;

        let mut remaining = Vec::new();
        for msg in pending.drain(..) {
            if let MessageTiming::AfterWrite { ref pattern } = msg.timing {
                if data.contains(pattern) {
                    triggered.push_back(msg.value);
                } else {
                    remaining.push(msg);
                }
            } else {
                remaining.push(msg);
            }
        }
        *pending = remaining;
    }

    /// Create a MockTransport from a scenario
    pub fn from_scenario(scenario: super::Scenario) -> Self {
        let mut all_messages = scenario.on_connect;
        for exchange in scenario.exchanges {
            // If exchange has a trigger_pattern, convert responses to AfterWrite timing
            if let Some(ref pattern) = exchange.trigger_pattern {
                for mut msg in exchange.responses {
                    msg.timing = MessageTiming::AfterWrite {
                        pattern: pattern.clone(),
                    };
                    all_messages.push(msg);
                }
            } else {
                all_messages.extend(exchange.responses);
            }
        }
        Self::new(all_messages, TimingConfig::default())
    }

    /// Check if there are pending triggered messages
    pub async fn has_triggered_messages(&self) -> bool {
        !self.triggered_messages.lock().await.is_empty()
    }
}

#[async_trait]
impl Transport for MockTransport {
    async fn connect(&self) -> Result<()> {
        if self.connected.swap(true, Ordering::SeqCst) {
            return Err(ClaudeError::Connection(ConnectionError {
                message: "Already connected".to_string(),
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
        let triggered_messages = Arc::clone(&self.triggered_messages);
        let injector_rx = self.injector_rx.clone();
        let timing = self.timing.clone();
        let rng = Arc::new(Mutex::new(StdRng::seed_from_u64(self.timing.seed)));
        let connected = &self.connected;

        Box::pin(async_stream::stream! {
            loop {
                // Check connection state
                if !connected.load(Ordering::SeqCst) {
                    break;
                }

                // Check for triggered messages first (from AfterWrite)
                {
                    let mut triggered = triggered_messages.lock().await;
                    if let Some(msg) = triggered.pop_front() {
                        yield Ok(msg);
                        continue;
                    }
                }

                // Check for injected messages (non-blocking)
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
                                let jitter = if jitter_ms > 0 {
                                    let mut rng_guard = rng.lock().await;
                                    rng_guard.gen_range(0..=jitter_ms)
                                } else {
                                    0
                                };
                                let delay = Duration::from_millis(
                                    ((base_ms + jitter) as f64 * timing.speed_factor) as u64
                                );
                                if delay > Duration::ZERO {
                                    tokio::time::sleep(delay).await;
                                }
                            },
                            MessageTiming::AfterWrite { .. } => {
                                // Should not reach here - AfterWrite messages are in pending_triggers
                            },
                        }
                        yield Ok(scheduled.value);
                    }
                    None => {
                        // No more pre-loaded messages, poll for injection or triggered messages
                        // Use a short timeout to periodically check for triggered messages
                        // This prevents deadlock when AfterWrite messages are triggered by writes
                        tokio::select! {
                            result = injector_rx.recv_async() => {
                                match result {
                                    Ok(msg) => yield Ok(msg),
                                    Err(_) => break, // Channel closed
                                }
                            }
                            _ = tokio::time::sleep(Duration::from_millis(10)) => {
                                // Check if we're still connected and if triggered messages appeared
                                if !connected.load(Ordering::SeqCst) {
                                    break;
                                }
                                // Loop back to check triggered_messages
                                continue;
                            }
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

/// Builder for MockTransport
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

    /// Add a message that triggers after a write pattern is matched
    pub fn message_after_write(
        mut self,
        msg: impl Into<serde_json::Value>,
        pattern: impl Into<String>,
    ) -> Self {
        self.messages.push(ScheduledMessage {
            value: msg.into(),
            timing: MessageTiming::AfterWrite {
                pattern: pattern.into(),
            },
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

    /// Set speed factor (0.0 = instant, 1.0 = normal, 2.0 = 2x slower)
    pub fn speed_factor(mut self, factor: f64) -> Self {
        self.timing
            .get_or_insert_with(TimingConfig::default)
            .speed_factor = factor;
        self
    }

    /// Build the transport
    pub fn build(self) -> MockTransport {
        MockTransport::new(self.messages, self.timing.unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    #[tokio::test]
    async fn test_mock_transport_basic() {
        let transport = MockTransport::builder()
            .message(serde_json::json!({"type": "system", "subtype": "init"}))
            .message(serde_json::json!({"type": "assistant", "message": {"content": []}}))
            .build();

        transport.connect().await.unwrap();
        assert!(transport.is_ready());

        let mut stream = transport.read_messages();

        let msg1 = stream.next().await.unwrap().unwrap();
        assert_eq!(msg1["type"], "system");

        let msg2 = stream.next().await.unwrap().unwrap();
        assert_eq!(msg2["type"], "assistant");

        transport.close().await.unwrap();
        assert!(!transport.is_ready());
    }

    #[tokio::test]
    async fn test_mock_transport_write_capture() {
        let transport = MockTransport::builder().build();
        transport.connect().await.unwrap();

        transport
            .write(r#"{"type": "user", "message": "hello"}"#)
            .await
            .unwrap();

        let written = transport.written_messages_async().await;
        assert_eq!(written.len(), 1);
        assert!(written[0].data.contains("hello"));
        assert!(written[0].parsed.is_some());
    }

    #[tokio::test]
    async fn test_mock_transport_injection() {
        let transport = MockTransport::builder().build();
        transport.connect().await.unwrap();

        transport.inject(serde_json::json!({"type": "injected"}));

        let mut stream = transport.read_messages();
        let msg = stream.next().await.unwrap().unwrap();
        assert_eq!(msg["type"], "injected");
    }
}
