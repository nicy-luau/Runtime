# Custom Modules

NicyRuntime provides a powerful custom `require()` implementation with caching, aliases, and circular dependency detection.

## Basic Usage

**`math_utils.luau`**:
```luau
local MathUtils = {}

function MathUtils.add(a, b)
    return a + b
end

function MathUtils.multiply(a, b)
    return a * b
end

return MathUtils
```

**`main.luau`**:
```luau
local MathUtils = require("math_utils")
print(MathUtils.add(10, 20))  -- 30
```

## Module Resolution Order

When you call `require("module")`, NicyRuntime searches for files in this order:

1. `module.luauc` (compiled bytecode — fastest)
2. `module.luau` (Luau source)
3. `module.lua` (Lua source)
4. `module/init.luauc`
5. `module/init.luau`
6. `module/init.lua`

## Relative Paths

Modules can be required using relative paths:

```luau
-- From: /project/src/main.luau
local utils = require("./utils/math_utils")  -- /project/src/utils/math_utils.luau
local core = require("../lib/core")          -- /project/lib/core.luau
```

## Absolute Paths

```luau
local config = require("/etc/myapp/config")
```

## Module Caching

Loaded modules are **cached** by their resolved file path. Subsequent `require()` calls for the same module return the cached result without re-loading.

### Cache Invalidation

The cache uses **file fingerprints** based on:
- **Modification time** (`mtime`) — nanosecond precision
- **File size** — in bytes

A cached module is reloaded when **either** the modification time **or** the file size changes. This provides fast cache validation without the overhead of computing content hashes.

### ⚠️ Cache Limitations

**False positive scenario**: The cache uses `mtime + size` fingerprinting, NOT content hashing. This means:

1. If a file is edited and reverted to different content with the same size
2. AND the filesystem preserves the original `mtime` (e.g., via `touch -t` on Linux, or certain backup/restore tools)
3. The runtime will serve the **stale cached version** instead of reloading the file

**Example of cache miss**:
```bash
# File has content "A" at time T1
echo "print('version A')" > module.luau
require("module")  -- Loads and caches version A

# File is edited to content "B" (same size), mtime is artificially reset to T1
echo "print('version B')" > module.luau
touch -t 202604150000 module.luau  -- Resets mtime to original time
require("module")  -- ❌ Still returns cached version A!
```

**When this matters**:
- Automated backup/restore tools that preserve timestamps
- Git operations that restore files with original timestamps
- Manual `touch` commands that reset modification times
- File synchronization tools (rsync, robocopy) with timestamp preservation

**When this is NOT a problem**:
- Normal development workflow (editing files changes mtime)
- Build pipelines that copy/regenerate files (mtime changes)
- Production deployments (files are static)

### Workarounds

If you need robust cache invalidation:

1. **Manual cache clear**: Restart the runtime (clears all cached modules)
2. **Future feature**: `robust-cache` feature flag using SHA-256 content hashing (planned)
3. **Development workflow**: Always save files normally (don't manipulate timestamps)

### Cache Scope

- Cache is **per-runtime-instance**. Each `nicy_start()` call creates a fresh cache.
- Cache is **not shared** between separate runtime invocations.
- Cache entries are automatically cleaned up when the runtime shuts down.

## Circular Dependencies

NicyRuntime detects circular dependencies and throws a clear error:

```luau
-- a.luau
local B = require("b")  -- Error: Cyclic require detected: a -> b -> a
```

Error output:
```
Error: Cyclic require detected
  require chain:
    a.luau
    → b.luau
    → a.luau (circular)
```

## Concurrent Loading

If a module is already being loaded by another coroutine, `require()` will yield and wait for the loading to complete. This prevents duplicate loading of the same module.

## The `@self` Alias

`@self` refers to the directory of the current script:

```luau
-- From: /project/src/main.luau
local utils = require("@self/utils/math")  -- /project/src/utils/math.luau
```

This is especially useful in libraries where you want to require sibling modules without knowing the absolute path.

## See Also

- [`.luaurc` Aliases](luaurc-aliases.md)
- [Bytecode Compilation](bytecode-compilation.md)
