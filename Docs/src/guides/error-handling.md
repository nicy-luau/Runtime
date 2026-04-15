# Error Handling

NicyRuntime provides a robust error handling system with concise output by default and verbose mode for debugging.

## Concise Mode (Default)

Errors are displayed in a compact, readable format:

```
Error: module 'missing_module' not found
  searched:
    ./missing_module.luauc
    ./missing_module.luau
    ./missing_module.lua
```

## Verbose Mode

Enable verbose mode with the `NICY_VERBOSE_ERRORS` environment variable:

```bash
NICY_VERBOSE_ERRORS=1 nicy run broken.luau
```

Verbose output includes:
- Full stack trace
- Require chain (which modules required which)
- File paths and line numbers
- Exception details (PowerShell-style formatting)

## Error Codes

NicyRuntime extends standard Luau error codes with custom codes for better error categorization.

### Standard Luau Codes

| Code | Name | Description |
|------|------|-------------|
| 0 | `LUA_OK` | Success |
| 1 | `LUA_YIELD` | Coroutine yielded |
| 2 | `LUA_ERRRUN` | Runtime error |
| 3 | `LUA_ERRSYNTAX` | Syntax error |
| 4 | `LUA_ERRMEM` | Memory error |
| 5 | `LUA_ERRERR` | Error handler error |
| 6 | `LUA_ERRFILE` | File error |

### Nicy-Specific Codes

| Code | Name | Description | Luau Equivalent |
|------|------|-------------|-----------------|
| 100 | `NICY_ERR_MODULE_NOT_FOUND` | Require failed to resolve module | `LUA_ERRFILE` |
| 101 | `NICY_ERR_MODULE_LOAD_FAILED` | Module found but failed to load/compile | `LUA_ERRSYNTAX` |
| 102 | `NICY_ERR_MODULE_INIT_FAILED` | Module loaded but init function failed | `LUA_ERRRUN` |
| 103 | `NICY_ERR_CYCLIC_REQUIRE` | Cyclic dependency detected | `LUA_ERRRUN` |
| 104 | `NICY_ERR_TASK_CRASH` | Task/coroutine crashed | `LUA_ERRRUN` |
| 105 | `NICY_ERR_NATIVE_CRASH` | Native DLL crashed | `LUA_ERRRUN` |
| 106 | `NICY_ERR_TIMEOUT` | Operation timed out | `LUA_ERRRUN` |
| 107 | `NICY_ERR_PERMISSION_DENIED` | Access denied | `LUA_ERRFILE` |

### Accessing Error Codes via FFI

When using NicyRuntime via FFI (C, C++, Rust, etc.), errors are returned as integer codes. The mapping is:

```c
// In your C/C++ code:
int error_code = nicy_start("script.luau");

switch (error_code) {
    case 0:   // LUA_OK
        printf("Success\n");
        break;
    case 100: // NICY_ERR_MODULE_NOT_FOUND
        fprintf(stderr, "Module not found\n");
        break;
    case 103: // NICY_ERR_CYCLIC_REQUIRE
        fprintf(stderr, "Cyclic require detected\n");
        break;
    // ... etc
}
```

> 🔧 **Future**: A `nicy_error_code()` function will be exposed to convert error objects to codes.
> Currently, error codes are embedded in the error message returned by `lua_tostring()`.

## Error Colors

Errors are colorized by default using ANSI escape codes:
- **Red** — Error messages
- **Yellow** — Warnings
- **Cyan** — Info/context

### Disabling Colors

```bash
NICY_NO_COLOR=1 nicy run script.luau
```

## pcall and Error Suppression

Errors inside `pcall` are **not reported** to the console. The error reporter tracks `pcall` state and suppresses errors accordingly:

```luau
local success, err = pcall(function()
    error("This error is caught")
end)

if not success then
    print("Caught: " .. err)
end
-- No error output to console
```

## Require Chain Tracking

When an error occurs inside a `require()` chain, NicyRuntime tracks the full chain:

```
RequireChain:
  main.luau → a.luau → b.luau
```

## SEH Crash Protection (Windows)

On Windows, `runtime.loadlib()` is wrapped in SEH (Structured Exception Handling) to catch access violations during library loading. This prevents the entire process from crashing when a library has bugs.
