//! MockClient wrapper for testing

use std::sync::Arc;

use super::Transport;
use super::scenario::Scenario;
use super::transport::{MockTransport, WrittenMessage};
use crate::client::ClaudeClient;
use crate::types::config::ClaudeAgentOptions;

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
    pub fn from_scenario_with_options(scenario: Scenario, options: ClaudeAgentOptions) -> Self {
        let transport = Arc::new(MockTransport::from_scenario(scenario));
        let client = ClaudeClient::with_transport(
            Arc::clone(&transport) as Arc<dyn crate::internal::transport::Transport>,
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
            Arc::clone(&transport) as Arc<dyn crate::internal::transport::Transport>,
            options.clone(),
        );

        Self {
            client,
            transport,
            options,
        }
    }

    /// Access the underlying client mutably
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
            written
                .iter()
                .filter_map(|w| w.parsed.as_ref())
                .any(&matcher),
            "No written JSON matched the predicate"
        );
    }

    /// Get all written messages
    pub fn written_messages(&self) -> Vec<WrittenMessage> {
        self.transport.written_messages()
    }

    /// Inject a message dynamically during test
    pub fn inject_message(&self, msg: crate::types::messages::Message) {
        self.transport
            .inject(serde_json::to_value(msg).expect("Message serialization should not fail"));
    }

    /// Inject an error
    pub fn inject_error(&self, err: &str) {
        self.transport.inject(serde_json::json!({
            "type": "error",
            "error": { "message": err }
        }));
    }

    // === Stream Access ===

    /// Access the raw message stream for receiving messages
    ///
    /// This provides lower-level access to messages without going through
    /// the client's response handling logic.
    pub fn receive_messages(
        &self,
    ) -> std::pin::Pin<
        Box<
            dyn futures::stream::Stream<
                    Item = crate::errors::Result<crate::types::messages::Message>,
                > + Send
                + '_,
        >,
    > {
        use futures::StreamExt;

        let stream = self.transport.read_messages();

        Box::pin(
            stream.map(|result| {
                result.and_then(crate::internal::message_parser::MessageParser::parse)
            }),
        )
    }

    // === Convenience Assertions ===

    /// Assert no messages were written
    pub fn assert_no_writes(&self) {
        let written = self.transport.written_messages();
        assert!(
            written.is_empty(),
            "Expected no writes, but got {} messages: {:?}",
            written.len(),
            written.iter().map(|w| &w.data).collect::<Vec<_>>()
        );
    }

    /// Assert exact number of writes
    pub fn assert_write_count(&self, expected: usize) {
        let written = self.transport.written_messages();
        assert_eq!(
            written.len(),
            expected,
            "Expected {} writes, got {}",
            expected,
            written.len()
        );
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::ScenarioBuilder;
    use crate::testing::builders::{
        AssistantMessageBuilder, ResultMessageBuilder, SystemMessageBuilder,
    };
    use futures::StreamExt;

    #[tokio::test]
    async fn test_mock_client_basic() {
        let scenario = ScenarioBuilder::new("basic")
            .on_connect(SystemMessageBuilder::default().build())
            .exchange()
            .respond(AssistantMessageBuilder::new().text("Hello!").build())
            .then_result(ResultMessageBuilder::default().build())
            .build();

        let mut client = MockClient::from_scenario(scenario);
        client.connect_with_transport().await.unwrap();

        // Send query
        client.query("Hi").await.unwrap();

        // Receive response
        let messages: Vec<_> = client.receive_response().collect().await;

        assert!(!messages.is_empty());
        client.assert_wrote("Hi");
    }
}
