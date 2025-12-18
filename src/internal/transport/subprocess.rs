//! Subprocess transport implementation for Claude Code CLI

use async_trait::async_trait;
use futures::stream::Stream;
use std::collections::HashMap;
use std::path::PathBuf;
use std::pin::Pin;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;
use tracing::warn;

use crate::errors::{
    ClaudeError, CliNotFoundError, ConnectionError, JsonDecodeError, ProcessError, Result,
};
use crate::types::config::ClaudeAgentOptions;
use crate::version::{
    check_version, ENTRYPOINT, MIN_CLI_VERSION, SDK_VERSION, SKIP_VERSION_CHECK_ENV,
};

use super::Transport;

const DEFAULT_MAX_BUFFER_SIZE: usize = 10 * 1024 * 1024; // 10MB

/// Query prompt type
#[derive(Clone)]
pub enum QueryPrompt {
    /// Text prompt (one-shot mode)
    Text(String),
    /// Streaming mode (no initial prompt)
    Streaming,
}

impl From<String> for QueryPrompt {
    fn from(text: String) -> Self {
        QueryPrompt::Text(text)
    }
}

impl From<&str> for QueryPrompt {
    fn from(text: &str) -> Self {
        QueryPrompt::Text(text.to_string())
    }
}

/// Subprocess transport for communicating with Claude Code CLI
pub struct SubprocessTransport {
    cli_path: PathBuf,
    cwd: Option<PathBuf>,
    options: ClaudeAgentOptions,
    prompt: QueryPrompt,
    process: Option<Child>,
    pub(crate) stdin: Arc<Mutex<Option<ChildStdin>>>,
    pub(crate) stdout: Arc<Mutex<Option<BufReader<ChildStdout>>>>,
    max_buffer_size: usize,
    ready: bool,
}

impl SubprocessTransport {
    /// Create a new subprocess transport
    pub fn new(prompt: QueryPrompt, options: ClaudeAgentOptions) -> Result<Self> {
        let cli_path = if let Some(ref path) = options.cli_path {
            path.clone()
        } else {
            Self::find_cli()?
        };

        let cwd = options.cwd.clone().or_else(|| std::env::current_dir().ok());
        let max_buffer_size = options.max_buffer_size.unwrap_or(DEFAULT_MAX_BUFFER_SIZE);

        Ok(Self {
            cli_path,
            cwd,
            options,
            prompt,
            process: None,
            stdin: Arc::new(Mutex::new(None)),
            stdout: Arc::new(Mutex::new(None)),
            max_buffer_size,
            ready: false,
        })
    }

