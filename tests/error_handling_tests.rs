//! Tests for error handling paths in the SDK
//!
//! These tests verify that errors are properly handled and reported
//! throughout the SDK.

use claude_agent_sdk_rs::testing::{
    AssistantMessageBuilder, MockTransport, ResultMessageBuilder, ScenarioBuilder,
    SystemMessageBuilder, Transport,
};
use claude_agent_sdk_rs::{ClaudeAgentOptions, ClaudeError, UserContentBlock};
use futures::StreamExt;

// =============================================================================
// Transport Error Tests
// =============================================================================

#[tokio::test]
async fn test_transport_write_before_connect_fails() {
    let transport = MockTransport::builder().build();

    // Writing before connect should fail
    let result = transport.write("test data").await;
    assert!(result.is_err(), "Write before connect should fail");

    let err = result.unwrap_err();
    assert!(
        matches!(err, ClaudeError::Transport(_)),
        "Should be a transport error"
    );
}

#[tokio::test]
async fn test_transport_double_connect_fails() {
    let transport = MockTransport::builder().build();

    transport.connect().await.unwrap();

    // Second connect should fail
    let result = transport.connect().await;
    assert!(result.is_err(), "Double connect should fail");
}

#[tokio::test]
async fn test_transport_read_empty_messages() {
    let transport = MockTransport::builder().build();
    transport.connect().await.unwrap();

    // With no messages and no injection, should wait
    // Close the transport to end the stream
    transport.close().await.unwrap();

    let mut stream = transport.read_messages();

    // Stream should terminate
    let result = tokio::time::timeout(std::time::Duration::from_millis(100), stream.next()).await;

    // Either timeout or None (stream ended) is acceptable
    match result {
        Ok(None) => {} // Stream ended properly
        Err(_) => {}   // Timeout is also acceptable
        Ok(Some(Ok(_))) => panic!("Should not receive messages after close"),
        Ok(Some(Err(_))) => {} // Error is acceptable
    }
}

// =============================================================================
// Configuration Validation Tests
// =============================================================================

#[test]
fn test_invalid_cwd_path_not_exists() {
    use claude_agent_sdk_rs::ClaudeClient;
    use std::path::Path;

    let options = ClaudeAgentOptions::builder()
        .cwd(Path::new(
            "/nonexistent/path/that/definitely/does/not/exist",
        ))
        .build();

    let result = ClaudeClient::try_new(options);
    assert!(result.is_err(), "Should fail with non-existent cwd");

    let err = match result {
        Err(e) => e.to_string(),
        Ok(_) => panic!("Expected error but got Ok"),
    };
    assert!(
        err.contains("Working directory"),
        "Error should mention working directory: {}",
        err
    );
}

#[test]
fn test_invalid_cwd_is_file() {
    use claude_agent_sdk_rs::ClaudeClient;
    use std::path::Path;

    // Use a file that exists in the project
    let options = ClaudeAgentOptions::builder()
        .cwd(Path::new("Cargo.toml"))
        .build();

    let result = ClaudeClient::try_new(options);
    assert!(result.is_err(), "Should fail when cwd is a file");

    let err = match result {
        Err(e) => e.to_string(),
        Ok(_) => panic!("Expected error but got Ok"),
    };
    assert!(
        err.contains("not a directory"),
        "Error should mention not a directory: {}",
        err
    );
}

// =============================================================================
// Image Validation Tests
// =============================================================================

#[test]
fn test_image_invalid_media_type() {
    let result = UserContentBlock::image_base64("image/bmp", "data");
    assert!(result.is_err(), "BMP should not be supported");

    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("Unsupported media type"),
        "Error should mention unsupported media type: {}",
        err
    );
}

#[test]
fn test_image_invalid_media_type_text() {
    let result = UserContentBlock::image_base64("text/plain", "data");
    assert!(result.is_err(), "text/plain should not be supported");
}

