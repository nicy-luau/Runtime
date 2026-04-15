# Test Structure

NicyRuntime's test suite is organized by functionality area.

## Directory Layout

```
Runtime/tests/
├── Core/              # Basic Luau functionality
│   ├── api.luau       # Lua C API wrapper testing
│   ├── bit32.luau     # Bit32 library
│   ├── buffers.luau   # Buffer handling
│   ├── edge_cases.luau
│   ├── gc.luau        # Garbage collection
│   ├── io_files.luau  # File I/O
│   ├── luaurc.luau    # .luaurc parsing
│   ├── metatables.luau
│   ├── os_ext.luau    # OS extensions
│   ├── stdlib.luau    # Standard library
│   └── vectors.luau   # Vector4 type
├── Require/           # Module system
│   ├── aliases.luau
│   ├── bytecode.luau
│   ├── circular.luau
│   ├── concurrent.luau
│   ├── relative.luau
│   ├── resolution.luau
│   └── fixtures/      # Test fixtures
│       ├── simple.luau
│       └── nested.luau
├── Runtime/           # Runtime behavior
│   ├── debug.luau
│   ├── error_handler.luau
│   ├── globals.luau
│   ├── loadlib_errors.luau
│   ├── shutdown.luau
│   └── traceback.luau
├── Task/              # Task scheduler
│   ├── cancel.luau
│   ├── defer.luau
│   ├── delay.luau
│   ├── precision.luau
│   ├── spawn.luau
│   ├── stress.luau
│   └── stress_extreme.luau
├── helpers/           # Test utilities
│   ├── expect.luau    # Assertion library
│   ├── init.luau
│   └── report.luau    # Test reporting
└── run_all.luau       # Master test runner
```

## Test Helper Files

### `helpers/expect.luau`

Provides assertion-style testing:

```luau
local expect = require("helpers/expect")

expect(value).to_be(true)
expect(table).to_equal(expected)
expect(func).to_error("expected error message")
```

### `helpers/report.luau`

Generates test summary reports:

```
Tests: 42 passed, 0 failed, 0 skipped
Time: 1.234s
```

## Writing Tests

### Basic Structure

```luau
-- Import helpers
local helpers = require("helpers/init")

-- Test 1
local result = my_function(1, 2)
assert(result == 3, "my_function should return 3")
print("✓ my_function basic test")

-- Test 2 with expect
expect(result).to_equal(3)
print("✓ my_function expect test")
```

### Error Testing

```luau
-- Test that an error is thrown
local success, err = pcall(function()
    error_function()
end)

assert(not success, "Should fail")
assert(err:find("expected error"), "Wrong error message: " .. err)
print("✓ error handling test")
```

### Async Testing

```luau
-- Test task scheduler
task.spawn(function()
    task.wait(0.1)
    print("✓ async test completed")
end)

task.wait(0.5)  -- Wait for task to complete
```
