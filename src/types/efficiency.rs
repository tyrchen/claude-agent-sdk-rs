//! Efficiency tracking and hook injection for Claude Agent SDK
//!
//! This module provides built-in hooks that help improve agent execution efficiency by:
//! - Tracking execution metrics (tool calls, edits per file, etc.)
//! - Injecting working directory reminders at prompt submission
//! - Providing efficiency feedback with specific warnings when execution stops

use futures::future::BoxFuture;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::sync::RwLock;

use super::hooks::{
    HookCallback, HookContext, HookEvent, HookInput, HookJsonOutput, HookMatcher,
    HookSpecificOutput, PostToolUseHookInput, StopHookInput, SyncHookJsonOutput,
    UserPromptSubmitHookInput, UserPromptSubmitHookSpecificOutput,
};

/// Execution metrics collected during agent execution
///
/// These metrics help identify efficiency issues like fragmented edits,
/// excessive directory checks, and build failures.
#[derive(Debug, Default)]
pub struct ExecutionMetrics {
    /// Total tool calls by tool name
    tool_calls: RwLock<HashMap<String, u32>>,
    /// Edit calls per file path (to detect fragmented edits)
    edit_calls_per_file: RwLock<HashMap<String, u32>>,
    /// Number of pwd/cd related bash commands
    directory_checks: AtomicU32,
    /// Number of build/test attempts
    build_attempts: AtomicU32,
    /// Number of TodoWrite calls
    todo_write_calls: AtomicU32,
}

