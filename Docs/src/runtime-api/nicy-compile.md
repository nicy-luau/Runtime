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

### Returns

Returns the coroutine thread (useful for `task.cancel`).

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

Use `task.defer` when you want to ensure a function runs **after** the current iteration of the scheduler.

## `task.delay`

Schedule a function to run after a delay (in seconds).

```luau
local id = task.delay(2.0, function()
    print("2 seconds later!")
end)

print("Waiting...")
task.wait(3.0)
print("Done!")
```

### Returns

Returns a numeric ID that can be used with `task.cancel`:

```luau
local id = task.delay(5.0, function()
    print("This won't fire")
end)

task.cancel(id)
```

### Safe Integer Validation

`task.cancel` validates IDs against the safe integer range (2^53). IDs exceeding this range are rejected to prevent incorrect cancellations.

## `task.wait`

Pause the current coroutine for the specified duration (in seconds).

```luau
print("Waiting 1 second...")
local elapsed = task.wait(1.0)
print(string.format("Waited %.3f seconds", elapsed))
```

### In the Main Thread

When called from the main thread (not inside `task.spawn`), `task.wait` runs the scheduler for the specified duration:

```luau
task.spawn(function()
    for i = 1, 10 do
        print("Spawned: " .. i)
        task.wait(0.1)
    end
end)

-- This runs the scheduler for 2 seconds
task.wait(2.0)
print("Main thread resumed")
```

### In a Spawned Task

When called inside `task.spawn`, `task.wait` yields the current coroutine and the scheduler resumes it after the delay.

### Auto-registration

If `task.wait` is called from a coroutine that wasn't created via `task.spawn` (e.g., `coroutine.create`), the scheduler automatically registers it so it can be managed.

## `task.cancel`

Cancel a running task or a scheduled delay.

```luau
-- Cancel a spawned task
local t = task.spawn(function()
    while true do
        print("Running...")
        task.wait(0.5)
    end
end)

task.wait(2.0)
task.cancel(t)

-- Cancel a delayed task
local id = task.delay(5.0, function()
    print("Won't execute")
end)

task.wait(1.0)
task.cancel(id)
```

### Behavior

- Returns `true` if the task was found and cancelled
- Returns `false` if the task was not found or already completed
- Safely handles invalid IDs (no errors)

## Scheduler Behavior

### Execution Order

The scheduler processes tasks in this order:

1. **Ready queue** (`task.spawn`) — Tasks that are ready to run
2. **Yielded queue** (`task.defer`) — Tasks that yielded and are ready to resume
3. **Timers** (`task.delay`, `task.wait`) — Delayed tasks whose time has come

### Cooperative Multitasking

Tasks must **cooperate** by calling `task.wait()` or completing. A task that runs an infinite loop without yielding will block all other tasks:

```luau
-- Bad: Blocks the scheduler
task.spawn(function()
    while true do
        -- No yield!
    end
end)

-- Good: Cooperative
task.spawn(function()
    while true do
        -- Do work
        task.wait(0)  -- Yield to other tasks
    end
end)
```

### Main Thread Behavior

When running via `nicy run`, the main thread:

1. Executes the entry script
2. Runs the scheduler until all tasks complete
3. Exits

This means `nicy run` will **wait** for all spawned tasks to finish before exiting.

## Advanced Patterns

### Worker Pool

```luau
local function worker(id, jobs)
    while true do
        local job = table.remove(jobs, 1)
        if not job then break end

        print("Worker " .. id .. " processing: " .. job)
        task.wait(0.5)
    end
end

local jobs = {"job1", "job2", "job3", "job4", "job5"}

for i = 1, 3 do
    task.spawn(worker, i, jobs)
end

task.wait(10)
print("All jobs complete!")
```

### Timeout Pattern

```luau
local function withTimeout(duration, fn)
    local done = false
    local result = nil

    task.spawn(function()
        result = fn()
        done = true
    end)

    task.wait(duration)
    if not done then
        error("Operation timed out!")
    end

    return result
end
```
