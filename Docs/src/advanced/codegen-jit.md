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

### Feature Flags by Platform

NicyRuntime uses platform-specific feature flags to ensure ABI compatibility and stability. The following table shows which Luau features are enabled on each platform:

| Platform | Architecture | CodeGen/JIT | Vector4 | Notes |
|----------|-------------|-------------|---------|-------|
| Windows | x64 | ✅ | ✅ | Full support |
| Windows | ARM64 | ✅ | ✅ | Full support |
| Windows | x86 | ✅ | ✅ | Full support |
| macOS | x64 | ✅ | ✅ | Full support |
| macOS | ARM64 | ✅ | ✅ | Full support |
| Linux | x64 | ✅ | ✅ | Full support |
| Linux | ARM64 | ✅ | ✅ | Full support |
| Linux | x86 (32-bit) | ✅ | ❌ | TValue ABI mismatch — `lua_TValue` size differs on i686 |
| Android | ARM64 | ❌ | ❌ | Disabled for stability — JIT can cause crashes on some devices |
| Android | ARMv7 | ❌ | ❌ | Disabled for stability |

### Why These Differences?

**Vector4 (Linux x86)**: The `luau-vector4` feature is disabled on 32-bit Linux because of a static assertion failure: `sizeof(lua_TValue) == 24`. On i686, the TValue structure has a different size, causing ABI incompatibility with Luau bytecode compiled on other platforms.

**CodeGen (Android)**: The `luau-codegen` feature is disabled on Android because LLVM JIT compilation can cause instability on certain Android devices, especially those with SELinux restrictions or limited memory.

### What This Means for You

- **`--!native` directive**: On platforms without CodeGen, this directive is silently ignored. Use `runtime.hasJIT(path)` to check if JIT is active for a specific file.
- **Cross-platform bytecode**: Bytecode compiled with `--!native` on x64 will NOT run on x86 or ARM due to architecture-specific machine code.
- **Vector4**: If your code uses `vector4` type, it will NOT work on Linux x86 (32-bit). Test on your target platform.

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
