# Quick Start

Let's run your first Luau script with NicyRuntime.

## Hello, Luau!

Create a file called `hello.luau`:

```luau
-- hello.luau
print("Hello from NicyRuntime!")
print("Luau version: " .. _VERSION)
print("Runtime: " .. runtime.version)

-- Check if JIT is available
if runtime.hasJIT() then
    print("CodeGen/JIT is enable")
else
    print("Running in interpreted mode")
end
```

Run it:

```bash
nicy run hello.luau
```

Output:
```
Hello from NicyRuntime!
Luau version: Luau
Runtime: 1.0.0-alpha
CodeGen/JIT is enabled! ⚡
```

## Evaluate Inline Code

No need to create a file for quick tests:

```bash
nicy eval 'print("Hello from CLI!")'
```

## Working with Modules

Create two files to see the module resolver in action:

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
-- main.luau
local MathUtils = require("math_utils")

local sum = MathUtils.add(10, 20)
local product = MathUtils.multiply(5, 6)

print("10 + 20 = " .. sum)
print("5 * 6 = " .. product)
```

Run:

```bash
nicy run main.luau
```

Output:
```
10 + 20 = 30
5 * 6 = 30
```

## Compile to Bytecode

Compile your script to `.luauc` bytecode for faster loading and obfuscation:

```bash
nicy compile hello.luau
# Creates hello.luauc

# Run the bytecode directly
nicy run hello.luauc
```

## Using Native Compiler Directives

Enable Luau's native code generation for specific functions:

**`fast_math.luau`**:
```luau
--!native
--!optimize 2

local function fibonacci(n)
    if n <= 1 then return n end
    return fibonacci(n - 1) + fibonacci(n - 2)
end

local start = os.clock()
local result = fibonacci(35)
local elapsed = os.clock() - start

print("fibonacci(35) = " .. result)
print("Time: " .. string.format("%.4f", elapsed) .. "s")
```

Run with native compilation:
```bash
nicy run fast_math.luau
```

## Task Scheduler Demo

Try the async task scheduler:

**`tasks.luau`**:
```luau
-- Spawn concurrent tasks
task.spawn(function()
    for i = 1, 3 do
        print("Task A: " .. i)
        task.wait(0.5)
    end
end)

task.spawn(function()
    for i = 1, 3 do
        print("Task B: " .. i)
        task.wait(0.3)
    end
end)

-- Delayed execution
task.delay(1.0, function()
    print("This runs after 1 second!")
end)

-- Wait in the main thread (non-blocking)
print("Main thread waiting...")
task.wait(2.0)
print("Done!")
```

Run:
```bash
nicy run tasks.luau
```

Output:
```
Main thread waiting...
Task A: 1
Task B: 1
Task B: 2
Task A: 2
Task B: 3
This runs after 1 second!
Task A: 3
Done!
```

## What's Next?

- Learn about [all CLI commands](../cli/commands)
- Explore the [Runtime API](../runtime-api/nicy-start) for embedding
- Read the [Custom Modules guide](../guides/custom-modules) for advanced `require()` usage
