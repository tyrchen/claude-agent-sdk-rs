#!/usr/bin/env python3
"""Capture MCP Server Config type variants for Rust unit tests.

This script captures: McpStdioServerConfig, McpSSEServerConfig,
McpHttpServerConfig, McpSdkServerConfig.
"""

import json
from pathlib import Path


FIXTURES_DIR = Path(__file__).parent.parent / "fixtures" / "mcp_configs"


def save_fixture(name: str, data: dict):
    """Save a fixture to the fixtures directory."""
    file_path = FIXTURES_DIR / f"{name}.json"
    with open(file_path, "w") as f:
        json.dump(data, f, indent=2)
    print(f"✓ Saved {name}.json")


def capture_mcp_configs():
    """Capture all MCP server config variants."""
    print("\n=== Capturing MCP Server Configs ===")

    # McpStdioServerConfig - minimal (type optional for backward compat)
    config = {"command": "node", "args": ["server.js"]}
    save_fixture("mcp_stdio_minimal", config)

    # McpStdioServerConfig - with explicit type
    config = {"type": "stdio", "command": "python", "args": ["-m", "my_mcp_server"]}
    save_fixture("mcp_stdio_with_type", config)

    # McpStdioServerConfig - with env vars
    config = {
        "type": "stdio",
        "command": "/usr/local/bin/my-server",
        "args": ["--port", "8080", "--verbose"],
        "env": {"API_KEY": "secret123", "DEBUG": "true", "LOG_LEVEL": "info"},
    }
    save_fixture("mcp_stdio_with_env", config)

    # McpStdioServerConfig - command only
    config = {"command": "/usr/bin/simple-server"}
    save_fixture("mcp_stdio_command_only", config)

    # McpSSEServerConfig - minimal
    config = {"type": "sse", "url": "https://example.com/mcp/events"}
    save_fixture("mcp_sse_minimal", config)

    # McpSSEServerConfig - with headers
    config = {
        "type": "sse",
        "url": "https://api.example.com/mcp/stream",
        "headers": {
            "Authorization": "Bearer token123",
            "X-API-Key": "key456",
            "Accept": "text/event-stream",
        },
    }
    save_fixture("mcp_sse_with_headers", config)

    # McpHttpServerConfig - minimal
    config = {"type": "http", "url": "https://example.com/mcp"}
    save_fixture("mcp_http_minimal", config)

    # McpHttpServerConfig - with headers
    config = {
        "type": "http",
        "url": "https://api.example.com/mcp/v1",
        "headers": {
            "Authorization": "Bearer token789",
            "Content-Type": "application/json",
            "X-Custom-Header": "custom-value",
        },
    }
    save_fixture("mcp_http_with_headers", config)

    # McpSdkServerConfig
    # Note: This type contains an actual server instance which can't be serialized
    # We'll create a representation showing the structure
    config = {
        "type": "sdk",
        "name": "my_custom_server",
        "instance": "<McpServer instance>",
    }
    save_fixture("mcp_sdk_structure", config)

    # Example of multiple server configs in a dict (as used in ClaudeAgentOptions)
    configs = {
        "weather": {
            "type": "stdio",
            "command": "npx",
            "args": ["-y", "@modelcontextprotocol/server-weather"],
        },
        "filesystem": {
            "type": "stdio",
            "command": "python",
            "args": ["-m", "mcp_server_filesystem"],
        },
        "api_service": {
            "type": "http",
            "url": "https://api.example.com/mcp",
            "headers": {"Authorization": "Bearer token"},
        },
        "event_stream": {"type": "sse", "url": "https://events.example.com/mcp"},
    }
    save_fixture("mcp_configs_collection", configs)

    print("  ✓ Created all MCP config variants")


def main():
    """Run all capture functions."""
    print("Starting MCP config capture...")
    print(f"Fixtures will be saved to: {FIXTURES_DIR}")

    # Create fixtures directory
    FIXTURES_DIR.mkdir(parents=True, exist_ok=True)

    # Capture all MCP config types
    capture_mcp_configs()

    print("\n✅ All MCP config fixtures captured successfully!")


if __name__ == "__main__":
    main()
