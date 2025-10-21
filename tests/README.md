# Rust Unit Tests for Claude Agent SDK

## Overview

This directory contains comprehensive unit tests that validate Rust type compatibility with the Python Claude Agent SDK using **real data** captured from actual API interactions.

## Test Files

### `real_fixtures_test.rs`

Comprehensive integration tests using 130 real JSON fixtures captured from Claude API interactions.

**Test Coverage:**
- ✅ **28 test functions** - All passing
- ✅ **130 real fixtures** - All deserialize successfully
- ✅ **Message types** - Assistant, User, System, Result, StreamEvent
- ✅ **Content blocks** - Text, ToolUse, ToolResult, Thinking
- ✅ **Real API data** - Actual IDs, usage stats, costs
- ✅ **Streaming events** - 97 real streaming deltas
- ✅ **Serialization** - Round-trip validation

## Running Tests

```bash
# Run all tests
cargo test

# Run just the real fixtures tests
cargo test --test real_fixtures_test

# Run with output
cargo test --test real_fixtures_test -- --nocapture

# Run specific test
cargo test test_real_assistant_001_basic_text
```

## Test Categories

### 1. Assistant Message Tests (6 tests)
- Basic text responses
- Tool use planning
- Tool execution messages
- Comprehensive validation of all 16 assistant fixtures

### 2. User Message Tests (3 tests)
- Tool result messages
- Error handling
- All 5 user message variants

### 3. System Message Tests (2 tests)
- Session initialization
- All 6 system message variants

### 4. Result Message Tests (2 tests)
- Query completion with cost/usage
- All 6 result message variants

### 5. Stream Event Tests (4 tests)
- Message start events
- Content delta events
- Message stop events
- All 97 streaming event variants

### 6. Content Block Tests (2 tests)
- Text blocks
- Tool use blocks with real tool IDs

### 7. Validation Tests (9 tests)
- Usage statistics structure
- Cost calculations
- Session ID consistency
- UUID uniqueness
- Message ID format (`msg_*`)
- Tool use ID format (`toolu_*`)
- Serialization round-trips
- Error handling
- Comprehensive 130-fixture validation

## Real Data Validation

All tests use **real JSON responses** captured from actual Claude API calls, ensuring:

1. **Authentic Structure** - Real API response format
2. **Real IDs** - Actual message IDs, tool use IDs, session IDs
3. **Real Usage Data** - Genuine token counts, cache stats, costs
4. **Real Tool Execution** - Actual tool inputs/outputs
5. **Real Streaming** - Complete streaming sequences

## Fixtures Location

All fixtures are stored in `../fixtures/raw_messages/`:

```
fixtures/raw_messages/
├── assistant_001.json - assistant_016.json (16 files)
├── user_001.json - user_005.json (5 files)
├── system_001.json - system_006.json (6 files)
├── result_001.json - result_006.json (6 files)
└── stream_event_001.json - stream_event_097.json (97 files)
```

Total: **130 real JSON fixtures**

## Test Results

```
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Key Findings

1. ✅ All real fixtures deserialize correctly
2. ✅ Message type discrimination works properly
3. ✅ Optional fields handled correctly
4. ✅ Nested structures parse correctly
5. ✅ Real API IDs match expected formats
6. ✅ Serialization is reversible (round-trip works)
7. ✅ Error cases handled gracefully

## Adding New Tests

To add tests for new fixtures:

1. Capture new real data using `tools/capture_real_interactions.py`
2. Add fixtures to `fixtures/raw_messages/`
3. Write test in `real_fixtures_test.rs`:

```rust
#[test]
fn test_my_new_fixture() {
    let msg = test_fixture!("my_fixture.json");
    match msg {
        Message::Assistant(assistant) => {
            // Your assertions here
        }
        _ => panic!("Expected Assistant message"),
    }
}
```

## Continuous Integration

These tests should be run on every commit to ensure:
- Type compatibility with Python SDK
- No regressions in deserialization
- All real-world data formats supported

## Performance

- **Compilation**: ~0.4s
- **Execution**: ~0.03s (all 28 tests)
- **Fixtures loaded**: 130 JSON files
- **Total test time**: <1 second

## Related Documentation

- `../fixtures/README.md` - Fixture data organization
- `../tools/REAL_DATA_SUMMARY.md` - Fixture capture methodology
- `../src/types/messages.rs` - Type definitions being tested
