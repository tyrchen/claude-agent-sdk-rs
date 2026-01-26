//! Hook recorder for testing

use futures::future::BoxFuture;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::types::hooks::{
    HookCallback, HookContext, HookEvent, HookInput, HookJsonOutput, SyncHookJsonOutput,
};

/// A recorded hook invocation
#[derive(Debug, Clone)]
pub struct HookInvocation {
    /// The hook event type
    pub event: HookEvent,
    /// Tool name (if applicable)
    pub tool_name: Option<String>,
    /// Tool input (if applicable)
    pub input: Option<serde_json::Value>,
    /// Tool output (if applicable)
    pub output: Option<serde_json::Value>,
    /// Timestamp of invocation
    pub timestamp: std::time::Instant,
}

/// Records hook invocations for assertions
#[derive(Clone)]
pub struct HookRecorder {
    invocations: Arc<Mutex<Vec<HookInvocation>>>,
}

impl HookRecorder {
    /// Create a new hook recorder
    pub fn new() -> Self {
        Self {
            invocations: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a pre_tool_use hook callback
    pub fn pre_tool_use_callback(&self) -> HookCallback {
        let invocations = Arc::clone(&self.invocations);
        Arc::new(
            move |input: HookInput,
                  _tool_use_id: Option<String>,
                  _context: HookContext|
                  -> BoxFuture<'static, HookJsonOutput> {
                let invocations = Arc::clone(&invocations);
                Box::pin(async move {
                    let (tool_name, tool_input) = match &input {
                        HookInput::PreToolUse(pre) => {
                            (Some(pre.tool_name.clone()), Some(pre.tool_input.clone()))
                        }
                        _ => (None, None),
                    };

                    invocations.lock().await.push(HookInvocation {
                        event: HookEvent::PreToolUse,
                        tool_name,
                        input: tool_input,
                        output: None,
                        timestamp: std::time::Instant::now(),
                    });

                    HookJsonOutput::Sync(SyncHookJsonOutput {
                        continue_: Some(true),
                        suppress_output: None,
                        stop_reason: None,
                        decision: None,
                        system_message: None,
                        reason: None,
                        hook_specific_output: None,
                    })
                })
            },
        )
    }

    /// Create a post_tool_use hook callback
    pub fn post_tool_use_callback(&self) -> HookCallback {
        let invocations = Arc::clone(&self.invocations);
        Arc::new(
            move |input: HookInput,
                  _tool_use_id: Option<String>,
                  _context: HookContext|
                  -> BoxFuture<'static, HookJsonOutput> {
                let invocations = Arc::clone(&invocations);
                Box::pin(async move {
                    let (tool_name, tool_input, tool_output) = match &input {
                        HookInput::PostToolUse(post) => (
                            Some(post.tool_name.clone()),
                            Some(post.tool_input.clone()),
                            Some(post.tool_response.clone()),
                        ),
                        _ => (None, None, None),
                    };

                    invocations.lock().await.push(HookInvocation {
                        event: HookEvent::PostToolUse,
                        tool_name,
                        input: tool_input,
                        output: tool_output,
                        timestamp: std::time::Instant::now(),
                    });

                    HookJsonOutput::Sync(SyncHookJsonOutput {
                        continue_: Some(true),
                        suppress_output: None,
                        stop_reason: None,
                        decision: None,
                        system_message: None,
                        reason: None,
                        hook_specific_output: None,
                    })
                })
            },
        )
    }

    /// Get all invocations
    pub async fn invocations(&self) -> Vec<HookInvocation> {
        self.invocations.lock().await.clone()
    }

    /// Assert hook was called a specific number of times
    pub async fn assert_called(&self, event: HookEvent, times: usize) {
        let invocations = self.invocations.lock().await;
        let count = invocations.iter().filter(|i| i.event == event).count();
        assert_eq!(
            count, times,
            "Expected {:?} to be called {} times, but was called {} times",
            event, times, count
        );
    }

    /// Assert a specific tool was used
    pub async fn assert_tool_used(&self, tool_name: &str) {
        let invocations = self.invocations.lock().await;
        assert!(
            invocations
                .iter()
                .any(|i| i.tool_name.as_deref() == Some(tool_name)),
            "Expected tool '{}' to be used, but it wasn't. Used tools: {:?}",
            tool_name,
            invocations
                .iter()
                .filter_map(|i| i.tool_name.as_ref())
                .collect::<Vec<_>>()
        );
    }

    /// Assert tool was not used
    pub async fn assert_tool_not_used(&self, tool_name: &str) {
        let invocations = self.invocations.lock().await;
        assert!(
            !invocations
                .iter()
                .any(|i| i.tool_name.as_deref() == Some(tool_name)),
            "Expected tool '{}' to NOT be used, but it was",
            tool_name
        );
    }

    /// Clear recorded invocations
    pub async fn clear(&self) {
        self.invocations.lock().await.clear();
    }
}

impl Default for HookRecorder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hook_recorder() {
        let recorder = HookRecorder::new();

        // Simulate recording
        {
            let mut invocations = recorder.invocations.lock().await;
            invocations.push(HookInvocation {
                event: HookEvent::PreToolUse,
                tool_name: Some("Read".to_string()),
                input: Some(serde_json::json!({"file_path": "/tmp/test"})),
                output: None,
                timestamp: std::time::Instant::now(),
            });
        }

        recorder.assert_called(HookEvent::PreToolUse, 1).await;
        recorder.assert_tool_used("Read").await;
    }
}