impl ExecutionMetrics {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a tool call
    pub async fn record_tool_call(&self, tool_name: &str, tool_input: &serde_json::Value) {
        // Increment tool call count
        {
            let mut calls = self.tool_calls.write().await;
            *calls.entry(tool_name.to_string()).or_insert(0) += 1;
        }

        // Track specific patterns
        match tool_name {
            "Edit" => {
                if let Some(file_path) = tool_input.get("file_path").and_then(|v| v.as_str()) {
                    let mut edits = self.edit_calls_per_file.write().await;
                    *edits.entry(file_path.to_string()).or_insert(0) += 1;
                }
            }
            "Bash" => {
                if let Some(command) = tool_input.get("command").and_then(|v| v.as_str()) {
                    let cmd_lower = command.to_lowercase();
                    // Detect directory confusion commands
                    if cmd_lower.starts_with("pwd")
                        || cmd_lower.contains("&& pwd")
                        || (cmd_lower.starts_with("cd ") && !cmd_lower.contains("&&"))
                    {
                        self.directory_checks.fetch_add(1, Ordering::Relaxed);
                    }
                    // Detect build/test attempts
                    if cmd_lower.contains("npm run build")
                        || cmd_lower.contains("npm test")
                        || cmd_lower.contains("cargo build")
                        || cmd_lower.contains("cargo test")
                        || cmd_lower.contains("make build")
                        || cmd_lower.contains("make test")
                        || cmd_lower.contains("pytest")
                        || cmd_lower.contains("go build")
                        || cmd_lower.contains("go test")
                    {
                        self.build_attempts.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
            "TodoWrite" => {
                self.todo_write_calls.fetch_add(1, Ordering::Relaxed);
            }
            _ => {}
        }
    }

    /// Generate efficiency warnings based on collected metrics
    pub async fn generate_warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check for fragmented edits (>3 edits to same file)
        let edits = self.edit_calls_per_file.read().await;
        let fragmented_files: Vec<_> = edits
            .iter()
            .filter(|&(_, count)| *count > 3)
            .map(|(file, count)| format!("  - {} ({} edits)", file, count))
            .collect();
        if !fragmented_files.is_empty() {
            warnings.push(format!(
                "FRAGMENTED EDITS: Consider batching edits to same file:\n{}",
                fragmented_files.join("\n")
            ));
        }

        // Check for directory confusion
        let dir_checks = self.directory_checks.load(Ordering::Relaxed);
        if dir_checks > 2 {
            warnings.push(format!(
                "DIRECTORY CONFUSION: {} pwd/cd commands detected. Track directory state mentally, use absolute paths.",
                dir_checks
            ));
        }

        // Check for excessive build attempts
        let builds = self.build_attempts.load(Ordering::Relaxed);
        if builds > 2 {
            warnings.push(format!(
                "BUILD ITERATIONS: {} build/test attempts. Pre-validate code before building (check types, extensions, imports).",
                builds
            ));
        }

        // Check for excessive TodoWrite
        let todos = self.todo_write_calls.load(Ordering::Relaxed);
        if todos > 8 {
            warnings.push(format!(
                "EXCESSIVE TODO UPDATES: {} TodoWrite calls. Update only at phase boundaries, not every small step.",
                todos
            ));
        }

        warnings
    }

    /// Get summary statistics
    pub async fn get_summary(&self) -> MetricsSummary {
        let tool_calls = self.tool_calls.read().await;
        let total_tool_calls: u32 = tool_calls.values().sum();

        let edits = self.edit_calls_per_file.read().await;
        let total_edits: u32 = edits.values().sum();
        let files_with_multiple_edits = edits.values().filter(|&&c| c > 1).count() as u32;

        MetricsSummary {
            total_tool_calls,
            total_edits,
            files_with_multiple_edits,
            directory_checks: self.directory_checks.load(Ordering::Relaxed),
            build_attempts: self.build_attempts.load(Ordering::Relaxed),
            todo_write_calls: self.todo_write_calls.load(Ordering::Relaxed),
        }
    }
}

/// Summary of execution metrics
#[derive(Debug, Clone)]
pub struct MetricsSummary {
    pub total_tool_calls: u32,
    pub total_edits: u32,
    pub files_with_multiple_edits: u32,
    pub directory_checks: u32,
    pub build_attempts: u32,
    pub todo_write_calls: u32,
}

/// Configuration for efficiency-related features
#[derive(Clone, Default)]
pub struct EfficiencyConfig {
    /// Inject working directory reminder in UserPromptSubmit hook
    pub inject_cwd_reminder: bool,
    /// Inject efficiency tips in Stop hook
    pub inject_stop_tips: bool,
    /// Track execution metrics and provide specific warnings
    pub track_metrics: bool,
    /// Working directory to remind about (uses cwd from options if not set)
    pub cwd: Option<PathBuf>,
    /// Shared metrics collector (created internally when track_metrics is true)
    metrics: Option<Arc<ExecutionMetrics>>,
}

impl std::fmt::Debug for EfficiencyConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EfficiencyConfig")
            .field("inject_cwd_reminder", &self.inject_cwd_reminder)
            .field("inject_stop_tips", &self.inject_stop_tips)
            .field("track_metrics", &self.track_metrics)
            .field("cwd", &self.cwd)
            .field("metrics", &self.metrics.is_some())
            .finish()
    }
}

impl EfficiencyConfig {
    /// Create a new efficiency config with all features enabled
    pub fn enabled() -> Self {
        Self {
            inject_cwd_reminder: true,
            inject_stop_tips: true,
            track_metrics: true,
            cwd: None,
            metrics: Some(Arc::new(ExecutionMetrics::new())),
        }
    }

    /// Create a new efficiency config with only CWD reminder enabled
    pub fn cwd_reminder_only() -> Self {
        Self {
            inject_cwd_reminder: true,
            inject_stop_tips: false,
            track_metrics: false,
            cwd: None,
            metrics: None,
        }
    }

    /// Create a new efficiency config with only stop tips enabled
    pub fn stop_tips_only() -> Self {
        Self {
            inject_cwd_reminder: false,
            inject_stop_tips: true,
            track_metrics: false,
            cwd: None,
            metrics: None,
        }
    }

    /// Create a new efficiency config with metrics tracking enabled
    pub fn with_metrics() -> Self {
        Self {
            inject_cwd_reminder: false,
            inject_stop_tips: true,
            track_metrics: true,
            cwd: None,
            metrics: Some(Arc::new(ExecutionMetrics::new())),
        }
    }

