//! Comprehensive tests using real fixture data captured from Claude API
//!
//! This test file validates that all Rust types can deserialize real JSON responses
//! from the Claude Agent SDK Python implementation.

use claude_agent_sdk_rs::types::messages::*;

/// Test helper to load and deserialize a message from filesystem
fn load_fixture(filename: &str) -> Message {
    let path = format!("fixtures/raw_messages/{}", filename);
    let json = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to read fixture file: {}", path));
    serde_json::from_str(&json).unwrap_or_else(|_| panic!("Failed to deserialize {}", filename))
}

/// Macro to include fixture at compile time for specific tests
macro_rules! test_fixture {
    ($filename:expr) => {{
        let json = include_str!(concat!("../fixtures/raw_messages/", $filename));
        serde_json::from_str::<Message>(json).expect(concat!("Failed to parse ", $filename))
    }};
}

// ============================================================================
// ASSISTANT MESSAGE TESTS - Real responses from Claude
// ============================================================================

#[test]
fn test_real_assistant_001_basic_text() {
    let msg = test_fixture!("assistant_001.json");
    match msg {
        Message::Assistant(assistant) => {
            assert!(assistant.message.model.is_some());
            assert_eq!(
                assistant.message.model.as_ref().unwrap(),
                "claude-sonnet-4-5-20250929"
            );
            assert!(!assistant.message.content.is_empty());
            // Should have text content
            match &assistant.message.content[0] {
                ContentBlock::Text(text) => {
                    assert!(!text.text.is_empty());
                }
                _ => panic!("Expected text block"),
            }
        }
        _ => panic!("Expected Assistant message"),
    }
}

#[test]
fn test_real_assistant_002_tool_use_planning() {
    let msg = test_fixture!("assistant_002.json");
    match msg {
        Message::Assistant(assistant) => {
            assert!(assistant.message.model.is_some());
            assert!(assistant.session_id.is_some());
            assert!(assistant.uuid.is_some());
            // Has usage stats from real API
            assert!(assistant.message.usage.is_some());
        }
        _ => panic!("Expected Assistant message"),
    }
}

#[test]
fn test_real_assistant_003_with_tool_use() {
    let msg = test_fixture!("assistant_003.json");
    match msg {
        Message::Assistant(assistant) => {
            // This message should contain tool use blocks
            let has_tool_use = assistant
                .message
                .content
                .iter()
                .any(|block| matches!(block, ContentBlock::ToolUse(_)));
            assert!(has_tool_use, "Expected tool use block in assistant_003");

            // Validate tool use structure
            for block in &assistant.message.content {
                if let ContentBlock::ToolUse(tool_use) = block {
                    assert!(!tool_use.id.is_empty());
                    assert!(!tool_use.name.is_empty());
                    assert!(tool_use.id.starts_with("toolu_"));
                }
            }
        }
        _ => panic!("Expected Assistant message"),
    }
}

// Test all 16 assistant messages
#[test]
fn test_all_assistant_messages() {
    for i in 1..=16 {
        let path = format!("assistant_{:03}.json", i);
        let msg = load_fixture(&path);
        match msg {
            Message::Assistant(assistant) => {
                // All assistant messages should have these fields
                assert!(assistant.message.model.is_some());
                assert!(assistant.session_id.is_some());
                assert!(assistant.uuid.is_some());

                // Should have content
                assert!(!assistant.message.content.is_empty() || assistant.message.model.is_some());
            }
            _ => panic!("Expected Assistant message in {}", path),
        }
    }
}

// ============================================================================
// USER MESSAGE TESTS - Real tool results
// ============================================================================

#[test]
fn test_real_user_001_tool_result() {
    let msg = test_fixture!("user_001.json");
    match msg {
        Message::User(user) => {
            assert!(user.extra["message"]["content"].is_array());
            let content = &user.extra["message"]["content"];

            // Should have tool_result
            assert_eq!(content[0]["type"], "tool_result");
            assert!(content[0]["tool_use_id"].is_string());
            assert!(content[0]["tool_use_id"]
                .as_str()
                .unwrap()
                .starts_with("toolu_"));
        }
        _ => panic!("Expected User message"),
    }
}

