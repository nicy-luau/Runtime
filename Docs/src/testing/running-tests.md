# Running Tests

NicyRuntime includes a comprehensive test suite for validating runtime behavior.

## Quick Start

Run all tests:

```bash
nicy run Runtime/tests/run_all.luau
```

Run a single test file:

```bash
nicy run Runtime/tests/Task/spawn.luau
```

## Test Categories

Tests are organized by category:

| Category | Files | Description |
|----------|-------|-------------|
| **Core** | 11 | Basic Luau functionality (stdlib, GC, metatables, vectors, etc.) |
| **Require** | 6 + fixtures | Module resolution, aliases, circular deps, bytecode loading |
| **Runtime** | 6 | Error handling, globals, debug, shutdown |
| **Task** | 7 | Scheduler functionality (spawn, defer, delay, wait, cancel, stress) |

## Running Individual Tests

Each test is a standalone `.luau` file that can be executed with:

```bash
# From project root
nicy run Runtime/tests/Core/api.luau
nicy run Runtime/tests/Require/aliases.luau
nicy run Runtime/tests/Runtime/error_handler.luau
nicy run Runtime/tests/Task/precision.luau
```

## Test Output Format

Tests use the built-in test helper (`Runtime/tests/helpers/expect.luau`) for assertions:

```luau
-- Example test
local result = some_function()
expect(result).to_equal("expected_value")
print("✓ Test name")
```

Output:
```
✓ Test name
✗ Failed test
  Expected: "expected_value"
  Got: "actual_value"
```

## Test Fixtures

Some tests use fixture files in `Runtime/tests/Require/fixtures/`:

- `simple.luau` — Basic module for require testing
- `nested.luau` — Nested module structure for path resolution testing

## Continuous Integration

Tests are run automatically on every push via GitHub Actions. See `.github/workflows/`.

## Adding New Tests

1. Create a `.luau` file in the appropriate category directory
2. Use `expect()` from `helpers/expect.luau` for assertions
3. Print `✓ Test name` on success, descriptive error on failure
4. Add to `run_all.luau` if not auto-discovered
