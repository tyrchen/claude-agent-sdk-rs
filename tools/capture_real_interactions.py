#!/usr/bin/env python3
"""Capture REAL JSON data from Claude Agent SDK interactions.

This script intercepts the raw JSON messages from the CLI before they are
parsed by the SDK, giving us the actual wire format data.
"""

import asyncio
import json
from pathlib import Path
from typing import Any

from claude_agent_sdk import ClaudeAgentOptions, ClaudeSDKClient


FIXTURES_DIR = Path(__file__).parent.parent / "fixtures" / "raw_messages"
FIXTURES_DIR.mkdir(parents=True, exist_ok=True)

# Counter for unique filenames
message_counter = {}


def save_raw_message(msg_type: str, data: dict[str, Any]):
    """Save raw message JSON to fixtures."""
    if msg_type not in message_counter:
        message_counter[msg_type] = 0
    message_counter[msg_type] += 1

    filename = f"{msg_type}_{message_counter[msg_type]:03d}.json"
    file_path = FIXTURES_DIR / filename

    with open(file_path, "w") as f:
        json.dump(data, f, indent=2)
    print(f"✓ Saved {filename}")


async def capture_basic_conversation():
    """Capture a basic conversation with text responses."""
    print("\n=== Capturing Basic Conversation ===")

    options = ClaudeAgentOptions(
        max_turns=2,
    )

    # Monkey-patch the message parser to capture raw data
    from claude_agent_sdk._internal import message_parser

    original_parse = message_parser.parse_message

    def capturing_parse(data: dict[str, Any]):
        # Save the raw message before parsing
        msg_type = data.get("type", "unknown")
        save_raw_message(msg_type, data)
        return original_parse(data)

    message_parser.parse_message = capturing_parse

    try:
        async with ClaudeSDKClient(options) as client:
            await client.query(
                "Tell me a very short joke about programming. Just one sentence."
            )

            async for _ in client.receive_response():
                pass  # Messages are captured in the parser
    finally:
        message_parser.parse_message = original_parse


async def capture_tool_usage():
    """Capture messages with tool usage."""
    print("\n=== Capturing Tool Usage ===")

    options = ClaudeAgentOptions(
        allowed_tools=["Bash", "Read", "Write"],
        permission_mode="bypassPermissions",
        max_turns=3,
    )

    from claude_agent_sdk._internal import message_parser

    original_parse = message_parser.parse_message

    def capturing_parse(data: dict[str, Any]):
        msg_type = data.get("type", "unknown")
        save_raw_message(msg_type, data)
        return original_parse(data)

    message_parser.parse_message = capturing_parse

    try:
        async with ClaudeSDKClient(options) as client:
            await client.query(
                "Create a file called test.txt with the text 'Hello World' and then read it back to verify."
            )

            async for _ in client.receive_response():
                pass
    finally:
        message_parser.parse_message = original_parse


async def capture_with_streaming():
    """Capture streaming messages."""
    print("\n=== Capturing Streaming Mode ===")

    options = ClaudeAgentOptions(
        include_partial_messages=True,
        max_turns=1,
    )

    from claude_agent_sdk._internal import message_parser

    original_parse = message_parser.parse_message

    def capturing_parse(data: dict[str, Any]):
        msg_type = data.get("type", "unknown")
        save_raw_message(msg_type, data)
        return original_parse(data)

    message_parser.parse_message = capturing_parse

    try:
        async with ClaudeSDKClient(options) as client:
            await client.query(
                "Count from 1 to 5, with a brief explanation of each number."
            )

            async for _ in client.receive_response():
                pass
    finally:
        message_parser.parse_message = original_parse