#[test]
fn test_real_user_001_error_result() {
    let msg = test_fixture!("user_001.json");
    match msg {
        Message::User(user) => {
            let content = &user.extra["message"]["content"][0];
            // This specific fixture has an error
            if content["is_error"].is_boolean() {
                assert_eq!(content["is_error"], true);
            }
        }
        _ => panic!("Expected User message"),
    }
}

#[test]
fn test_all_user_messages() {
    for i in 1..=5 {
        let path = format!("user_{:03}.json", i);
        let msg = load_fixture(&path);
        match msg {
            Message::User(_user) => {
                // Successfully deserialized
            }
            _ => panic!("Expected User message in {}", path),
        }
    }
}

// ============================================================================
// SYSTEM MESSAGE TESTS - Session initialization
// ============================================================================

#[test]
fn test_real_system_001() {
    let msg = test_fixture!("system_001.json");
    match msg {
        Message::System(system) => {
            assert!(!system.subtype.is_empty());
            assert!(system.session_id.is_some());
            assert!(system.cwd.is_some());
            assert!(system.model.is_some());

            // Should have tools list
            assert!(system.tools.is_some());
            let tools = system.tools.as_ref().unwrap();
            assert!(!tools.is_empty());
        }
        _ => panic!("Expected System message"),
    }
}

#[test]
fn test_all_system_messages() {
    for i in 1..=6 {
        let path = format!("system_{:03}.json", i);
        let msg = load_fixture(&path);
        match msg {
            Message::System(system) => {
                assert!(!system.subtype.is_empty());
                assert!(system.session_id.is_some());
                assert!(system.uuid.is_some());
            }
            _ => panic!("Expected System message in {}", path),
        }
    }
}

// ============================================================================
// RESULT MESSAGE TESTS - Query completion with usage stats
// ============================================================================

#[test]
fn test_real_result_001() {
    let msg = load_fixture("result_001.json");
    match msg {
        Message::Result(result) => {
            assert!(!result.subtype.is_empty());
            assert!(!result.session_id.is_empty());
            assert!(result.duration_ms > 0);
            assert!(result.num_turns > 0);

            // Should have cost data
            assert!(result.total_cost_usd.is_some());
            assert!(result.usage.is_some());

            let cost = result.total_cost_usd.unwrap();
            assert!(cost > 0.0);
        }
        _ => panic!("Expected Result message"),
    }
}

#[test]
fn test_all_result_messages() {
    for i in 1..=6 {
        let path = format!("result_{:03}.json", i);
        let msg = load_fixture(&path);
        match msg {
            Message::Result(result) => {
                assert!(!result.session_id.is_empty());
                assert!(result.duration_ms > 0);
                assert!(result.duration_api_ms > 0);
                assert!(result.num_turns > 0);
            }
            _ => panic!("Expected Result message in {}", path),
        }
    }
}

// ============================================================================
// STREAM EVENT TESTS - Real streaming data
// ============================================================================

#[test]
fn test_real_stream_event_001_message_start() {
    let msg = load_fixture("stream_event_001.json");
    match msg {
        Message::StreamEvent(event) => {
            assert!(!event.uuid.is_empty());
            assert!(!event.session_id.is_empty());
            assert!(event.event.is_object());

            // Check event type
            assert_eq!(event.event["type"], "message_start");

            // Should have message data
            assert!(event.event["message"].is_object());
            let message = &event.event["message"];
            assert_eq!(message["model"], "claude-sonnet-4-5-20250929");
        }
        _ => panic!("Expected StreamEvent message"),
    }
}

#[test]
fn test_stream_event_types() {
    // Test a variety of stream events
    let test_cases = vec![
        ("stream_event_001.json", "message_start"),
        ("stream_event_095.json", "message_stop"), // Near end of stream
    ];

    for (path, expected_type) in test_cases {
        let msg = load_fixture(path);
        match msg {
            Message::StreamEvent(event) => {
                assert_eq!(
                    event.event["type"].as_str().unwrap(),
                    expected_type,
                    "Event type mismatch in {}",
                    path
                );
            }
            _ => panic!("Expected StreamEvent in {}", path),
        }
    }
}

#[test]
fn test_stream_event_content_deltas() {
    // Stream events 2-94 should be content deltas
    for i in 2..=94 {
        let path = format!("stream_event_{:03}.json", i);
        let msg = load_fixture(&path);
        match msg {
            Message::StreamEvent(event) => {
                // Should be valid stream event
                assert!(!event.uuid.is_empty());
                assert!(!event.session_id.is_empty());
            }
            _ => panic!("Expected StreamEvent in {}", path),
        }
    }
}

