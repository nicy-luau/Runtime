# Flags

The NicyRuntime CLI does **not** use flags for compilation configuration.

## Environment Variables

| Variable | Effect |
|----------|--------|
| `NICY_VERBOSE_ERRORS=1` | Enable verbose error output |
| `NICY_NO_COLOR=1` | Disable ANSI colors in errors |
| `NICY_HIRES_TIMER=1` | Enable high-resolution timer (Windows) |

## Compiler Configuration

Compiler configuration is done via **in-source compiler directives**, not CLI flags:

```luau
--!native          -- Enable CodeGen/JIT
--!optimize 2      -- Optimization level (0-2)
```

```bash
# Correct: configure via source directives
nicy compile myscript.luau

# Wrong: CLI flags do not exist
nicy compile myscript.luau --native  # DOES NOT EXIST
```

See [Compiler Directives](../runtime-api/compiler-directives.md) for details.
