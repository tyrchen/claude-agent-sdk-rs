#!/usr/bin/env python3
"""Capture SDK Control Protocol type variants for Rust unit tests.

This script captures: SDKControlRequest, SDKControlResponse,
and all their subtypes.
"""

import json
from pathlib import Path


FIXTURES_DIR = Path(__file__).parent.parent / "fixtures" / "control_protocol"


def save_fixture(name: str, data: dict):
    """Save a fixture to the fixtures directory."""
    file_path = FIXTURES_DIR / f"{name}.json"
    with open(file_path, "w") as f:
        json.dump(data, f, indent=2)
    print(f"✓ Saved {name}.json")


def capture_control_requests():
    """Capture all SDKControlRequest variants."""
    print("\n=== Capturing SDK Control Requests ===")

    # SDKControlInterruptRequest
    request = {
        "type": "control_request",
        "request_id": "req_001",
        "request": {"subtype": "interrupt"},
    }
    save_fixture("control_request_interrupt", request)

    # SDKControlPermissionRequest - basic
    request = {
        "type": "control_request",
        "request_id": "req_002",
        "request": {
            "subtype": "can_use_tool",
            "tool_name": "Write",
            "input": {"file_path": "/tmp/test.txt", "content": "Hello, World!"},
            "permission_suggestions": None,
            "blocked_path": None,
        },
    }
    save_fixture("control_request_permission_basic", request)

    # SDKControlPermissionRequest - with suggestions and blocked path
    request = {
        "type": "control_request",
        "request_id": "req_003",
        "request": {
            "subtype": "can_use_tool",
            "tool_name": "Bash",
            "input": {"command": "rm -rf /"},
            "permission_suggestions": [
                {
                    "type": "addRules",
                    "rules": [{"toolName": "Bash", "ruleContent": "rm *"}],
                    "behavior": "deny",
                    "destination": "session",
                }
            ],
            "blocked_path": "/etc/sensitive",
        },
    }
    save_fixture("control_request_permission_with_suggestions", request)

    # SDKControlInitializeRequest - no hooks
    request = {
        "type": "control_request",
        "request_id": "req_004",
        "request": {"subtype": "initialize", "hooks": None},
    }
    save_fixture("control_request_initialize_no_hooks", request)

    # SDKControlInitializeRequest - with hooks
    request = {
        "type": "control_request",
        "request_id": "req_005",
        "request": {
            "subtype": "initialize",
            "hooks": {
                "PreToolUse": [{"matcher": "Bash", "callback_id": "cb_001"}],
                "PostToolUse": [{"matcher": None, "callback_id": "cb_002"}],
            },
        },
    }
    save_fixture("control_request_initialize_with_hooks", request)

    # SDKControlSetPermissionModeRequest
    for mode in ["default", "acceptEdits", "plan", "bypassPermissions"]:
        request = {
            "type": "control_request",
            "request_id": f"req_mode_{mode}",
            "request": {"subtype": "set_permission_mode", "mode": mode},
        }
        save_fixture(f"control_request_set_mode_{mode}", request)

    # SDKHookCallbackRequest - PreToolUse
    request = {
        "type": "control_request",
        "request_id": "req_006",
        "request": {
            "subtype": "hook_callback",
            "callback_id": "cb_001",
            "input": {
                "hook_event_name": "PreToolUse",
                "session_id": "session_123",
                "transcript_path": "/path/to/transcript.jsonl",
                "cwd": "/Users/test/project",
                "tool_name": "Bash",
                "tool_input": {"command": "echo 'test'"},
            },
            "tool_use_id": "toolu_abc123",
        },
    }
    save_fixture("control_request_hook_callback_pretooluse", request)

    # SDKHookCallbackRequest - PostToolUse
    request = {
        "type": "control_request",
        "request_id": "req_007",
        "request": {
            "subtype": "hook_callback",
            "callback_id": "cb_002",
            "input": {
                "hook_event_name": "PostToolUse",
                "session_id": "session_123",
                "transcript_path": "/path/to/transcript.jsonl",
                "cwd": "/Users/test/project",
                "tool_name": "Bash",
                "tool_input": {"command": "echo 'test'"},
                "tool_response": "test\n",
            },
            "tool_use_id": "toolu_abc123",
        },
    }
    save_fixture("control_request_hook_callback_posttooluse", request)

    # SDKHookCallbackRequest - UserPromptSubmit
    request = {
        "type": "control_request",
        "request_id": "req_008",
        "request": {
            "subtype": "hook_callback",
            "callback_id": "cb_003",
            "input": {
                "hook_event_name": "UserPromptSubmit",
                "session_id": "session_123",
                "transcript_path": "/path/to/transcript.jsonl",
                "cwd": "/Users/test/project",
                "prompt": "What is the weather today?",
            },
            "tool_use_id": None,
        },
    }
    save_fixture("control_request_hook_callback_userpromptsubmit", request)

    # SDKControlMcpMessageRequest
    request = {
        "type": "control_request",
        "request_id": "req_009",
        "request": {
            "subtype": "mcp_message",
            "server_name": "my_mcp_server",
            "message": {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "get_weather",
                    "arguments": {"location": "San Francisco"},
                },
            },
        },
    }
    save_fixture("control_request_mcp_message", request)

    print("  ✓ Created all control request variants")


