# nicy_start

Initialize the Luau runtime and execute a script file.

## C Signature

```c
void nicy_start(const char* file_path);
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `file_path` | `const char*` | Path to the `.luau`, `.lua`, or `.luauc` file to execute |

## Description

`nicy_start` is the main entry point for executing Luau scripts. It:

1. Loads the dynamic runtime library
2. Creates a new Luau state with standard libraries
3. Installs the error handler and `runtime`/`task` globals
4. Loads, compiles (with optional CodeGen), and executes the script
5. Runs the task scheduler until idle (processes all `task.spawn`, `task.delay`, etc.)
6. Cleans up (unloads libraries, destroys state, resets static state)

## Example (C)

```c
#include "NicyRuntime.h"

int main() {
    nicy_start("myscript.luau");
    return 0;
}
```

## Compiler Directives

The source file can include compiler directives on the first lines:

```luau
--!native          -- Enable CodeGen/JIT
--!optimize 2      -- Set optimization level (0-2)
--!coverage        -- Enable coverage tracking
--!profile         -- Enable profiling
--!typeinfo 1      -- Enable type info generation
```

## Global Environment

After initialization, the following globals are available:

### `runtime` table

| Property | Type | Description |
|----------|------|-------------|
| `runtime.version` | `string` | NicyRuntime version |
| `runtime.loadlib` | `function` | Dynamically load a native library |
| `runtime.hasJIT(spec?)` | `function` | Check if JIT is available for a spec |
| `runtime.entry_file` | `string` | Path of the executed script |
| `runtime.entry_dir` | `string` | Directory of the executed script |

### `task` table

| Function | Description |
|----------|-------------|
| `task.spawn(fn, ...)` | Spawn a concurrent task |
| `task.defer(fn, ...)` | Defer execution to the end of the queue |
| `task.delay(seconds, fn, ...)` | Schedule delayed execution |
| `task.wait(seconds?)` | Non-blocking wait |
| `task.cancel(thread_or_id)` | Cancel a task or delay |

### OS extensions

| Function | Description |
|----------|-------------|
| `os.exit(code)` | Exit the process |
| `os.getenv(name)` | Get environment variable |
| `os.remove(path)` | Delete a file |
| `os.rename(old, new)` | Rename a file |
| `os.sleep(ms)` | Sleep (in milliseconds) |
| `os.tmpname()` | Generate a unique temp filename |

Standard `os.clock()`, `os.time()`, `os.date()`, `os.difftime()` are also available.

## Environment Variables

| Variable | Effect |
|----------|--------|
| `NICY_VERBOSE_ERRORS=1` | Enable verbose error output |
| `NICY_NO_COLOR=1` | Disable ANSI colors in errors |
| `NICY_HIRES_TIMER=1` | Enable high-resolution timer (Windows) |

## Thread Safety

`nicy_start` is **not thread-safe**. Each call creates and destroys its own state. Static state is cleaned up between calls.
