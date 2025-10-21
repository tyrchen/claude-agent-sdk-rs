# Fixture Capture Scripts Summary

This document summarizes the Python scripts created to capture JSON fixtures for Rust unit tests.

## Overview

Created 7 Python scripts using the Python Claude Agent SDK to capture all possible JSON data variations that match the types defined in `vendors/claude-agent-sdk-python/src/claude_agent_sdk/types.py`.

Total fixtures generated: **114 JSON files**

## Scripts Created

### 1. capture_messages.py
Captures Message type variants:
- UserMessage
- AssistantMessage (simple and with tool use)
- SystemMessage
- ResultMessage
- StreamEvent (streaming mode)

**Fixtures**: 5 files in `fixtures/messages/`

### 2. capture_content_blocks.py
Captures ContentBlock type variants:
- TextBlock
- ThinkingBlock
- ToolUseBlock
- ToolResultBlock (success, error, structured variants)

**Fixtures**: 6 files in `fixtures/content_blocks/`

### 3. capture_hooks.py
Captures Hook type variants:
- PreToolUseHookInput
- PostToolUseHookInput
- UserPromptSubmitHookInput
- StopHookInput
- SubagentStopHookInput (synthetic)
- PreCompactHookInput (synthetic, manual and auto triggers)
- All HookJSONOutput variants (async, sync, with various hook-specific outputs)

**Fixtures**: 17 files in `fixtures/hooks/`

### 4. capture_permissions.py
Captures Permission type variants:
- PermissionUpdate (all 6 types: addRules, replaceRules, removeRules, setMode, addDirectories, removeDirectories)
- PermissionRuleValue (with and without content)
- PermissionResultAllow (simple, with updated input, with updated permissions, complete)
- PermissionResultDeny (simple, with message, with interrupt)
- ToolPermissionContext (with and without suggestions)

**Fixtures**: 20 files in `fixtures/permissions/`

### 5. capture_control_protocol.py
Captures SDK Control Protocol type variants:
- SDKControlRequest (all subtypes):
  - interrupt
  - can_use_tool (permission requests)
  - initialize (with and without hooks)
  - set_permission_mode (all 4 modes)
  - hook_callback (PreToolUse, PostToolUse, UserPromptSubmit)
  - mcp_message
- SDKControlResponse (success and error variants)

**Fixtures**: 21 files in `fixtures/control_protocol/`

### 6. capture_mcp_configs.py
Captures MCP Server Config type variants:
- McpStdioServerConfig (minimal, with type, with env, command only)
- McpSSEServerConfig (minimal, with headers)
- McpHttpServerConfig (minimal, with headers)
- McpSdkServerConfig (structure example)
- Collection example with multiple server types

**Fixtures**: 10 files in `fixtures/mcp_configs/`

### 7. capture_agents.py
Captures Agent and Options type variants:
- AgentDefinition (minimal, with tools, with model, complete, all model types)
- SystemPromptPreset (basic, with append)
- ClaudeAgentOptions (all configuration options):
  - allowed_tools, disallowed_tools
  - system_prompt (string and preset)
  - permission_mode (all 4 modes)
  - model, max_turns
  - continue_conversation, resume, fork_session
  - include_partial_messages (streaming)
  - setting_sources (all combinations)
  - cwd, add_dirs, env, extra_args
  - agents (custom agent definitions)
  - comprehensive example

**Fixtures**: 35 files in `fixtures/agents/`

## Running the Scripts

All scripts can be run using `uv run` from the tools directory:

```bash
cd tools

# Run synthetic data generators (no API calls)
uv run capture_control_protocol.py
uv run capture_mcp_configs.py
uv run capture_permissions.py
uv run capture_agents.py

# Run scripts that make API calls (these take longer)
uv run capture_messages.py
uv run capture_content_blocks.py
uv run capture_hooks.py
```

## Fixture Organization

```
fixtures/
├── agents/              # 35 files - Agent definitions and options
├── content_blocks/      # 6 files - Text, Thinking, ToolUse, ToolResult blocks
├── control_protocol/    # 21 files - SDK control protocol messages
├── hooks/               # 17 files - Hook inputs and outputs
├── mcp_configs/         # 10 files - MCP server configurations
├── messages/            # 5 files - Message types (User, Assistant, System, Result, Stream)
└── permissions/         # 20 files - Permission updates and results
```

## Usage in Rust Tests

These fixtures can be used in Rust unit tests to verify:
1. Correct deserialization of all type variants
2. Serialization behavior matches Python SDK
3. Edge cases and optional fields are handled properly
4. Type safety and validation

Example usage in Rust:
```rust
#[test]
fn test_deserialize_assistant_message() {
    let fixture = include_str!("../fixtures/messages/assistant_message_simple.json");
    let msg: AssistantMessage = serde_json::from_str(fixture).unwrap();
    assert_eq!(msg.model, "claude-sonnet-4-5-20250929");
}
```

## Dependencies

The tools project uses:
- Python 3.12+
- uv for dependency management
- claude-agent-sdk (local path to vendors/claude-agent-sdk-python)

See `tools/pyproject.toml` for configuration.