    /// Set the working directory for reminders
    pub fn with_cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    /// Enable metrics tracking
    pub fn enable_metrics(mut self) -> Self {
        self.track_metrics = true;
        if self.metrics.is_none() {
            self.metrics = Some(Arc::new(ExecutionMetrics::new()));
        }
        self
    }

    /// Get the metrics collector (if tracking is enabled)
    pub fn metrics(&self) -> Option<Arc<ExecutionMetrics>> {
        self.metrics.clone()
    }
}

/// Build efficiency hooks based on configuration
///
/// Returns a HashMap of hook events to matchers that can be merged with user-defined hooks.
pub fn build_efficiency_hooks(config: &EfficiencyConfig) -> HashMap<HookEvent, Vec<HookMatcher>> {
    let mut hooks: HashMap<HookEvent, Vec<HookMatcher>> = HashMap::new();

    if config.inject_cwd_reminder {
        let cwd = config.cwd.clone();
        let callback = create_user_prompt_submit_hook(cwd);
        hooks.insert(
            HookEvent::UserPromptSubmit,
            vec![HookMatcher {
                matcher: None,
                hooks: vec![callback],
                timeout: None,
            }],
        );
    }

    // Add PostToolUse hook for metrics collection
    if config.track_metrics && config.metrics.is_some() {
        let metrics = config.metrics.as_ref().unwrap();
        let callback = create_post_tool_use_hook(Arc::clone(metrics));
        hooks.insert(
            HookEvent::PostToolUse,
            vec![HookMatcher {
                matcher: None,
                hooks: vec![callback],
                timeout: None,
            }],
        );
    }

    if config.inject_stop_tips {
        let metrics = config.metrics.clone();
        let callback = create_stop_hook(metrics);
        hooks.insert(
            HookEvent::Stop,
            vec![HookMatcher {
                matcher: None,
                hooks: vec![callback],
                timeout: None,
            }],
        );
    }

    hooks
}

/// Merge efficiency hooks with user-defined hooks
///
/// User-defined hooks take precedence - efficiency hooks are added only for events
/// that don't already have user-defined hooks.
pub fn merge_hooks(
    user_hooks: Option<HashMap<HookEvent, Vec<HookMatcher>>>,
    efficiency_hooks: HashMap<HookEvent, Vec<HookMatcher>>,
) -> Option<HashMap<HookEvent, Vec<HookMatcher>>> {
    if efficiency_hooks.is_empty() {
        return user_hooks;
    }

    let mut merged = user_hooks.unwrap_or_default();

    for (event, matchers) in efficiency_hooks {
        merged.entry(event).or_default().extend(matchers);
    }

    if merged.is_empty() {
        None
    } else {
        Some(merged)
    }
}

/// Create the PostToolUse hook for metrics collection
fn create_post_tool_use_hook(metrics: Arc<ExecutionMetrics>) -> HookCallback {
    Arc::new(
        move |input: HookInput, _tool_use_id: Option<String>, _context: HookContext| {
            let metrics = Arc::clone(&metrics);
            Box::pin(async move {
                if let HookInput::PostToolUse(PostToolUseHookInput {
                    tool_name,
                    tool_input,
                    ..
                }) = input
                {
                    metrics.record_tool_call(&tool_name, &tool_input).await;
                }

                // Don't modify the response, just collect metrics
                HookJsonOutput::Sync(SyncHookJsonOutput::default())
            }) as BoxFuture<'static, HookJsonOutput>
        },
    )
}