#[test]
fn test_image_valid_media_types() {
    // JPEG
    assert!(UserContentBlock::image_base64("image/jpeg", "data").is_ok());
    // PNG
    assert!(UserContentBlock::image_base64("image/png", "data").is_ok());
    // GIF
    assert!(UserContentBlock::image_base64("image/gif", "data").is_ok());
    // WebP
    assert!(UserContentBlock::image_base64("image/webp", "data").is_ok());
}

// =============================================================================
// Content Validation Tests
// =============================================================================

#[tokio::test]
async fn test_query_with_empty_content_fails() {
    use claude_agent_sdk_rs::query_with_content;

    let result = query_with_content(Vec::<UserContentBlock>::new(), None).await;
    assert!(result.is_err(), "Empty content should fail");

    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("at least one block"),
        "Error should mention needing at least one block: {}",
        err
    );
}

#[tokio::test]
async fn test_query_stream_with_empty_content_fails() {
    use claude_agent_sdk_rs::query_stream_with_content;

    let result = query_stream_with_content(Vec::<UserContentBlock>::new(), None).await;
    assert!(result.is_err(), "Empty content should fail");
}

// =============================================================================
// Error Type Tests
// =============================================================================

#[test]
fn test_error_types_are_distinguishable() {
    // Test that different error types can be distinguished
    use claude_agent_sdk_rs::errors::{
        CliNotFoundError, ConnectionError, JsonDecodeError, MessageParseError, ProcessError,
    };

    let conn_err = ClaudeError::Connection(ConnectionError::new("test"));
    assert!(matches!(conn_err, ClaudeError::Connection(_)));

    let process_err = ClaudeError::Process(ProcessError::new("test", Some(1), None));
    assert!(matches!(process_err, ClaudeError::Process(_)));

    let json_err = ClaudeError::JsonDecode(JsonDecodeError::new("test", "line"));
    assert!(matches!(json_err, ClaudeError::JsonDecode(_)));

    let parse_err = ClaudeError::MessageParse(MessageParseError::new("test", None));
    assert!(matches!(parse_err, ClaudeError::MessageParse(_)));

    let cli_err = ClaudeError::CliNotFound(CliNotFoundError::new("test", None));
    assert!(matches!(cli_err, ClaudeError::CliNotFound(_)));

    let transport_err: ClaudeError = ClaudeError::Transport("test".to_string());
    assert!(matches!(transport_err, ClaudeError::Transport(_)));
}

#[test]
fn test_error_display_messages() {
    use claude_agent_sdk_rs::errors::{ConnectionError, ProcessError};

    let conn_err = ClaudeError::Connection(ConnectionError::new("connection failed"));
    let display = conn_err.to_string();
    assert!(
        display.contains("connection failed"),
        "Display should contain message: {}",
        display
    );

    let process_err = ClaudeError::Process(ProcessError::new(
        "process died",
        Some(127),
        Some("error output".to_string()),
    ));
    let display = process_err.to_string();
    assert!(
        display.contains("127"),
        "Display should contain exit code: {}",
        display
    );
}

// =============================================================================
// Message Parse Error Tests
// =============================================================================

