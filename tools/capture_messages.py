#!/usr/bin/env python3
"""Capture all Message type variants for Rust unit tests.

This script uses the Python Claude SDK to capture real JSON responses
for all message types: UserMessage, AssistantMessage, SystemMessage,
ResultMessage, and StreamEvent.
"""

import asyncio
import json
from pathlib import Path

from claude_agent_sdk import (
    AssistantMessage,
    ClaudeAgentOptions,
    ClaudeSDKClient,
    ResultMessage,
    SystemMessage,
    TextBlock,
    ThinkingBlock,
    ToolUseBlock,
    ToolResultBlock,
)
from claude_agent_sdk.types import StreamEvent


FIXTURES_DIR = Path(__file__).parent.parent / "fixtures" / "messages"


def save_fixture(name: str, data: dict):
    """Save a fixture to the fixtures directory."""
    file_path = FIXTURES_DIR / f"{name}.json"
    with open(file_path, "w") as f:
        json.dump(data, f, indent=2)
    print(f"✓ Saved {name}.json")


def message_to_dict(message) -> dict:
    """Convert a message object to a dictionary for JSON serialization."""
    if isinstance(message, AssistantMessage):
        content = []
        for block in message.content:
            if isinstance(block, TextBlock):
                content.append({"type": "text", "text": block.text})
            elif isinstance(block, ThinkingBlock):
                content.append(
                    {
                        "type": "thinking",
                        "thinking": block.thinking,
                        "signature": block.signature,
                    }
                )
            elif isinstance(block, ToolUseBlock):
                content.append(
                    {
                        "type": "tool_use",
                        "id": block.id,
                        "name": block.name,
                        "input": block.input,
                    }
                )
            elif isinstance(block, ToolResultBlock):
                block_dict = {"type": "tool_result", "tool_use_id": block.tool_use_id}
                if block.content is not None:
                    block_dict["content"] = block.content
                if block.is_error is not None:
                    block_dict["is_error"] = block.is_error
                content.append(block_dict)

        result = {"type": "assistant", "content": content, "model": message.model}
        if message.parent_tool_use_id:
            result["parent_tool_use_id"] = message.parent_tool_use_id
        return result

    elif isinstance(message, SystemMessage):
        return {"type": "system", "subtype": message.subtype, "data": message.data}

    elif isinstance(message, ResultMessage):
        result = {
            "type": "result",
            "subtype": message.subtype,
            "duration_ms": message.duration_ms,
            "duration_api_ms": message.duration_api_ms,
            "is_error": message.is_error,
            "num_turns": message.num_turns,
            "session_id": message.session_id,
        }
        if message.total_cost_usd is not None:
            result["total_cost_usd"] = message.total_cost_usd
        if message.usage is not None:
            result["usage"] = message.usage
        if message.result is not None:
            result["result"] = message.result
        return result

    elif isinstance(message, StreamEvent):
        result = {
            "type": "stream",
            "uuid": message.uuid,
            "session_id": message.session_id,
            "event": message.event,
        }
        if message.parent_tool_use_id:
            result["parent_tool_use_id"] = message.parent_tool_use_id
        return result

    return {}


async def capture_simple_assistant_message():
    """Capture a simple AssistantMessage with text."""
    print("\n=== Capturing Simple Assistant Message ===")

    options = ClaudeAgentOptions(
        max_turns=1,
        system_prompt="You are a helpful assistant. Keep responses very brief.",
    )

    async with ClaudeSDKClient(options) as client:
        await client.query("Say 'Hello, World!' and nothing else.")

        async for message in client.receive_response():
            if isinstance(message, AssistantMessage):
                save_fixture("assistant_message_simple", message_to_dict(message))
                break


async def capture_assistant_with_tool_use():
    """Capture an AssistantMessage with tool use."""
    print("\n=== Capturing Assistant Message with Tool Use ===")

    options = ClaudeAgentOptions(
        allowed_tools=["Bash"], max_turns=1, permission_mode="bypassPermissions"
    )

    async with ClaudeSDKClient(options) as client:
        await client.query("Run the command: echo 'test'")

        async for message in client.receive_response():
            if isinstance(message, AssistantMessage):
                # Check if this message has tool use
                has_tool_use = any(
                    isinstance(block, ToolUseBlock) for block in message.content
                )
                if has_tool_use:
                    save_fixture(
                        "assistant_message_with_tool_use", message_to_dict(message)
                    )
                    break


async def capture_result_message():
    """Capture a ResultMessage."""
    print("\n=== Capturing Result Message ===")

    options = ClaudeAgentOptions(max_turns=1)

    async with ClaudeSDKClient(options) as client:
        await client.query("What is 1+1?")

        async for message in client.receive_response():
            if isinstance(message, ResultMessage):
                save_fixture("result_message", message_to_dict(message))
                break


async def capture_system_message():
    """Capture SystemMessage variants."""
    print("\n=== Capturing System Messages ===")

    options = ClaudeAgentOptions(max_turns=1)

    async with ClaudeSDKClient(options) as client:
        await client.query("Hello")

        async for message in client.receive_response():
            if isinstance(message, SystemMessage):
                save_fixture(
                    f"system_message_{message.subtype}", message_to_dict(message)
                )


async def capture_stream_event():
    """Capture StreamEvent (requires streaming mode)."""
    print("\n=== Capturing Stream Event ===")

    options = ClaudeAgentOptions(max_turns=1, include_partial_messages=True)

    async with ClaudeSDKClient(options) as client:
        await client.query("Say hello")

        async for message in client.receive_response():
            if isinstance(message, StreamEvent):
                save_fixture("stream_event", message_to_dict(message))
                break


async def main():
    """Run all capture functions."""
    print("Starting message capture...")
    print(f"Fixtures will be saved to: {FIXTURES_DIR}")

    # Create fixtures directory
    FIXTURES_DIR.mkdir(parents=True, exist_ok=True)

    # Capture all message types
    await capture_simple_assistant_message()
    await capture_assistant_with_tool_use()
    await capture_result_message()
    await capture_system_message()
    await capture_stream_event()

    print("\n✅ All message fixtures captured successfully!")


if __name__ == "__main__":
    asyncio.run(main())
