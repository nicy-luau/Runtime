# Project Structure

Understanding the NicyRuntime project layout.

## Workspace Overview

NicyRuntime is a **Cargo workspace** with two crates:

```
NicyRuntime/
├── Cargo.toml              # Workspace root
├── Runtime/                # Core library (cdylib)
│   ├── Cargo.toml
│   ├── NicyRuntime.h       # C header for embedding
│   ├── README.md
│   ├── libs/
│   │   └── libunwind.a     # Static unwind library
│   ├── src/
│   │   ├── lib.rs              # Main entry point
│   │   ├── require_resolver.rs # Module resolver
│   │   ├── task_scheduler.rs   # Async scheduler
│   │   ├── ffi_exports.rs      # C-ABI exports
│   │   └── error.rs            # Error system
│   └── tests/              # Luau test suite
├── Nicy/                   # CLI executable
│   ├── Cargo.toml
│   ├── README.md
│   ├── libs/
│   │   └── libunwind.a
│   └── src/
│       └── main.rs         # CLI entry point
├── build.ps1               # Build script
├── Docs/                   # Documentation site
└── .github/workflows/      # CI/CD
```

## Runtime Crate (`Runtime/`)

The **core library** — a `cdylib` (dynamic library) that contains:

### Source Files

| File | Lines | Purpose |
|------|-------|---------|
| `lib.rs` | ~1,250 | Main entry, Luau state init, FFI functions, OS extensions |
| `require_resolver.rs` | ~1,205 | Custom `require()` with caching, aliases, circular detection |
| `task_scheduler.rs` | ~782 | Cooperative async scheduler with coroutines |
| `ffi_exports.rs` | ~522 | 70+ Lua C API wrappers with stable C-ABI |
| `error.rs` | ~1,997 | Error reporting (concise/verbose modes) |

### Key Features

- **`nicy_start()`** — Initialize runtime and execute a script file
- **`nicy_eval()`** — Evaluate inline code in an isolated state
- **`nicy_compile()`** — Compile source to `.luauc` bytecode
- **`nicy_version()`** — Return runtime version string
- **70+ `nicy_lua_*` functions** — Full Lua C API exposed via FFI

## Nicy Crate (`Nicy/`)

The **CLI executable** — a minimal wrapper (~361 lines) that:

1. Dynamically loads `nicyruntime` via `libloading`
2. Routes commands (`run`, `eval`, `compile`)
3. Handles argument parsing and output

### Why Dynamic Loading?

The CLI doesn't link against the runtime at compile time. Instead, it:

- Discovers the runtime library at runtime
- Allows swapping runtime versions without recompiling the CLI
- Demonstrates the embedding pattern for end users

## Build Artifacts

After building, you'll find:

```
target/
├── release/
│   ├── nicy                  # CLI executable (or nicy.exe)
│   └── libnicyruntime.so     # Runtime library (.dll / .dylib)
```

## Test Suite

Located in `Runtime/tests/` — 32 Luau files covering:

| Category | Files | Tests |
|----------|-------|-------|
| Core API | 11 | stdlib, bit32, buffers, GC, IO, metatables, vectors, etc. |
| Require System | 6 + fixtures | aliases, bytecode, circular deps, concurrent loading |
| Runtime | 6 | debug, error handler, globals, shutdown, traceback |
| Task Scheduler | 7 | spawn, defer, delay, wait, cancel, stress tests |

Run all tests:
```bash
nicy run Runtime/tests/run_all.luau
```

## Configuration Files

### `Cargo.toml` (workspace)

```toml
[workspace]
members = ["Nicy", "Runtime"]
resolver = "2"

[profile.release]
strip = "symbols"
lto = true
codegen-units = 1
opt-level = "z"  # Optimize for size
```

### `Runtime/Cargo.toml` (dependencies)

```toml
[dependencies]
libloading = "0.9"

[target.'cfg(not(target_os = "android"))'.dependencies]
mlua-sys = { version = "0.10.0", features = ["luau", "luau-codegen", "luau-vector4", "vendored"] }
```

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `NICY_VERBOSE_ERRORS=1` | Enable verbose error output |
| `NICY_NO_COLOR=1` | Disable ANSI colors in error messages |
| `NICY_HIRES_TIMER=1` | Enable high-resolution timer on Windows |
