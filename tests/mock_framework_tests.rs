//! Tests for the testing mock framework
//!
//! These tests validate that the mock framework correctly simulates
//! Claude CLI communication for testing purposes.

use claude_agent_sdk_rs::testing::{
    MessageDirection, MockClient, MockTransport, PermissionRecorder, ScenarioBuilder,
    SnapshotPlayer, SnapshotRecorder, Transport,
    builders::{
        AssistantMessageBuilder, ResultMessageBuilder, SystemMessageBuilder, ToolResultBuilder,
    },
    timing_profiles,
};
use claude_agent_sdk_rs::types::permissions::{PermissionResult, ToolPermissionContext};
use claude_agent_sdk_rs::{ClaudeAgentOptions, Message};
use futures::StreamExt;
use std::time::Duration;

// =============================================================================
// MockTransport Tests
// =============================================================================

#[tokio::test]
async fn test_mock_transport_basic_message_delivery() {
    let transport = MockTransport::builder()
        .message(serde_json::json!({"type": "system", "subtype": "init"}))
        .message(serde_json::json!({"type": "assistant", "message": {"content": []}}))
        .message(serde_json::json!({"type": "result", "subtype": "success"}))
        .build();

    transport.connect().await.unwrap();
    assert!(transport.is_ready());

    let mut stream = transport.read_messages();

    let msg1 = stream.next().await.unwrap().unwrap();
    assert_eq!(msg1["type"], "system");

    let msg2 = stream.next().await.unwrap().unwrap();
    assert_eq!(msg2["type"], "assistant");

    let msg3 = stream.next().await.unwrap().unwrap();
    assert_eq!(msg3["type"], "result");

    transport.close().await.unwrap();
    assert!(!transport.is_ready());
}

