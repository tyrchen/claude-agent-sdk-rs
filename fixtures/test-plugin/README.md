# Test Plugin for Claude Agent SDK Rust

This is a minimal Claude Code plugin used for integration testing of the Rust SDK.

## Structure

Claude Code plugins follow this structure:

```
test-plugin/
├── .claude-plugin/
│   └── plugin.json      # Plugin metadata
├── commands/
│   └── test-cmd.md      # Custom command definition
└── README.md            # This file
```

## Purpose

This plugin is used to verify that:
- Plugins can be loaded from local paths using `--plugin-dir`
- Plugin configuration is passed correctly to Claude CLI
- Multiple plugins can be loaded simultaneously
- System messages contain plugin information

## Plugin Structure

### .claude-plugin/plugin.json
Contains plugin metadata:
- name
- description
- version
- author

### commands/
Contains custom slash commands as markdown files. Each `.md` file becomes a `/command-name` that Claude can execute.

## Usage in Tests

This plugin is referenced in integration tests (`test_plugin_integration`) to verify plugin loading functionality with the real Claude CLI.

## Testing

To test this plugin manually with Claude CLI:

```bash
claude --plugin-dir ./test-fixtures/test-plugin "Run /test-cmd"
```

The plugin should load and the `/test-cmd` command should be available to Claude.
