//! Snapshot testing - record and replay sessions

use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use futures::StreamExt;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use super::scenario::Scenario;
use super::transport::{MessageTiming, ScheduledMessage};
use crate::errors::Result;
use crate::internal::transport::Transport;

/// Message direction in a recorded session
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum MessageDirection {
    /// Message received from CLI (response)
    Received,
    /// Message sent to CLI (query)
    Sent,
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

/// A complete session snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    /// Snapshot format version
    pub version: u32,
    /// When the snapshot was recorded
    pub recorded_at: String,
    /// SDK version used
    pub sdk_version: String,
    /// CLI version used (if available)
    pub cli_version: Option<String>,
    /// Options used (sanitized)
    pub options: serde_json::Value,
    /// Recorded messages
    pub messages: Vec<RecordedMessage>,
}

/// Records a live session for later replay
#[derive(Clone)]
pub struct SnapshotRecorder {
    messages: Arc<Mutex<Vec<RecordedMessage>>>,
    start_time: std::time::Instant,
}

impl SnapshotRecorder {
    /// Create a new snapshot recorder
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
            recorded_at: chrono_lite_now(),
            sdk_version: env!("CARGO_PKG_VERSION").to_string(),
            cli_version: None,
            options: serde_json::Value::Null,
            messages: self.messages.lock().await.clone(),
        };

        let json = serde_json::to_string_pretty(&snapshot)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(path, json)
    }

    /// Get recorded messages
    pub async fn messages(&self) -> Vec<RecordedMessage> {
        self.messages.lock().await.clone()
    }

    /// Create a wrapping transport that records all traffic
    pub fn wrap_transport(&self, inner: Arc<dyn Transport>) -> RecordingTransport {
        RecordingTransport {
            inner,
            recorder: self.clone(),
        }
    }
}

impl Default for SnapshotRecorder {
    fn default() -> Self {
        Self::new()
    }
}

/// Plays back a recorded session
pub struct SnapshotPlayer {
    snapshot: SessionSnapshot,
}

impl SnapshotPlayer {
    /// Load from file
    pub fn load(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        let snapshot: SessionSnapshot = serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(Self { snapshot })
    }

    /// Load from embedded string (for include_str! usage)
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        let snapshot: SessionSnapshot = serde_json::from_str(json)?;
        Ok(Self { snapshot })
    }

    /// Get the snapshot
    pub fn snapshot(&self) -> &SessionSnapshot {
        &self.snapshot
    }

    /// Convert to a scenario
    ///
    /// Note: This creates a basic scenario structure. For more accurate replay
    /// with timing, use `to_mock_transport()` directly.
    pub fn to_scenario(&self) -> Scenario {
        Scenario {
            name: "snapshot_replay".to_string(),
            on_connect: Vec::new(),
            exchanges: Vec::new(),
        }
    }

    /// Create a MockTransport from this snapshot
    ///
    /// This directly converts recorded messages to ScheduledMessages with proper timing,
    /// bypassing the Scenario abstraction for more accurate replay.
    pub fn to_mock_transport(&self) -> super::MockTransport {
        let mut scheduled_messages = Vec::new();
        let mut last_offset = 0u64;

        for msg in &self.snapshot.messages {
            match msg.direction {
                MessageDirection::Sent => {
                    // Sent messages are queries - we don't emit them, just track timing
                    last_offset = msg.offset_ms;
                }
                MessageDirection::Received => {
                    // Calculate delay from last message
                    let delay = msg.offset_ms.saturating_sub(last_offset);
                    last_offset = msg.offset_ms;

                    let scheduled = ScheduledMessage {
                        value: msg.content.clone(),
                        timing: if delay > 0 {
                            MessageTiming::Delayed {
                                base_ms: delay,
                                jitter_ms: 0, // No jitter for replay - preserve exact timing
                            }
                        } else {
                            MessageTiming::Immediate
                        },
                    };
                    scheduled_messages.push(scheduled);
                }
            }
        }

        super::MockTransport::new(
            scheduled_messages,
            super::transport::TimingConfig::default(),
        )
    }

    /// Get received messages only (for assertions)
    pub fn received_messages(&self) -> Vec<&RecordedMessage> {
        self.snapshot
            .messages
            .iter()
            .filter(|m| m.direction == MessageDirection::Received)
            .collect()
    }

    /// Get sent messages only (for assertions)
    pub fn sent_messages(&self) -> Vec<&RecordedMessage> {
        self.snapshot
            .messages
            .iter()
            .filter(|m| m.direction == MessageDirection::Sent)
            .collect()
    }
}

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

/// Simple timestamp without chrono dependency
fn chrono_lite_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_snapshot_recorder() {
        let recorder = SnapshotRecorder::new();

        recorder
            .record_sent(serde_json::json!({"type": "user"}))
            .await;
        recorder
            .record_received(serde_json::json!({"type": "assistant"}))
            .await;

        let messages = recorder.messages().await;
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].direction, MessageDirection::Sent);
        assert_eq!(messages[1].direction, MessageDirection::Received);
    }

    #[test]
    fn test_snapshot_player_from_json() {
        let json = r#"{
            "version": 1,
            "recorded_at": "1234567890",
            "sdk_version": "0.6.0",
            "cli_version": null,
            "options": null,
            "messages": [
                {"offset_ms": 0, "direction": "Sent", "content": {"type": "user"}},
                {"offset_ms": 100, "direction": "Received", "content": {"type": "assistant"}}
            ]
        }"#;

        let player = SnapshotPlayer::from_json(json).unwrap();
        assert_eq!(player.snapshot().messages.len(), 2);
    }
}
