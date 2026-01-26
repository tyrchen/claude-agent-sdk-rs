//! Scenario system for defining test conversations

use super::transport::{MessageTiming, ScheduledMessage};
use crate::types::messages::Message;

/// Default timing for scenario messages
#[derive(Debug, Clone)]
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

/// A single request-response exchange
#[derive(Debug, Clone)]
pub struct Exchange {
    /// Messages to emit for this exchange
    pub responses: Vec<ScheduledMessage>,
    /// Optional: only trigger after write matching this pattern
    pub trigger_pattern: Option<String>,
}

/// A test scenario defining a sequence of messages
#[derive(Debug, Clone)]
pub struct Scenario {
    /// Initial messages sent on connect (e.g., SystemMessage)
    pub on_connect: Vec<ScheduledMessage>,
    /// Conversation exchanges
    pub exchanges: Vec<Exchange>,
    /// Name for debugging
    pub name: String,
}

/// Builder for scenarios
pub struct ScenarioBuilder {
    name: String,
    on_connect: Vec<ScheduledMessage>,
    exchanges: Vec<Exchange>,
    current_exchange: Option<ExchangeBuilder>,
    timing_defaults: TimingDefaults,
}

impl ScenarioBuilder {
    /// Create a new scenario builder
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

    /// Set seed for deterministic jitter
    #[allow(dead_code)]
    pub fn seed(self, _seed: u64) -> Self {
        // Seed is applied at transport level, not scenario level
        self
    }

    /// Add system message on connect
    pub fn on_connect(mut self, msg: Message) -> Self {
        self.on_connect.push(ScheduledMessage {
            value: serde_json::to_value(msg).expect("Message serialization should not fail"),
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
            value: serde_json::to_value(msg).expect("Message serialization should not fail"),
            timing,
        });
    }

    fn add_response_delayed(&mut self, msg: Message, base_ms: u64, jitter_ms: u64) {
        self.is_first = false;
        self.responses.push(ScheduledMessage {
            value: serde_json::to_value(msg).expect("Message serialization should not fail"),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::builders::{
        AssistantMessageBuilder, ResultMessageBuilder, SystemMessageBuilder,
    };

    #[test]
    fn test_scenario_builder_basic() {
        let scenario = ScenarioBuilder::new("test")
            .on_connect(SystemMessageBuilder::default().build())
            .exchange()
            .respond(AssistantMessageBuilder::new().text("Hello").build())
            .then_result(ResultMessageBuilder::default().build())
            .build();

        assert_eq!(scenario.name, "test");
        assert_eq!(scenario.on_connect.len(), 1);
        assert_eq!(scenario.exchanges.len(), 1);
        assert_eq!(scenario.exchanges[0].responses.len(), 2);
    }

    #[test]
    fn test_scenario_builder_multiple_exchanges() {
        let scenario = ScenarioBuilder::new("multi")
            .on_connect(SystemMessageBuilder::default().build())
            .exchange()
            .respond(AssistantMessageBuilder::new().text("First").build())
            .then_result(ResultMessageBuilder::default().build())
            .exchange()
            .respond(AssistantMessageBuilder::new().text("Second").build())
            .then_result(ResultMessageBuilder::default().build())
            .build();

        assert_eq!(scenario.exchanges.len(), 2);
    }
}
