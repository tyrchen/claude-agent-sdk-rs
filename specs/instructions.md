# Instructions

## Capture raw json data

carefully craft many python script under ./tools to use python claude agent sdk to capture all possible use cases of all kinds of json data agent produces that matches vendors/claude-agent-sdk-python/src/claude_agent_sdk/types.py. Store these captured data in ./fixtures for Rust unit tests. The ./tools is using uv to manage python env and project. For python claude sdk usage see vendors/claude-agent-sdk-python. Run this in parallel. Once scripts are done please run it use `uv run ...` to verify proper data are captured.

## Improve hook usage

currently adding a hook is a bit hard:

```rust

    // Configure hooks using ClaudeAgentOptions builder
    let mut hooks: HashMap<HookEvent, Vec<HookMatcher>> = HashMap::new();

    // Add PreToolUse hook that prints info for all tools
    hooks.insert(
        HookEvent::PreToolUse,
        vec![
            // First hook: print info for all tools - using builder pattern
            HookMatcher::builder()
                .hooks(vec![Arc::new(|input, tool_use_id, context| {
                    Box::pin(print_tool_info(input, tool_use_id, context))
                })])
                .build(),
            // Second hook: block dangerous bash commands - using builder pattern
            HookMatcher::builder()
                .matcher("Bash") // Only match Bash tool
                .hooks(vec![Arc::new(|input, tool_use_id, context| {
                    Box::pin(block_dangerous_bash(input, tool_use_id, context))
                })])
                .build(),
        ],
    );
```

Can we improve the API to make it easier to use, something like (please improve the API design when necessary):

```rust
let mut hooks = Hooks::new();
hooks.add_pre_tooluse(None, print_tool_info); // no matcher
hooks.add_pre_tooluse("Bash", block_dangerous_bash); // matcher is "Bash"
hooks.add_pre_tooluse("TodoWrite", capture_todo_update_hook); // matcher is "TodoWrite"
hooks.add_post_user_prompt_submit(print_user_prompt);
hooks.add_post_stop(print_summary);

async fn print_tool_info(input: HookInput, tool_use_id: Option<String>, context: HookContext) -> HookJsonOutput {
  ...
}
...
```

The `add_xxx` functions are generated using a macro based on the HookEvent enum (hooks.rs).

Please think ultra hard and make this user friendly and easy to use. Update 05 example once done. Run it to see if works.
