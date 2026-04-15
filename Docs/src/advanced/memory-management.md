# Memory Management

Understanding how NicyRuntime manages memory and how to avoid leaks.

## Luau Garbage Collection

Luau uses **incremental garbage collection**. The collector runs automatically in the background, so you typically don't need to manage memory manually.

### Manual GC Control

```luau
-- Force a full collection
collectgarbage("collect")

-- Get memory usage (KB)
local mem = collectgarbage("count")
print("Memory: " .. mem .. " KB")

-- Stop the collector
collectgarbage("stop")

-- Restart the collector
collectgarbage("restart")

-- Perform a single step
collectgarbage("step")
```

### GC via FFI

```c
// Full collection
nicy_lua_gc(L, LUA_GCCOLLECT, 0);

// Get memory usage (KB)
int kb = nicy_lua_gc(L, LUA_GCCOUNT, 0);

// Stop/restart
nicy_lua_gc(L, LUA_GCSTOP, 0);
nicy_lua_gc(L, LUA_GCRESTART, 0);
```

## NicyRuntime Memory Safety

### Static State Cleanup

NicyRuntime cleans up all static state between calls to `nicy_start()` (FIX C-1). This prevents:

- Stale module caches from previous executions
- Dangling coroutine references
- Memory leaks from loaded libraries

### Library Unloading

Libraries loaded via `runtime.loadlib()` are unloaded when the runtime shuts down (FIX C-5). The cleanup sequence:

1. Stop the task scheduler
2. Unref all registry references
3. Unload all dynamically loaded libraries
4. Destroy the Luau state
5. Clear static state (module cache, require chain, etc.)

## Avoiding Memory Leaks

### In Luau Scripts

```luau
-- Good: Let GC collect unused data
local function processData()
    local largeTable = {}
    for i = 1, 1000000 do
        largeTable[i] = i * 2
    end
    local result = compute(largeTable)
    return result  -- largeTable is now eligible for GC
end

-- Bad: Holding references indefinitely
local cache = {}
local function cachedProcess(key)
    if not cache[key] then
        cache[key] = loadData(key)  -- Cache grows forever!
    end
    return cache[key]
end
```

### In Host Applications

```c
// Good: Clean up after each execution
for (int i = 0; i < 100; i++) {
    nicy_start("script.luau", 0, NULL);
    // Static state is cleaned up automatically
}

// Good: Use nicy_eval for isolated evaluations
for (int i = 0; i < 100; i++) {
    char code[64];
    sprintf(code, "return %d * 2", i);
    const char* result = nicy_eval(code);
    // Each call creates and destroys its own state
}
```

## Memory Profiling

Enable verbose errors to see more details about memory-related issues:

```bash
NICY_VERBOSE_ERRORS=1 nicy run memory_test.luau
```

Monitor memory usage in your host application:

```c
// Get Luau memory usage
int kb = nicy_lua_gc(L, LUA_GCCOUNT, 0);
printf("Luau memory: %d KB\n", kb);
```

## Known Issues

| Issue | Status | Fix |
|-------|--------|-----|
| Stale module cache between `nicy_start` calls | ✅ Fixed (FIX C-1) | Static state cleanup |
| Loaded libraries not unloaded | ✅ Fixed (FIX C-5) | Library unload on shutdown |
| Coroutine references leaked | ✅ Fixed | Scheduler shutdown with unref |
