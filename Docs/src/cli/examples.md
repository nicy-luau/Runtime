# Examples

Common usage patterns for the NicyRuntime CLI.

## Basic Script Execution

### Run a Simple Script

```bash
nicy run hello.luau
```

### Evaluate Inline Code

```bash
nicy eval "print(42)"
```

## Compilation

### Compile and Run

```bash
# Compile (respects --!native, --!optimize from source)
nicy compile game.luau

# Run the bytecode
nicy run game.luauc
```

## Module System

### Simple Module

**`math_utils.luau`**:
```luau
return {
    add = function(a, b) return a + b end,
    mul = function(a, b) return a * b end,
}
```

**`main.luau`**:
```luau
local utils = require("math_utils")
print(utils.add(10, 20))
```

```bash
nicy run main.luau
```

## Async Tasks

### Concurrent Execution

```luau
for i = 1, 5 do
    task.spawn(function()
        print("Task " .. i .. " starting")
        task.wait(math.random() * 2)
        print("Task " .. i .. " done")
    end)
end

task.wait(3)
print("All done!")
```

### Delayed Execution

```luau
local id = task.delay(2.0, function()
    print("2 seconds elapsed!")
end)

task.wait(1.0)
task.cancel(id)
print("Cancelled before firing")
```

## Error Handling

```bash
# Concise error (default)
nicy run broken.luau

# Verbose error
NICY_VERBOSE_ERRORS=1 nicy run broken.luau

# No colors
NICY_NO_COLOR=1 nicy run broken.luau
```