def capture_control_responses():
    """Capture all SDKControlResponse variants."""
    print("\n=== Capturing SDK Control Responses ===")

    # ControlResponse - success with no data
    response = {
        "type": "control_response",
        "response": {"subtype": "success", "request_id": "req_001", "response": None},
    }
    save_fixture("control_response_success_no_data", response)

    # ControlResponse - success with permission allow
    response = {
        "type": "control_response",
        "response": {
            "subtype": "success",
            "request_id": "req_002",
            "response": {"behavior": "allow"},
        },
    }
    save_fixture("control_response_success_permission_allow", response)

    # ControlResponse - success with permission deny
    response = {
        "type": "control_response",
        "response": {
            "subtype": "success",
            "request_id": "req_003",
            "response": {
                "behavior": "deny",
                "message": "Operation not permitted",
                "interrupt": False,
            },
        },
    }
    save_fixture("control_response_success_permission_deny", response)

    # ControlResponse - success with hook output
    response = {
        "type": "control_response",
        "response": {
            "subtype": "success",
            "request_id": "req_004",
            "response": {
                "continue": True,
                "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": "allow",
                },
            },
        },
    }
    save_fixture("control_response_success_hook_output", response)

    # ControlResponse - success with MCP response
    response = {
        "type": "control_response",
        "response": {
            "subtype": "success",
            "request_id": "req_005",
            "response": {
                "jsonrpc": "2.0",
                "id": 1,
                "result": {"temperature": 72, "conditions": "sunny"},
            },
        },
    }
    save_fixture("control_response_success_mcp_result", response)

    # ControlErrorResponse - generic error
    response = {
        "type": "control_response",
        "response": {
            "subtype": "error",
            "request_id": "req_006",
            "error": "Invalid request: missing required field 'tool_name'",
        },
    }
    save_fixture("control_response_error_generic", response)

    # ControlErrorResponse - permission denied
    response = {
        "type": "control_response",
        "response": {
            "subtype": "error",
            "request_id": "req_007",
            "error": "Permission denied: user callback rejected tool use",
        },
    }
    save_fixture("control_response_error_permission", response)

    # ControlErrorResponse - hook execution error
    response = {
        "type": "control_response",
        "response": {
            "subtype": "error",
            "request_id": "req_008",
            "error": "Hook execution failed: callback threw exception",
        },
    }
    save_fixture("control_response_error_hook", response)

    print("  ✓ Created all control response variants")


def main():
    """Run all capture functions."""
    print("Starting control protocol capture...")
    print(f"Fixtures will be saved to: {FIXTURES_DIR}")

    # Create fixtures directory
    FIXTURES_DIR.mkdir(parents=True, exist_ok=True)

    # Capture all control protocol types
    capture_control_requests()
    capture_control_responses()

    print("\n✅ All control protocol fixtures captured successfully!")


if __name__ == "__main__":
    main()
