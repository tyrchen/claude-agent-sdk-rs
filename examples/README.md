# Claude Agent SDK Examples

This directory contains working examples demonstrating the Claude Agent SDK for Rust.

## Quick Start Examples

```bash
# Run example 1: Basic Hello World
cargo run --example 01_hello_world

# Run example 2: Limit Tool Use
cargo run --example 02_limit_tool_use

# Run example 3: Monitor Tool Use
cargo run --example 03_monitor_tools
```

## All Examples

### Basic Examples

**01_hello_world.rs** - Basic SDK usage to have Claude write a Python program
- Uses the `query()` function
- Sets `allowed_tools`
- Processes messages

**02_limit_tool_use.rs** - Restrict which tools Claude can use
- Uses `allowed_tools` and `disallowed_tools`
- Verifies tool restrictions

**03_monitor_tools.rs** - Comprehensive tool usage monitoring
- Tracks every tool invocation
- Provides detailed usage summary

### Advanced Control Examples

**04_permission_callbacks.rs** - Tool permission callbacks
- Control tool execution with callbacks
- Modify tool inputs dynamically
- Log tool usage

**05_hooks_pretooluse.rs** - PreToolUse hooks
- Intercept tools before execution
- Approve or deny tool usage

**06_bidirectional_client.rs** - Bidirectional streaming client
- Real-time message streaming
- Interactive conversations

**07_dynamic_control.rs** - Dynamic control patterns
- Runtime tool control
- Multi-turn conversations

### MCP Integration

**08_mcp_server_integration.rs** - MCP server integration
- Create in-process MCP servers
- Define custom tools
- Integrate with Claude

### Configuration Examples

**09_agents.rs** - Custom agents with specific tools and prompts
```bash
cargo run --example 09_agents
```

**10_include_partial_messages.rs** - Stream partial messages
```bash
cargo run --example 10_include_partial_messages
```

**11_setting_sources.rs** - Control settings sources (user/project/local)
```bash
# Run all examples
cargo run --example 11_setting_sources -- all

# Run specific example
cargo run --example 11_setting_sources -- default
cargo run --example 11_setting_sources -- user_only
cargo run --example 11_setting_sources -- project_and_user
```

**12_stderr_callback.rs** - Capture stderr output for debugging
```bash
cargo run --example 12_stderr_callback
```

**13_system_prompt.rs** - Different system prompt configurations
```bash
cargo run --example 13_system_prompt
```

### Streaming Mode Examples

**14_streaming_mode.rs** - Comprehensive streaming patterns
```bash
# Run all streaming examples
cargo run --example 14_streaming_mode -- all

# Run specific examples
cargo run --example 14_streaming_mode -- basic_streaming
cargo run --example 14_streaming_mode -- multi_turn
cargo run --example 14_streaming_mode -- with_options
cargo run --example 14_streaming_mode -- manual_handling
```

### Hooks Examples

**15_hooks_comprehensive.rs** - Complete hooks system demonstration
```bash
# Run all hook examples
cargo run --example 15_hooks_comprehensive -- all

# Run specific examples
cargo run --example 15_hooks_comprehensive -- PreToolUse
cargo run --example 15_hooks_comprehensive -- PostToolUse
cargo run --example 15_hooks_comprehensive -- UserPromptSubmit
cargo run --example 15_hooks_comprehensive -- DecisionFields
cargo run --example 15_hooks_comprehensive -- ContinueControl
```

## Key Concepts Covered

- **Basic Usage**: Simple queries, message processing
- **Tool Control**: Allowing/denying tools, permission callbacks
- **Hooks**: PreToolUse, PostToolUse, UserPromptSubmit
- **Streaming**: Real-time message streaming, multi-turn conversations
- **MCP Integration**: Creating and using MCP servers
- **Configuration**: Agents, system prompts, setting sources
- **Debugging**: Stderr callbacks, tool monitoring

## Requirements

- Claude Code CLI 2.0.0+
- Rust 1.70+
- Python 3 (for running generated scripts in some examples)
