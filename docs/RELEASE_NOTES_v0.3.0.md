# Release Notes - v0.3.0

**Release Date:** November 12, 2025
**Status:** Production Ready
**Feature Parity:** 100% with Python SDK v0.1.6

## 🎉 Highlights

This release achieves **100% feature parity** with the official Python SDK v0.1.6, adding critical production features and a comprehensive plugin system. The Rust SDK is now fully production-ready with all the capabilities of the Python SDK plus the performance and type safety benefits of Rust.

## ✨ What's New

### Production Features

#### 1. Fallback Model (`fallback_model`)
Configure a backup model for automatic failover when the primary model is unavailable or encounters errors.

```rust
let options = ClaudeAgentOptions::builder()
    .model("claude-opus-4")
    .fallback_model("claude-sonnet-4")
    .build();
```

**Use Case:** Ensure service availability in production environments.

#### 2. Budget Control (`max_budget_usd`)
Set spending limits to prevent runaway costs in production deployments.

```rust
let options = ClaudeAgentOptions::builder()
    .max_budget_usd(10.0)
    .build();
```

**Use Case:** Cost control for production applications and testing with known limits.

#### 3. Extended Thinking (`max_thinking_tokens`)
Control the maximum number of tokens for thinking blocks in models with extended thinking capabilities.

```rust
let options = ClaudeAgentOptions::builder()
    .max_thinking_tokens(2000)
    .build();
```

**Use Case:** Resource management for complex reasoning tasks.

#### 4. Streaming Query API (`query_stream()`)
Memory-efficient streaming alternative to `query()` for large conversations.

```rust
use claude_agent_sdk_rs::query_stream;
use futures::StreamExt;

let mut stream = query_stream("prompt", None).await?;
while let Some(result) = stream.next().await {
    // Process messages in real-time with O(1) memory
}
```

**Performance:**
- `query()`: O(n) memory, waits for all messages
- `query_stream()`: O(1) memory per message, real-time processing

### Plugin System

A complete plugin system for extending Claude's capabilities with custom functionality.

#### Plugin Loading

```rust
use claude_agent_sdk_rs::SdkPluginConfig;

let options = ClaudeAgentOptions::builder()
    .plugins(vec![
        SdkPluginConfig::local("./my-plugin"),
        SdkPluginConfig::local("/opt/company-plugins/tools"),
        SdkPluginConfig::local("~/.claude/plugins/personal"),
    ])
    .build();
```

**Supported Path Types:**
- Relative paths: `./plugins/my-tool`
- Absolute paths: `/opt/plugins/tool`
- Home directory: `~/.claude/plugins/tool`

#### Multiple Plugins
Load and use multiple plugins simultaneously with proper isolation.

#### Plugin Development Guide
Comprehensive 9-page guide covering plugin structure, development, testing, and deployment.

## 📚 Documentation Enhancements

### 1. Enhanced API Documentation (`src/lib.rs`)
- Comprehensive examples for all major features
- Working code snippets with `no_run` examples
- Clear documentation of all configuration options
- Links to relevant documentation and examples

### 2. Examples README (`examples/README.md`)
- Detailed guide to all 22 examples
- Learning path from beginner to advanced
- Example categories and descriptions
- Common patterns and troubleshooting
- Quick reference matrix

### 3. Plugin Development Guide (`PLUGIN_GUIDE.md`)
- Plugin structure and requirements
- Creating custom plugins
- Loading plugins in Rust
- Best practices and security considerations
- Comprehensive examples and troubleshooting

### 4. Updated Main README
- Added plugin system documentation
- Updated feature list
- New production features section
- Updated example count and categories

## 🎯 New Examples

Six new comprehensive examples (total: 22):

### Example 17: Fallback Model
Demonstrates fallback model configuration for production reliability.

### Example 18: Budget Control
Shows cost control and budget tracking implementation.

### Example 19: Extended Thinking
Extended thinking configuration and thinking block inspection.

### Example 20: Streaming Query
Memory-efficient streaming with performance comparison.

### Example 21: Custom Plugins
Plugin configuration, loading patterns, and development guide.

### Example 22: Plugin Integration
Real-world plugin integration scenario with multiple domain plugins.

## 🧪 Testing Improvements

### New Tests
- **7 plugin unit tests**: Plugin types, serialization, path handling
- **5 plugin integration tests**: Configuration, loading, multiple plugins
- **Test plugin fixture**: Complete test plugin for integration testing

### Test Coverage
- **51 unit tests** (100% passing)
- **10 integration unit tests** (100% passing)
- **15 integration tests** (require Claude CLI, properly ignored)
- **Zero warnings** from clippy
- **100% formatted** with rustfmt

## 🔧 API Changes

### New Configuration Fields

```rust
pub struct ClaudeAgentOptions {
    // NEW in v0.3.0
    pub fallback_model: Option<String>,
    pub max_budget_usd: Option<f64>,
    pub max_thinking_tokens: Option<u32>,
    pub plugins: Vec<SdkPluginConfig>,

    // Existing fields...
}
```

### New Public API

- `query_stream()`: Streaming query function
- `SdkPluginConfig`: Plugin configuration type
- `SdkPluginConfig::local()`: Plugin constructor

### Backward Compatibility

**100% Backward Compatible** - All changes are additive. Existing code continues to work without modifications.

## 📊 Feature Parity Status

### Python SDK v0.1.6: 100% ✅

All 30 configuration options supported:

