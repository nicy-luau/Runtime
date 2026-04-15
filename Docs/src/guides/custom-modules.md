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

The cache is invalidated when:
- The file's modification time changes
- The file size changes

This means you can edit a module during development and the changes will be picked up on the next `require()`.

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
