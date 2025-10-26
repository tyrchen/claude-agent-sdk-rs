//! Integration tests for Claude Agent SDK
//!
//! These tests verify the SDK functionality end-to-end.
//! Note: Most tests are marked as #[ignore] by default since they require
//! a working Claude CLI installation and API access.
//!
//! ## Session ID Behavior
//!
//! When using `query_with_session()` or `new_session()`, the session_id parameter
//! is passed to Claude CLI, but the CLI may generate its own UUID session IDs.
//! Therefore, tests should NOT assert exact session_id matches. Instead, tests
//! should verify that:
//! 1. Messages are received successfully
//! 2. Session IDs are present and non-empty
//! 3. The API accepts session_id parameters without errors

use claude_agent_sdk_rs::{
    ClaudeAgentOptions, ClaudeClient, HookEvent, HookInput, HookJsonOutput, HookMatcher, Message,
    PermissionMode, SyncHookJsonOutput,
};
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::test]
#[ignore] // Requires Claude CLI
async fn test_basic_client_connection() -> anyhow::Result<()> {
    let options = ClaudeAgentOptions {
        max_turns: Some(1),
        ..Default::default()
    };

    let mut client = ClaudeClient::new(options);
    client.connect().await?;
    client.disconnect().await?;

    Ok(())
}

#[tokio::test]
#[ignore] // Requires Claude CLI
async fn test_simple_query_with_bash() -> anyhow::Result<()> {
    let options = ClaudeAgentOptions {
        allowed_tools: vec!["Bash".to_string()],
        permission_mode: Some(PermissionMode::BypassPermissions),
        max_turns: Some(3),
        ..Default::default()
    };

    let mut client = ClaudeClient::new(options);
    client.connect().await?;

    client.query("Run the command: echo 'test'").await?;

    let mut found_result = false;
    {
        let mut stream = client.receive_response();

        use futures::StreamExt;
        while let Some(message) = stream.next().await {
            let message = message?;
            if let Message::Result(result) = message {
                assert!(!result.is_error);
                found_result = true;
            }
        }
    }

    assert!(found_result, "Should receive a result message");

    client.disconnect().await?;
    Ok(())
}

#[tokio::test]
#[ignore] // Requires Claude CLI
async fn test_session_management() -> anyhow::Result<()> {
    let options = ClaudeAgentOptions {
        max_turns: Some(1),
        permission_mode: Some(PermissionMode::BypassPermissions),
        ..Default::default()
    };

    let mut client = ClaudeClient::new(options);
    client.connect().await?;

    // Query with different session IDs - Claude CLI may generate UUIDs instead
    // We just verify that we can send queries with session_id parameter
    client
        .query_with_session("What is 2 + 2?", "session-1")
        .await?;

    {
        let mut stream = client.receive_response();
        use futures::StreamExt;
        let mut found_session_1 = false;
        let mut session_1_id = String::new();
        while let Some(message) = stream.next().await {
            let message = message?;
            if let Message::Result(result) = message {
                session_1_id = result.session_id.clone();
                found_session_1 = true;
            }
        }
        assert!(found_session_1, "Should receive result for session-1");
        assert!(!session_1_id.is_empty(), "Session ID should not be empty");
    }

    // Different session should have different context
    client
        .query_with_session("What is 3 + 3?", "session-2")
        .await?;

    {
        let mut stream = client.receive_response();
        use futures::StreamExt;
        let mut found_session_2 = false;
        let mut session_2_id = String::new();
        while let Some(message) = stream.next().await {
            let message = message?;
            if let Message::Result(result) = message {
                session_2_id = result.session_id.clone();
                found_session_2 = true;
            }
        }
        assert!(found_session_2, "Should receive result for session-2");
        assert!(!session_2_id.is_empty(), "Session ID should not be empty");
    }

    client.disconnect().await?;
    Ok(())
}

