//! Tests for permission control protocol (`can_use_tool`) handling.
//!
//! These tests use the mock framework to simulate the Claude Code CLI sending
//! `control_request` messages to the SDK. The SDK should reply with a
//! `control_response` containing a serialized `PermissionResult`.

use claude_agent_sdk_rs::ClaudeAgentOptions;
use claude_agent_sdk_rs::testing::{MockClient, MockTransport, PermissionRecorder};
use std::time::Duration;

async fn wait_for_written_json<F>(transport: &MockTransport, predicate: F) -> serde_json::Value
where
    F: Fn(&serde_json::Value) -> bool,
{
    tokio::time::timeout(Duration::from_secs(1), async {
        loop {
            let written = transport.written_messages_async().await;
            if let Some(value) = written
                .iter()
                .filter_map(|w| w.parsed.as_ref())
                .find(|json| predicate(json))
            {
                return value.clone();
            }

            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("timeout waiting for written message")
}

#[tokio::test]
async fn test_can_use_tool_control_request_invokes_callback_and_allows() {
    let recorder = PermissionRecorder::allow_all();
    let options = ClaudeAgentOptions::builder()
        .can_use_tool(recorder.as_callback())
        .build();

    let transport = MockTransport::builder().build();
    let mut client = MockClient::from_transport(transport, options);
    client.connect_with_transport().await.unwrap();

    client.transport().inject(serde_json::json!({
        "type": "control_request",
        "request_id": "req_permission_allow",
        "request": {
            "subtype": "can_use_tool",
            "tool_name": "Bash",
            "tool_input": {"command": "echo hello"},
            "suggestions": [
                {
                    "type": "addRules",
                    "rules": [{"toolName": "Bash", "ruleContent": "echo *"}],
                    "behavior": "allow",
                    "destination": "session"
                }
            ],
            "tool_use_id": "toolu_abc123"
        }
    }));

    let response = wait_for_written_json(client.transport(), |json| {
        json.get("type") == Some(&serde_json::json!("control_response"))
            && json["response"]["request_id"] == "req_permission_allow"
    })
    .await;

    assert_eq!(response["type"], "control_response");
    assert_eq!(response["response"]["subtype"], "success");
    assert_eq!(response["response"]["request_id"], "req_permission_allow");
    assert_eq!(response["response"]["response"]["behavior"], "allow");

    recorder.assert_asked("Bash").await;
    let decisions = recorder.decisions().await;
    assert_eq!(decisions.len(), 1);
    assert_eq!(
        decisions[0].context.tool_use_id.as_deref(),
        Some("toolu_abc123")
    );
    assert_eq!(decisions[0].context.suggestions.len(), 1);

    client.disconnect().await.unwrap();
}

#[tokio::test]
async fn test_can_use_tool_without_callback_denies_by_default() {
    let transport = MockTransport::builder().build();
    let mut client = MockClient::from_transport(transport, ClaudeAgentOptions::default());
    client.connect_with_transport().await.unwrap();

    client.transport().inject(serde_json::json!({
        "type": "control_request",
        "request_id": "req_permission_deny",
        "request": {
            "subtype": "can_use_tool",
            "tool_name": "Read",
            "input": {"file_path": "/tmp/test.txt"},
            "permission_suggestions": null,
            "blocked_path": null
        }
    }));

    let response = wait_for_written_json(client.transport(), |json| {
        json.get("type") == Some(&serde_json::json!("control_response"))
            && json["response"]["request_id"] == "req_permission_deny"
    })
    .await;

    assert_eq!(response["type"], "control_response");
    assert_eq!(response["response"]["subtype"], "success");
    assert_eq!(response["response"]["request_id"], "req_permission_deny");
    assert_eq!(response["response"]["response"]["behavior"], "deny");
    assert_eq!(
        response["response"]["response"]["message"],
        "No permission callback registered"
    );

    client.disconnect().await.unwrap();
}
