# NicyRuntime

**A blazing-fast Luau runtime environment with a modular architecture. Built in Rust.**

NicyRuntime is a high-performance runtime for executing [Luau](https://luau-lang.org/) scripts — Roblox's gradual typing dialect of Lua. It's designed for embedding in applications, game engines, or any system that needs a fast, sandboxable scripting layer.

## Why NicyRuntime?

### ⚡ Performance

Built entirely in **Rust**, NicyRuntime leverages the safety and speed of systems-level programming. With optional **Luau CodeGen/JIT** support, your scripts compile to native machine code at runtime for maximum performance.

### 🔌 Dynamic Library Architecture

The core runtime is a **`cdylib`** (dynamic library) that can be loaded at runtime by any host application. This means:

- **Zero coupling**: Your host app doesn't need to link against the runtime at compile time
- **Hot-reloadable**: Update the runtime without recompiling your host
- **Language agnostic**: Embed from C, C++, Rust, Python, Node.js — anything with C FFI support

### 📦 Custom Module Resolver

A sophisticated `require()` implementation with:

- **Smart caching** based on file fingerprint (mtime + size)
- **`.luaurc` alias support** with directory tree inheritance
- **Circular dependency detection** with clear error messages
- **Concurrent loading** support with cooperative yielding
- **Bytecode priority** (`.luauc` > `.luau` > `.lua`)

### 🔄 Async Task Scheduler

Cooperative multitasking built on Luau coroutines:

```luau
task.spawn(function()
    print("Running concurrently!")
end)

local id = task.delay(2.0, function()
    print("Delayed execution")
end)

task.wait(1.0)  -- Non-blocking wait

task.cancel(id) -- Cancel a delayed task
```

### 🌍 Cross-Platform

Pre-built binaries for every major platform:

| Platform | Architecture | Status |
|----------|-------------|--------|
| Windows | x64, x86, ARM64 | ✅ Stable |
| macOS | x64, ARM64 (Apple Silicon) | ✅ Stable |
| Linux | x64, ARM64 | ✅ Stable |
| Android | ARM64, ARMv7 | ✅ Stable |

### 🛡️ Robust Error Handling

- **Concise errors** by default — clean, readable output
- **Verbose mode** via `NICY_VERBOSE_ERRORS=1` — full stack traces with require chain tracking
- **SEH crash protection** on Windows for `runtime.loadlib()`
- **Memory safety** with complete static state cleanup between runtime calls

## Architecture Overview

```
┌─────────────────────────────────────────────────┐
│                  Host Application                 │
│              (C / C++ / Rust / etc.)             │
├─────────────────────────────────────────────────┤
│              nicyruntime (cdylib)                 │
│  ┌─────────────┬─────────────┬─────────────────┐  │
│  │  Luau VM    │  Module     │  Task           │  │
│  │  (mlua-sys) │  Resolver   │  Scheduler      │  │
│  │             │             │                 │  │
│  │  CodeGen    │  Cache      │  Coroutines     │  │
│  │  (JIT)      │  System     │  (async)        │  │
│  └─────────────┴─────────────┴─────────────────┘  │
├─────────────────────────────────────────────────┤
│                  nicy (CLI)                       │
│            (Dynamic loader + router)              │
└─────────────────────────────────────────────────┘
```

## Project Structure

```
NicyRuntime/
├── Runtime/              # Core cdylib library
│   ├── src/
│   │   ├── lib.rs                # Main entry point, FFI functions
│   │   ├── require_resolver.rs   # Custom module resolver
│   │   ├── task_scheduler.rs     # Async task scheduler
│   │   ├── ffi_exports.rs        # 70+ C-ABI Lua API wrappers
│   │   └── error.rs              # Error reporting system
│   └── tests/            # Luau test suite (32 files)
├── Nicy/                 # CLI executable
│   └── src/main.rs       # Dynamic loading & command routing
├── build.ps1             # Multi-platform build script
└── Docs/                 # This documentation site
```

## Quick Links

- **[Getting Started](getting-started/installation)** — Install and run your first script
- **[CLI Reference](cli/commands)** — All `nicy` commands and flags
- **[Runtime API](runtime-api/nicy-start)** — FFI functions for embedding
- **[FFI Reference](ffi-reference/overview)** — Complete Lua C API wrapper docs
- **[Guides](guides/embedding-c)** — Practical tutorials

## License

NicyRuntime is licensed under the **Mozilla Public License Version 2.0** (MPL 2.0).

[Source Code](https://github.com/nicy-luau/Runtime) · [Releases](https://github.com/nicy-luau/Runtime/releases) · [Report Bug](https://github.com/nicy-luau/Runtime/issues)
