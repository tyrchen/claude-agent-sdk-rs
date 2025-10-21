#!/usr/bin/env python3
"""Capture Agent and Options type variants for Rust unit tests.

This script captures: AgentDefinition, SystemPromptPreset,
ClaudeAgentOptions with various configurations.
"""

import json
from pathlib import Path

from claude_agent_sdk.types import AgentDefinition


FIXTURES_DIR = Path(__file__).parent.parent / "fixtures" / "agents"


def save_fixture(name: str, data: dict):
    """Save a fixture to the fixtures directory."""
    file_path = FIXTURES_DIR / f"{name}.json"
    with open(file_path, "w") as f:
        json.dump(data, f, indent=2)
    print(f"✓ Saved {name}.json")


def agent_to_dict(agent: AgentDefinition) -> dict:
    """Convert AgentDefinition to dict."""
    result = {"description": agent.description, "prompt": agent.prompt}
    if agent.tools is not None:
        result["tools"] = agent.tools
    if agent.model is not None:
        result["model"] = agent.model
    return result


def capture_agent_definitions():
    """Capture AgentDefinition variants."""
    print("\n=== Capturing Agent Definitions ===")

    # Basic agent - minimal fields
    agent = AgentDefinition(
        description="A simple helper agent", prompt="You are a helpful assistant."
    )
    save_fixture("agent_definition_minimal", agent_to_dict(agent))

    # Agent with tools
    agent = AgentDefinition(
        description="Code review agent",
        prompt="You are an expert code reviewer. Review code for bugs and best practices.",
        tools=["Read", "Grep", "Glob"],
    )
    save_fixture("agent_definition_with_tools", agent_to_dict(agent))

    # Agent with model
    agent = AgentDefinition(
        description="Fast responder",
        prompt="You provide quick, concise answers.",
        model="haiku",
    )
    save_fixture("agent_definition_with_model", agent_to_dict(agent))

    # Agent with all fields
    agent = AgentDefinition(
        description="Full-featured agent",
        prompt="You are a comprehensive assistant with specific tools and model.",
        tools=["Read", "Write", "Edit", "Bash", "Grep"],
        model="sonnet",
    )
    save_fixture("agent_definition_complete", agent_to_dict(agent))

    # Agent with inherit model
    agent = AgentDefinition(
        description="Inheriting agent",
        prompt="You inherit the parent's model configuration.",
        tools=["Read"],
        model="inherit",
    )
    save_fixture("agent_definition_inherit_model", agent_to_dict(agent))

    # Agent with opus model
    agent = AgentDefinition(
        description="High-capability agent",
        prompt="You handle complex reasoning tasks.",
        model="opus",
    )
    save_fixture("agent_definition_opus_model", agent_to_dict(agent))

    # Collection of agents (as used in ClaudeAgentOptions)
    agents = {
        "reviewer": agent_to_dict(
            AgentDefinition(
                description="Code reviewer",
                prompt="Review code for quality and bugs.",
                tools=["Read", "Grep"],
            )
        ),
        "writer": agent_to_dict(
            AgentDefinition(
                description="Documentation writer",
                prompt="Write clear documentation.",
                tools=["Write", "Read"],
                model="sonnet",
            )
        ),
        "tester": agent_to_dict(
            AgentDefinition(
                description="Test runner",
                prompt="Run and validate tests.",
                tools=["Bash", "Read"],
                model="haiku",
            )
        ),
    }
    save_fixture("agent_definitions_collection", agents)

    print("  ✓ Created all agent definition variants")


def capture_system_prompt_presets():
    """Capture SystemPromptPreset variants."""
    print("\n=== Capturing System Prompt Presets ===")

    # Basic preset
    preset = {"type": "preset", "preset": "claude_code"}
    save_fixture("system_prompt_preset_basic", preset)

    # Preset with append
    preset = {
        "type": "preset",
        "preset": "claude_code",
        "append": "Additional instructions: Always be concise and use examples.",
    }
    save_fixture("system_prompt_preset_with_append", preset)

    print("  ✓ Created system prompt preset variants")


