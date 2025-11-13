# Plugin Development Guide

This guide explains how to create, configure, and use custom plugins with the Claude Agent SDK for Rust.

## Table of Contents

- [Overview](#overview)
- [Plugin Structure](#plugin-structure)
- [Creating a Plugin](#creating-a-plugin)
- [Loading Plugins](#loading-plugins)
- [Best Practices](#best-practices)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)

## Overview

Plugins allow you to extend Claude Code's capabilities with custom tools, integrations, and functionality. The Rust SDK provides full support for loading and using plugins through the `SdkPluginConfig` type.

### Key Features

- **Local Plugin Loading**: Load plugins from filesystem paths
- **Multiple Plugins**: Use multiple plugins simultaneously
- **Type-Safe Configuration**: Strongly-typed plugin configuration
- **Path Flexibility**: Support for relative, absolute, and home directory paths

## Plugin Structure

A Claude Code plugin typically has the following structure:

```
my-plugin/
├── plugin.json          # Plugin metadata
├── index.js             # Main plugin file (Node.js)
├── package.json         # Dependencies (optional)
└── README.md            # Documentation
```

### plugin.json

```json
{
  "name": "my-plugin",
  "version": "1.0.0",
  "description": "My custom Claude Code plugin",
  "main": "index.js",
  "author": "Your Name",
  "license": "MIT"
}
```

### index.js

```javascript
module.exports = {
  name: 'my-plugin',
  version: '1.0.0',

  async initialize() {
    console.log('Plugin initialized');
  },

  tools: [
    {
      name: 'my_tool',
      description: 'A custom tool',
      input_schema: {
        type: 'object',
        properties: {
          input: { type: 'string' }
        },
        required: ['input']
      },
      async execute(input) {
        return {
          content: `Processed: ${input.input}`
        };
      }
    }
  ]
};
```

## Creating a Plugin

### Step 1: Create Plugin Directory

```bash
mkdir -p my-plugin
cd my-plugin
```

### Step 2: Create plugin.json

```json
{
  "name": "my-plugin",
  "version": "1.0.0",
  "description": "My custom plugin",
  "main": "index.js"
}
```

### Step 3: Implement Plugin Logic

Create `index.js` with your plugin implementation (see structure above).

### Step 4: Test Locally

Test your plugin with Claude CLI before integrating with the Rust SDK:

```bash
claude --plugin ./my-plugin "Test my plugin"
```

## Loading Plugins in Rust

### Basic Usage

```rust
use claude_agent_sdk_rs::{ClaudeAgentOptions, SdkPluginConfig};

// Single plugin
let options = ClaudeAgentOptions::builder()
    .plugins(vec![
        SdkPluginConfig::local("./my-plugin")
    ])
    .build();
```

### Multiple Plugins

```rust
let options = ClaudeAgentOptions::builder()
    .plugins(vec![
        SdkPluginConfig::local("./plugins/database-tools"),
        SdkPluginConfig::local("./plugins/api-integration"),
        SdkPluginConfig::local("~/.claude/plugins/global-tools"),
    ])
    .build();
```

### Path Types

#### Relative Path

```rust
SdkPluginConfig::local("./my-plugin")
SdkPluginConfig::local("../shared-plugins/tool-suite")
```

#### Absolute Path

```rust
SdkPluginConfig::local("/opt/company-plugins/tools")
SdkPluginConfig::local("/usr/local/share/claude-plugins/my-plugin")
```

#### Home Directory

```rust
SdkPluginConfig::local("~/.claude/plugins/personal-tools")
SdkPluginConfig::local("~/Development/claude-plugins/dev-tools")
```

## Best Practices

### 1. Plugin Organization

**Do:**

- Group related tools in a single plugin
- Use clear, descriptive plugin names
- Version your plugins properly
- Document plugin requirements

**Don't:**

- Create too many small plugins (overhead)
- Mix unrelated functionality
- Forget to version plugins

### 2. Path Management

**Development:**

```rust
// Use relative paths for project-specific plugins
SdkPluginConfig::local("./plugins/my-tool")
```

**Production:**

```rust
// Use absolute paths for stability
SdkPluginConfig::local("/opt/app/plugins/my-tool")
```

**User-Specific:**

```rust
// Use home directory for personal tools
SdkPluginConfig::local("~/.claude/plugins/personal")
```

### 3. Error Handling

```rust
use claude_agent_sdk_rs::{ClaudeAgentOptions, SdkPluginConfig};

// Check plugin exists before loading
let plugin_path = "./my-plugin";
if std::path::Path::new(plugin_path).exists() {
    let options = ClaudeAgentOptions::builder()
        .plugins(vec![SdkPluginConfig::local(plugin_path)])
        .build();
} else {
    eprintln!("Warning: Plugin not found at {}", plugin_path);
    // Use fallback configuration
}
```

### 4. Security

- **Only load trusted plugins**: Review plugin code before use
- **Validate plugin paths**: Ensure paths are safe and expected
- **Use permission modes**: Control what plugins can do
- **Isolate testing**: Test plugins in isolated environments first

```rust
let options = ClaudeAgentOptions::builder()
    .plugins(vec![SdkPluginConfig::local("./my-plugin")])
    .permission_mode(PermissionMode::Default) // Ask for permission
    .max_budget_usd(1.0) // Limit costs during testing
    .build();
```

### 5. Plugin Development Workflow

1. **Design**: Plan plugin functionality and tools
2. **Implement**: Write plugin code (JavaScript/Node.js)
3. **Test**: Test with Claude CLI directly
4. **Integrate**: Load in Rust SDK
5. **Deploy**: Use in production with proper paths

## Examples

### Example 1: Database Tools Plugin

```rust
use claude_agent_sdk_rs::{ClaudeClient, ClaudeAgentOptions, SdkPluginConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let options = ClaudeAgentOptions::builder()
        .plugins(vec![
            SdkPluginConfig::local("./plugins/postgres-tools")
        ])
        .allowed_tools(vec![
            "Read".to_string(),
            "Write".to_string(),
        ])
        .build();

    let mut client = ClaudeClient::new(options);
    client.connect().await?;

    client.query("Show me the database schema").await?;

    // Handle responses...

    client.disconnect().await?;
    Ok(())
}
```

### Example 2: Multiple Domain Plugins

```rust
let options = ClaudeAgentOptions::builder()
    .plugins(vec![
        // Database operations
        SdkPluginConfig::local("./plugins/database"),
        // API testing
        SdkPluginConfig::local("./plugins/api-test"),
        // Code generation
        SdkPluginConfig::local("./plugins/codegen"),
    ])
    .permission_mode(PermissionMode::Default)
    .max_turns(10)
    .build();
```

### Example 3: Conditional Plugin Loading

```rust
use std::path::Path;

let mut plugins = vec![];

// Always load core plugin
plugins.push(SdkPluginConfig::local("./plugins/core"));

// Conditionally load dev tools
if cfg!(debug_assertions) {
    if Path::new("./plugins/dev-tools").exists() {
        plugins.push(SdkPluginConfig::local("./plugins/dev-tools"));
    }
}

// Load user's personal plugins if available
let user_plugins = std::env::var("HOME")
    .map(|home| format!("{}/.claude/plugins/personal", home))
    .ok();

if let Some(path) = user_plugins {
    if Path::new(&path).exists() {
        plugins.push(SdkPluginConfig::local(path));
    }
}

let options = ClaudeAgentOptions::builder()
    .plugins(plugins)
    .build();
```

## Troubleshooting

### Plugin Not Found

**Problem**: Plugin fails to load

**Solutions**:

1. Verify the path exists:

   ```rust
   let path = "./my-plugin";
   println!("Exists: {}", std::path::Path::new(path).exists());
   ```

2. Use absolute paths in production
3. Check plugin.json is present and valid

### Plugin Tools Not Available

**Problem**: Plugin tools don't appear to Claude

**Solutions**:

1. Check Claude CLI supports plugins (version >= 2.0.0)
2. Verify plugin.json has correct structure
3. Test plugin with CLI directly first: `claude --plugin ./my-plugin "test"`

### Multiple Plugin Conflicts

**Problem**: Plugins interfere with each other

**Solutions**:

1. Ensure unique tool names across plugins
2. Load plugins in specific order if order matters
3. Test plugins individually first

### Permission Issues

**Problem**: Plugin operations are blocked

**Solutions**:

1. Use appropriate permission mode:

   ```rust
   .permission_mode(PermissionMode::AcceptEdits)
   ```

2. Check plugin permissions in Claude Code settings
3. Test with `BypassPermissions` for debugging only

## Testing Plugins

### Unit Testing

```rust
#[test]
fn test_plugin_config() {
    let plugin = SdkPluginConfig::local("./my-plugin");
    assert!(plugin.path().is_some());
}

#[test]
fn test_plugin_paths() {
    let options = ClaudeAgentOptions::builder()
        .plugins(vec![
            SdkPluginConfig::local("./plugin1"),
            SdkPluginConfig::local("/opt/plugin2"),
        ])
        .build();

    assert_eq!(options.plugins.len(), 2);
}
```

### Integration Testing

```rust
#[tokio::test]
#[ignore] // Requires Claude CLI
async fn test_plugin_integration() -> anyhow::Result<()> {
    let options = ClaudeAgentOptions::builder()
        .plugins(vec![SdkPluginConfig::local("./test-plugin")])
        .build();

    let mut client = ClaudeClient::new(options);
    client.connect().await?;

    // Test plugin functionality...

    client.disconnect().await?;
    Ok(())
}
```

## Resources

- [Claude Code Documentation](https://docs.claude.com/claude-code)
- [Examples Directory](./examples/)
  - `examples/21_custom_plugins.rs` - Plugin basics
  - `examples/22_plugin_integration.rs` - Real-world usage
- [Test Plugin](./fixtures/test-plugin/) - Example plugin structure

## Contributing

When contributing plugin-related features:

1. Add tests for new plugin types
2. Update this guide with new features
3. Provide example code
4. Document breaking changes

## License

This guide is part of the Claude Agent SDK for Rust, distributed under the MIT License.
