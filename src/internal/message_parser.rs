//! Message parser for converting JSON to typed messages

use crate::errors::{MessageParseError, Result};
use crate::types::messages::Message;

/// Message parser for CLI output
pub struct MessageParser;

impl MessageParser {
    /// Parse a JSON value into a Message, consuming the value
    ///
    /// This method consumes the JSON value to avoid unnecessary cloning.
    /// On parse error, the original data is not available in the error
    /// since it was consumed during the parse attempt.
    pub fn parse(data: serde_json::Value) -> Result<Message> {
        serde_json::from_value(data).map_err(|e| {
            MessageParseError::new(
                format!("Failed to parse message: {}", e),
                None, // Don't include original data to avoid cloning overhead
            )
            .into()
        })
    }
}