    /// Find the Claude CLI executable
    fn find_cli() -> Result<PathBuf> {
        // Strategy 1: Try executing 'claude' directly from PATH
        // This is the most reliable method as it respects the shell's PATH resolution
        if let Ok(output) = std::process::Command::new("claude")
            .arg("--version")
            .output()
        {
            if output.status.success() {
                // 'claude' is in PATH and executable, return it as-is
                // The OS will resolve it when we spawn the process
                return Ok(PathBuf::from("claude"));
            }
        }

        // Strategy 2: Use 'which' command to locate claude in PATH (Unix-like systems)
        #[cfg(not(target_os = "windows"))]
        if let Ok(output) = std::process::Command::new("which").arg("claude").output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout);
                let path = PathBuf::from(path_str.trim());
                // Verify the path exists and is executable
                if path.exists() && path.is_file() {
                    return Ok(path);
                }
            }
        }

        // Strategy 3: Use 'where' command on Windows
        #[cfg(target_os = "windows")]
        if let Ok(output) = std::process::Command::new("where").arg("claude").output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout);
                // 'where' returns all matches, take the first one
                if let Some(first_line) = path_str.lines().next() {
                    let path = PathBuf::from(first_line.trim());
                    if path.exists() && path.is_file() {
                        return Ok(path);
                    }
                }
            }
        }

        // Strategy 4: Check common installation locations
        // Get home directory for path expansion
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE")) // Windows fallback
            .ok()
            .map(PathBuf::from);

        // Common installation locations
        let mut common_paths: Vec<PathBuf> = vec![];

        // Unix-like paths
        #[cfg(not(target_os = "windows"))]
        {
            common_paths.extend(vec![
                PathBuf::from("/usr/local/bin/claude"),
                PathBuf::from("/opt/homebrew/bin/claude"),
                PathBuf::from("/usr/bin/claude"),
            ]);

            // Add home-relative paths if home directory is available
            if let Some(ref home) = home_dir {
                common_paths.push(home.join(".local/bin/claude"));
                common_paths.push(home.join("bin/claude"));
            }
        }

        // Windows paths
        #[cfg(target_os = "windows")]
        {
            if let Some(ref home) = home_dir {
                common_paths.extend(vec![
                    home.join("AppData\\Local\\Programs\\Claude\\claude.exe"),
                    home.join("AppData\\Roaming\\npm\\claude.cmd"),
                    home.join("AppData\\Roaming\\npm\\claude.exe"),
                ]);
            }
            common_paths.extend(vec![
                PathBuf::from("C:\\Program Files\\Claude\\claude.exe"),
                PathBuf::from("C:\\Program Files (x86)\\Claude\\claude.exe"),
            ]);
        }

        // Check each common path
        for path in common_paths {
            if path.exists() && path.is_file() {
                return Ok(path);
            }
        }

        // Strategy 5: Check if CLAUDE_CLI_PATH environment variable is set
        if let Ok(cli_path) = std::env::var("CLAUDE_CLI_PATH") {
            let path = PathBuf::from(cli_path);
            if path.exists() && path.is_file() {
                return Ok(path);
            }
        }

        Err(ClaudeError::CliNotFound(CliNotFoundError::new(
            "Claude Code CLI not found. Please ensure 'claude' is in your PATH or set CLAUDE_CLI_PATH environment variable.",
            None,
        )))
    }

    /// Build command arguments from options
    fn build_command(&self) -> Vec<String> {
        let mut args = vec![
            "--output-format".to_string(),
            "stream-json".to_string(),
            "--verbose".to_string(),
        ];

        // For streaming mode, enable bidirectional stream-json input
        if matches!(self.prompt, QueryPrompt::Streaming) {
            args.push("--input-format".to_string());
            args.push("stream-json".to_string());
        }

        // Add system prompt
        // Note: Python SDK behavior (lines 91-102 of subprocess_cli.py):
        // - If None: skip
        // - If string: use --system-prompt
        // - If preset with append: use --append-system-prompt (NOT --system-prompt-preset)
        //   This relies on default Claude Code prompt and just appends to it
        if let Some(ref system_prompt) = self.options.system_prompt {
            match system_prompt {
                crate::types::config::SystemPrompt::Text(text) => {
                    args.push("--system-prompt".to_string());
                    args.push(text.clone());
                }
                crate::types::config::SystemPrompt::Preset(preset) => {
                    // Only add append if present (uses default Claude Code prompt)
                    if let Some(ref append) = preset.append {
                        args.push("--append-system-prompt".to_string());
                        args.push(append.clone());
                    }
                    // Note: preset.preset field is ignored - CLI uses default prompt
                }
            }
        }

        // Add permission mode
        if let Some(mode) = self.options.permission_mode {
            let mode_str = match mode {
                crate::types::config::PermissionMode::Default => "default",
                crate::types::config::PermissionMode::AcceptEdits => "acceptEdits",
                crate::types::config::PermissionMode::Plan => "plan",
                crate::types::config::PermissionMode::BypassPermissions => "bypassPermissions",
            };
            args.push("--permission-mode".to_string());
            args.push(mode_str.to_string());
        }

        // Add allowed tools
        for tool in &self.options.allowed_tools {
            args.push("--allowed-tools".to_string());
            args.push(tool.clone());
        }

        // Add disallowed tools
        for tool in &self.options.disallowed_tools {
            args.push("--disallowed-tools".to_string());
            args.push(tool.clone());
        }

        // Add model
        if let Some(ref model) = self.options.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }

        // Add fallback model
        if let Some(ref fallback_model) = self.options.fallback_model {
            args.push("--fallback-model".to_string());
            args.push(fallback_model.clone());
        }

        // Add max budget USD
        if let Some(max_budget) = self.options.max_budget_usd {
            args.push("--max-budget-usd".to_string());
            args.push(max_budget.to_string());
        }

        // Add max thinking tokens
        if let Some(max_thinking) = self.options.max_thinking_tokens {
            args.push("--max-thinking-tokens".to_string());
            args.push(max_thinking.to_string());
        }

        // Add output format (structured outputs / JSON schema)
        // Expected format: {"type": "json_schema", "schema": {...}}
        if let Some(ref output_format) = self.options.output_format {
            if output_format.get("type") == Some(&serde_json::json!("json_schema")) {
                if let Some(schema) = output_format.get("schema") {
                    args.push("--json-schema".to_string());
                    args.push(schema.to_string());
                }
            }
        }

        // Add max turns
        if let Some(max_turns) = self.options.max_turns {
            args.push("--max-turns".to_string());
            args.push(max_turns.to_string());
        }

        // Add resume session
        if let Some(ref session_id) = self.options.resume {
            args.push("--resume".to_string());
            args.push(session_id.clone());
        }

        // Add continue conversation
        if self.options.continue_conversation {
            args.push("--continue-conversation".to_string());
        }

        // Add fork session
        if self.options.fork_session {
            args.push("--fork-session".to_string());
        }

        // Add plugins
        for plugin in &self.options.plugins {
            if let Some(path) = plugin.path() {
                args.push("--plugin-dir".to_string());
                args.push(path.display().to_string());
            }
        }

        // Add extra args
        for (key, value) in &self.options.extra_args {
            args.push(format!("--{}", key));
            if let Some(ref v) = value {
                args.push(v.clone());
            }
        }

        args
    }

    /// Check Claude CLI version
    async fn check_claude_version(&self) -> Result<()> {
        // Skip if environment variable is set
        if std::env::var(SKIP_VERSION_CHECK_ENV).is_ok() {
            return Ok(());
        }

        let output = Command::new(&self.cli_path)
            .arg("--version")
            .output()
            .await
            .map_err(|e| {
                ClaudeError::Connection(ConnectionError::new(format!(
                    "Failed to get Claude version: {}",
                    e
                )))
            })?;

        let version_output = String::from_utf8_lossy(&output.stdout);
        let version = version_output
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().next())
            .unwrap_or("")
            .trim();

        if !check_version(version) {
            warn!(
                "Claude Code CLI ({}) version {} is below minimum required version {}. Some features may not work correctly.",
                self.cli_path.display(),
                version,
                MIN_CLI_VERSION
            );
        }

        Ok(())
    }

    /// Build environment variables
    fn build_env(&self) -> HashMap<String, String> {
        let mut env = self.options.env.clone();
        env.insert("CLAUDE_CODE_ENTRYPOINT".to_string(), ENTRYPOINT.to_string());
        env.insert(
            "CLAUDE_AGENT_SDK_VERSION".to_string(),
            SDK_VERSION.to_string(),
        );
        env
    }
}

