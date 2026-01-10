//! Transport trait definition

use async_trait::async_trait;
use futures::stream::Stream;
use std::pin::Pin;

use crate::errors::Result;

/// Transport trait for communicating with Claude Code CLI
///
/// All methods use `&self` because implementations handle their own
/// internal synchronization (e.g., Mutex for stdin/stdout, AtomicBool for ready state).
/// This allows the transport to be shared via `Arc<dyn Transport>` without an outer Mutex.
#[async_trait]
pub trait Transport: Send + Sync {
    /// Connect the transport
    async fn connect(&self) -> Result<()>;

    /// Write raw data to the transport
    async fn write(&self, data: &str) -> Result<()>;

    /// Read messages as a stream of JSON values
    fn read_messages(&self) -> Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send + '_>>;

    /// Close the transport
    async fn close(&self) -> Result<()>;

    /// Check if the transport is ready
    #[allow(dead_code)]
    fn is_ready(&self) -> bool;

    /// End input stream (close stdin)
    async fn end_input(&self) -> Result<()>;
}