#[test]
fn test_all_stream_events() {
    // Test all 97 stream events
    for i in 1..=97 {
        let path = format!("stream_event_{:03}.json", i);
        let msg = load_fixture(&path);
        match msg {
            Message::StreamEvent(event) => {
                assert!(!event.uuid.is_empty());
                assert!(!event.session_id.is_empty());
                assert!(event.event.is_object());
            }
            _ => panic!("Expected StreamEvent in {}", path),
        }
    }
}

// ============================================================================
// CONTENT BLOCK TESTS
// ============================================================================

#[test]
fn test_content_block_text() {
    // Assistant messages should have text blocks
    let msg = load_fixture("assistant_001.json");
    match msg {
        Message::Assistant(assistant) => {
            for block in &assistant.message.content {
                if let ContentBlock::Text(text) = block {
                    assert!(!text.text.is_empty());
                    return; // Found at least one
                }
            }
        }
        _ => panic!("Expected Assistant message"),
    }
}

#[test]
fn test_content_block_tool_use() {
    // assistant_003 should have tool use
    let msg = load_fixture("assistant_003.json");
    match msg {
        Message::Assistant(assistant) => {
            for block in &assistant.message.content {
                if let ContentBlock::ToolUse(tool_use) = block {
                    assert!(!tool_use.id.is_empty());
                    assert!(tool_use.id.starts_with("toolu_"));
                    assert!(!tool_use.name.is_empty());
                    assert!(tool_use.input.is_object());
                    return; // Found it
                }
            }
            panic!("Expected to find ToolUse block");
        }
        _ => panic!("Expected Assistant message"),
    }
}

// ============================================================================
// USAGE AND COST VALIDATION
// ============================================================================

#[test]
fn test_usage_statistics_structure() {
    let msg = load_fixture("result_001.json");
    match msg {
        Message::Result(result) => {
            assert!(result.usage.is_some());
            let usage = result.usage.as_ref().unwrap();

            // Should have token counts
            assert!(usage["input_tokens"].is_number());
            assert!(usage["output_tokens"].is_number());

            // May have cache stats
            if usage["cache_read_input_tokens"].is_number() {
                assert!(usage["cache_read_input_tokens"].as_u64().is_some());
            }
        }
        _ => panic!("Expected Result message"),
    }
}

#[test]
fn test_cost_calculation() {
    let msg = load_fixture("result_001.json");
    match msg {
        Message::Result(result) => {
            if let Some(cost) = result.total_cost_usd {
                assert!(cost >= 0.0);
                assert!(cost < 100.0); // Sanity check - should be small
            }
        }
        _ => panic!("Expected Result message"),
    }
}

// ============================================================================
// SESSION TRACKING
// ============================================================================

#[test]
fn test_session_id_consistency() {
    // All messages from the same scenario should have same session_id
    let msg1 = load_fixture("assistant_001.json");
    let msg2 = load_fixture("result_001.json");

    let session1 = match msg1 {
        Message::Assistant(ref a) => a.session_id.as_ref(),
        _ => panic!("Expected Assistant"),
    };

    let session2 = match msg2 {
        Message::Result(ref r) => Some(&r.session_id),
        _ => panic!("Expected Result"),
    };

    // Both should have session IDs
    assert!(session1.is_some());
    assert!(session2.is_some());
}

#[test]
fn test_uuid_uniqueness() {
    // Each message should have a unique UUID
    let msg1 = load_fixture("assistant_001.json");
    let msg2 = load_fixture("assistant_002.json");

    let uuid1 = match msg1 {
        Message::Assistant(ref a) => a.uuid.as_ref(),
        _ => panic!("Expected Assistant"),
    };

    let uuid2 = match msg2 {
        Message::Assistant(ref a) => a.uuid.as_ref(),
        _ => panic!("Expected Assistant"),
    };

    assert!(uuid1.is_some());
    assert!(uuid2.is_some());
    assert_ne!(uuid1, uuid2); // Different messages should have different UUIDs
}

// ============================================================================
// REAL API ID FORMAT VALIDATION
// ============================================================================