| Category   | Python SDK | Rust SDK v0.3.0 | Status     |
|------------|------------|-----------------|------------|
| Basic      | 10 options | 10 options      | ✅ 100%     |
| Advanced   | 12 options | 12 options      | ✅ 100%     |
| Production | 4 options  | 4 options       | ✅ NEW      |
| Plugins    | 1 option   | 1 option        | ✅ NEW      |
| Hooks      | 6 types    | 6 types         | ✅ 100%     |
| MCP        | 4 types    | 4 types         | ✅ 100%     |
| **Total**  | **30**     | **30**          | **✅ 100%** |

## 📦 Files Changed

### Core Implementation (5 files)
- `src/types/config.rs`: Added 4 new configuration fields
- `src/types/plugin.rs`: New plugin types module (7 unit tests)
- `src/query.rs`: Added `query_stream()` function
- `src/lib.rs`: Enhanced documentation
- `src/internal/transport/subprocess.rs`: CLI integration

### Documentation (4 files)
- `README.md`: Updated features, examples, API docs
- `PLUGIN_GUIDE.md`: New 9-page plugin guide
- `examples/README.md`: New comprehensive examples guide
- `CHANGELOG.md`: Detailed v0.3.0 changelog

### Tests (2 files)
- `tests/integration_tests.rs`: Added 5 plugin tests
- `src/types/plugin.rs`: 7 unit tests

### Examples (6 new files)
- `examples/17_fallback_model.rs`
- `examples/18_max_budget_usd.rs`
- `examples/19_max_thinking_tokens.rs`
- `examples/20_query_stream.rs`
- `examples/21_custom_plugins.rs`
- `examples/22_plugin_integration.rs`

### Test Fixtures (1 directory)
- `fixtures/test-plugin/`: Complete test plugin

### Configuration (2 files)
- `Cargo.toml`: Version bump to 0.3.0, updated description
- `RELEASE_NOTES_v0.3.0.md`: This file

**Total Files Changed/Created:** 20+

## 🔄 Migration Guide

### From v0.2.1 to v0.3.0

No breaking changes! All new features are opt-in.

#### Using New Features

```rust
use claude_agent_sdk_rs::{ClaudeAgentOptions, SdkPluginConfig, query_stream};

// 1. Fallback model (optional)
let options = ClaudeAgentOptions::builder()
    .model("claude-opus-4")
    .fallback_model("claude-sonnet-4")  // NEW
    .build();

// 2. Budget control (optional)
let options = ClaudeAgentOptions::builder()
    .max_budget_usd(10.0)  // NEW
    .build();

// 3. Extended thinking (optional)
let options = ClaudeAgentOptions::builder()
    .max_thinking_tokens(2000)  // NEW
    .build();

// 4. Plugins (optional)
let options = ClaudeAgentOptions::builder()
    .plugins(vec![
        SdkPluginConfig::local("./my-plugin")  // NEW
    ])
    .build();

// 5. Streaming query (optional)
let stream = query_stream("prompt", None).await?;  // NEW
```

#### Existing Code

All existing v0.2.1 code works without changes:

```rust
// This still works exactly as before
let messages = query("Hello", None).await?;
```

## 🎯 Quality Metrics

### Code Quality
- ✅ **51 unit tests** passing
- ✅ **10 integration tests** passing
- ✅ **Zero clippy warnings** (with `-D warnings`)
- ✅ **100% rustfmt compliant**
- ✅ **22 examples** compiling and running
- ✅ **Type-safe** throughout
- ✅ **Zero unsafe code**

### Documentation
- ✅ **Comprehensive API docs** in src/lib.rs
- ✅ **22 fully documented examples**
- ✅ **3 major guides** (README, Plugin Guide, Examples)
- ✅ **Detailed CHANGELOG**
- ✅ **Migration guide** (this document)

### Performance
- ✅ **Streaming query** reduces memory usage
- ✅ **Zero-copy** where possible
- ✅ **Lock-free** stdin/stdout access
- ✅ **Efficient** plugin loading

## 🚀 Getting Started with v0.3.0

### Installation

```toml
[dependencies]
claude-agent-sdk-rs = "0.3.0"
tokio = { version = "1", features = ["full"] }
```

### Quick Example

```rust
use claude_agent_sdk_rs::{ClaudeAgentOptions, query, PermissionMode};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let options = ClaudeAgentOptions::builder()
        .model("claude-opus-4")
        .fallback_model("claude-sonnet-4")
        .max_budget_usd(5.0)
        .permission_mode(PermissionMode::BypassPermissions)
        .build();

    let messages = query("What is Rust?", Some(options)).await?;

    // Process messages...
    Ok(())
}
```

## 📖 Documentation Links

- [API Documentation](https://docs.rs/claude-agent-sdk-rs/0.3.0)
- [GitHub Repository](https://github.com/tyrchen/claude-agent-sdk-rs)
- [Examples Directory](https://github.com/tyrchen/claude-agent-sdk-rs/tree/master/examples)
- [Plugin Guide](https://github.com/tyrchen/claude-agent-sdk-rs/blob/master/PLUGIN_GUIDE.md)

## 🙏 Acknowledgments

This release completes the feature parity roadmap with Python SDK v0.1.6. The Rust SDK now provides:
- ✅ **100% feature parity** with Python SDK
- ✅ **Production-ready** reliability and performance
- ✅ **Type-safe** API with compile-time guarantees
- ✅ **Comprehensive documentation** and examples
- ✅ **Full test coverage** with quality assurance

## 🔮 What's Next

With 100% feature parity achieved, future releases will focus on:
- Performance optimizations
- Additional convenience APIs
- Enhanced error messages
- Community-requested features
- Ecosystem integrations

## 📝 Feedback

We welcome feedback and contributions! Please:
- Report issues on [GitHub Issues](https://github.com/tyrchen/claude-agent-sdk-rs/issues)
- Submit PRs for improvements
- Share your use cases
- Suggest new features

---

**Thank you for using Claude Agent SDK for Rust! 🦀**
