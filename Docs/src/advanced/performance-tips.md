# Performance Tips

Maximize the performance of your Luau scripts on NicyRuntime.

## 1. Enable CodeGen/JIT

The single biggest performance improvement:

```luau
--!native
--!optimize 2

-- Your compute-intensive code here
```

Or compile with native code generation:

```bash
nicy compile myscript.luau --native --optimize 2
```

See [CodeGen/JIT](codegen-jit.md) for details.

## 2. Use Local Variables

Local variables are significantly faster than globals:

```luau
-- Slow: Global access
function compute()
    math.sin(x)  -- Global lookup each call
end

-- Fast: Local reference
local sin = math.sin
function compute()
    sin(x)  -- Local access
end
```

## 3. Avoid Table Allocations in Loops

```luau
-- Bad: Creates a new table every iteration
for i = 1, 1000000 do
    local t = {x = i, y = i * 2}
    process(t)
end

-- Good: Reuse a single table
local t = {}
for i = 1, 1000000 do
    t.x = i
    t.y = i * 2
    process(t)
end
```

## 4. Use Integer Arithmetic When Possible

```luau
-- Slower: Floating point
local x = 10.0 + 20.0

-- Faster: Integer
local x = 10 + 20
```

Luau distinguishes between integers and floats internally. Integer operations are faster.

## 5. Minimize Function Call Overhead

```luau
-- Bad: Function call in tight loop
for i = 1, 1000000 do
    result = add(result, i)
end

-- Good: Inline the operation
for i = 1, 1000000 do
    result = result + i
end
```

## 6. Use `rawget`/`rawset` for Performance-Critical Access

```luau
-- With metamethod lookup
local value = table.key

-- Without metamethod lookup (faster)
local value = rawget(table, "key")
```

## 7. Pre-allocate Tables

```luau
-- Bad: Table grows dynamically
local t = {}
for i = 1, 1000000 do
    t[i] = i
end

-- Good: Pre-allocate
local t = table.create(1000000)
for i = 1, 1000000 do
    t[i] = i
end
```

## 8. Use `task.defer` for Non-Urgent Work

```luau
-- Spawn heavy work to run after current frame
task.defer(function()
    heavyComputation()
end)

-- Continue with urgent work
urgentWork()
```

## 9. Profile Your Code

```luau
--!profile

local function slowFunction()
    -- This will be profiled
end
```

Or use `os.clock()` for manual profiling:

```luau
local start = os.clock()
myFunction()
local elapsed = os.clock() - start
print(string.format("myFunction took %.6f seconds", elapsed))
```

## 10. Enable High-Resolution Timer (Windows)

For accurate profiling on Windows:

```bash
NICY_HIRES_TIMER=1 nicy run profile.luau
```

## Benchmark Example

```luau
--!native
--!optimize 2

local function benchmark(name, fn, iterations)
    local start = os.clock()
    for i = 1, iterations do
        fn()
    end
    local elapsed = os.clock() - start
    print(string.format("%s: %.4f ms/op", name, elapsed / iterations * 1000))
end

-- Compare approaches
benchmark("global math.sin", function()
    math.sin(1.5)
end, 1000000)

local sin = math.sin
benchmark("local sin", function()
    sin(1.5)
end, 1000000)
```

## See Also

- [CodeGen/JIT](codegen-jit.md)
- [Memory Management](memory-management.md)
