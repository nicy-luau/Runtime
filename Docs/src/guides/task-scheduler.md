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

**Main thread**: runs the scheduler for the specified duration.
**Spawned tasks**: yields the coroutine; scheduler resumes after delay.

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
