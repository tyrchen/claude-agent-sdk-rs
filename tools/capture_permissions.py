#!/usr/bin/env python3
"""Capture all Permission type variants for Rust unit tests.

This script captures: PermissionUpdate, PermissionRuleValue,
PermissionResultAllow, PermissionResultDeny.
"""

import asyncio
import json
from pathlib import Path

from claude_agent_sdk import (
    ClaudeAgentOptions,
    ClaudeSDKClient,
    PermissionResultAllow,
    PermissionResultDeny,
    PermissionUpdate,
    ToolPermissionContext,
)
from claude_agent_sdk.types import PermissionRuleValue


FIXTURES_DIR = Path(__file__).parent.parent / "fixtures" / "permissions"


def save_fixture(name: str, data: dict):
    """Save a fixture to the fixtures directory."""
    file_path = FIXTURES_DIR / f"{name}.json"
    with open(file_path, "w") as f:
        json.dump(data, f, indent=2)
    print(f"✓ Saved {name}.json")


def capture_permission_updates():
    """Capture all PermissionUpdate variants."""
    print("\n=== Capturing Permission Updates ===")

    # addRules variant
    update = PermissionUpdate(
        type="addRules",
        rules=[
            PermissionRuleValue(tool_name="Write", rule_content="*.txt"),
            PermissionRuleValue(tool_name="Edit", rule_content="*.md"),
        ],
        behavior="allow",
        destination="userSettings",
    )
    save_fixture("permission_update_add_rules", update.to_dict())

    # replaceRules variant
    update = PermissionUpdate(
        type="replaceRules",
        rules=[
            PermissionRuleValue(tool_name="Bash", rule_content="echo *"),
        ],
        behavior="deny",
        destination="projectSettings",
    )
    save_fixture("permission_update_replace_rules", update.to_dict())

    # removeRules variant
    update = PermissionUpdate(
        type="removeRules",
        rules=[
            PermissionRuleValue(tool_name="Write", rule_content="/tmp/*"),
        ],
        behavior="ask",
        destination="localSettings",
    )
    save_fixture("permission_update_remove_rules", update.to_dict())

    # setMode variant
    update = PermissionUpdate(type="setMode", mode="plan", destination="session")
    save_fixture("permission_update_set_mode", update.to_dict())

    # setMode with different modes
    for mode in ["default", "acceptEdits", "bypassPermissions"]:
        update = PermissionUpdate(type="setMode", mode=mode, destination="session")
        save_fixture(f"permission_update_set_mode_{mode}", update.to_dict())

    # addDirectories variant
    update = PermissionUpdate(
        type="addDirectories",
        directories=["/path/to/safe/dir1", "/path/to/safe/dir2"],
        destination="userSettings",
    )
    save_fixture("permission_update_add_directories", update.to_dict())

    # removeDirectories variant
    update = PermissionUpdate(
        type="removeDirectories",
        directories=["/path/to/remove"],
        destination="projectSettings",
    )
    save_fixture("permission_update_remove_directories", update.to_dict())

    print("  ✓ Created all permission update variants")


async def capture_permission_results():
    """Capture PermissionResult variants through callback."""
    print("\n=== Capturing Permission Results ===")

    # PermissionResultAllow - simple
    result = PermissionResultAllow()
    save_fixture("permission_result_allow_simple", {"behavior": result.behavior})

    # PermissionResultAllow - with updated input
    result = PermissionResultAllow(
        updated_input={
            "file_path": "/safe/path/file.txt",
            "content": "modified content",
        }
    )
    save_fixture(
        "permission_result_allow_updated_input",
        {"behavior": result.behavior, "updated_input": result.updated_input},
    )

    # PermissionResultAllow - with updated permissions
    result = PermissionResultAllow(
        updated_permissions=[
            PermissionUpdate(
                type="addRules",
                rules=[PermissionRuleValue(tool_name="Write", rule_content="*.txt")],
                behavior="allow",
                destination="session",
            )
        ]
    )
    save_fixture(
        "permission_result_allow_updated_permissions",
        {
            "behavior": result.behavior,
            "updated_permissions": [p.to_dict() for p in result.updated_permissions],
        },
    )

    # PermissionResultAllow - with both updated input and permissions
    result = PermissionResultAllow(
        updated_input={"command": "echo 'safe'"},
        updated_permissions=[
            PermissionUpdate(type="setMode", mode="acceptEdits", destination="session")
        ],
    )
    save_fixture(
        "permission_result_allow_complete",
        {
            "behavior": result.behavior,
            "updated_input": result.updated_input,
            "updated_permissions": [p.to_dict() for p in result.updated_permissions],
        },
    )

    # PermissionResultDeny - simple
    result = PermissionResultDeny()
    save_fixture(
        "permission_result_deny_simple",
        {
            "behavior": result.behavior,
            "message": result.message,
            "interrupt": result.interrupt,
        },
    )

    # PermissionResultDeny - with message
    result = PermissionResultDeny(
        message="Operation not permitted: insufficient privileges"
    )
    save_fixture(
        "permission_result_deny_message",
        {
            "behavior": result.behavior,
            "message": result.message,
            "interrupt": result.interrupt,
        },
    )

    # PermissionResultDeny - with interrupt
    result = PermissionResultDeny(
        message="Critical security violation detected", interrupt=True
    )
    save_fixture(
        "permission_result_deny_interrupt",
        {
            "behavior": result.behavior,
            "message": result.message,
            "interrupt": result.interrupt,
        },
    )

    print("  ✓ Created all permission result variants")


def capture_permission_rule_values():
    """Capture PermissionRuleValue examples."""
    print("\n=== Capturing Permission Rule Values ===")

    # With rule content
    rule = PermissionRuleValue(tool_name="Write", rule_content="*.txt")
    save_fixture(
        "permission_rule_value_with_content",
        {"toolName": rule.tool_name, "ruleContent": rule.rule_content},
    )

    # Without rule content (applies to all uses of the tool)
    rule = PermissionRuleValue(tool_name="Bash", rule_content=None)
    save_fixture(
        "permission_rule_value_no_content",
        {"toolName": rule.tool_name, "ruleContent": rule.rule_content},
    )

    print("  ✓ Created permission rule value variants")


async def capture_tool_permission_context():
    """Capture ToolPermissionContext examples."""
    print("\n=== Capturing Tool Permission Context ===")

    # Context with suggestions
    context_dict = {
        "signal": None,
        "suggestions": [
            PermissionUpdate(
                type="addRules",
                rules=[PermissionRuleValue(tool_name="Write", rule_content="*.txt")],
                behavior="allow",
                destination="session",
            ).to_dict()
        ],
    }
    save_fixture("tool_permission_context_with_suggestions", context_dict)

    # Empty context
    context_dict = {"signal": None, "suggestions": []}
    save_fixture("tool_permission_context_empty", context_dict)

    print("  ✓ Created tool permission context variants")


async def main():
    """Run all capture functions."""
    print("Starting permission capture...")
    print(f"Fixtures will be saved to: {FIXTURES_DIR}")

    # Create fixtures directory
    FIXTURES_DIR.mkdir(parents=True, exist_ok=True)

    # Capture all permission types
    capture_permission_updates()
    await capture_permission_results()
    capture_permission_rule_values()
    await capture_tool_permission_context()

    print("\n✅ All permission fixtures captured successfully!")


if __name__ == "__main__":
    asyncio.run(main())