/// Create the UserPromptSubmit hook for CWD reminder injection
fn create_user_prompt_submit_hook(cwd: Option<PathBuf>) -> HookCallback {
    Arc::new(
        move |input: HookInput, _tool_use_id: Option<String>, _context: HookContext| {
            let cwd = cwd.clone();
            Box::pin(async move {
                let actual_cwd = match &input {
                    HookInput::UserPromptSubmit(UserPromptSubmitHookInput { cwd, .. }) => {
                        cwd.clone()
                    }
                    _ => return HookJsonOutput::Sync(SyncHookJsonOutput::default()),
                };

                // Use provided cwd or fall back to the one from hook input
                let display_cwd = cwd.map(|p| p.display().to_string()).unwrap_or(actual_cwd);

                let reminder = format!(
                    r#"<system-efficiency-reminder>
Working directory: {}

EFFICIENCY REMINDERS:
- Use absolute paths or track directory state - avoid pwd
- Batch multiple edits to same file in one Edit call
- Write code correctly the first time (check extensions, mock syntax, types)
- Update TodoWrite only at phase boundaries, not every small step
- Parallel tool calls: Read/Grep multiple files in single turn
</system-efficiency-reminder>"#,
                    display_cwd
                );

                HookJsonOutput::Sync(SyncHookJsonOutput {
                    continue_: Some(true),
                    hook_specific_output: Some(HookSpecificOutput::UserPromptSubmit(
                        UserPromptSubmitHookSpecificOutput {
                            additional_context: Some(reminder),
                        },
                    )),
                    ..Default::default()
                })
            }) as BoxFuture<'static, HookJsonOutput>
        },
    )
}

