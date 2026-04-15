# Compiler Directives

Compiler directives are special comments at the top of Luau source files that control compilation behavior.

## Syntax

Directives are placed at the **beginning** of the file (before any code):

```luau
--!native
--!optimize 2

-- Your code starts here
local function main()
    -- ...
end
```

## Available Directives

### `--!native`

Enable Luau CodeGen (native code generation) for this file.

```luau
--!native

local function hot_function(x)
    -- This will be compiled to native code
    return x * x * x
end
```

**Effect**: The Luau VM will generate native machine code for functions in this file, providing faster execution.

**Platform Support**:
- ✅ Windows, macOS, Linux (x64, ARM64)
- ❌ Android (disabled)

**Equivalent CLI flag**: `--native`

### `--!optimize <level>`

Set the optimization level for this file.

```luau
--!optimize 2

local function compute()
    -- Aggressively optimized
end
```

**Levels**:

| Level | Description |
|-------|-------------|
| `0` | No optimization (fastest compilation, slowest execution) |
| `1` | Default optimization (balanced) |
| `2` | Aggressive optimization (slowest compilation, fastest execution) |

**Equivalent CLI flag**: `--optimize <level>`

### `--!coverage`

Enable code coverage instrumentation.

```luau
--!coverage

-- Code will track which lines are executed
```

**Effect**: Generates coverage data that can be used to analyze which parts of the code are executed during a run.

### `--!profile`

Enable profiling instrumentation.

```luau
--!profile

-- Functions will include profiling hooks
```

**Effect**: Adds overhead to track function call counts and execution times.

### `--!typeinfo <level>`

Enable type info generation.

```luau
--!typeinfo 1

local function add(a: number, b: number): number
    return a + b
end
```

**Levels**:
- `0` — No type info
- `1` — Basic type info
- `2` — Full type info (includes inferred types)

## Multiple Directives

Multiple directives can be combined:

```luau
--!native
--!optimize 2
--!typeinfo 1

-- This file uses native code generation,
-- aggressive optimization, and full type info
```

## Directive Precedence

1. **CLI flags** (highest priority) — Flags passed to `nicy compile` or `nicy_compile()` override source directives
2. **Source directives** — Directives in the source file
3. **Defaults** — If neither is specified, defaults apply (no native, optimize level 1, no type info)

## Implementation Details

Directives are parsed from the **first lines** of the source file before compilation. The parser:

1. Reads lines starting with `--!`
2. Extracts the directive name and optional value
3. Strips the directive lines from the source (they are not executed)
4. Applies the directives during compilation

Lines after the first non-directive line are treated as regular code.

## Examples

### Production Build

```luau
--!native
--!optimize 2

-- Production code with maximum performance
```

### Debug Build

```luau
--!optimize 0

-- Debug code with no optimization for easier debugging
```

### Typed Code

```luau
--!native
--!typeinfo 2

local function factorial(n: number): number
    if n <= 1 then return 1 end
    return n * factorial(n - 1)
end
```

## See Also

- [`nicy_compile`](nicy-compile.md)
- [CodeGen/JIT Guide](../advanced/codegen-jit)