def capture_claude_agent_options():
    """Capture ClaudeAgentOptions variants."""
    print("\n=== Capturing Claude Agent Options ===")

    # Minimal options
    options = {}
    save_fixture("options_minimal", options)

    # With allowed tools
    options = {"allowed_tools": ["Read", "Write", "Bash"]}
    save_fixture("options_allowed_tools", options)

    # With disallowed tools
    options = {"disallowed_tools": ["Bash", "Edit"]}
    save_fixture("options_disallowed_tools", options)

    # With system prompt string
    options = {"system_prompt": "You are a helpful coding assistant."}
    save_fixture("options_system_prompt_string", options)

    # With system prompt preset
    options = {
        "system_prompt": {
            "type": "preset",
            "preset": "claude_code",
            "append": "Focus on Rust development.",
        }
    }
    save_fixture("options_system_prompt_preset", options)

    # With permission mode
    for mode in ["default", "acceptEdits", "plan", "bypassPermissions"]:
        options = {"permission_mode": mode}
        save_fixture(f"options_permission_mode_{mode}", options)

    # With model
    options = {"model": "claude-sonnet-4-5-20250929"}
    save_fixture("options_model", options)

    # With max turns
    options = {"max_turns": 5}
    save_fixture("options_max_turns", options)

    # With continue conversation
    options = {"continue_conversation": True}
    save_fixture("options_continue_conversation", options)

    # With resume session
    options = {"resume": "session_abc123"}
    save_fixture("options_resume", options)

    # With fork session
    options = {"resume": "session_abc123", "fork_session": True}
    save_fixture("options_fork_session", options)

    # With include partial messages (streaming)
    options = {"include_partial_messages": True}
    save_fixture("options_streaming", options)

    # With setting sources
    for sources in [
        ["user"],
        ["project"],
        ["local"],
        ["user", "project"],
        ["user", "project", "local"],
    ]:
        options = {"setting_sources": sources}
        save_fixture(f"options_setting_sources_{'_'.join(sources)}", options)

    # With cwd
    options = {"cwd": "/Users/test/project"}
    save_fixture("options_cwd", options)

    # With add_dirs
    options = {"add_dirs": ["/path/to/dir1", "/path/to/dir2"]}
    save_fixture("options_add_dirs", options)

    # With env vars
    options = {"env": {"API_KEY": "secret", "DEBUG": "true"}}
    save_fixture("options_env", options)

    # With extra args
    options = {"extra_args": {"--verbose": None, "--output-format": "json"}}
    save_fixture("options_extra_args", options)

    # With agents
    options = {
        "agents": {
            "reviewer": {
                "description": "Code reviewer",
                "prompt": "Review code carefully.",
                "tools": ["Read", "Grep"],
            }
        }
    }
    save_fixture("options_with_agents", options)

    # Comprehensive options
    options = {
        "allowed_tools": ["Read", "Write", "Bash", "Grep"],
        "disallowed_tools": ["Edit"],
        "system_prompt": {
            "type": "preset",
            "preset": "claude_code",
            "append": "Focus on security and best practices.",
        },
        "permission_mode": "acceptEdits",
        "model": "claude-sonnet-4-5-20250929",
        "max_turns": 10,
        "include_partial_messages": False,
        "setting_sources": ["user", "project"],
        "cwd": "/Users/test/project",
        "add_dirs": ["/safe/dir1", "/safe/dir2"],
        "env": {"CUSTOM_VAR": "value"},
        "agents": {
            "reviewer": {
                "description": "Code reviewer agent",
                "prompt": "Review code for quality.",
                "tools": ["Read", "Grep"],
                "model": "sonnet",
            },
            "tester": {
                "description": "Test runner agent",
                "prompt": "Run tests and report results.",
                "tools": ["Bash", "Read"],
                "model": "haiku",
            },
        },
    }
    save_fixture("options_comprehensive", options)

    print("  ✓ Created all option variants")


def main():
    """Run all capture functions."""
    print("Starting agent and options capture...")
    print(f"Fixtures will be saved to: {FIXTURES_DIR}")

    # Create fixtures directory
    FIXTURES_DIR.mkdir(parents=True, exist_ok=True)

    # Capture all types
    capture_agent_definitions()
    capture_system_prompt_presets()
    capture_claude_agent_options()

    print("\n✅ All agent and option fixtures captured successfully!")


if __name__ == "__main__":
    main()
