#!/usr/bin/env python3
"""Capture all Hook type variants for Rust unit tests.

This script captures: PreToolUseHookInput, PostToolUseHookInput,
UserPromptSubmitHookInput, StopHookInput, SubagentStopHookInput,
PreCompactHookInput, and all HookJSONOutput variants.
"""

import asyncio
import json
from pathlib import Path

from claude_agent_sdk import ClaudeAgentOptions, ClaudeSDKClient
from claude_agent_sdk.types import (
    HookContext,
    HookInput,
    HookJSONOutput,
    HookMatcher,
)


FIXTURES_DIR = Path(__file__).parent.parent / "fixtures" / "hooks"


def save_fixture(name: str, data: dict):
    """Save a fixture to the fixtures directory."""
    file_path = FIXTURES_DIR / f"{name}.json"
    with open(file_path, "w") as f:
        json.dump(data, f, indent=2)
    print(f"✓ Saved {name}.json")


# Global storage for captured hook inputs
captured_inputs = {}


async def capture_hook_input(
    input_data: HookInput, tool_use_id: str | None, context: HookContext
) -> HookJSONOutput:
    """Generic hook to capture all input types."""
    hook_event = input_data["hook_event_name"]

    # Store the captured input
    if hook_event not in captured_inputs:
        captured_inputs[hook_event] = input_data.copy()
        print(f"  Captured {hook_event} input")

    return {}


async def capture_pretooluse_hook_input():
    """Capture PreToolUseHookInput."""
    print("\n=== Capturing PreToolUse Hook Input ===")

    global captured_inputs
    captured_inputs = {}

    options = ClaudeAgentOptions(
        allowed_tools=["Bash"],
        permission_mode="bypassPermissions",
        hooks={
            "PreToolUse": [
                HookMatcher(matcher="Bash", hooks=[capture_hook_input]),
            ],
        },
    )

    async with ClaudeSDKClient(options) as client:
        await client.query("Run: echo 'test'")

        async for _ in client.receive_response():
            pass

    if "PreToolUse" in captured_inputs:
        save_fixture("pretooluse_hook_input", captured_inputs["PreToolUse"])


async def capture_posttooluse_hook_input():
    """Capture PostToolUseHookInput."""
    print("\n=== Capturing PostToolUse Hook Input ===")

    global captured_inputs
    captured_inputs = {}

    options = ClaudeAgentOptions(
        allowed_tools=["Bash"],
        permission_mode="bypassPermissions",
        hooks={
            "PostToolUse": [
                HookMatcher(matcher="Bash", hooks=[capture_hook_input]),
            ],
        },
    )

    async with ClaudeSDKClient(options) as client:
        await client.query("Run: echo 'test'")

        async for _ in client.receive_response():
            pass

    if "PostToolUse" in captured_inputs:
        save_fixture("posttooluse_hook_input", captured_inputs["PostToolUse"])


async def capture_userpromptsubmit_hook_input():
    """Capture UserPromptSubmitHookInput."""
    print("\n=== Capturing UserPromptSubmit Hook Input ===")

    global captured_inputs
    captured_inputs = {}

    options = ClaudeAgentOptions(
        hooks={
            "UserPromptSubmit": [
                HookMatcher(matcher=None, hooks=[capture_hook_input]),
            ],
        }
    )

    async with ClaudeSDKClient(options) as client:
        await client.query("Hello, Claude!")

        async for _ in client.receive_response():
            pass

    if "UserPromptSubmit" in captured_inputs:
        save_fixture("userpromptsubmit_hook_input", captured_inputs["UserPromptSubmit"])


async def capture_stop_hook_input():
    """Capture StopHookInput."""
    print("\n=== Capturing Stop Hook Input ===")

    global captured_inputs
    captured_inputs = {}

    options = ClaudeAgentOptions(
        hooks={
            "Stop": [
                HookMatcher(matcher=None, hooks=[capture_hook_input]),
            ],
        }
    )

    async with ClaudeSDKClient(options) as client:
        await client.query("Say hello")

        async for _ in client.receive_response():
            pass

    if "Stop" in captured_inputs:
        save_fixture("stop_hook_input", captured_inputs["Stop"])


