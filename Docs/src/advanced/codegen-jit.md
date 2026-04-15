# CodeGen / JIT

Luau CodeGen (Code Generation) compiles hot Luau functions to native machine code at runtime, providing near-native performance.

## How It Works

1. Luau code is initially interpreted by the VM
2. The CodeGen profiler identifies "hot" functions (frequently executed)
3. Hot functions are compiled to native machine code via LLVM
4. Subsequent calls execute the native code directly

## Enabling CodeGen

### Via CLI

```bash
nicy compile myscript.luau --native
```

### Via Compiler Directive

```luau
--!native

local function hot_function()
    -- This will be compiled to native code
end
```

### Via FFI

```c
nicy_compile("myscript.luau", "myscript.luauc", 1, 1);
// native=1
```

## Checking Availability

```luau
if runtime.hasJIT then
    print("CodeGen is available!")
else
    print("Running in interpreted mode only")
end
```

## Platform Support

| Platform | Architecture | CodeGen |
|----------|-------------|---------|
| Windows | x64 | ✅ Yes |
| Windows | x86 | ✅ Yes |
| Windows | ARM64 | ✅ Yes |
| macOS | x64 | ✅ Yes |
| macOS | ARM64 | ✅ Yes |
| Linux | x64 | ✅ Yes |
| Linux | ARM64 | ✅ Yes |
| Linux | x86 | ✅ Yes (no vector4) |
| Android | ARM64 | ❌ Disabled |
| Android | ARMv7 | ❌ Disabled |

CodeGen is disabled on Android for stability reasons.

## Performance Impact

CodeGen can provide **2-10x** speedup for compute-intensive code:

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

print(string.format("fibonacci(35) = %d", result))
print(string.format("Time: %.4f seconds", elapsed))
```

Typical results:
- Interpreted: ~2.5 seconds
- With CodeGen: ~0.3 seconds (~8x faster)

## Limitations

- Not all Luau constructs can be compiled to native code
- Fallback to interpreter for unsupported operations
- Native code increases bytecode file size
- Native bytecode is architecture-specific

## Optimization Levels

Combine CodeGen with optimization levels:

```luau
--!native
--!optimize 2  -- Aggressive optimization
```

| Level | Description |
|-------|-------------|
| 0 | No optimization (debugging) |
| 1 | Default (balanced) |
| 2 | Aggressive (production) |

## See Also

- [Compiler Directives](../runtime-api/compiler-directives.md)
- [Bytecode Compilation](../guides/bytecode-compilation.md)
