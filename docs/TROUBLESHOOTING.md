# Troubleshooting Guide

This guide helps you diagnose and resolve common issues when using the Claude Agent SDK for Rust.

## Common Issues

### Connection Errors

**Problem:** `CLI connection error: Failed to spawn Claude CLI process`

**Possible Causes:**
- Claude CLI is not installed or not in PATH
- Claude CLI version is incompatible (need >= 2.0.0)
- Incorrect CLI path specified

**Solutions:**

1. **Verify Claude CLI is installed:**
   ```bash
   which claude
   ```
   If this returns nothing, Claude CLI is not installed or not in your PATH.

2. **Check CLI version:**
   ```bash
   claude --version
   ```
   Ensure the version is >= 2.0.0.

3. **Set explicit path:**
   ```rust
   let options = ClaudeAgentOptions::builder()
       .cli_path(Some("/path/to/claude".into()))
       .build();
   ```

4. **Check permissions:**
   ```bash
   ls -l $(which claude)
   # Should show execute permissions
   ```

---

### Timeout Issues

**Problem:** Operations hang indefinitely or timeout

**Possible Causes:**
- Claude CLI is waiting for user input
- Permission mode not set correctly
- Control request taking longer than timeout (default: 30s)

**Solutions:**

1. **Set appropriate permission mode:**
   ```rust
   let options = ClaudeAgentOptions::builder()
       .permission_mode(Some(PermissionMode::AcceptEdits))
       .build();
   ```

2. **Increase control request timeout:**
   ```rust
   let options = ClaudeAgentOptions::builder()
       .control_request_timeout(Duration::from_secs(60))
       .build();
   ```

3. **Enable debug logging to see what's happening:**
   ```rust
   tracing_subscriber::fmt()
       .with_max_level(tracing::Level::DEBUG)
       .init();
   ```

---

### Memory Leaks

**Problem:** Memory usage grows over time

**Possible Causes:**
- Not calling `disconnect()` on ClaudeClient
- Not dropping stream before starting next query
- Circular Arc references in custom code

**Solutions:**

1. **Always call disconnect():**
   ```rust
   let mut client = ClaudeClient::new(options);
   client.connect().await?;

   // ... use client ...

   client.disconnect().await?; // Important!
   ```

2. **Drop stream before next query:**
   ```rust
   client.query("First question").await?;
   {
       let mut stream = client.receive_response();
       while let Some(msg) = stream.next().await {
           // handle message
       }
   } // Stream dropped here

   // Now safe to send next query
   client.query("Second question").await?;
   ```

3. **Use helper methods that handle cleanup:**
   ```rust
   // This handles stream lifecycle automatically
   let text = client.query_for_text("What is Rust?").await?;
   ```

---

### Transport Errors

**Problem:** `Transport error: Buffer size exceeded` or `Transport error: Failed to write to stdin`

**Possible Causes:**
- Message size exceeds buffer limits
- CLI process has terminated unexpectedly
- Stdin pipe is broken

**Solutions:**

1. **Increase buffer size:**
   ```rust
   let options = ClaudeAgentOptions::builder()
       .max_buffer_size(Some(20_000_000)) // 20MB
       .build();
   ```

2. **Check if CLI is still running:**
   ```bash
   ps aux | grep claude
   ```

3. **Monitor stderr for CLI errors:**
   ```rust
   let options = ClaudeAgentOptions::builder()
       .stderr_callback(Some(Arc::new(|line| {
           eprintln!("CLI stderr: {}", line);
       })))
       .build();
   ```

---

### Parse Errors

**Problem:** `Message parse error` or `JSON decode error`

**Possible Causes:**
- Claude CLI output format has changed
- SDK version incompatible with CLI version
- Corrupted message from CLI

**Solutions:**

1. **Check version compatibility:**
   ```bash
   claude --version
   # Compare with SDK's MIN_CLI_VERSION
   ```

2. **Update SDK and CLI to latest versions:**
   ```bash
   cargo update -p claude-agent-sdk-rs
   # Update Claude CLI through its installation method
   ```

