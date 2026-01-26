//! Builder for ResultMessage

use crate::types::messages::{Message, ResultMessage};

/// Builder for ResultMessage (query completion)
pub struct ResultMessageBuilder {
    subtype: String,
    cost_usd: f64,
    duration_ms: u64,
    duration_api_ms: u64,
    turns: u32,
    is_error: bool,
    session_id: String,
    result: Option<String>,
}

impl ResultMessageBuilder {
    /// Create a new builder with defaults
    pub fn new() -> Self {
        Self {
            subtype: "success".to_string(),
            cost_usd: 0.01,
            duration_ms: 1000,
            duration_api_ms: 800,
            turns: 1,
            is_error: false,
            session_id: format!("test-session-{}", uuid::Uuid::new_v4()),
            result: None,
        }
    }

    /// Set the subtype
    pub fn subtype(mut self, subtype: impl Into<String>) -> Self {
        self.subtype = subtype.into();
        self
    }

    /// Set the total cost in USD
    pub fn cost_usd(mut self, cost: f64) -> Self {
        self.cost_usd = cost;
        self
    }

    /// Set the total duration in milliseconds
    pub fn duration_ms(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }

    /// Set the API duration in milliseconds
    pub fn duration_api_ms(mut self, ms: u64) -> Self {
        self.duration_api_ms = ms;
        self
    }

    /// Set the number of turns
    pub fn turns(mut self, turns: u32) -> Self {
        self.turns = turns;
        self
    }

    /// Mark as error
    pub fn error(mut self) -> Self {
        self.is_error = true;
        self.subtype = "error".to_string();
        self
    }

    /// Set the session ID
    pub fn session_id(mut self, id: impl Into<String>) -> Self {
        self.session_id = id.into();
        self
    }

    /// Set the result text
    pub fn result(mut self, result: impl Into<String>) -> Self {
        self.result = Some(result.into());
        self
    }

    /// Build the message
    pub fn build(self) -> Message {
        Message::Result(ResultMessage {
            subtype: self.subtype,
            total_cost_usd: Some(self.cost_usd),
            duration_ms: self.duration_ms,
            duration_api_ms: self.duration_api_ms,
            num_turns: self.turns,
            is_error: self.is_error,
            session_id: self.session_id,
            usage: None,
            result: self.result,
            structured_output: None,
        })
    }

    /// Build as JSON value
    pub fn build_json(self) -> serde_json::Value {
        serde_json::to_value(self.build()).expect("Message serialization should not fail")
    }
}

impl Default for ResultMessageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_builder_defaults() {
        let msg = ResultMessageBuilder::default().build();

        if let Message::Result(result) = msg {
            assert_eq!(result.subtype, "success");
            assert!(!result.is_error);
            assert_eq!(result.num_turns, 1);
        } else {
            panic!("Expected result message");
        }
    }

    #[test]
    fn test_result_builder_error() {
        let msg = ResultMessageBuilder::new().error().build();

        if let Message::Result(result) = msg {
            assert!(result.is_error);
            assert_eq!(result.subtype, "error");
        } else {
            panic!("Expected result message");
        }
    }

    #[test]
    fn test_result_builder_custom() {
        let msg = ResultMessageBuilder::new()
            .cost_usd(0.05)
            .duration_ms(2000)
            .turns(3)
            .build();

        if let Message::Result(result) = msg {
            assert_eq!(result.total_cost_usd, Some(0.05));
            assert_eq!(result.duration_ms, 2000);
            assert_eq!(result.num_turns, 3);
        } else {
            panic!("Expected result message");
        }
    }
}
