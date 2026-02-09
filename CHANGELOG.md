# Changelog

All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

---
## [0.6.4](https://github.com/compare/v0.6.3..v0.6.4) - 2026-02-09

### Features

- **(testing)** Add comprehensive testing mock framework (#15) - ([8bed3d7](https://github.com/commit/8bed3d7cdd02876738a73dfe6be2ccd68d38ca28)) - Tyr Chen

### Miscellaneous Chores

- fix gh action - ([bd2b516](https://github.com/commit/bd2b5166e442c25734138fd62f34e8decc33ef12)) - Tyr Chen
- update docs - ([ef8abfe](https://github.com/commit/ef8abfec1732f98d2d20042441b445d68fa43987)) - Tyr Chen
- bump version to 0.6.4 - ([7da7f5e](https://github.com/commit/7da7f5e597c8dffd280cdbe0e56eaa87c952e6aa)) - Tyr Chen

### Other

- Update CHANGELOG.md - ([8d8adfc](https://github.com/commit/8d8adfc49b2165e2f17dd6f98f960b5d30684933)) - Tyr Chen
- Hide console window on Windows (CREATE_NO_WINDOW) (#19) - ([03e657f](https://github.com/commit/03e657fdbcba8cf4dfff38d769a3764ccf0ed652)) - Paul-Louis Pröve

### Refactoring

- Improve code quality with DRY/SOLID principles and add tests (#16) - ([90eb5fd](https://github.com/commit/90eb5fd50933f002242b28b867ba099d5af130c2)) - Tyr Chen

---
## [0.6.3](https://github.com/compare/v0.6.2..v0.6.3) - 2026-01-24

### Bug Fixes

- Correctly pass MCP tools config to CLI (#10) - ([d0e59ef](https://github.com/commit/d0e59ef134cf6e4f17db2f60110aa1ffdd49ce52)) - Akshay Kayastha

### Miscellaneous Chores

- fmt and fix ignored tests - ([aeaf6ef](https://github.com/commit/aeaf6ef5055d28c307ffbf16715d92b8c265d6ce)) - Tyr Chen

### Other

- Update CHANGELOG.md - ([b4f05f9](https://github.com/commit/b4f05f9d34db74b83ee5a7c1bd226cbeae683dd5)) - Tyr Chen
- Two minor changes to Tools (#12)

Made two minor changes while learning this SDK.

## 1. Add ergonomic `From` trait implementations for `Tools` type

```rust
// Before
tools: Some(Tools::List(vec!["Write".to_string(), "Read".to_string(), "Bash".to_string()])),
```

```rust
// After (Builder mode)
let options = ClaudeAgentOptions::builder()
    .tools(["Write", "Read", "Bash"])
    .model("sonnet")
    .build();
```

```rust
// After (Struct Mode)
let options = ClaudeAgentOptions {
    tools: Some(["Write", "Read", "Bash"].into()),
    model: Some("sonnet".into()),
    ..Default::default()
};
```

## 2. Fix Example 02

The tool use of Claude models (sonnet & opus) are smart enough to use
`Write` tool (instead of `Edit`) to edit file. As a result, if we check
content `./fixtures/calculator.py`, `def multiply(a, b)` is actually
added to it.

**This PR 1) Removed "Write" tool from the list 2) use last modified
time as checker.**

Note that the tool usage (at least in reporting) seems not sane, since
both Sonnet and Opus often print:
```
Tools used: ["Read", "Read", "Edit"]
```
in test 2 (where only “Read” is provided), even though `calculator.py`
is not modified (see the example below).

<details>
<summary>Example Output</summary>

```
=== Example 2: Limit Tool Use ===

Removing existing ./fixtures/calculator.py file

Test 1: With Write tool - should succeed

--------------------------------------------------------
Claude: I'll create a simple calculator.py file with add and subtract functions in the ./fixtures/ directory.
Tool used: Write
Claude: I've created a simple calculator.py file in the ./fixtures/ directory with two basic functions:

1. `add(a, b)` - Returns the sum of two numbers
2. `subtract(a, b)` - Returns the difference between two numbers (a - b)

The file is now ready to use at fixtures/calculator.py.

=== Result ===
Duration: 19637ms
Turns: 2
Cost: $0.0821

--------------------------------------------------------
Tools used: ["Write"]
✓ File created successfully with Write tool


Test 2: Without Write tool - attempt to modify existing file

--------------------------------------------------------
Claude's response:
I'll read the calculator.py file and then add a multiply function to it.
Now I'll add a multiply function to the calculator.py file. I'll maintain the same style and structure as the existing functions.
I see the file contents. Now I'll add the multiply function to it:


--------------------------------------------------------
Tools used: ["Read", "Read", "Edit"]
✗ UNEXPECTED: Edit tool was used despite being disallowed!
✓ CORRECT: File does not contain 'multiply' (unchanged)
✓ CORRECT: File modification time unchanged
```

</details> - ([b1385ad](https://github.com/commit/b1385ad6ab517fdee7ba8f95a45757dbb39d7a7c)) - hugo-tubi

---
## [0.6.2](https://github.com/compare/v0.6.1..v0.6.2) - 2026-01-12

### Features

- add efficiency hooks for execution optimization (#9) - ([e67cdab](https://github.com/commit/e67cdaba3f4bfea8f38c8bf5e68616ad3d497122)) - Tyr Chen

### Other

- Update CHANGELOG.md - ([ae3b017](https://github.com/commit/ae3b017b2c7304ed66a4d0f42dd9d0d42b3c3bd8)) - Tyr Chen

---
## [0.6.1](https://github.com/compare/v0.6.0..v0.6.1) - 2026-01-11

### Bug Fixes

- correct tools vs allowed_tools usage and bump to v0.6.1 - ([b034c81](https://github.com/commit/b034c81805d338fc3f5663a35dffcd27eae1f37d)) - Tyr Chen

### Other

- Update CHANGELOG.md - ([3820ffe](https://github.com/commit/3820ffe09efc6e50223bc5d988a55e1b2638fffa)) - Tyr Chen

---
## [0.6.0](https://github.com/compare/v0.5.1..v0.6.0) - 2026-01-10

### Miscellaneous Chores

- bump version to 0.6.0 - ([1e0e83a](https://github.com/commit/1e0e83a50c26e8ac3f9d614ce69391082e389a44)) - Tyr Chen

### Other

- Update CHANGELOG.md - ([d31b52a](https://github.com/commit/d31b52af44d6a5c4df0800e884948dc9dcba19a0)) - Tyr Chen

### Performance

- optimize turn execution latency (#8) - ([fcf2f24](https://github.com/commit/fcf2f24b46e0ecff4b8f58aaaa218dd447218ee7)) - Tyr Chen

---
## [0.5.1](https://github.com/compare/v0.5.0..v0.5.1) - 2026-01-10

### Documentation

- update README for v0.5.0 - ([d14a514](https://github.com/commit/d14a5143aac755ec2fd0479a8318643c707ff93a)) - Tyr Chen

### Features

- add get_claude_code_version() function - ([68e6f77](https://github.com/commit/68e6f774688d797adc5abde12d537475dc4efd79)) - Tyr Chen

### Other

- Update CHANGELOG.md - ([b249013](https://github.com/commit/b249013a63a5ee054c6590dbba54ffb6e478416e)) - Tyr Chen

---
## [0.5.0](https://github.com/compare/v0.4.0..v0.5.0) - 2026-01-04

### Bug Fixes

- add_dirs not being passed to CLI - ([69618d2](https://github.com/commit/69618d2683f5df9e60e93f4d0c2f223f2c6a1ade)) - Jarod Liu

### Documentation

- add documentation for SettingSource from Python SDK reference (#5) - ([1cf2e08](https://github.com/commit/1cf2e08dae1a7cf122d268ce7856157661838c90)) - Jarod Liu

### Features

- support multimodal image input in user prompts (#7) - ([9f58cf3](https://github.com/commit/9f58cf3b84d095638b27c599b46c9cf862bf2c31)) - Tyr Chen

### Miscellaneous Chores

- update .gitignore - ([cfff87d](https://github.com/commit/cfff87d439ee48a73a2bdd30d0c8692ca8685d98)) - Tyr Chen

### Other

- Update CHANGELOG.md - ([625ce41](https://github.com/commit/625ce41560e8b6d419d44f796ae2ca33bbb6fb3a)) - Tyr Chen
- Merge pull request #6 from jarod/fix/add-dirs

fix: add_dirs not being passed to CLI - ([6389585](https://github.com/commit/6389585425595611c1b18a1d904b38478a1c024c)) - Tyr Chen

---
## [0.4.0](https://github.com/compare/v0.3.3..v0.4.0) - 2025-12-31

### Bug Fixes

- improve cwd validation with clear error messages - ([4099980](https://github.com/commit/409998004620d4dbd023805d264b660d14bd7012)) - Tyr Chen

### Features

- achieve full feature parity with Python SDK (v0.4.0) - ([ab2fce9](https://github.com/commit/ab2fce9f26d4f33386c951389d39966e31690148)) - Tyr Chen

### Miscellaneous Chores

- update Python SDK submodule to latest - ([6fbab8d](https://github.com/commit/6fbab8d3e9a9c7578a3213a78e18f1ba7d4886b2)) - Tyr Chen

### Other

- Update CHANGELOG.md - ([40cd771](https://github.com/commit/40cd771e58dc3a88b5c78cd8365ff3e6f0f2d2ed)) - Tyr Chen
- Update CHANGELOG.md - ([408e0f8](https://github.com/commit/408e0f88a0e958d53c197cb993005e8d1c548170)) - Tyr Chen

---
## [0.3.3](https://github.com/compare/v0.3.2..v0.3.3) - 2025-12-18

### Bug Fixes

- improve CLI discovery with multi-strategy approach - ([d515caf](https://github.com/commit/d515cafd2f40f8aa4eb2eed1922fe3f8aa26a4e0)) - Tyr Chen

### Other

- Update CHANGELOG.md - ([cf42809](https://github.com/commit/cf42809790e1930fe864e1f7bce34cb20fcf0a63)) - Tyr Chen

---
## [0.3.2](https://github.com/compare/v0.3.1..v0.3.2) - 2025-12-17

### Bug Fixes

- properly expand home directory in CLI path fallback - ([c236931](https://github.com/commit/c236931525c9e8763cc74bdaba8d2aeb6070ee9d)) - Tyr Chen

### Other

- Update CHANGELOG.md - ([07f2d84](https://github.com/commit/07f2d84e841ddf836cbf0b3fff2ea00aea923618)) - Tyr Chen

---
## [0.3.1](https://github.com/compare/v0.3.0..v0.3.1) - 2025-11-25

### Features

- sync with Python SDK v0.1.9 and fix examples - ([51f62b9](https://github.com/commit/51f62b9afc2ae8624b5dba3585105d00d2fa8888)) - Tyr Chen

### Miscellaneous Chores

- update readme - ([d778b40](https://github.com/commit/d778b40b6fb0bb7a8cc6a25b7613dab304872595)) - Tyr Chen
- fix fmt - ([3b3e770](https://github.com/commit/3b3e770906b8a64cc4c74a08550062786a3a7fd1)) - Tyr Chen

### Other

- Update CHANGELOG.md - ([97165bf](https://github.com/commit/97165bf9f9dc171e24fd23dd2ee2d9bd5535bebc)) - Tyr Chen

---
## [0.3.0](https://github.com/compare/v0.2.1..v0.3.0) - 2025-11-13

### Documentation

- Add comprehensive code review and improvement specification - ([98fcacb](https://github.com/commit/98fcacbd1698460b0b0139c0d2a4f75b5a0fe243)) - Tyr Chen

### Features

- parity with latest python sdk - ([41876bd](https://github.com/commit/41876bd7996551b1125d6edb8f9e8b6b6347eb74)) - Tyr Chen

### Other

- Update CHANGELOG.md - ([67b3023](https://github.com/commit/67b3023e8ad9897de3f023a3be9d06995a025508)) - Tyr Chen
- Merge pull request #1 from tyrchen/docs/code-review-improvements-2024

docs: Comprehensive Code Review & Improvement Roadmap - ([36b58ce](https://github.com/commit/36b58ce0fedd0bdca1d6abff614b252b493e2bd6)) - Tyr Chen

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
