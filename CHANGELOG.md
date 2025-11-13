# Changelog

All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

---
## [0.3.0] - 2025-11-12

### ✨ Features

This release achieves **100% feature parity** with Python SDK v0.1.6, adding critical production features and plugin support.

#### Production Features

- **Fallback Model** (`fallback_model`): Configure backup model for automatic failover when primary model is unavailable
- **Budget Control** (`max_budget_usd`): Set spending limits to prevent runaway costs in production
- **Extended Thinking** (`max_thinking_tokens`): Control maximum tokens for thinking blocks in models with extended thinking capabilities
- **Streaming Query API** (`query_stream()`): Memory-efficient streaming alternative to `query()` for large conversations

#### Plugin System

- **Plugin Loading** (`plugins`): Load custom plugins from local filesystem paths to extend Claude's capabilities
- **Plugin Configuration** (`SdkPluginConfig`): Type-safe plugin configuration with support for relative, absolute, and home directory paths
- **Multiple Plugins**: Load and use multiple plugins simultaneously
- **Plugin Guide**: Comprehensive 9-page plugin development guide (PLUGIN_GUIDE.md)

### 📚 Documentation

- **Enhanced API Documentation**: Updated lib.rs with comprehensive examples and API overview
- **Examples README**: Detailed guide to all 22 examples with learning paths and common patterns
- **Plugin Development Guide**: Complete guide for creating and integrating custom plugins
- **Updated Main README**: Added plugin system, streaming query, and production features documentation

### 🎯 Examples

Added 4 new examples (total: 22):

- **Example 17** (`17_fallback_model.rs`): Demonstrates fallback model configuration for reliability
- **Example 18** (`18_max_budget_usd.rs`): Shows budget control and cost tracking
- **Example 19** (`19_max_thinking_tokens.rs`): Extended thinking configuration and inspection
- **Example 20** (`20_query_stream.rs`): Memory-efficient streaming with performance comparison
- **Example 21** (`21_custom_plugins.rs`): Plugin configuration and loading patterns
- **Example 22** (`22_plugin_integration.rs`): Real-world plugin integration scenario

### 🧪 Testing

- **Unit Tests**: Added 7 plugin-specific tests (total: 51 passing)
- **Integration Tests**: Added 5 plugin integration tests (total: 10 non-ignored tests passing)
- **Test Plugin**: Created complete test plugin fixture for integration testing
- **100% Test Pass Rate**: All tests passing with zero warnings

### 🔧 API Changes

#### New Configuration Options

```rust
ClaudeAgentOptions {
    fallback_model: Option<String>,       // NEW: Backup model
    max_budget_usd: Option<f64>,          // NEW: Cost control
    max_thinking_tokens: Option<u32>,     // NEW: Thinking limit
    plugins: Vec<SdkPluginConfig>,        // NEW: Plugin loading
    // ... existing options
}
```

#### New Functions

- `query_stream()`: Streaming alternative to `query()` for memory efficiency

#### New Types

- `SdkPluginConfig`: Plugin configuration enum
  - `SdkPluginConfig::Local { path }`: Local filesystem plugins

### 🎨 Improvements

- **Type Safety**: All new configuration options fully typed and documented
- **Builder Pattern**: Seamless integration with existing TypedBuilder API
- **Error Handling**: Comprehensive error handling for all new features
- **Performance**: Streaming query reduces memory usage for large conversations
- **Code Quality**: Zero clippy warnings, properly formatted with rustfmt

### 📦 Files Changed

**Core Implementation:**
- `src/types/config.rs`: Added 4 new configuration fields
- `src/types/plugin.rs`: New plugin types module
- `src/query.rs`: Added `query_stream()` function
- `src/lib.rs`: Enhanced documentation with examples
- `src/internal/transport/subprocess.rs`: Plugin and new option CLI integration

