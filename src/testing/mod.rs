//! Testing mock framework for Claude Agent SDK
//!
//! This module provides comprehensive testing utilities for writing deterministic,
//! fast, and reliable tests without requiring the Claude Code CLI or network access.
//!
//! # Features
//!
//! - **MockTransport**: Drop-in replacement for subprocess transport
//! - **Message Builders**: Ergonomic builders for all message types
//! - **Scenario System**: Linear message sequences with timing simulation
//! - **Snapshot Testing**: Record real sessions, replay in tests
//! - **Hook/Permission Testing**: Verify callbacks are invoked correctly
//! - **Deterministic Timing**: Reproducible delays with seeded random jitters
//!
//! # Example
//!
//! ```no_run
//! use claude_agent_sdk_rs::testing::*;
//!
//! #[tokio::test]
//! async fn test_simple_query() {
//!     let scenario = ScenarioBuilder::new("simple_query")
//!         .on_connect(SystemMessageBuilder::default().build())
//!         .exchange()
//!             .respond(AssistantMessageBuilder::new()
//!                 .text("Hello! I'm Claude.")
//!                 .build())
//!             .then_result(ResultMessageBuilder::default().build())
//!         .build();
//!
//!     let mut client = MockClient::from_scenario(scenario);
//!     client.connect().await.unwrap();
//!     client.query("Hi there!").await.unwrap();
//!
//!     // Verify messages received
//!     // ...
//! }
//! ```

mod client;
mod scenario;
mod snapshot;
mod timing;
mod transport;

pub mod builders;
pub mod recorders;

// Transport
pub use transport::{
    MessageTiming, MockTransport, MockTransportBuilder, ScheduledMessage, TimingConfig,
    WrittenMessage,
};

// Client
pub use client::MockClient;

// Scenario
pub use scenario::{Exchange, Scenario, ScenarioBuilder, TimingDefaults};

// Timing
pub use timing::{TimingSimulator, timing_profiles};

// Snapshot Testing
pub use snapshot::{
    MessageDirection, RecordedMessage, SessionSnapshot, SnapshotPlayer, SnapshotRecorder,
};

// Message Builders
pub use builders::{
    AssistantMessageBuilder, ResultMessageBuilder, SystemMessageBuilder, ToolResultBuilder,
};

// Recorders
pub use recorders::{HookInvocation, HookRecorder, PermissionDecision, PermissionRecorder};

// Convenience re-exports
pub use crate::types::hooks::HookEvent;
pub use crate::types::messages::*;

// Re-export Transport trait for testing
pub use crate::internal::transport::Transport;
