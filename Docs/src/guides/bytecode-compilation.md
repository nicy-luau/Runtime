# Bytecode Compilation

NicyRuntime can compile Luau source files to bytecode (`.luauc`) for faster loading and source obfuscation.

## Compiling

### Via CLI

```bash
# Basic compilation (respects --!native, --!optimize from source)
nicy compile myscript.luau

# Creates: myscript.luauc (same directory, same base name)
```

### Via FFI

```c
#include "NicyRuntime.h"

int main() {
    nicy_compile("myscript.luau");
    return 0;
}
```

## Compiler Configuration

The `compile` command does **not** accept CLI flags like `--native`, `--optimize`, or `--output`. All configuration is done via **in-source compiler directives**:

```luau
-- myscript.luau
--!native
--!optimize 2

local function hot_function()
    -- This will be compiled to native code with optimization level 2
end
```

### Supported Directives

| Directive | Description |
|-----------|-------------|
| `--!native` | Enable CodeGen/JIT for this file |
| `--!optimize N` | Set optimization level (0-2, default: 1) |
| `--!coverage` | Enable coverage tracking |
| `--!profile` | Enable profiling |
| `--!typeinfo N` | Enable type info generation (0-1) |

## Running Bytecode

Bytecode files are executed the same way as source files:

```bash
# Compile
nicy compile game.luau

# Run the bytecode
nicy run game.luauc
```

The runtime automatically detects the `.luauc` extension and loads the bytecode directly.

## Bytecode Priority

When requiring a module, NicyRuntime checks for bytecode first:

1. `module.luauc` — bytecode (fastest loading)
2. `module.luau` — source
3. `module.lua` — source

This means you can distribute bytecode alongside source, and the runtime will prefer the compiled version.

## Portability

### Source Files (`.luau`, `.lua`)
- ✅ Portable across platforms
- ✅ Portable across Luau versions

### Bytecode Files (`.luauc`)
- ❌ **Not portable** across Luau versions
- ❌ May not be portable across architectures (especially with `--!native`)

Always recompile bytecode when updating NicyRuntime.

## Native Bytecode

When compiled with `--!native`, the bytecode includes native machine code for the target architecture.

### Benefits
- ⚡ Faster execution (hot paths run as native code)
- 🔒 Source code not included

### Drawbacks
- 📦 Larger file size
- 🏗️ Architecture-specific (x64 bytecode won't work on ARM)