#[async_trait]
impl Transport for SubprocessTransport {
    async fn connect(&mut self) -> Result<()> {
        // Check version
        self.check_claude_version().await?;

        // Build command
        let args = self.build_command();
        let env = self.build_env();

        // Build command
        let mut cmd = Command::new(&self.cli_path);
        cmd.args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .envs(&env);

        if let Some(ref cwd) = self.cwd {
            cmd.current_dir(cwd);
        }

        // Spawn process
        let mut child = cmd.spawn().map_err(|e| {
            ClaudeError::Process(ProcessError::new(
                format!("Failed to spawn Claude CLI process: {}", e),
                None,
                None,
            ))
        })?;

        // Take stdin and stdout
        let stdin = child.stdin.take().ok_or_else(|| {
            ClaudeError::Connection(ConnectionError::new("Failed to get stdin".to_string()))
        })?;

        let stdout = child.stdout.take().ok_or_else(|| {
            ClaudeError::Connection(ConnectionError::new("Failed to get stdout".to_string()))
        })?;

        let stderr = child.stderr.take();

        // Spawn stderr handler if callback is provided
        if let (Some(stderr), Some(ref callback)) = (stderr, &self.options.stderr_callback) {
            let callback = Arc::clone(callback);
            tokio::spawn(async move {
                let mut reader = BufReader::new(stderr);
                let mut line = String::new();
                while let Ok(n) = reader.read_line(&mut line).await {
                    if n == 0 {
                        break;
                    }
                    callback(line.clone());
                    line.clear();
                }
            });
        }

        *self.stdin.lock().await = Some(stdin);
        *self.stdout.lock().await = Some(BufReader::new(stdout));
        self.process = Some(child);
        self.ready = true;

        // Send initial prompt if it's text (one-shot mode)
        match &self.prompt {
            QueryPrompt::Text(text) => {
                let text_owned = text.clone();
                self.write(&text_owned).await?;
                self.end_input().await?;
            }
            QueryPrompt::Streaming => {
                // Don't send initial prompt or close stdin - leave it open for streaming
            }
        }

        Ok(())
    }

