#!/usr/bin/env python3
"""Capture all ContentBlock type variants for Rust unit tests.

This script captures: TextBlock, ThinkingBlock, ToolUseBlock, ToolResultBlock.
"""

import asyncio
import json
from pathlib import Path

from claude_agent_sdk import (
    AssistantMessage,
    ClaudeAgentOptions,
    ClaudeSDKClient,
    TextBlock,
    ThinkingBlock,
    ToolUseBlock,
    ToolResultBlock,
)


FIXTURES_DIR = Path(__file__).parent.parent / "fixtures" / "content_blocks"


def save_fixture(name: str, data: dict):
    """Save a fixture to the fixtures directory."""
    file_path = FIXTURES_DIR / f"{name}.json"
    with open(file_path, "w") as f:
        json.dump(data, f, indent=2)
    print(f"✓ Saved {name}.json")


async def capture_text_block():
    """Capture a TextBlock."""
    print("\n=== Capturing Text Block ===")

    options = ClaudeAgentOptions(max_turns=1)

    async with ClaudeSDKClient(options) as client:
        await client.query("Say 'Hello from Claude!'")

        async for message in client.receive_response():
            if isinstance(message, AssistantMessage):
                for block in message.content:
                    if isinstance(block, TextBlock):
                        save_fixture("text_block", {"type": "text", "text": block.text})
                        return


async def capture_thinking_block():
    """Capture a ThinkingBlock (requires extended thinking model)."""
    print("\n=== Capturing Thinking Block ===")

    # Note: Thinking blocks may require specific model or configuration
    options = ClaudeAgentOptions(
        max_turns=1,
        model="claude-sonnet-4-5-20250929",  # Use a model that supports thinking
    )

    async with ClaudeSDKClient(options) as client:
        await client.query("Think carefully: what is 2+2?")

        async for message in client.receive_response():
            if isinstance(message, AssistantMessage):
                for block in message.content:
                    if isinstance(block, ThinkingBlock):
                        save_fixture(
                            "thinking_block",
                            {
                                "type": "thinking",
                                "thinking": block.thinking,
                                "signature": block.signature,
                            },
                        )
                        return

    # If we don't get a thinking block, create a synthetic one for testing
    print("  (Creating synthetic thinking block for testing)")
    save_fixture(
        "thinking_block",
        {
            "type": "thinking",
            "thinking": "Let me think about this problem step by step...",
            "signature": "sha256:abc123",
        },
    )


async def capture_tool_use_block():
    """Capture a ToolUseBlock."""
    print("\n=== Capturing Tool Use Block ===")

    options = ClaudeAgentOptions(
        allowed_tools=["Bash"], max_turns=1, permission_mode="bypassPermissions"
    )

    async with ClaudeSDKClient(options) as client:
        await client.query("Run the bash command: echo 'test output'")

        async for message in client.receive_response():
            if isinstance(message, AssistantMessage):
                for block in message.content:
                    if isinstance(block, ToolUseBlock):
                        save_fixture(
                            "tool_use_block",
                            {
                                "type": "tool_use",
                                "id": block.id,
                                "name": block.name,
                                "input": block.input,
                            },
                        )
                        return


async def capture_tool_result_block():
    """Capture a ToolResultBlock."""
    print("\n=== Capturing Tool Result Block ===")

    # Tool result blocks are typically in system messages after tool execution
    # We'll create synthetic examples since they're part of the internal protocol

    # Success case
    save_fixture(
        "tool_result_block_success",
        {
            "type": "tool_result",
            "tool_use_id": "toolu_01234567890ABC",
            "content": "Command executed successfully\ntest output",
        },
    )

    # Error case
    save_fixture(
        "tool_result_block_error",
        {
            "type": "tool_result",
            "tool_use_id": "toolu_01234567890XYZ",
            "content": "Error: Command failed with exit code 1",
            "is_error": True,
        },
    )

    # With structured content
    save_fixture(
        "tool_result_block_structured",
        {
            "type": "tool_result",
            "tool_use_id": "toolu_01234567890DEF",
            "content": [{"type": "text", "text": "Result data"}],
        },
    )

    print("  ✓ Created synthetic tool result blocks")


async def main():
    """Run all capture functions."""
    print("Starting content block capture...")
    print(f"Fixtures will be saved to: {FIXTURES_DIR}")

    # Create fixtures directory
    FIXTURES_DIR.mkdir(parents=True, exist_ok=True)

    # Capture all content block types
    await capture_text_block()
    await capture_thinking_block()
    await capture_tool_use_block()
    await capture_tool_result_block()

    print("\n✅ All content block fixtures captured successfully!")


if __name__ == "__main__":
    asyncio.run(main())
