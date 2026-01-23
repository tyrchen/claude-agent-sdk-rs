# Claude Agent SDK for Rust

[![Crates.io](https://img.shields.io/crates/v/claude-agent-sdk-rs.svg)](https://crates.io/crates/claude-agent-sdk-rs)
[![Documentation](https://docs.rs/claude-agent-sdk-rs/badge.svg)](https://docs.rs/claude-agent-sdk-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE.md)

[English](README.md) | [中文](README_zh-CN.md)

Rust SDK 用于与 Claude Code CLI 交互，提供对 Claude 功能的编程访问，**完全支持双向流式传输**。

**状态**: ✅ **生产就绪** - 与 Python SDK 100% 功能对等

## ✨ 特性

- 🚀 **简单查询 API**: 用于无状态交互的一次性查询，支持收集和流式两种模式
- 🔄 **双向流式传输**: 使用 `ClaudeClient` 进行实时流式通信
- 🎛️ **动态控制**: 中断、更改权限、执行中切换模型
- 🪝 **钩子系统**: 运行时拦截和控制 Claude 的行为，提供简洁的构建器 API
- 🛠️ **自定义工具**: 进程内 MCP 服务器，提供简洁的 `tool!` 宏
- 🔌 **插件系统**: 加载自定义插件以扩展 Claude 的能力
- 🔐 **权限管理**: 对工具执行的细粒度控制
- 💰 **成本控制**: 预算限制和后备模型，提供生产可靠性
- 🧠 **扩展思考**: 配置最大思考令牌数以进行复杂推理
- 📊 **会话管理**: 使用 fork_session 实现独立上下文和内存清除
- 🦀 **类型安全**: 强类型的消息、配置、钩子和权限
- ⚡ **零死锁**: 无锁架构，支持并发读写
- 📚 **全面示例**: 22 个完整示例涵盖所有功能
- 🧪 **充分测试**: 广泛的单元测试和集成测试覆盖

## 📦 安装

在你的 `Cargo.toml` 中添加:

```toml
[dependencies]
claude-agent-sdk-rs = "0.3"
tokio = { version = "1", features = ["full"] }
```

或使用 cargo-add:

```bash
cargo add claude-agent-sdk-rs
cargo add tokio --features full
```

## 🎯 前置要求

- **Rust**: 1.90 或更高版本
- **Claude Code CLI**: 2.0.0 或更高版本 ([安装指南](https://docs.claude.com/claude-code))
- **API 密钥**: 在环境变量或 Claude Code 配置中设置 Anthropic API 密钥

## 🚀 快速开始

### 简单查询（一次性）

```rust
use claude_agent_sdk_rs::{query, ClaudeAgentOptions, Message, ContentBlock};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 使用默认选项的简单查询
    let messages = query("2 + 2 等于多少?", None).await?;

    for message in messages {
        if let Message::Assistant(msg) = message {
            for block in msg.message.content {
                if let ContentBlock::Text(text) = block {
                    println!("Claude: {}", text.text);
                }
            }
        }
    }

    Ok(())
}
```

使用自定义选项:

```rust
use claude_agent_sdk_rs::{ClaudeAgentOptions, query};

let options = ClaudeAgentOptions {
    model: Some("sonnet".to_string()),  // 使用 Sonnet 降低成本
    max_turns: Some(5),
    tools: Some(["Read", "Write"].into()),
    ..Default::default()
};

let messages = query("创建一个 hello.txt 文件", Some(options)).await?;
```

### 工具配置：`tools` vs `allowed_tools`

SDK 提供两个不同的工具配置参数：

| 参数 | CLI 标志 | 用途 |
|-----------|----------|---------|
| `tools` | `--tools` | **限制** Claude 可以使用的工具 |
| `allowed_tools` | `--allowedTools` | **授权** 特定工具的权限（主要用于 MCP 工具） |

**使用 `tools`** 来限制 Claude 只能使用特定的内置工具：

```rust
// Claude 只能使用 Read、Write 和 Bash
let options = ClaudeAgentOptions {
    tools: Some(["Read", "Write", "Bash"].into()),
    ..Default::default()
};
```

**使用 `allowed_tools`** 来授权自定义 MCP 工具：

```rust
// 授权自定义 MCP 工具（格式：mcp__{服务器}__{工具}）
let options = ClaudeAgentOptions {
    allowed_tools: vec!["mcp__my-tools__greet".to_string()],
    ..Default::default()
};
```

### 双向对话（多轮）

```rust
use claude_agent_sdk_rs::{ClaudeClient, ClaudeAgentOptions, Message, ContentBlock};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = ClaudeClient::new(ClaudeAgentOptions::default());

    // 连接到 Claude
    client.connect().await?;

    // 第一个问题
    client.query("法国的首都是什么?").await?;

    // 接收响应
    loop {
        match client.receive_message().await? {
            Some(Message::Assistant(msg)) => {
                for block in msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        println!("Claude: {}", text.text);
                    }
                }
            }
            Some(Message::Result(_)) => break,
            Some(_) => continue,
            None => break,
        }
    }

    // 后续问题 - Claude 会记住上下文！
    client.query("那个城市的人口是多少?").await?;

    loop {
        match client.receive_message().await? {
            Some(Message::Assistant(msg)) => {
                for block in msg.message.content {
                    if let ContentBlock::Text(text) = block {
                        println!("Claude: {}", text.text);
                    }
                }
            }
            Some(Message::Result(_)) => break,
            Some(_) => continue,
            None => break,
        }
    }

    client.disconnect().await?;
    Ok(())
}
```

### 自定义工具（SDK MCP 服务器）

创建 Claude 可以使用的自定义进程内工具:

```rust
use claude_agent_sdk_rs::{tool, create_sdk_mcp_server, ToolResult, McpToolResultContent};
use serde_json::json;

async fn greet_handler(args: serde_json::Value) -> anyhow::Result<ToolResult> {
    let name = args["name"].as_str().unwrap_or("世界");
    Ok(ToolResult {
        content: vec![McpToolResultContent::Text {
            text: format!("你好，{}！", name),
        }],
        is_error: false,
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let greet_tool = tool!(
        "greet",
        "问候用户",
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            },
            "required": ["name"]
        }),
        greet_handler
    );

    let server = create_sdk_mcp_server("my-tools", "1.0.0", vec![greet_tool]);

    // 使用 MCP 服务器和允许的工具配置 ClaudeClient
    let mut mcp_servers = HashMap::new();
    mcp_servers.insert("my-tools".to_string(), McpServerConfig::Sdk(server));

    // 注意：MCP 工具使用 `allowed_tools`（不是 `tools`）
    // - allowed_tools: 授权自定义 MCP 工具
    // - tools: 限制内置工具（Read、Write、Bash 等）
    let options = ClaudeAgentOptions {
        mcp_servers: McpServers::Dict(mcp_servers),
        allowed_tools: vec!["mcp__my-tools__greet".to_string()],
        model: Some("sonnet".to_string()),  // 使用 Sonnet 降低成本
        permission_mode: Some(PermissionMode::AcceptEdits),
        ..Default::default()
    };

    let mut client = ClaudeClient::new(options);
    client.connect().await?;

    // Claude 现在可以使用你的自定义工具了！
    client.query("问候 Alice").await?;
    // ... 处理响应

    client.disconnect().await?;
    Ok(())
}
```

**注意**: 工具必须使用格式 `mcp__{服务器名}__{工具名}` 明确允许。

完整指南请参阅 [examples/MCP_INTEGRATION.md](examples/MCP_INTEGRATION.md)。

## 架构

SDK 采用分层结构:

```
┌─────────────────────────────────────────────────────────┐
│                    公共 API 层                          │
│  (query(), ClaudeClient, tool!(), create_sdk_server())  │
└─────────────────────────────────────────────────────────┘
                           │
┌─────────────────────────────────────────────────────────┐
│                  控制协议层                              │
│        (Query: 处理双向控制)                             │
└─────────────────────────────────────────────────────────┘
                           │
┌─────────────────────────────────────────────────────────┐
│                   传输层                                 │
│     (SubprocessTransport, 自定义实现)                    │
└─────────────────────────────────────────────────────────┘
                           │
┌─────────────────────────────────────────────────────────┐
│                  Claude Code CLI                        │
│         (通过 stdio/subprocess 的外部进程)              │
└─────────────────────────────────────────────────────────┘
```

## 类型系统

SDK 为所有 Claude 交互提供强类型的 Rust 接口:

- **消息**: `Message`, `ContentBlock`, `TextBlock`, `ToolUseBlock` 等
- **配置**: `ClaudeAgentOptions`, `SystemPrompt`, `PermissionMode`
- **钩子**: `HookEvent`, `HookCallback`, `HookInput`, `HookJsonOutput`
- **权限**: `PermissionResult`, `PermissionUpdate`, `CanUseToolCallback`
- **MCP**: `McpServers`, `SdkMcpServer`, `ToolHandler`, `ToolResult`

## 📚 示例

SDK 包含 22 个全面的示例，演示所有功能。详见 [examples/README.md](examples/README.md)。

### 快速示例

```bash
# 基础用法
cargo run --example 01_hello_world        # 带工具使用的简单查询
cargo run --example 02_limit_tool_use     # 限制允许的工具
cargo run --example 03_monitor_tools      # 监控工具执行

# 流式传输和对话
cargo run --example 06_bidirectional_client  # 多轮对话
cargo run --example 14_streaming_mode -- all # 全面的流式传输模式

# 钩子和控制
cargo run --example 05_hooks_pretooluse      # PreToolUse 钩子
cargo run --example 15_hooks_comprehensive -- all  # 所有钩子类型
cargo run --example 07_dynamic_control       # 运行时控制

# 自定义工具和 MCP
cargo run --example 08_mcp_server_integration  # 进程内 MCP 服务器

# 配置
cargo run --example 09_agents               # 自定义代理
cargo run --example 11_setting_sources -- all  # 设置控制
cargo run --example 13_system_prompt        # 系统提示配置
```

### 示例分类

| 类别     | 示例  | 描述                           |
| -------- | ----- | ------------------------------ |
| **基础** | 01-03 | 简单查询、工具控制、监控       |
| **高级** | 04-07 | 权限、钩子、流式传输、动态控制 |
| **MCP**  | 08    | 自定义工具和 MCP 服务器集成    |
| **配置** | 09-13 | 代理、设置、提示、调试         |
| **模式** | 14-15 | 全面的流式传输和钩子模式       |

## 📖 API 概览

### 核心类型

```rust
// 双向流式传输的主客户端
ClaudeClient

// 用于一次性交互的简单查询函数
query(prompt: &str, options: Option<ClaudeAgentOptions>) -> Vec<Message>

// 配置
ClaudeAgentOptions {
    model: Option<String>,
    max_turns: Option<u32>,
    tools: Option<Tools>,                // 限制可用工具 (--tools)
    allowed_tools: Vec<String>,          // 授权 MCP 工具权限 (--allowedTools)
    system_prompt: Option<SystemPromptConfig>,
    hooks: Option<HashMap<String, Vec<HookMatcher>>>,
    mcp_servers: Option<HashMap<String, McpServer>>,
    // ... 更多
}

// 消息
Message::Assistant(AssistantMessage)
Message::User(UserMessage)
Message::System(SystemMessage)
Message::Result(ResultMessage)
```

### ClaudeClient（双向流式传输）

```rust
// 创建并连接
let mut client = ClaudeClient::new(options);
client.connect().await?;

// 发送查询
client.query("你好").await?;

// 接收消息
loop {
    match client.receive_message().await? {
        Some(Message::Assistant(msg)) => { /* 处理 */ }
        Some(Message::Result(_)) => break,
        None => break,
        _ => continue,
    }
}

// 动态控制（执行中）
client.interrupt().await?;  // 停止当前操作
// 客户端会自动处理中断

// 断开连接
client.disconnect().await?;
```

### 钩子系统

```rust
use claude_agent_sdk_rs::{Hook, HookMatcher, HookInput, HookContext, HookJSONOutput};

async fn my_hook(
    input: HookInput,
    tool_use_id: Option<String>,
    context: HookContext,
) -> anyhow::Result<HookJSONOutput> {
    // 阻止危险命令
    if let Some(command) = input.get("tool_input")
        .and_then(|v| v.get("command"))
        .and_then(|v| v.as_str())
    {
        if command.contains("rm -rf") {
            return Ok(serde_json::json!({
                "hookSpecificOutput": {
                    "permissionDecision": "deny",
                    "permissionDecisionReason": "危险命令已阻止"
                }
            }));
        }
    }
    Ok(serde_json::json!({}))
}

let mut hooks = HashMap::new();
hooks.insert("PreToolUse".to_string(), vec![
    HookMatcher {
        matcher: Some("Bash".to_string()),
        hooks: vec![Hook::new(my_hook)],
    }
]);

let options = ClaudeAgentOptions {
    hooks: Some(hooks),
    ..Default::default()
};
```

## 🧪 开发

### 运行测试

```bash
# 运行所有测试
cargo test

# 带输出运行测试
cargo test -- --nocapture

# 运行特定测试
cargo test test_name
```

### 代码质量

```bash
# 使用 clippy 检查代码
cargo clippy --all-targets --all-features

# 格式化代码
cargo fmt

# 检查格式化
cargo fmt -- --check
```

### 构建

```bash
# 构建库
cargo build

# 使用发布优化构建
cargo build --release

# 构建所有示例
cargo build --examples

# 构建文档
cargo doc --open
```

## 🔧 故障排除

### 常见问题

**"找不到 Claude Code CLI"**

- 安装 Claude Code CLI: <https://docs.claude.com/claude-code>
- 确保 `claude` 在你的 PATH 中

**"API 密钥未配置"**

- 设置 `ANTHROPIC_API_KEY` 环境变量
- 或通过 Claude Code CLI 设置配置

**"权限被拒绝"错误**

- 对于自动化工作流，使用 `permission_mode: PermissionMode::AcceptEdits`
- 或实现自定义权限回调

### 调试模式

启用调试输出以查看正在发生的事情:

```rust
let options = ClaudeAgentOptions {
    stderr_callback: Some(Arc::new(|msg| eprintln!("DEBUG: {}", msg))),
    extra_args: Some({
        let mut args = HashMap::new();
        args.insert("debug-to-stderr".to_string(), None);
        args
    }),
    ..Default::default()
};
```

## Python SDK 对比

Rust SDK 紧密镜像 Python SDK API:

| Python                                        | Rust                                        |
| --------------------------------------------- | ------------------------------------------- |
| `async with ClaudeClient() as client:`     | `client.connect().await?`                   |
| `await client.query("...")`                   | `client.query("...").await?`                |
| `async for msg in client.receive_response():` | `while let Some(msg) = stream.next().await` |
| `await client.interrupt()`                    | `client.interrupt().await?`                 |
| `await client.disconnect()`                   | `client.disconnect().await?`                |

## 🤝 贡献

欢迎贡献！请随时提交 Pull Request。

### 开发设置

```bash
# 克隆仓库
git clone https://github.com/tyrchen/claude-agent-sdk-rs
cd claude-agent-sdk-rs

# 安装依赖
cargo build

# 运行测试
cargo test

# 运行示例
cargo run --example 01_hello_world
```

### 指南

- 遵循 Rust 约定和惯用法
- 为新功能添加测试
- 更新文档和示例
- 提交前运行 `cargo fmt` 和 `cargo clippy`

本 SDK 基于 [claude-agent-sdk-python](https://github.com/anthropics/claude-agent-sdk-python) 规范。

## 许可证

本项目根据 MIT 许可证条款分发。

详见 [LICENSE.md](LICENSE.md)。

## 🔗 相关项目

- [Claude Code CLI](https://docs.claude.com/claude-code) - 官方 Claude Code 命令行界面
- [Claude Agent SDK for Python](https://github.com/anthropics/claude-agent-sdk-python) - 官方 Python SDK
- [Anthropic API](https://www.anthropic.com/api) - Claude API 文档

## ⭐ 支持

如果你觉得这个项目有用，请考虑在 GitHub 上给它一个星标！

## 📝 更新日志

版本历史和更改请参阅 [CHANGELOG.md](CHANGELOG.md)。