/// Create the Stop hook for efficiency feedback injection
fn create_stop_hook(metrics: Option<Arc<ExecutionMetrics>>) -> HookCallback {
    Arc::new(
        move |input: HookInput, _tool_use_id: Option<String>, _context: HookContext| {
            let metrics = metrics.clone();
            Box::pin(async move {
                // Verify this is actually a Stop hook input
                if !matches!(input, HookInput::Stop(StopHookInput { .. })) {
                    return HookJsonOutput::Sync(SyncHookJsonOutput::default());
                }

                // Generate feedback based on metrics if available
                let feedback = if let Some(metrics) = metrics {
                    let warnings = metrics.generate_warnings().await;
                    let summary = metrics.get_summary().await;

                    if warnings.is_empty() {
                        // No specific warnings, provide general tips
                        format!(
                            r#"<system-efficiency-feedback>
EXECUTION COMPLETE - Stats: {} tool calls, {} edits, {} build attempts

No major efficiency issues detected. Keep following these principles:
- Batch Read/Grep calls for independent files in single turn
- Combine multiple edits to same file in one Edit call
- Update TodoWrite only at phase boundaries
</system-efficiency-feedback>"#,
                            summary.total_tool_calls, summary.total_edits, summary.build_attempts
                        )
                    } else {
                        // Provide specific warnings
                        format!(
                            r#"<system-efficiency-feedback>
EXECUTION COMPLETE - Stats: {} tool calls, {} edits, {} build attempts

EFFICIENCY WARNINGS DETECTED:

{}

Addressing these issues could reduce turns by 30-50% in future executions.
</system-efficiency-feedback>"#,
                            summary.total_tool_calls,
                            summary.total_edits,
                            summary.build_attempts,
                            warnings.join("\n\n")
                        )
                    }
                } else {
                    // No metrics, provide general tips
                    r#"<system-efficiency-feedback>
EXECUTION COMPLETE - Review these efficiency principles for next time:

1. CODE QUALITY: Write correct code first time
   - Check file extensions (.ts vs .tsx for JSX)
   - Verify mock syntax matches framework (vi.mock vs jest.mock)
   - Complete type definitions before implementation

2. TOOL EFFICIENCY:
   - Batch Read/Grep calls for independent files in single turn
   - Combine multiple edits to same file in one Edit call
   - Use absolute paths, avoid pwd/cd confusion

3. PROGRESS TRACKING:
   - TodoWrite only at phase boundaries (start, phase complete, end)
   - Not after every small edit or file read

Following these principles can reduce turns by 50% and costs by 40%.
</system-efficiency-feedback>"#
                        .to_string()
                };

                HookJsonOutput::Sync(SyncHookJsonOutput {
                    continue_: Some(true),
                    system_message: Some(feedback),
                    ..Default::default()
                })
            }) as BoxFuture<'static, HookJsonOutput>
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_efficiency_config_default() {
        let config = EfficiencyConfig::default();
        assert!(!config.inject_cwd_reminder);
        assert!(!config.inject_stop_tips);
        assert!(!config.track_metrics);
        assert!(config.cwd.is_none());
        assert!(config.metrics.is_none());
    }

    #[test]
    fn test_efficiency_config_enabled() {
        let config = EfficiencyConfig::enabled();
        assert!(config.inject_cwd_reminder);
        assert!(config.inject_stop_tips);
        assert!(config.track_metrics);
        assert!(config.metrics.is_some());
    }

    #[test]
    fn test_efficiency_config_with_cwd() {
        let config = EfficiencyConfig::enabled().with_cwd("/test/path");
        assert_eq!(config.cwd, Some(PathBuf::from("/test/path")));
    }

    #[test]
    fn test_efficiency_config_with_metrics() {
        let config = EfficiencyConfig::with_metrics();
        assert!(!config.inject_cwd_reminder);
        assert!(config.inject_stop_tips);
        assert!(config.track_metrics);
        assert!(config.metrics.is_some());
    }

    #[test]
    fn test_efficiency_config_enable_metrics() {
        let config = EfficiencyConfig::stop_tips_only().enable_metrics();
        assert!(config.track_metrics);
        assert!(config.metrics.is_some());
    }

    #[test]
    fn test_build_efficiency_hooks_empty() {
        let config = EfficiencyConfig::default();
        let hooks = build_efficiency_hooks(&config);
        assert!(hooks.is_empty());
    }

    #[test]
    fn test_build_efficiency_hooks_cwd_only() {
        let config = EfficiencyConfig::cwd_reminder_only();
        let hooks = build_efficiency_hooks(&config);
        assert!(hooks.contains_key(&HookEvent::UserPromptSubmit));
        assert!(!hooks.contains_key(&HookEvent::Stop));
        assert!(!hooks.contains_key(&HookEvent::PostToolUse));
    }

    #[test]
    fn test_build_efficiency_hooks_stop_only() {
        let config = EfficiencyConfig::stop_tips_only();
        let hooks = build_efficiency_hooks(&config);
        assert!(!hooks.contains_key(&HookEvent::UserPromptSubmit));
        assert!(hooks.contains_key(&HookEvent::Stop));
        assert!(!hooks.contains_key(&HookEvent::PostToolUse));
    }

    #[test]
    fn test_build_efficiency_hooks_with_metrics() {
        let config = EfficiencyConfig::enabled();
        let hooks = build_efficiency_hooks(&config);
        assert!(hooks.contains_key(&HookEvent::UserPromptSubmit));
        assert!(hooks.contains_key(&HookEvent::Stop));
        assert!(hooks.contains_key(&HookEvent::PostToolUse));
    }

    #[test]
    fn test_merge_hooks_empty_user() {
        let config = EfficiencyConfig::enabled();
        let efficiency_hooks = build_efficiency_hooks(&config);
        let merged = merge_hooks(None, efficiency_hooks);

        assert!(merged.is_some());
        let merged = merged.unwrap();
        assert!(merged.contains_key(&HookEvent::UserPromptSubmit));
        assert!(merged.contains_key(&HookEvent::Stop));
        assert!(merged.contains_key(&HookEvent::PostToolUse));
    }

    #[test]
    fn test_merge_hooks_with_user_hooks() {
        let config = EfficiencyConfig::enabled();
        let efficiency_hooks = build_efficiency_hooks(&config);

        // Create user hooks with PreToolUse
        let mut user_hooks = HashMap::new();
        user_hooks.insert(HookEvent::PreToolUse, vec![]);

        let merged = merge_hooks(Some(user_hooks), efficiency_hooks);

        assert!(merged.is_some());
        let merged = merged.unwrap();
        assert!(merged.contains_key(&HookEvent::PreToolUse));
        assert!(merged.contains_key(&HookEvent::UserPromptSubmit));
        assert!(merged.contains_key(&HookEvent::Stop));
    }

    #[test]
    fn test_merge_hooks_user_takes_precedence() {
        let config = EfficiencyConfig::cwd_reminder_only();
        let efficiency_hooks = build_efficiency_hooks(&config);

        // User already has UserPromptSubmit hook
        let mut user_hooks = HashMap::new();
        user_hooks.insert(
            HookEvent::UserPromptSubmit,
            vec![HookMatcher {
                matcher: Some("custom".to_string()),
                hooks: vec![],
                timeout: None,
            }],
        );

        let merged = merge_hooks(Some(user_hooks), efficiency_hooks);

        assert!(merged.is_some());
        let merged = merged.unwrap();
        // Should have both user and efficiency hooks merged
        let matchers = &merged[&HookEvent::UserPromptSubmit];
        assert_eq!(matchers.len(), 2); // user + efficiency
    }

    #[tokio::test]
    async fn test_metrics_record_tool_call() {
        let metrics = ExecutionMetrics::new();

        // Record some tool calls
        metrics
            .record_tool_call("Read", &serde_json::json!({"file_path": "/test.rs"}))
            .await;
        metrics
            .record_tool_call("Edit", &serde_json::json!({"file_path": "/test.rs"}))
            .await;
        metrics
            .record_tool_call("Edit", &serde_json::json!({"file_path": "/test.rs"}))
            .await;

        let summary = metrics.get_summary().await;
        assert_eq!(summary.total_tool_calls, 3);
        assert_eq!(summary.total_edits, 2);
    }

    #[tokio::test]
    async fn test_metrics_detect_directory_checks() {
        let metrics = ExecutionMetrics::new();

        metrics
            .record_tool_call("Bash", &serde_json::json!({"command": "pwd"}))
            .await;
        metrics
            .record_tool_call("Bash", &serde_json::json!({"command": "cd /tmp && pwd"}))
            .await;
        metrics
            .record_tool_call("Bash", &serde_json::json!({"command": "cd /home"}))
            .await;

        let summary = metrics.get_summary().await;
        assert_eq!(summary.directory_checks, 3);
    }

    #[tokio::test]
    async fn test_metrics_detect_build_attempts() {
        let metrics = ExecutionMetrics::new();

        metrics
            .record_tool_call("Bash", &serde_json::json!({"command": "npm run build"}))
            .await;
        metrics
            .record_tool_call("Bash", &serde_json::json!({"command": "npm test"}))
            .await;
        metrics
            .record_tool_call("Bash", &serde_json::json!({"command": "cargo build"}))
            .await;

        let summary = metrics.get_summary().await;
        assert_eq!(summary.build_attempts, 3);
    }

    #[tokio::test]
    async fn test_metrics_generate_warnings_fragmented_edits() {
        let metrics = ExecutionMetrics::new();

        // Edit same file 5 times
        for _ in 0..5 {
            metrics
                .record_tool_call("Edit", &serde_json::json!({"file_path": "/test.rs"}))
                .await;
        }

        let warnings = metrics.generate_warnings().await;
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("FRAGMENTED EDITS"));
        assert!(warnings[0].contains("/test.rs"));
    }

    #[tokio::test]
    async fn test_metrics_generate_warnings_directory_confusion() {
        let metrics = ExecutionMetrics::new();

        for _ in 0..4 {
            metrics
                .record_tool_call("Bash", &serde_json::json!({"command": "pwd"}))
                .await;
        }

        let warnings = metrics.generate_warnings().await;
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("DIRECTORY CONFUSION"));
    }

    #[tokio::test]
    async fn test_metrics_no_warnings_when_efficient() {
        let metrics = ExecutionMetrics::new();

        // Efficient usage: few edits per file, no pwd
        metrics
            .record_tool_call("Read", &serde_json::json!({"file_path": "/a.rs"}))
            .await;
        metrics
            .record_tool_call("Edit", &serde_json::json!({"file_path": "/a.rs"}))
            .await;
        metrics
            .record_tool_call("Bash", &serde_json::json!({"command": "npm run build"}))
            .await;

        let warnings = metrics.generate_warnings().await;
        assert!(warnings.is_empty());
    }
}