3. **Enable raw message inspection:**
   ```rust
   // Set RUST_LOG=trace to see raw JSON messages
   std::env::set_var("RUST_LOG", "claude_agent_sdk_rs=trace");
   tracing_subscriber::fmt()
       .with_max_level(tracing::Level::TRACE)
       .init();
   ```

---

## Performance Issues

### Slow Response Times

**Diagnostics:**

1. **Enable tracing to identify bottlenecks:**
   ```rust
   tracing_subscriber::fmt()
       .with_max_level(tracing::Level::TRACE)
       .init();
   ```

2. **Check network latency:**
   - Claude CLI makes API calls to Anthropic
   - Network issues can cause slowness

3. **Monitor subprocess CPU/memory:**
   ```bash
   top -p $(pgrep -f "claude")
   ```

**Solutions:**

1. **Use streaming for faster perceived response:**
   ```rust
   client.query("Long analysis task").await?;
   let mut stream = client.receive_response();

   while let Some(msg) = stream.next().await {
       // Process messages as they arrive
       // Don't wait for entire response
   }
   ```

2. **Reduce max_turns if Claude is doing too many iterations:**
   ```rust
   let options = ClaudeAgentOptions::builder()
       .max_turns(Some(3))
       .build();
   ```

---

## Debugging Tips

### Enable Detailed Logging

**Trace level (most verbose):**
```rust
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::TRACE)
    .init();
```

**Debug level (recommended for troubleshooting):**
```rust
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();
```

**Via environment variable:**
```bash
RUST_LOG=claude_agent_sdk_rs=debug cargo run
```

### Inspect Raw Messages

To see the exact JSON messages being exchanged:

```rust
let options = ClaudeAgentOptions::builder()
    .stderr_callback(Some(Arc::new(|line| {
        eprintln!("CLI: {}", line);
    })))
    .build();
```

### Use Minimal Reproduction

When reporting issues, create a minimal example:

```rust
use claude_agent_sdk_rs::{ClaudeClient, ClaudeAgentOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let mut client = ClaudeClient::new(ClaudeAgentOptions::default());
    client.connect().await?;

    // Minimal code that reproduces the issue
    client.query("test").await?;

    client.disconnect().await?;
    Ok(())
}
```

---

## Error Message Reference

### `CLI not found: {message}`
- **Cause:** Claude CLI executable not found
- **Solution:** Install Claude CLI or set `cli_path` in options

### `Connection error: {message}`
- **Cause:** Failed to establish connection to CLI
- **Solution:** Check CLI installation, permissions, and version

### `Process error (exit code N): {message}`
- **Cause:** CLI process exited unexpectedly
- **Solution:** Check stderr output for CLI-specific errors

### `Control protocol error: Request timeout after Nms`
- **Cause:** Control request took longer than configured timeout
- **Solution:** Increase `control_request_timeout` in options

### `Control protocol error: Unknown control request subtype: {subtype}`
- **Cause:** CLI sent an unrecognized control request
- **Solution:** Update SDK to match CLI version

### `Transport error: Stdin not available`
- **Cause:** Cannot write to CLI stdin (process may have exited)
- **Solution:** Check if CLI process is still running

### `Transport error: Buffer size exceeded: {current} > {max}`
- **Cause:** Response size exceeds configured buffer limit
- **Solution:** Increase `max_buffer_size` in options

---

## Getting Help

If you're still experiencing issues after consulting this guide:

1. **Check GitHub Issues:** https://github.com/tyrchen/claude-agent-sdk-rs/issues
2. **Review Examples:** The `examples/` directory contains working code for common scenarios
3. **Enable Debug Logging:** Capture logs with `RUST_LOG=claude_agent_sdk_rs=debug`
4. **Create Minimal Reproduction:** Simplify your code to the smallest example that shows the problem
5. **Report Issue:** Include:
   - SDK version (`Cargo.toml`)
   - CLI version (`claude --version`)
   - OS and Rust version
   - Full error message
   - Debug logs
   - Minimal reproduction code