async def capture_with_hooks():
    """Capture messages with hooks that actually modify behavior."""
    print("\n=== Capturing With Hooks ===")

    from claude_agent_sdk.types import (
        HookContext,
        HookInput,
        HookJSONOutput,
        HookMatcher,
    )

    async def logging_hook(
        input_data: HookInput, tool_use_id: str | None, context: HookContext
    ) -> HookJSONOutput:
        """Hook that logs but allows everything."""
        print(f"  Hook triggered: {input_data['hook_event_name']}")
        if input_data.get("tool_name"):
            print(f"    Tool: {input_data['tool_name']}")
        return {}

    options = ClaudeAgentOptions(
        allowed_tools=["Bash"],
        permission_mode="bypassPermissions",
        max_turns=2,
        hooks={
            "PreToolUse": [HookMatcher(matcher="Bash", hooks=[logging_hook])],
            "PostToolUse": [HookMatcher(matcher="Bash", hooks=[logging_hook])],
        },
    )

    from claude_agent_sdk._internal import message_parser

    original_parse = message_parser.parse_message

    def capturing_parse(data: dict[str, Any]):
        msg_type = data.get("type", "unknown")
        save_raw_message(msg_type, data)
        return original_parse(data)

    message_parser.parse_message = capturing_parse

    try:
        async with ClaudeSDKClient(options) as client:
            await client.query("Run the command: echo 'Hook test successful'")

            async for _ in client.receive_response():
                pass
    finally:
        message_parser.parse_message = original_parse


async def capture_with_permissions():
    """Capture messages with permission callbacks."""
    print("\n=== Capturing With Permission Callbacks ===")

    from claude_agent_sdk import (
        PermissionResultAllow,
        PermissionResultDeny,
        ToolPermissionContext,
    )

    deny_count = 0

    async def permission_callback(
        tool_name: str, input_data: dict, context: ToolPermissionContext
    ):
        nonlocal deny_count
        print(f"  Permission requested for: {tool_name}")

        # Deny the first write, allow the second
        if tool_name == "Write":
            deny_count += 1
            if deny_count == 1:
                print(f"    Denying first write")
                return PermissionResultDeny(message="First write denied for testing")
            else:
                print(f"    Allowing second write")
                return PermissionResultAllow()

        return PermissionResultAllow()

    options = ClaudeAgentOptions(
        allowed_tools=["Write"],
        permission_mode="default",  # Use default to trigger callbacks
        can_use_tool=permission_callback,
        max_turns=3,
    )

    from claude_agent_sdk._internal import message_parser

    original_parse = message_parser.parse_message

    def capturing_parse(data: dict[str, Any]):
        msg_type = data.get("type", "unknown")
        save_raw_message(msg_type, data)
        return original_parse(data)

    message_parser.parse_message = capturing_parse

    try:
        async with ClaudeSDKClient(options) as client:
            await client.query(
                "Try to write 'test' to a file called output.txt. If it fails, try again."
            )

            async for _ in client.receive_response():
                pass
    finally:
        message_parser.parse_message = original_parse


async def capture_with_agents():
    """Capture messages using custom agents."""
    print("\n=== Capturing With Custom Agents ===")

    from claude_agent_sdk.types import AgentDefinition

    options = ClaudeAgentOptions(
        agents={
            "calculator": AgentDefinition(
                description="A simple calculator agent",
                prompt="You are a calculator. Perform mathematical calculations accurately.",
                tools=["Bash"],
                model="haiku",
            )
        },
        permission_mode="bypassPermissions",
        max_turns=2,
    )

    from claude_agent_sdk._internal import message_parser

    original_parse = message_parser.parse_message

    def capturing_parse(data: dict[str, Any]):
        msg_type = data.get("type", "unknown")
        save_raw_message(msg_type, data)
        return original_parse(data)

    message_parser.parse_message = capturing_parse

    try:
        async with ClaudeSDKClient(options) as client:
            await client.query("Calculate 15 * 23 + 47 using bash")

            async for _ in client.receive_response():
                pass
    finally:
        message_parser.parse_message = original_parse


async def main():
    """Run all capture scenarios."""
    print("Starting real data capture...")
    print(f"Fixtures will be saved to: {FIXTURES_DIR}")
    print("\nThis will make real API calls and may take a few minutes...")

    try:
        await capture_basic_conversation()
        await capture_tool_usage()
        await capture_with_streaming()
        await capture_with_hooks()
        await capture_with_permissions()
        await capture_with_agents()

        print("\n✅ All real data captured successfully!")
        print(f"\nTotal files created: {sum(message_counter.values())}")
        print("\nBreakdown:")
        for msg_type, count in sorted(message_counter.items()):
            print(f"  {msg_type}: {count} messages")

    except Exception as e:
        print(f"\n❌ Error during capture: {e}")
        import traceback

        traceback.print_exc()


if __name__ == "__main__":
    asyncio.run(main())
