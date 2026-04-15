# runtime Table

The `runtime` global table provides NicyRuntime-specific functionality.

## Properties

### `runtime.version`

The NicyRuntime version string.

```luau
print(runtime.version)
-- "1.0.0-alpha"
```

### `runtime.hasJIT(spec?)`

A function that checks if CodeGen/JIT is available.

```luau
-- Check default
if runtime.hasJIT() then
    print("JIT is available")
end

-- Check for a specific module spec
if runtime.hasJIT("@self/perf_module") then
    print("JIT available for this module")
end
```

### `runtime.loadlib(path)`

Load a dynamic library (`.dll`, `.so`, `.dylib`) from a specified path.

```luau
local result = runtime.loadlib("@self/mylib.dll")
```

#### Path Formats

| Format | Description |
|--------|-------------|
| `@self/path` | Relative to the entry script's directory |
| `./relative/path` | Relative to the current working directory |
| `/absolute/path` | Absolute path |

#### Caching

Libraries are cached by their resolved path. Subsequent calls with the same path return the cached result.

#### SEH Crash Protection (Windows)

On Windows, library loading is wrapped in SEH to catch access violations. If a library crashes during load, an error is returned instead of crashing the process.

### `runtime.entry_file`

The absolute path of the script passed to `nicy_start`.

```luau
print(runtime.entry_file)
-- e.g., "/path/to/myscript.luau"
```

### `runtime.entry_dir`

The directory containing the entry script.

```luau
print(runtime.entry_dir)
-- e.g., "/path/to/"
```

## `warn` Override

NicyRuntime provides a custom `warn()` implementation that integrates with the error reporting system. If the global `warn` is `nil`, it's replaced with NicyRuntime's version.