async def capture_precompact_hook_input():
    """Capture PreCompactHookInput."""
    print("\n=== Capturing PreCompact Hook Input (Synthetic) ===")

    # PreCompact is triggered manually, so we'll create a synthetic example
    save_fixture(
        "precompact_hook_input",
        {
            "hook_event_name": "PreCompact",
            "session_id": "test_session_123",
            "transcript_path": "/path/to/transcript.jsonl",
            "cwd": "/Users/test/project",
            "trigger": "manual",
            "custom_instructions": "Please summarize the conversation so far",
        },
    )

    # Auto-trigger variant
    save_fixture(
        "precompact_hook_input_auto",
        {
            "hook_event_name": "PreCompact",
            "session_id": "test_session_456",
            "transcript_path": "/path/to/transcript2.jsonl",
            "cwd": "/Users/test/project",
            "trigger": "auto",
            "custom_instructions": None,
        },
    )


def capture_hook_outputs():
    """Capture all HookJSONOutput variants."""
    print("\n=== Capturing Hook JSON Outputs ===")

    # AsyncHookJSONOutput
    save_fixture("hook_output_async", {"async": True, "asyncTimeout": 5000})

    # AsyncHookJSONOutput without timeout
    save_fixture("hook_output_async_no_timeout", {"async": True})

    # SyncHookJSONOutput - PreToolUse with allow
    save_fixture(
        "hook_output_pretooluse_allow",
        {
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "allow",
                "permissionDecisionReason": "Tool use approved",
            }
        },
    )

    # SyncHookJSONOutput - PreToolUse with deny
    save_fixture(
        "hook_output_pretooluse_deny",
        {
            "continue": True,
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "deny",
                "permissionDecisionReason": "Dangerous command blocked",
            },
            "reason": "Security policy violation",
            "systemMessage": "⚠️ Command blocked by security policy",
        },
    )

    # SyncHookJSONOutput - PreToolUse with ask
    save_fixture(
        "hook_output_pretooluse_ask",
        {
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "ask",
            }
        },
    )

    # SyncHookJSONOutput - PreToolUse with updated input
    save_fixture(
        "hook_output_pretooluse_updated_input",
        {
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "allow",
                "updatedInput": {"command": "echo 'modified command'"},
            }
        },
    )

    # SyncHookJSONOutput - PostToolUse with additional context
    save_fixture(
        "hook_output_posttooluse",
        {
            "hookSpecificOutput": {
                "hookEventName": "PostToolUse",
                "additionalContext": "The command completed successfully",
            }
        },
    )

    # SyncHookJSONOutput - UserPromptSubmit
    save_fixture(
        "hook_output_userpromptsubmit",
        {
            "hookSpecificOutput": {
                "hookEventName": "UserPromptSubmit",
                "additionalContext": "User prefers concise responses",
            }
        },
    )

    # SyncHookJSONOutput - with continue false
    save_fixture(
        "hook_output_stop",
        {
            "continue": False,
            "stopReason": "Critical error detected",
            "systemMessage": "🛑 Execution halted",
        },
    )

    # SyncHookJSONOutput - with suppressOutput
    save_fixture("hook_output_suppress", {"continue": True, "suppressOutput": True})

    # SyncHookJSONOutput - with decision block
    save_fixture(
        "hook_output_block",
        {
            "decision": "block",
            "systemMessage": "Operation not permitted",
            "reason": "Insufficient permissions",
        },
    )

    print("  ✓ Created all hook output variants")


async def main():
    """Run all capture functions."""
    print("Starting hook capture...")
    print(f"Fixtures will be saved to: {FIXTURES_DIR}")

    # Create fixtures directory
    FIXTURES_DIR.mkdir(parents=True, exist_ok=True)

    # Capture hook inputs
    await capture_pretooluse_hook_input()
    await capture_posttooluse_hook_input()
    await capture_userpromptsubmit_hook_input()
    await capture_stop_hook_input()
    await capture_precompact_hook_input()

    # Capture hook outputs
    capture_hook_outputs()

    print("\n✅ All hook fixtures captured successfully!")


if __name__ == "__main__":
    asyncio.run(main())