**Documentation:**
- `README.md`: Updated features, examples, and API documentation
- `PLUGIN_GUIDE.md`: New comprehensive plugin development guide
- `examples/README.md`: New detailed examples documentation
- `CHANGELOG.md`: This file

**Tests:**
- `tests/integration_tests.rs`: Added 5 plugin tests
- `src/types/plugin.rs`: 7 unit tests for plugin types

**Examples:**
- `examples/17_fallback_model.rs`: New example
- `examples/18_max_budget_usd.rs`: New example
- `examples/19_max_thinking_tokens.rs`: New example
- `examples/20_query_stream.rs`: New example
- `examples/21_custom_plugins.rs`: New example
- `examples/22_plugin_integration.rs`: New example

**Test Fixtures:**
- `fixtures/test-plugin/`: Complete test plugin for integration tests

### 🔄 Migration Guide

This release is **fully backward compatible**. All new features are additive.

To use new features:

```rust
// Fallback model
let options = ClaudeAgentOptions::builder()
    .model("claude-opus-4")
    .fallback_model("claude-sonnet-4")  // NEW
    .build();

// Budget control
let options = ClaudeAgentOptions::builder()
    .max_budget_usd(10.0)  // NEW
    .build();

// Extended thinking
let options = ClaudeAgentOptions::builder()
    .max_thinking_tokens(2000)  // NEW
    .build();

// Plugins
use claude_agent_sdk_rs::SdkPluginConfig;
let options = ClaudeAgentOptions::builder()
    .plugins(vec![SdkPluginConfig::local("./my-plugin")])  // NEW
    .build();

// Streaming query
use claude_agent_sdk_rs::query_stream;  // NEW
let stream = query_stream("prompt", None).await?;
```

### 📊 Feature Parity

**Python SDK v0.1.6 Parity: 100%**

All 30 configuration options from Python SDK are now supported:
- ✅ 26 options from v0.2.1
- ✅ 4 new options: `fallback_model`, `max_budget_usd`, `max_thinking_tokens`, `plugins`
- ✅ Streaming query API: `query_stream()`

### 🙏 Acknowledgments

This release completes the feature parity roadmap with Python SDK v0.1.6, providing a production-ready Rust SDK for Claude Agent development.

---
## [0.2.1](https://github.com/compare/v0.2.0..v0.2.1) - 2025-10-26

### Features

- add session management interface - ([3ff20d2](https://github.com/commit/3ff20d211fa30c607ddbdd0733da73a0f7027d4f)) - Tyr Chen

### Other

- Update CHANGELOG.md - ([2663900](https://github.com/commit/26639008db1e4153e8950c88ea14cab9513d5940)) - Tyr Chen

---
## [0.2.0] - 2025-10-26

### Features

- support rust claude agent sdk based on python version - ([15ba8f8](https://github.com/commit/15ba8f889cef49a420ef848337457cd6fd9d4944)) - Tyr Chen
- improve and make mcp server work - ([0d64b0a](https://github.com/commit/0d64b0a5023c0c65d60ddf0db1496c3d75195fa3)) - Tyr Chen
- add new examples, feature parity with python and more test cases - ([d8ebffd](https://github.com/commit/d8ebffd05c527a9453dc49452fd9342f85790744)) - Tyr Chen
- user friendly interface for claude agent and hooks - ([48846ea](https://github.com/commit/48846ea5ba5ba5337643fd166ad7b78bec4af996)) - Tyr Chen

### Miscellaneous Chores

- add Chinese readme - ([3d8392a](https://github.com/commit/3d8392a23f06baf8004644fc12688b6f10feae3a)) - Tyr Chen
- rename to claude-agent-sdk-rs - ([07691f6](https://github.com/commit/07691f6a6ab7addd2c20c88cb349a7afa8dc1cb6)) - Tyr Chen
- bump version - ([d273884](https://github.com/commit/d273884777956e3c2dbcd3d5d3eb1a7b4d7c43ab)) - Tyr Chen

<!-- generated by git-cliff -->
