//! Permission recorder for testing

use futures::future::BoxFuture;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::types::permissions::{
    CanUseToolCallback, PermissionResult, PermissionResultAllow, PermissionResultDeny,
    ToolPermissionContext,
};

/// A recorded permission decision
#[derive(Debug, Clone)]
pub struct PermissionDecision {
    /// Tool name
    pub tool_name: String,
    /// Tool input
    pub input: serde_json::Value,
    /// Permission context
    pub context: ToolPermissionContext,
    /// The result returned
    pub result: PermissionResult,
    /// Timestamp
    pub timestamp: std::time::Instant,
}

/// Records and controls permission decisions for testing
#[derive(Clone)]
pub struct PermissionRecorder {
    decisions: Arc<Mutex<Vec<PermissionDecision>>>,
    default_response: PermissionResult,
    tool_responses: Arc<Mutex<HashMap<String, PermissionResult>>>,
}

impl PermissionRecorder {
    /// Create with default allow behavior
    pub fn allow_all() -> Self {
        Self {
            decisions: Arc::new(Mutex::new(Vec::new())),
            default_response: PermissionResult::Allow(PermissionResultAllow {
                updated_input: None,
                updated_permissions: None,
            }),
            tool_responses: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create with default deny behavior
    pub fn deny_all() -> Self {
        Self {
            decisions: Arc::new(Mutex::new(Vec::new())),
            default_response: PermissionResult::Deny(PermissionResultDeny {
                message: "Denied by test".into(),
                interrupt: false,
            }),
            tool_responses: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Allow specific tools
    pub fn allow_tools(tools: &[&str]) -> Self {
        let mut tool_responses = HashMap::new();
        for tool in tools {
            tool_responses.insert(
                tool.to_string(),
                PermissionResult::Allow(PermissionResultAllow {
                    updated_input: None,
                    updated_permissions: None,
                }),
            );
        }

        Self {
            decisions: Arc::new(Mutex::new(Vec::new())),
            default_response: PermissionResult::Deny(PermissionResultDeny {
                message: "Not in allow list".into(),
                interrupt: false,
            }),
            tool_responses: Arc::new(Mutex::new(tool_responses)),
        }
    }

    /// Set response for a specific tool
    pub async fn set_response(&self, tool: &str, response: PermissionResult) {
        self.tool_responses
            .lock()
            .await
            .insert(tool.to_string(), response);
    }

    /// Get as callback for ClaudeAgentOptions
    pub fn as_callback(&self) -> CanUseToolCallback {
        let decisions = Arc::clone(&self.decisions);
        let default_response = self.default_response.clone();
        let tool_responses = Arc::clone(&self.tool_responses);

        Arc::new(
            move |tool_name: String,
                  input: serde_json::Value,
                  context: ToolPermissionContext|
                  -> BoxFuture<'static, PermissionResult> {
                let decisions = Arc::clone(&decisions);
                let default_response = default_response.clone();
                let tool_responses = Arc::clone(&tool_responses);
                let tool_name_clone = tool_name.clone();
                let input_clone = input.clone();
                let context_clone = context.clone();

                Box::pin(async move {
                    let result = {
                        let responses = tool_responses.lock().await;
                        responses
                            .get(&tool_name_clone)
                            .cloned()
                            .unwrap_or(default_response)
                    };

                    decisions.lock().await.push(PermissionDecision {
                        tool_name: tool_name_clone,
                        input: input_clone,
                        context: context_clone,
                        result: result.clone(),
                        timestamp: std::time::Instant::now(),
                    });

                    result
                })
            },
        )
    }

    /// Get all decisions
    pub async fn decisions(&self) -> Vec<PermissionDecision> {
        self.decisions.lock().await.clone()
    }

    /// Assert permission was asked for tool
    pub async fn assert_asked(&self, tool_name: &str) {
        let decisions = self.decisions.lock().await;
        assert!(
            decisions.iter().any(|d| d.tool_name == tool_name),
            "Expected permission to be asked for '{}', but it wasn't. Asked for: {:?}",
            tool_name,
            decisions.iter().map(|d| &d.tool_name).collect::<Vec<_>>()
        );
    }

    /// Assert permission was NOT asked for tool
    pub async fn assert_not_asked(&self, tool_name: &str) {
        let decisions = self.decisions.lock().await;
        assert!(
            !decisions.iter().any(|d| d.tool_name == tool_name),
            "Expected permission to NOT be asked for '{}', but it was",
            tool_name
        );
    }

    /// Clear recorded decisions
    pub async fn clear(&self) {
        self.decisions.lock().await.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_permission_recorder_allow_all() {
        let recorder = PermissionRecorder::allow_all();
        let callback = recorder.as_callback();

        let result = callback(
            "Read".to_string(),
            serde_json::json!({"file_path": "/tmp/test"}),
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
    async fn test_permission_recorder_allow_specific() {
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
}
