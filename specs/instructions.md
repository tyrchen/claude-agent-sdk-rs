# Instructions

## Capture raw json data

carefully craft many python script under ./tools to use python claude agent sdk to capture all possible use cases of all kinds of json data agent produces that matches vendors/claude-agent-sdk-python/src/claude_agent_sdk/types.py. Store these captured data in ./fixtures for Rust unit tests. The ./tools is using uv to manage python env and project. For python claude sdk usage see vendors/claude-agent-sdk-python. Run this in parallel. Once scripts are done please run it use `uv run ...` to verify proper data are captured.
