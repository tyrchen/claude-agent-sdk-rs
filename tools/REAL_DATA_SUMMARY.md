# Real Data Capture Summary

## Overview

This document describes the **real-world JSON data** captured from actual Claude Agent SDK interactions. Unlike synthetic test data, these fixtures contain genuine API responses, tool executions, hooks, permission callbacks, and streaming events.

**Total Real Fixtures: 130 JSON files**

## Capture Method

The script `capture_real_interactions.py` intercepts raw JSON messages from the Claude CLI **before** they are parsed by the Python SDK. This gives us the actual wire format data exactly as it comes from the API.

### How It Works

1. Monkey-patches the message parser to capture raw JSON
2. Runs real SDK interactions with actual API calls
3. Saves each message to a separate JSON file
4. Preserves all metadata, IDs, usage stats, and content

## Captured Data Breakdown

### By Message Type
- **assistant**: 16 messages - Claude's responses with real API data
- **user**: 5 messages - Tool results and user inputs
- **system**: 6 messages - Session init and system messages
- **result**: 6 messages - Session results with usage/cost data
- **stream_event**: 97 messages - Real streaming events

### By Scenario

#### 1. Basic Conversation (3 messages)
Simple text conversation capturing:
- System initialization
- Assistant text response
- Result with cost/usage

**Files**: `system_001.json`, `assistant_001.json`, `result_001.json`

#### 2. Tool Usage (9 messages)
Real tool execution with file operations:
- Write tool creating a file
- Read tool verifying the file
- Tool results with actual output
- Multi-turn conversation

**Files**: `system_002.json`, `assistant_002-006.json`, `user_001-002.json`, `result_002.json`

**Features**:
- Real tool use IDs (e.g., `toolu_01D369H4hkD2NH6N7QXY3sg4`)
- Actual file paths and tool responses
- Cache token usage statistics
- Tool result blocks with real content

#### 3. Streaming Mode (100 messages)
Complete streaming session capturing:
- `message_start` event
- Multiple `content_block_delta` events
- `content_block_stop` event
- `message_delta` event
- `message_stop` event

**Files**: `system_003.json`, `stream_event_001-097.json`, `assistant_007.json`, `result_003.json`

**Features**:
- Real streaming deltas with partial text
- Usage stats updates
- Stop reasons
- UUID tracking for each event

#### 4. With Hooks (6 messages)
Real hook execution during tool use:
- PreToolUse hook before Bash execution
- PostToolUse hook after completion
- Tool execution with hook logging

**Files**: `system_004.json`, `assistant_008-010.json`, `user_003.json`, `result_004.json`

**Features**:
- Hooks triggered on real tool use
- Bash command execution
- Echo command output captured

#### 5. With Permission Callbacks (6 messages)
Real permission system in action:
- First write denied by callback
- Agent adapts and retries
- Second write allowed

**Files**: `system_005.json`, `assistant_011-013.json`, `user_004.json`, `result_005.json`

**Features**:
- Permission callback invoked
- Real denial and retry behavior
- Tool permission flow

#### 6. With Custom Agents (6 messages)
Custom agent definition usage:
- Calculator agent with haiku model
- Bash tool usage for calculation
- Agent-specific configuration

**Files**: `system_006.json`, `assistant_014-016.json`, `user_005.json`, `result_006.json`

**Features**:
- Custom agent in system message
- Model override (haiku)
- Tool restrictions applied

## Real Data Characteristics

### Authentic API Data
All messages contain:
- Real Claude API message IDs
- Actual model identifiers (claude-sonnet-4-5-20250929)
- True usage statistics (input/output tokens)
- Cache metrics (ephemeral 5m/1h tokens)
- Genuine session/UUID tracking

### Real Tool Executions
Tool use includes:
- Actual tool use IDs from API
- Real file system operations
- True command outputs
- Authentic error messages (when applicable)

### Real Streaming Data
Streaming events contain:
- Genuine delta text as Claude generates
- Real-time token updates
- Actual stop reasons
- True streaming event types

## Usage in Rust Tests

These fixtures provide real-world test cases for:

1. **Message Deserialization**
   ```rust
   #[test]
   fn test_real_assistant_message() {
       let data = include_str!("../fixtures/raw_messages/assistant_001.json");
       let msg: AssistantMessage = serde_json::from_str(data).unwrap();
       assert_eq!(msg.message.model, "claude-sonnet-4-5-20250929");
   }
   ```

2. **Tool Use Parsing**
   ```rust
   #[test]
   fn test_real_tool_result() {
       let data = include_str!("../fixtures/raw_messages/user_001.json");
       let msg: UserMessage = serde_json::from_str(data).unwrap();
       // Verify tool_result block structure
   }
   ```

3. **Streaming Event Handling**
   ```rust
   #[test]
   fn test_real_stream_event() {
       let data = include_str!("../fixtures/raw_messages/stream_event_001.json");
       let event: StreamEvent = serde_json::from_str(data).unwrap();
       assert_eq!(event.event.type, "message_start");
   }
   ```

4. **Usage Statistics**
   ```rust
   #[test]
   fn test_real_usage_data() {
       let data = include_str!("../fixtures/raw_messages/result_001.json");
       let result: ResultMessage = serde_json::from_str(data).unwrap();
       assert!(result.usage.is_some());
       assert!(result.total_cost_usd.is_some());
   }
   ```

## File Naming Convention

- `<type>_<number>.json` - Numbered sequentially
- Types: `assistant`, `user`, `system`, `result`, `stream_event`
- Numbers are padded to 3 digits (001, 002, etc.)

## Re-running the Capture

To capture fresh data with new API responses:

```bash
cd tools
uv run capture_real_interactions.py
```

**Note**: This will make real API calls and may take several minutes. It will also incur API costs.

## Comparison with Synthetic Data

### Old Synthetic Data (fixtures/*)
- 114 hand-crafted JSON files
- Good for structure validation
- Missing real API metadata
- Simplified examples

### New Real Data (fixtures/raw_messages/*)
- 130 captured from actual API
- Contains real IDs, tokens, costs
- Shows real Claude behavior
- Includes streaming sequences
- Demonstrates tool execution flow
- Shows hook/permission interactions

## Key Insights from Real Data

1. **Message Structure**: Real messages have more nested structure than expected
2. **Usage Stats**: Cache tokens are prominent in real usage
3. **Streaming**: Many more events than anticipated (97 for one response)
4. **Tool IDs**: Follow specific format (`toolu_` prefix)
5. **Session Tracking**: UUIDs and session IDs on every message

## Recommended Usage

For Rust SDK development:
1. Use **real data** for integration tests
2. Use **synthetic data** for unit tests of specific edge cases
3. Test deserialization against **both** datasets
4. Validate serialization produces data matching real format
