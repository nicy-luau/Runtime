# .luaurc Aliases

The `.luaurc` file allows you to define module aliases for cleaner require paths.

## Creating a .luaurc File

Place a `.luaurc` file in your project root:

**`.luaurc`**:
```json
{
    "aliases": {
        "@modules": "src/modules",
        "@lib": "lib",
        "@config": "config"
    }
}
```

## Using Aliases

```luau
-- Instead of:
local myModule = require("src/modules/myModule")

-- You can write:
local myModule = require("@modules/myModule")
```

## Alias Resolution

Aliases are resolved relative to the directory containing the `.luaurc` file:

```
project/
├── .luaurc          ← aliases defined here
├── src/
│   ├── modules/
│   │   └── myModule.luau
│   └── main.luau
└── lib/
    └── utils.luau
```

**`main.luau`**:
```luau
local myModule = require("@modules/myModule")  -- src/modules/myModule.luau
local utils = require("@lib/utils")            -- lib/utils.luau
```

## Directory Inheritance

`.luaurc` files are inherited up the directory tree. If a `.luaurc` is not found in the current directory, NicyRuntime searches parent directories.

```
project/
├── .luaurc              ← global aliases
├── src/
│   ├── .luaurc          ← src-specific aliases (merged with parent)
│   └── main.luau
```

**`project/.luaurc`**:
```json
{
    "aliases": {
        "@lib": "lib"
    }
}
```

**`project/src/.luaurc`**:
```json
{
    "aliases": {
        "@components": "components"
    }
}
```

**`project/src/main.luau`**:
```luau
-- Both aliases are available:
local utils = require("@lib/utils")       -- from parent .luaurc
local comp = require("@components/button") -- from local .luaurc
```

## Multiple .luaurc Files

When multiple `.luaurc` files exist in the directory tree:
- Child aliases take precedence over parent aliases
- Aliases are merged (not replaced)

## JSON Format

The `.luaurc` file must be valid JSON. Only the `aliases` key is currently supported:

```json
{
    "aliases": {
        "alias_name": "path/to/directory"
    }
}
```

## The `@self` Alias

`@self` is a built-in alias that always refers to the directory of the current script. It does not need to be defined in `.luaurc`.

## See Also

- [Custom Modules](custom-modules.md)
