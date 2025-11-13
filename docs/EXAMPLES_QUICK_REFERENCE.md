# Examples Quick Reference

All 22 examples at a glance with run commands and purposes.

## Basics (01-03)

### 01_hello_world
```bash
cargo run --example 01_hello_world
```
**Purpose:** Basic SDK usage with file writing

### 02_limit_tool_use
```bash
cargo run --example 02_limit_tool_use
```
**Purpose:** Tool restriction and whitelisting

### 03_monitor_tools
```bash
cargo run --example 03_monitor_tools
```
**Purpose:** Track and monitor tool execution

## Advanced (04-07)

### 04_permission_callbacks
```bash
cargo run --example 04_permission_callbacks
```
**Purpose:** Dynamic permission decisions

### 05_hooks_pretooluse
```bash
cargo run --example 05_hooks_pretooluse
```
**Purpose:** PreToolUse hook for interception

### 06_bidirectional_client
```bash
cargo run --example 06_bidirectional_client
```
**Purpose:** Multi-turn conversations

### 07_dynamic_control
```bash
cargo run --example 07_dynamic_control
```
**Purpose:** Runtime control (interrupt, model change)

## MCP (08)

### 08_mcp_server_integration
```bash
cargo run --example 08_mcp_server_integration
```
**Purpose:** Custom in-process tools

## Configuration (09-13)

### 09_agents
```bash
cargo run --example 09_agents
```
**Purpose:** Custom agent definitions

### 10_include_partial_messages
```bash
cargo run --example 10_include_partial_messages
```
**Purpose:** Partial message streaming

### 11_setting_sources
```bash
cargo run --example 11_setting_sources -- all
```
**Purpose:** Settings source control

### 12_stderr_callback
```bash
cargo run --example 12_stderr_callback
```
**Purpose:** CLI stderr monitoring

### 13_system_prompt
```bash
cargo run --example 13_system_prompt
```
**Purpose:** System prompt configuration

## Patterns (14-16)

### 14_streaming_mode
```bash
cargo run --example 14_streaming_mode -- all
```
**Purpose:** All streaming patterns

### 15_hooks_comprehensive
```bash
cargo run --example 15_hooks_comprehensive -- all
```
**Purpose:** All 6 hook types

### 16_session_management
```bash
cargo run --example 16_session_management
```
**Purpose:** Session control and isolation

## Production (17-20) - NEW in v0.3.0

### 17_fallback_model
```bash
cargo run --example 17_fallback_model
```
**Purpose:** Model failover configuration

### 18_max_budget_usd
```bash
cargo run --example 18_max_budget_usd
```
**Purpose:** Cost control and limits

### 19_max_thinking_tokens
```bash
cargo run --example 19_max_thinking_tokens
```
**Purpose:** Extended thinking configuration

### 20_query_stream
```bash
cargo run --example 20_query_stream
```
**Purpose:** Memory-efficient streaming query

## Plugins (21-22) - NEW in v0.3.0

### 21_custom_plugins
```bash
cargo run --example 21_custom_plugins
```
**Purpose:** Plugin loading and configuration

### 22_plugin_integration
```bash
cargo run --example 22_plugin_integration
```
**Purpose:** Real-world plugin usage
