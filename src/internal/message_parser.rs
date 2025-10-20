//! Message parser for converting JSON to typed messages

use crate::errors::{MessageParseError, Result};
use crate::types::messages::Message;

/// Message parser for CLI output
pub struct MessageParser;

impl MessageParser {
    /// Parse a JSON value into a Message
    pub fn parse(data: serde_json::Value) -> Result<Message> {
        serde_json::from_value(data.clone()).map_err(|e| {
            MessageParseError::new(format!("Failed to parse message: {}", e), Some(data)).into()
        })
    }
}
