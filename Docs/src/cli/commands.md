# Commands

The `nicy` CLI supports three main commands plus version info.

## `nicy run`

Execute a Luau script file.

### Syntax

```bash
nicy run <file>
```

### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<file>` | Yes | Path to the `.luau`, `.lua`, or `.luauc` file to execute |

### Examples

```bash
# Run a script
nicy run myscript.luau

# Run compiled bytecode
nicy run myscript.luauc
```

## `nicy eval`

Evaluate inline Luau code.

### Syntax

```bash
nicy eval "<code>"
```

### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<code>` | Yes | Luau code string to evaluate (must be quoted) |

### Examples

```bash
# Simple expression
nicy eval "print(2 + 2)"

# Multi-line code
nicy eval "
local function greet(name)
    print('Hello, ' .. name .. '!')
end
greet('World')
"
```

## `nicy compile`

Compile a Luau source file to bytecode (`.luauc`).

### Syntax

```bash
nicy compile <file>
```

### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| `<file>` | Yes | Path to the `.luau` or `.lua` source file |

### Examples

```bash
# Compile with defaults
nicy compile myscript.luau
# Creates: myscript.luauc (same directory, same name)
```

### No CLI Flags

The `compile` command does **not** accept `--native`, `--optimize`, `--output`, or other flags. All compiler configuration is done via **in-source compiler directives**:

```luau
-- myscript.luau
--!native
--!optimize 2

-- Your code here
```

```bash
# Correct: configure via source
nicy compile myscript.luau

# Wrong: CLI flags do not exist
nicy compile myscript.luau --native  # DOES NOT EXIST
```

See [Compiler Directives](../runtime-api/compiler-directives.md) for details.

## `nicy version`

Display the CLI version.

```bash
nicy version
```

Output:
```
nicy 1.0.0-alpha
```

## `nicy runtime-version`

Display both CLI and runtime library versions.

```bash
nicy runtime-version
```

Output:
```
Engine: 1.0.0-alpha
Luau: 0.650
```

## `nicy help`

Show usage information.

```bash
nicy help
```

Output:
```
nicy - The Ultimate Luau Runtime
Usage:
  nicy run <script.luau>
  nicy eval <"code">
  nicy compile <script.luau>
  nicy help
  nicy version
  nicy runtime-version
```