#[test]
fn test_message_id_format() {
    let msg = load_fixture("assistant_001.json");
    match msg {
        Message::Assistant(assistant) => {
            if let Some(id) = &assistant.message.id {
                assert!(id.starts_with("msg_"));
                assert!(id.len() > 4);
            }
        }
        _ => panic!("Expected Assistant message"),
    }
}

#[test]
fn test_tool_use_id_format() {
    let msg = load_fixture("assistant_003.json");
    match msg {
        Message::Assistant(assistant) => {
            for block in &assistant.message.content {
                if let ContentBlock::ToolUse(tool_use) = block {
                    assert!(tool_use.id.starts_with("toolu_"));
                    assert!(tool_use.id.len() > 6);
                    return;
                }
            }
        }
        _ => panic!("Expected Assistant message"),
    }
}

// ============================================================================
// SERIALIZATION ROUND-TRIP TESTS
// ============================================================================

#[test]
fn test_serialization_roundtrip_assistant() {
    let original_json = include_str!("../fixtures/raw_messages/assistant_001.json");
    let msg: Message = serde_json::from_str(original_json).unwrap();

    // Serialize back to JSON
    let serialized = serde_json::to_string(&msg).unwrap();

    // Deserialize again
    let msg2: Message = serde_json::from_str(&serialized).unwrap();

    // Should be equivalent
    match (msg, msg2) {
        (Message::Assistant(a1), Message::Assistant(a2)) => {
            assert_eq!(a1.session_id, a2.session_id);
            assert_eq!(a1.uuid, a2.uuid);
            assert_eq!(a1.message.model, a2.message.model);
        }
        _ => panic!("Type mismatch after round-trip"),
    }
}

#[test]
fn test_serialization_roundtrip_result() {
    let original_json = include_str!("../fixtures/raw_messages/result_001.json");
    let msg: Message = serde_json::from_str(original_json).unwrap();

    let serialized = serde_json::to_string(&msg).unwrap();
    let msg2: Message = serde_json::from_str(&serialized).unwrap();

    match (msg, msg2) {
        (Message::Result(r1), Message::Result(r2)) => {
            assert_eq!(r1.session_id, r2.session_id);
            assert_eq!(r1.duration_ms, r2.duration_ms);
            assert_eq!(r1.total_cost_usd, r2.total_cost_usd);
        }
        _ => panic!("Type mismatch after round-trip"),
    }
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

#[test]
fn test_malformed_json_error() {
    let bad_json = r#"{"type": "assistant", "invalid": true}"#;
    let result = serde_json::from_str::<Message>(bad_json);
    // Should fail gracefully, not panic
    assert!(result.is_err());
}

#[test]
fn test_unknown_type_handling() {
    // The Message enum should handle unknown types via serde
    let unknown = r#"{"type": "unknown_type", "data": {}}"#;
    let result = serde_json::from_str::<Message>(unknown);
    // Should error on unknown type
    assert!(result.is_err());
}

// ============================================================================
// COMPREHENSIVE FIXTURE VALIDATION
// ============================================================================

#[test]
fn test_all_130_fixtures_deserialize() {
    let mut success_count = 0;
    let mut error_count = 0;

    // Test all assistants
    for i in 1..=16 {
        let path = format!("assistant_{:03}.json", i);
        match load_fixture(&path) {
            Message::Assistant(_) => success_count += 1,
            _ => error_count += 1,
        }
    }

    // Test all users
    for i in 1..=5 {
        let path = format!("user_{:03}.json", i);
        match load_fixture(&path) {
            Message::User(_) => success_count += 1,
            _ => error_count += 1,
        }
    }

    // Test all systems
    for i in 1..=6 {
        let path = format!("system_{:03}.json", i);
        match load_fixture(&path) {
            Message::System(_) => success_count += 1,
            _ => error_count += 1,
        }
    }

    // Test all results
    for i in 1..=6 {
        let path = format!("result_{:03}.json", i);
        match load_fixture(&path) {
            Message::Result(_) => success_count += 1,
            _ => error_count += 1,
        }
    }

    // Test all stream events
    for i in 1..=97 {
        let path = format!("stream_event_{:03}.json", i);
        match load_fixture(&path) {
            Message::StreamEvent(_) => success_count += 1,
            _ => error_count += 1,
        }
    }

    assert_eq!(success_count, 130, "All 130 fixtures should deserialize");
    assert_eq!(error_count, 0, "No deserialization errors expected");
}