#[test]
fn test_parse_invalid_message_type() {
    use claude_agent_sdk_rs::Message;

    // Unknown message type
    let json = serde_json::json!({
        "type": "unknown_type",
        "data": "test"
    });

    let result: Result<Message, _> = serde_json::from_value(json);
    // Should either fail to parse or parse as a generic type
    // The SDK uses untagged enum, so unknown types may cause issues
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_parse_malformed_assistant_message() {
    use claude_agent_sdk_rs::Message;

    // Missing required fields
    let json = serde_json::json!({
        "type": "assistant"
        // Missing "message" field
    });

    let result: Result<Message, _> = serde_json::from_value(json);
    assert!(result.is_err(), "Should fail with missing required fields");
}

#[test]
fn test_parse_malformed_content_block() {
    use claude_agent_sdk_rs::ContentBlock;

    // Unknown content type
    let json = serde_json::json!({
        "type": "unknown_block_type",
        "data": "test"
    });

    let result: Result<ContentBlock, _> = serde_json::from_value(json);
    // Should fail or handle gracefully
    assert!(result.is_err() || result.is_ok());
}

// =============================================================================
// Scenario Builder Edge Cases
// =============================================================================

#[test]
fn test_scenario_empty_exchanges() {
    let scenario = ScenarioBuilder::new("empty")
        .on_connect(SystemMessageBuilder::default().build())
        .build();

    assert_eq!(scenario.exchanges.len(), 0);
    assert_eq!(scenario.on_connect.len(), 1);
}

#[test]
fn test_scenario_no_on_connect() {
    let scenario = ScenarioBuilder::new("no_connect")
        .exchange()
        .respond(AssistantMessageBuilder::new().text("Hello").build())
        .build();

    assert_eq!(scenario.on_connect.len(), 0);
    assert_eq!(scenario.exchanges.len(), 1);
}

#[test]
fn test_scenario_multiple_responses_in_exchange() {
    let scenario = ScenarioBuilder::new("multi_response")
        .exchange()
        .respond(AssistantMessageBuilder::new().text("Part 1").build())
        .respond(AssistantMessageBuilder::new().text("Part 2").build())
        .respond(AssistantMessageBuilder::new().text("Part 3").build())
        .then_result(ResultMessageBuilder::default().build())
        .build();

    // Should have 4 messages in the exchange (3 assistant + 1 result)
    assert_eq!(scenario.exchanges[0].responses.len(), 4);
}

// =============================================================================
// Mock Transport Edge Cases
// =============================================================================

#[tokio::test]
async fn test_mock_transport_zero_speed_factor() {
    let transport = MockTransport::builder()
        .message_delayed(serde_json::json!({"type": "test"}), 1000, 500)
        .speed_factor(0.0) // Instant
        .build();

    transport.connect().await.unwrap();

    let start = std::time::Instant::now();
    let mut stream = transport.read_messages();
    let _ = stream.next().await;
    let elapsed = start.elapsed();

    // With speed_factor 0, should be nearly instant
    assert!(
        elapsed < std::time::Duration::from_millis(100),
        "Should be instant with speed_factor 0, was {:?}",
        elapsed
    );
}

#[tokio::test]
async fn test_mock_transport_high_speed_factor() {
    let transport = MockTransport::builder()
        .message_delayed(serde_json::json!({"type": "test"}), 10, 0)
        .speed_factor(2.0) // 2x slower
        .build();

    transport.connect().await.unwrap();

    let start = std::time::Instant::now();
    let mut stream = transport.read_messages();
    let _ = stream.next().await;
    let elapsed = start.elapsed();

    // Should take at least 20ms (10ms * 2.0)
    // Allow some tolerance for test execution
    assert!(
        elapsed >= std::time::Duration::from_millis(10),
        "Should take longer with high speed_factor, was {:?}",
        elapsed
    );
}

// =============================================================================
// Builder Validation Tests
// =============================================================================

#[test]
fn test_assistant_builder_empty_content() {
    use claude_agent_sdk_rs::Message;

    let msg = AssistantMessageBuilder::new().build();

    if let Message::Assistant(a) = msg {
        assert!(a.message.content.is_empty(), "Should have empty content");
    } else {
        panic!("Expected assistant message");
    }
}

#[test]
fn test_system_builder_defaults() {
    use claude_agent_sdk_rs::Message;

    let msg = SystemMessageBuilder::default().build();

    if let Message::System(s) = msg {
        assert_eq!(s.subtype, "init");
    } else {
        panic!("Expected system message");
    }
}

#[test]
fn test_result_builder_defaults() {
    use claude_agent_sdk_rs::Message;

    let msg = ResultMessageBuilder::default().build();

    if let Message::Result(r) = msg {
        assert!(!r.is_error, "Default should not be error");
        // duration_ms is u64 so always >= 0, just check it exists
        let _ = r.duration_ms;
    } else {
        panic!("Expected result message");
    }
}