    async fn write(&mut self, data: &str) -> Result<()> {
        let mut stdin_guard = self.stdin.lock().await;
        if let Some(ref mut stdin) = *stdin_guard {
            stdin
                .write_all(data.as_bytes())
                .await
                .map_err(|e| ClaudeError::Transport(format!("Failed to write to stdin: {}", e)))?;
            stdin
                .write_all(b"\n")
                .await
                .map_err(|e| ClaudeError::Transport(format!("Failed to write newline: {}", e)))?;
            stdin
                .flush()
                .await
                .map_err(|e| ClaudeError::Transport(format!("Failed to flush stdin: {}", e)))?;
            Ok(())
        } else {
            Err(ClaudeError::Transport("stdin not available".to_string()))
        }
    }

    fn read_messages(
        &mut self,
    ) -> Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send + '_>> {
        let stdout = Arc::clone(&self.stdout);
        let max_buffer_size = self.max_buffer_size;

        Box::pin(async_stream::stream! {
            let mut stdout_guard = stdout.lock().await;
            if let Some(ref mut reader) = *stdout_guard {
                let mut line = String::new();
                let mut buffer_size = 0;

                loop {
                    line.clear();
                    match reader.read_line(&mut line).await {
                        Ok(0) => {
                            // EOF
                            break;
                        }
                        Ok(n) => {
                            buffer_size += n;
                            if buffer_size > max_buffer_size {
                                yield Err(ClaudeError::Transport(format!(
                                    "Buffer size exceeded maximum of {} bytes",
                                    max_buffer_size
                                )));
                                break;
                            }

                            let trimmed = line.trim();
                            if trimmed.is_empty() {
                                continue;
                            }

                            match serde_json::from_str::<serde_json::Value>(trimmed) {
                                Ok(json) => {
                                    yield Ok(json);
                                }
                                Err(e) => {
                                    yield Err(ClaudeError::JsonDecode(JsonDecodeError::new(
                                        format!("Failed to parse JSON: {}", e),
                                        trimmed.to_string(),
                                    )));
                                }
                            }
                        }
                        Err(e) => {
                            yield Err(ClaudeError::Transport(format!("Failed to read line: {}", e)));
                            break;
                        }
                    }
                }
            }
        })
    }

    async fn close(&mut self) -> Result<()> {
        // Close stdin
        if let Some(mut stdin) = self.stdin.lock().await.take() {
            let _ = stdin.shutdown().await;
        }

        // Wait for process to exit
        if let Some(mut process) = self.process.take() {
            let status = process.wait().await.map_err(|e| {
                ClaudeError::Process(ProcessError::new(
                    format!("Failed to wait for process: {}", e),
                    None,
                    None,
                ))
            })?;

            if !status.success() {
                return Err(ClaudeError::Process(ProcessError::new(
                    "Claude CLI exited with non-zero status".to_string(),
                    status.code(),
                    None,
                )));
            }
        }

        self.ready = false;
        Ok(())
    }

    fn is_ready(&self) -> bool {
        self.ready
    }

    async fn end_input(&mut self) -> Result<()> {
        if let Some(mut stdin) = self.stdin.lock().await.take() {
            stdin
                .shutdown()
                .await
                .map_err(|e| ClaudeError::Transport(format!("Failed to close stdin: {}", e)))?;
        }
        Ok(())
    }
}

impl Drop for SubprocessTransport {
    fn drop(&mut self) {
        if let Some(mut process) = self.process.take() {
            let _ = process.start_kill();
        }
    }
}