#[tokio::test]
#[ignore] // Requires Claude CLI
async fn test_fork_session() -> anyhow::Result<()> {
    let options = ClaudeAgentOptions::builder()
        .fork_session(true)
        .max_turns(1)
        .permission_mode(PermissionMode::BypassPermissions)
        .build();

    let mut client = ClaudeClient::new(options);
    client.connect().await?;

    client.query("What is 2 + 2?").await?;

    {
        let mut stream = client.receive_response();
        use futures::StreamExt;
        let mut found_result = false;
        while let Some(message) = stream.next().await {
            let message = message?;
            if let Message::Result(_) = message {
                found_result = true;
            }
        }
        assert!(
            found_result,
            "Should receive a result with fork_session enabled"
        );
    }

    client.disconnect().await?;
    Ok(())
}

#[tokio::test]
#[ignore] // Requires Claude CLI
async fn test_new_session_convenience() -> anyhow::Result<()> {
    let options = ClaudeAgentOptions {
        max_turns: Some(1),
        permission_mode: Some(PermissionMode::BypassPermissions),
        ..Default::default()
    };

    let mut client = ClaudeClient::new(options);
    client.connect().await?;

    // Use convenience method - Claude CLI may generate UUID instead of using our session_id
    client.new_session("test-session", "Hello").await?;

    {
        let mut stream = client.receive_response();
        use futures::StreamExt;
        let mut found_result = false;
        let mut session_id = String::new();
        while let Some(message) = stream.next().await {
            let message = message?;
            if let Message::Result(result) = message {
                session_id = result.session_id.clone();
                found_result = true;
            }
        }
        assert!(found_result, "Should receive result for new session");
        assert!(!session_id.is_empty(), "Session ID should not be empty");
    }

    client.disconnect().await?;
    Ok(())
}

#[tokio::test]
#[ignore] // Requires Claude CLI
async fn test_hook_pretooluse() -> anyhow::Result<()> {
    let mut hooks: HashMap<HookEvent, Vec<HookMatcher>> = HashMap::new();

    // Add a PreToolUse hook that allows all tools
    hooks.insert(
        HookEvent::PreToolUse,
        vec![HookMatcher {
            matcher: None,
            hooks: vec![Arc::new(|_input, _tool_use_id, _context| {
                Box::pin(async {
                    HookJsonOutput::Sync(SyncHookJsonOutput {
                        continue_: Some(true),
                        ..Default::default()
                    })
                })
            })],
        }],
    );

    let options = ClaudeAgentOptions {
        allowed_tools: vec!["Bash".to_string()],
        permission_mode: Some(PermissionMode::BypassPermissions),
        hooks: Some(hooks),
        max_turns: Some(3),
        ..Default::default()
    };

    let mut client = ClaudeClient::new(options);
    client.connect().await?;

    client.query("Run echo 'hook test'").await?;

    let mut found_result = false;
    {
        let mut stream = client.receive_response();

        use futures::StreamExt;
        while let Some(message) = stream.next().await {
            let message = message?;
            if let Message::Result(_) = message {
                found_result = true;
            }
        }
    }

    assert!(found_result);

    client.disconnect().await?;
    Ok(())
}

#[tokio::test]
#[ignore] // Requires Claude CLI
async fn test_permission_mode_change() -> anyhow::Result<()> {
    let mut client = ClaudeClient::new(ClaudeAgentOptions::default());
    client.connect().await?;

    // Change permission mode dynamically
    client
        .set_permission_mode(PermissionMode::AcceptEdits)
        .await?;
    client.set_permission_mode(PermissionMode::Default).await?;

    client.disconnect().await?;
    Ok(())
}

#[tokio::test]
#[ignore] // Requires Claude CLI
async fn test_interrupt() -> anyhow::Result<()> {
    let options = ClaudeAgentOptions {
        max_turns: Some(10),
        ..Default::default()
    };

    let mut client = ClaudeClient::new(options);
    client.connect().await?;

    client.query("Count from 1 to 100").await?;

    // Give it a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Interrupt the execution
    client.interrupt().await?;

    client.disconnect().await?;
    Ok(())
}

