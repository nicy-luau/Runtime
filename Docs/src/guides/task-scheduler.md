# Task Scheduler

NicyRuntime includes a built-in async task scheduler based on Luau coroutines.

## Overview

The `task` global table provides these functions:

| Function | Description |
|----------|-------------|
| `task.spawn(fn, ...)` | Create and schedule a concurrent task |
| `task.defer(fn, ...)` | Schedule for execution after current tasks |
| `task.delay(seconds, fn, ...)` | Schedule execution after a delay |
| `task.wait(seconds?)` | Pause the current coroutine |
| `task.cancel(thread_or_id)` | Cancel a spawned task or delayed task |

## `task.spawn`

Create a new coroutine and schedule it for immediate execution.

```luau
task.spawn(function()
    print("Running in a separate task!")
end)

-- With arguments
task.spawn(function(name, count)
    for i = 1, count do
        print(name .. ": " .. i)
    end
end, "Worker", 5)
```

## `task.defer`

Schedule a function to run after all currently ready tasks yield or complete.

```luau
print("Before defer")

task.defer(function()
    print("Deferred!")
end)

print("After defer")

-- Output:
-- Before defer
-- After defer
-- Deferred!
```

## `task.delay`

Schedule a function to run after a delay (in seconds).

```luau
local id = task.delay(2.0, function()
    print("2 seconds later!")
end)

task.wait(3.0)
```

Returns a numeric ID for cancellation:

```luau
local id = task.delay(5.0, function()
    print("This won't fire")
end)

task.cancel(id)
```

## `task.wait`

Pause the current coroutine for the specified duration (in seconds).

```luau
print("Waiting 1 second...")
local elapsed = task.wait(1.0)
print(string.format("Waited %.3f seconds", elapsed))
```

**Returns**: Actual elapsed time in seconds (float).

### Main Thread vs Coroutine Behavior

| Context | Behavior | CPU Usage |
|---------|----------|-----------|
| **Main thread** (entry script) | Synchronous busy-wait loop with `run_one_iteration()` | ⚠️ High (calls `std::thread::yield_now()`) |
| **Spawned task** (via `task.spawn`) | Async yield to scheduler; coroutine is suspended | ✅ None (coroutine is parked) |

### Limitations

- **Minimum precision**: 1ms (internally rounded). Values < 0.001s are rounded up.
- **Maximum timeout**: 10 years (implementation limit). Values exceeding this are capped.
- **Main thread busy-wait**: When called from the main thread, `task.wait()` consumes CPU even with `yield_now()`. Use only for short waits; prefer `task.spawn` for long delays.
- **Non-finite values**: `nil`, `NaN`, `Infinity`, and negative values are treated as 0 (immediate yield).

### Example: Main Thread vs Task

```luau
-- Main thread: busy-wait (high CPU)
print("Main thread waiting...")
task.wait(1.0)  -- Blocks and consumes CPU

-- Spawned task: async yield (no CPU)
task.spawn(function()
    print("Task waiting...")
    task.wait(1.0)  -- Yields to scheduler, zero CPU
end)
```

## `task.cancel`

Cancel a running task or a scheduled delay.

```luau
local t = task.spawn(function()
    while true do
        print("Running...")
        task.wait(0.5)
    end
end)

task.wait(2.0)
task.cancel(t)
```

Returns `true` if cancelled, `false` if not found or already completed.

### Cancellation by Type

| Argument Type | Behavior |
|---------------|----------|
| **Thread** (from `task.spawn`) | Removes from all scheduler queues, clears pending timers, unreferences registry entry |
| **Delay ID** (number from `task.delay`) | Removes timer entry, prevents function from firing |

### Limitations

- **Delay ID precision**: IDs are passed as `f64` (Lua numbers). IDs exceeding `2^53` (9,007,199,254,740,992) are rejected due to IEEE 754 precision loss. A warning is logged: `"task.cancel: id X exceeds safe integer range (2^53), ignoring"`.
- **Silent failure**: Canceling a non-existent or already-completed task returns `false` without error. This is by design for safe cleanup.
- **Thread reference**: The thread must have been registered with the scheduler (via `task.spawn`, `task.delay`, or explicit `task.wait`). Coroutines created via `coroutine.create` are not tracked.

### Example: Delay ID Limit

```luau
-- This will fail silently with a warning
local huge_id = 9007199254740993.0  -- Exceeds 2^53
task.cancel(huge_id)  -- Returns false, warns: "id exceeds safe integer range"

-- Normal usage (safe range)
local id = task.delay(5.0, function() end)
task.cancel(id)  -- Works correctly
```

## Scheduler Behavior

The scheduler processes tasks in this order:

1. **Ready queue** (`task.spawn`)
2. **Yielded queue** (`task.defer`)
3. **Timers** (`task.delay`, `task.wait`)

### Cooperative Multitasking

Tasks must call `task.wait()` to yield. An infinite loop without yielding blocks all other tasks:

```luau
-- Bad: blocks scheduler
task.spawn(function()
    while true do
        -- No yield!
    end
end)

-- Good: cooperative
task.spawn(function()
    while true do
        -- Do work
        task.wait(0)  -- Yield
    end
end)
```

### Main Thread Behavior

When using `nicy run`, the main thread:

1. Executes the entry script
2. Runs the scheduler until all tasks complete
3. Exits
