# Claude Agent SDK Examples

This directory contains working examples demonstrating the Claude Agent SDK for Rust.

## Running the Examples

```bash
# Run example 1: Basic Hello World
cargo run --example 01_hello_world

# Run example 2: Limit Tool Use
cargo run --example 02_limit_tool_use

# Run example 3: Monitor Tool Use
cargo run --example 03_monitor_tools
```

## Example 1: Basic Hello World

**Purpose**: Demonstrates basic SDK usage to have Claude write a Python program.

**What it does**:
1. Asks Claude to write a Python "Hello, World!" script
2. Saves it to `./fixtures/hello.py`
3. Verifies the file was created
4. Runs the Python script to confirm it works

**Key concepts**: Using the `query()` function, setting `allowed_tools`, processing messages

## Example 2: Limit Tool Use

**Purpose**: Demonstrates how to restrict which tools Claude can use.

**What it does**:
1. Allows only `Write` tool, asks Claude to create a calculator
2. Disallows `Edit` tool, asks Claude to modify a file
3. Verifies Claude respects tool restrictions

**Key concepts**: `allowed_tools`, `disallowed_tools`, tool usage tracking

## Example 3: Monitor Tool Use

**Purpose**: Demonstrates comprehensive tool usage monitoring.

**What it does**:
1. Asks Claude to create a factorial function and test file
2. Tracks every tool invocation with full details
3. Provides usage summary and file verification

**Key concepts**: Detailed message processing, tracking ToolUse blocks, turn-by-turn monitoring

## Requirements

- Claude Code CLI 2.0.0+
- Rust 1.70+
- Python 3 (for running generated scripts)