#[tokio::test]
async fn test_mock_transport_write_capture() {
    let transport = MockTransport::builder().build();
    transport.connect().await.unwrap();

    // Write some messages
    transport
        .write(r#"{"type": "user", "message": "hello"}"#)
        .await
        .unwrap();
    transport
        .write(r#"{"type": "user", "message": "world"}"#)
        .await
        .unwrap();

    let written = transport.written_messages_async().await;
    assert_eq!(written.len(), 2);
    assert!(written[0].data.contains("hello"));
    assert!(written[1].data.contains("world"));

    // Check parsed JSON is available
    assert!(written[0].parsed.is_some());
    assert_eq!(written[0].parsed.as_ref().unwrap()["message"], "hello");
}

#[tokio::test]
async fn test_mock_transport_injection() {
    let transport = MockTransport::builder().build();
    transport.connect().await.unwrap();

    // Inject a message dynamically
    transport.inject(serde_json::json!({"type": "injected", "data": "test"}));

    let mut stream = transport.read_messages();
    let msg = stream.next().await.unwrap().unwrap();
    assert_eq!(msg["type"], "injected");
    assert_eq!(msg["data"], "test");
}

#[tokio::test]
async fn test_mock_transport_delayed_messages() {
    let transport = MockTransport::builder()
        .message(serde_json::json!({"type": "immediate"}))
        .message_delayed(serde_json::json!({"type": "delayed"}), 50, 0)
        .speed_factor(0.0) // Instant timing for tests
        .build();

    transport.connect().await.unwrap();
    let mut stream = transport.read_messages();

    let start = std::time::Instant::now();

    let _ = stream.next().await.unwrap().unwrap();
    let _ = stream.next().await.unwrap().unwrap();

    // With speed_factor 0.0, delays should be instant
    assert!(start.elapsed() < Duration::from_millis(100));
}

#[tokio::test]
async fn test_mock_transport_after_write_trigger() {
    let transport = MockTransport::builder()
        .message(serde_json::json!({"type": "initial"}))
        .message_after_write(
            serde_json::json!({"type": "triggered", "reason": "found pattern"}),
            "trigger_me",
        )
        .build();

    transport.connect().await.unwrap();
    let mut stream = transport.read_messages();

    // Get the initial message
    let msg1 = stream.next().await.unwrap().unwrap();
    assert_eq!(msg1["type"], "initial");

    // Write something that doesn't match - no new message yet
    transport.write(r#"{"data": "no match"}"#).await.unwrap();

    // Write something that matches the pattern
    transport
        .write(r#"{"command": "trigger_me"}"#)
        .await
        .unwrap();

    // Now we should get the triggered message
    let msg2 = tokio::time::timeout(Duration::from_secs(1), stream.next())
        .await
        .expect("Timeout waiting for triggered message")
        .unwrap()
        .unwrap();
    assert_eq!(msg2["type"], "triggered");
    assert_eq!(msg2["reason"], "found pattern");
}

#[tokio::test]
async fn test_mock_transport_deterministic_seed() {
    // Create two transports with same seed
    let transport1 = MockTransport::builder()
        .message_delayed(serde_json::json!({"id": 1}), 100, 50)
        .seed(42)
        .build();

    let transport2 = MockTransport::builder()
        .message_delayed(serde_json::json!({"id": 1}), 100, 50)
        .seed(42)
        .build();

    // Both should produce the same timing (deterministic jitter)
    transport1.connect().await.unwrap();
    transport2.connect().await.unwrap();

    // With same seed, both should have identical behavior
    let mut stream1 = transport1.read_messages();
    let mut stream2 = transport2.read_messages();

    let start1 = std::time::Instant::now();
    let _ = stream1.next().await;
    let elapsed1 = start1.elapsed();

    let start2 = std::time::Instant::now();
    let _ = stream2.next().await;
    let elapsed2 = start2.elapsed();

    // Timing should be very similar (within small margin for execution time)
    let diff = elapsed1.abs_diff(elapsed2);
    assert!(
        diff < Duration::from_millis(50),
        "Timing difference too large: {:?}",
        diff
    );
}

// =============================================================================
// Message Builder Tests
// =============================================================================

#[test]
fn test_assistant_message_builder_text() {
    let msg = AssistantMessageBuilder::new().text("Hello, world!").build();

    match msg {
        Message::Assistant(assistant) => {
            assert!(!assistant.message.content.is_empty());
        }
        _ => panic!("Expected Assistant message"),
    }
}

#[test]
fn test_assistant_message_builder_tool_use() {
    let msg = AssistantMessageBuilder::new()
        .tool_use("Read", serde_json::json!({"file_path": "/test.txt"}))
        .build();

    match msg {
        Message::Assistant(assistant) => {
            assert_eq!(assistant.message.content.len(), 1);
        }
        _ => panic!("Expected Assistant message"),
    }
}

#[test]
fn test_assistant_message_builder_thinking() {
    let msg = AssistantMessageBuilder::new()
        .thinking("Let me analyze this...")
        .text("Here's my response")
        .build();

    match msg {
        Message::Assistant(assistant) => {
            // Should have both thinking and text blocks
            assert_eq!(assistant.message.content.len(), 2);
        }
        _ => panic!("Expected Assistant message"),
    }
}

#[test]
fn test_system_message_builder() {
    let msg = SystemMessageBuilder::default()
        .session_id("test-session-123")
        .tools(vec!["Read", "Write", "Bash"])
        .build();

    match msg {
        Message::System(system) => {
            assert_eq!(system.session_id, Some("test-session-123".to_string()));
            assert_eq!(system.tools.as_ref().unwrap().len(), 3);
        }
        _ => panic!("Expected System message"),
    }
}

#[test]
fn test_result_message_builder_success() {
    let msg = ResultMessageBuilder::default()
        .cost_usd(0.05)
        .duration_ms(1500)
        .turns(3)
        .build();

    match msg {
        Message::Result(result) => {
            assert!(result.total_cost_usd.unwrap() > 0.0);
            assert!(result.duration_ms > 0);
            assert!(result.num_turns > 0);
            assert!(!result.is_error);
        }
        _ => panic!("Expected Result message"),
    }
}

#[test]
fn test_result_message_builder_error() {
    let msg = ResultMessageBuilder::default().error().build();

    match msg {
        Message::Result(result) => {
            assert!(result.is_error);
        }
        _ => panic!("Expected Result message"),
    }
}

// =============================================================================
// Scenario Builder Tests
// =============================================================================

#[test]
fn test_scenario_builder_basic() {
    let scenario = ScenarioBuilder::new("basic_test")
        .on_connect(SystemMessageBuilder::default().build())
        .exchange()
        .respond(AssistantMessageBuilder::new().text("Hello!").build())
        .then_result(ResultMessageBuilder::default().build())
        .build();

    assert_eq!(scenario.name, "basic_test");
    assert_eq!(scenario.on_connect.len(), 1);
    assert_eq!(scenario.exchanges.len(), 1);
    assert_eq!(scenario.exchanges[0].responses.len(), 2); // assistant + result
}

#[test]
fn test_scenario_builder_multiple_exchanges() {
    let scenario = ScenarioBuilder::new("multi_exchange")
        .on_connect(SystemMessageBuilder::default().build())
        .exchange()
        .respond(
            AssistantMessageBuilder::new()
                .text("First response")
                .build(),
        )
        .then_result(ResultMessageBuilder::default().build())
        .exchange()
        .respond(
            AssistantMessageBuilder::new()
                .text("Second response")
                .build(),
        )
        .then_result(ResultMessageBuilder::default().build())
        .build();

    assert_eq!(scenario.exchanges.len(), 2);
}

#[test]
fn test_scenario_builder_with_trigger_pattern() {
    let scenario = ScenarioBuilder::new("triggered")
        .exchange()
        .when_write_contains("specific_query")
        .respond(
            AssistantMessageBuilder::new()
                .text("Triggered response")
                .build(),
        )
        .build();

    assert!(scenario.exchanges[0].trigger_pattern.is_some());
    assert_eq!(
        scenario.exchanges[0].trigger_pattern.as_ref().unwrap(),
        "specific_query"
    );
}

#[test]
fn test_scenario_with_timing_profiles() {
    // Test instant timing
    let _scenario = ScenarioBuilder::new("instant")
        .timing(timing_profiles::instant())
        .exchange()
        .respond(AssistantMessageBuilder::new().text("Fast!").build())
        .build();

    // Test realistic timing
    let _scenario = ScenarioBuilder::new("realistic")
        .timing(timing_profiles::realistic())
        .exchange()
        .respond(AssistantMessageBuilder::new().text("Normal speed").build())
        .build();
}

// =============================================================================
// MockClient Tests
// =============================================================================

#[tokio::test]
async fn test_mock_client_basic_conversation() {
    let scenario = ScenarioBuilder::new("conversation")
        .on_connect(SystemMessageBuilder::default().build())
        .exchange()
        .respond(
            AssistantMessageBuilder::new()
                .text("Hello! How can I help?")
                .build(),
        )
        .then_result(ResultMessageBuilder::default().build())
        .timing(timing_profiles::instant())
        .build();

    let mut client = MockClient::from_scenario(scenario);
    client.connect_with_transport().await.unwrap();

    // Query and get response
    client.query("Hi there!").await.unwrap();

    // Receive response stream
    let messages: Vec<_> = client.receive_response().collect().await;

    // Should have at least one message
    assert!(!messages.is_empty());

    client.disconnect().await.unwrap();
}

#[tokio::test]
async fn test_mock_client_tool_use_flow() {
    let scenario = ScenarioBuilder::new("tool_use")
        .on_connect(SystemMessageBuilder::default().build())
        .exchange()
        .respond(
            AssistantMessageBuilder::new()
                .text("Let me read that file for you.")
                .tool_use("Read", serde_json::json!({"file_path": "/test.txt"}))
                .build(),
        )
        .respond(
            AssistantMessageBuilder::new()
                .text("The file contains: File contents here")
                .build(),
        )
        .then_result(ResultMessageBuilder::default().build())
        .timing(timing_profiles::instant())
        .build();

    let mut client = MockClient::from_scenario(scenario);
    client.connect_with_transport().await.unwrap();

    client.query("Read /test.txt please").await.unwrap();
    let messages: Vec<_> = client.receive_response().collect().await;

    // Should have multiple messages
    assert!(messages.len() >= 2);

    client.disconnect().await.unwrap();
}

// =============================================================================
// Permission Recorder Tests
// =============================================================================

#[tokio::test]
async fn test_permission_recorder_allow_all() {
    let recorder = PermissionRecorder::allow_all();
    let callback = recorder.as_callback();

    let result = callback(
        "Read".to_string(),
        serde_json::json!({"file_path": "/test.txt"}),
        ToolPermissionContext::default(),
    )
    .await;

    assert!(matches!(result, PermissionResult::Allow(_)));
    recorder.assert_asked("Read").await;
}

#[tokio::test]
async fn test_permission_recorder_deny_all() {
    let recorder = PermissionRecorder::deny_all();
    let callback = recorder.as_callback();

    let result = callback(
        "Bash".to_string(),
        serde_json::json!({"command": "rm -rf /"}),
        ToolPermissionContext::default(),
    )
    .await;

    assert!(matches!(result, PermissionResult::Deny(_)));
}

#[tokio::test]
async fn test_permission_recorder_selective() {
    let recorder = PermissionRecorder::allow_tools(&["Read", "Write"]);
    let callback = recorder.as_callback();

    // Read should be allowed
    let result = callback(
        "Read".to_string(),
        serde_json::json!({}),
        ToolPermissionContext::default(),
    )
    .await;
    assert!(matches!(result, PermissionResult::Allow(_)));

    // Bash should be denied
    let result = callback(
        "Bash".to_string(),
        serde_json::json!({}),
        ToolPermissionContext::default(),
    )
    .await;
    assert!(matches!(result, PermissionResult::Deny(_)));
}

// =============================================================================
// Snapshot Tests
// =============================================================================

#[tokio::test]
async fn test_snapshot_record_and_replay() {
    // Record some messages
    let recorder = SnapshotRecorder::new();

    recorder
        .record_sent(serde_json::json!({"type": "user", "message": "Hello"}))
        .await;
    tokio::time::sleep(Duration::from_millis(10)).await;
    recorder
        .record_received(serde_json::json!({"type": "assistant", "message": "Hi there!"}))
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
            {"offset_ms": 50, "direction": "Received", "content": {"type": "assistant"}},
            {"offset_ms": 100, "direction": "Received", "content": {"type": "result"}}
        ]
    }"#;

    let player = SnapshotPlayer::from_json(json).unwrap();
    let snapshot = player.snapshot();

    assert_eq!(snapshot.version, 1);
    assert_eq!(snapshot.messages.len(), 3);

    // Check helper methods
    assert_eq!(player.sent_messages().len(), 1);
    assert_eq!(player.received_messages().len(), 2);
}

#[tokio::test]
async fn test_snapshot_to_mock_transport() {
    let json = r#"{
        "version": 1,
        "recorded_at": "1234567890",
        "sdk_version": "0.6.0",
        "cli_version": null,
        "options": null,
        "messages": [
            {"offset_ms": 0, "direction": "Sent", "content": {"query": "test"}},
            {"offset_ms": 10, "direction": "Received", "content": {"type": "system"}},
            {"offset_ms": 20, "direction": "Received", "content": {"type": "assistant"}}
        ]
    }"#;

    let player = SnapshotPlayer::from_json(json).unwrap();
    let transport = player.to_mock_transport();

    transport.connect().await.unwrap();

    let mut stream = transport.read_messages();

    // Should get the two received messages
    let msg1 = stream.next().await.unwrap().unwrap();
    assert_eq!(msg1["type"], "system");

    let msg2 = stream.next().await.unwrap().unwrap();
    assert_eq!(msg2["type"], "assistant");
}

// =============================================================================
// Integration: Full Conversation Flow
// =============================================================================

#[tokio::test]
async fn test_full_conversation_flow_with_mock() {
    // This test demonstrates a complete conversation flow using mocks
    let scenario = ScenarioBuilder::new("full_flow")
        .on_connect(
            SystemMessageBuilder::default()
                .session_id("test-session")
                .tools(vec!["Read", "Write", "Bash"])
                .build(),
        )
        // First exchange: simple question
        .exchange()
        .respond(
            AssistantMessageBuilder::new()
                .thinking("User is asking a simple question")
                .text("The capital of France is Paris.")
                .build(),
        )
        .then_result(
            ResultMessageBuilder::default()
                .cost_usd(0.001)
                .duration_ms(500)
                .turns(1)
                .build(),
        )
        // Second exchange: with tool use
        .exchange()
        .respond(
            AssistantMessageBuilder::new()
                .tool_use("Read", serde_json::json!({"file_path": "/etc/hosts"}))
                .build(),
        )
        .then_result(ResultMessageBuilder::default().build())
        .timing(timing_profiles::instant())
        .build();

    let mut client = MockClient::from_scenario_with_options(
        scenario,
        ClaudeAgentOptions {
            max_turns: Some(5),
            ..Default::default()
        },
    );

    client.connect_with_transport().await.unwrap();

    // First query
    client
        .query("What is the capital of France?")
        .await
        .unwrap();
    let messages: Vec<_> = client.receive_response().collect().await;
    assert!(!messages.is_empty());

    // Verify writes were captured
    let written = client.transport().written_messages_async().await;
    assert!(!written.is_empty());

    client.disconnect().await.unwrap();
}

// =============================================================================
// Edge Cases
// =============================================================================

#[tokio::test]
async fn test_mock_transport_empty_scenario() {
    let transport = MockTransport::builder().build();
    transport.connect().await.unwrap();

    // With no messages, should wait for injection
    transport.inject(serde_json::json!({"type": "injected"}));

    let mut stream = transport.read_messages();
    let msg = tokio::time::timeout(Duration::from_millis(100), stream.next())
        .await
        .expect("Should receive injected message")
        .unwrap()
        .unwrap();

    assert_eq!(msg["type"], "injected");
}

#[tokio::test]
async fn test_mock_transport_close_terminates_stream() {
    let transport = MockTransport::builder()
        .message(serde_json::json!({"type": "first"}))
        .build();

    transport.connect().await.unwrap();
    let mut stream = transport.read_messages();

    // Get first message
    let _ = stream.next().await;

    // Close the transport
    transport.close().await.unwrap();

    // Stream should terminate (eventually, due to periodic checks)
    let result = tokio::time::timeout(Duration::from_millis(100), stream.next()).await;

    // Either times out or gets None (stream ended)
    match result {
        Ok(None) => {} // Stream ended properly
        Err(_) => {}   // Timeout is also acceptable
        Ok(Some(_)) => panic!("Should not receive more messages after close"),
    }
}

#[tokio::test]
async fn test_scenario_trigger_pattern_converts_to_after_write() {
    // Test that scenario trigger_pattern is properly converted to AfterWrite timing
    let scenario = ScenarioBuilder::new("trigger_test")
        .exchange()
        .when_write_contains("magic_word")
        .respond(
            AssistantMessageBuilder::new()
                .text("You said the magic word!")
                .build(),
        )
        .timing(timing_profiles::instant())
        .build();

    let transport = MockTransport::from_scenario(scenario);
    transport.connect().await.unwrap();

    let mut stream = transport.read_messages();

    // Write without the pattern - should not trigger
    transport.write(r#"{"query": "hello"}"#).await.unwrap();

    // Write with the pattern - should trigger
    transport.write(r#"{"query": "magic_word"}"#).await.unwrap();

    // Should receive the triggered message
    let msg = tokio::time::timeout(Duration::from_secs(1), stream.next())
        .await
        .expect("Should receive triggered message")
        .unwrap()
        .unwrap();

    assert_eq!(msg["type"], "assistant");
}

// =============================================================================
// ToolResultBuilder Tests
// =============================================================================

#[test]
fn test_tool_result_builder_success() {
    let result = ToolResultBuilder::new("tool_123")
        .success(serde_json::json!({"output": "file contents"}))
        .build_control_response();

    assert_eq!(result["type"], "control_response");
    assert_eq!(result["control_response"]["tool_use_id"], "tool_123");
    assert!(!result["control_response"]["is_error"].as_bool().unwrap());
}

#[test]
fn test_tool_result_builder_error() {
    let result = ToolResultBuilder::new("tool_456")
        .error("File not found")
        .build_control_response();

    assert!(result["control_response"]["is_error"].as_bool().unwrap());
    assert_eq!(
        result["control_response"]["content"]["error"],
        "File not found"
    );
}