#[tokio::test]
#[ignore] // Requires Claude CLI
async fn test_set_model() -> anyhow::Result<()> {
    let mut client = ClaudeClient::new(ClaudeAgentOptions::default());
    client.connect().await?;

    // Change model dynamically
    client.set_model(Some("claude-sonnet-4-5")).await?;
    client.set_model(None).await?; // Reset to default

    client.disconnect().await?;
    Ok(())
}

#[tokio::test]
#[ignore] // Requires Claude CLI
async fn test_get_server_info() -> anyhow::Result<()> {
    let mut client = ClaudeClient::new(ClaudeAgentOptions::default());
    client.connect().await?;

    // Get server info
    let info = client.get_server_info().await;
    assert!(info.is_some(), "Should have server info in streaming mode");

    if let Some(server_info) = info {
        // Should have some expected fields
        assert!(
            server_info.is_object(),
            "Server info should be a JSON object"
        );
    }

    client.disconnect().await?;
    Ok(())
}

/// Test that verifies hook input deserialization matches expected format
#[test]
fn test_hook_input_formats() {
    // PreToolUse
    let json = serde_json::json!({
        "hook_event_name": "PreToolUse",
        "session_id": "test",
        "transcript_path": "/path",
        "cwd": "/cwd",
        "tool_name": "Bash",
        "tool_input": {"command": "test"}
    });
    let input: HookInput = serde_json::from_value(json).unwrap();
    assert!(matches!(input, HookInput::PreToolUse(_)));

    // Stop with stop_hook_active
    let json = serde_json::json!({
        "hook_event_name": "Stop",
        "session_id": "test",
        "transcript_path": "/path",
        "cwd": "/cwd",
        "stop_hook_active": true
    });
    let input: HookInput = serde_json::from_value(json).unwrap();
    if let HookInput::Stop(stop) = input {
        assert!(stop.stop_hook_active);
    } else {
        panic!("Expected Stop hook input");
    }

    // PreCompact with trigger and custom_instructions
    let json = serde_json::json!({
        "hook_event_name": "PreCompact",
        "session_id": "test",
        "transcript_path": "/path",
        "cwd": "/cwd",
        "trigger": "manual",
        "custom_instructions": "Keep important parts"
    });
    let input: HookInput = serde_json::from_value(json).unwrap();
    if let HookInput::PreCompact(precompact) = input {
        assert_eq!(precompact.trigger, "manual");
        assert_eq!(
            precompact.custom_instructions,
            Some("Keep important parts".to_string())
        );
    } else {
        panic!("Expected PreCompact hook input");
    }
}

/// Test permission mode serialization format
#[test]
fn test_permission_mode_serialization() {
    assert_eq!(
        serde_json::to_string(&PermissionMode::AcceptEdits).unwrap(),
        "\"acceptEdits\""
    );
    assert_eq!(
        serde_json::to_string(&PermissionMode::BypassPermissions).unwrap(),
        "\"bypassPermissions\""
    );
}

/// Test that message types can be deserialized from CLI output
#[test]
fn test_message_deserialization() {
    let assistant_json = serde_json::json!({
        "type": "assistant",
        "message": {
            "content": [
                {"type": "text", "text": "Hello"}
            ]
        },
        "session_id": "test"
    });

    let msg: Message = serde_json::from_value(assistant_json).unwrap();
    assert!(matches!(msg, Message::Assistant(_)));

    let result_json = serde_json::json!({
        "type": "result",
        "subtype": "query_complete",
        "duration_ms": 1000,
        "duration_api_ms": 800,
        "is_error": false,
        "num_turns": 2,
        "session_id": "test"
    });

    let msg: Message = serde_json::from_value(result_json).unwrap();
    assert!(matches!(msg, Message::Result(_)));
}
