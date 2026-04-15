# Nicy Runtime

A blazing-fast Luau runtime environment with a modular architecture. Built in Rust.

## Overview

**Nicy Runtime** is a high-performance runtime environment for Luau scripts, structured as a modular Rust workspace:

- **Nicy** — Command-line interface (CLI) host that loads and executes scripts.
- **Runtime** (`nicyruntime`) — Dynamic library (`cdylib`) that provides the full Luau runtime environment.

## Project Structure

```
C:\NicyRuntime\
├── Cargo.toml              # Workspace root
├── Nicy/                   # CLI host
│   ├── Cargo.toml
│   └── src/main.rs
├── Runtime/                # Runtime cdylib (nicyruntime)
│   ├── Cargo.toml
│   └── src/lib.rs
├── libs/
│   └── libunwind.a
├── build.ps1
├── README.md
└── LICENSE
```

## Features

- **Dynamic Library Architecture** — The runtime (`nicyruntime`) is a reusable `cdylib` that can be loaded by any host application.
- **Native Code Integration** — Load native shared libraries directly from Luau via `runtime.loadlib()`.
- **Custom Module Resolver** — Sophisticated `require()` with module caching, file fingerprinting, `.luaurc` alias support, and circular dependency detection.
- **Asynchronous Task Scheduler** — Cooperative multitasking with `task.spawn`, `task.defer`, `task.delay`, `task.wait`, and `task.cancel`.
- **Cross-Platform** — Builds for Windows, macOS, Linux, and Android (Termux).
- **Luau CodeGen/JIT** — Add `--!native` on the first line of a file to enable native bytecode compilation (disabled on Android for stability).
- **{{FFI_COUNT}} FFI Functions** — Complete C-ABI API for embedding in any language (C, C++, Python, C#, Node.js, Rust via libloading), plus error code utilities.

## Luau API

### `runtime` object

- `runtime.version` — Version of the runtime library.
- `runtime.hasJIT(path?: string)` — Returns `true` if JIT/CodeGen is active.
- `runtime.entry_file` — Path to the main script being executed.
- `runtime.entry_dir` — Directory of the main script.
- `runtime.loadlib(path: string)` — Loads a dynamic library (supports `@self` alias).

### `task` library

- `task.spawn(f, ...)` — Spawns a new coroutine.
- `task.defer(f, ...)` — Defers coroutine execution.
- `task.delay(seconds, f, ...)` — Spawns a coroutine after a delay.
- `task.wait(seconds)` — Pauses the current coroutine.
- `task.cancel(thread|delay_id)` — Cancels a running task.

### `_VERSION` string

Returns the current Luau version (e.g., `0.709`).

## Usage

### CLI Commands

```powershell
nicy run script.luau          # Execute a script
nicy eval "print('hello')"    # Evaluate code inline
nicy compile script.luau      # Compile to bytecode (.luauc)
nicy help                     # Show help
nicy version                  # Show CLI version
nicy runtime-version          # Show engine and Luau versions
```

### FFI C-ABI Exports

The `nicyruntime` library exposes these `extern "C"` functions for embedding in any language:

| Function | Description |
|---|---|
| `nicy_start(filepath)` | Initialize runtime and execute the script |
| `nicy_eval(code)` | Evaluate raw Luau code in an isolated state |
| `nicy_compile(filepath)` | Compile source to bytecode (`.luauc`) |
| `nicy_version()` | Return runtime version string |
| `nicy_luau_version()` | Return Luau version string |

Plus full Lua C API wrappers (`nicy_lua_*`, `nicy_luaL_*`).

## Build

```powershell
# Build for current platform
./build.ps1

# Build for all targets
./build.ps1 -target all

# Force rebuild
./build.ps1 -force
```

## Embedding nicyruntime in Other Projects

The `nicyruntime` library is a `cdylib` designed for **dynamic loading via FFI** from any language that supports calling native libraries.

### Supported Languages

| Language | Method | Example |
|----------|--------|---------|
| C/C++ | `dlopen` / `LoadLibrary` | See [Embedding in C](Docs/src/guides/embedding-c.md) |
| Rust | `libloading` | See [Embedding in Rust](Docs/src/guides/embedding-rust.md) |
| Python | `ctypes` / `cffi` | Load `nicyruntime.dll` / `libnicyruntime.so` |
| C# | `DllImport` / `NativeLibrary` | Use `[DllImport("nicyruntime")]` |
| Node.js | `ffi-napi` | Load via `ffi.Library('nicyruntime', ...)` |

### Quick Example (Rust with libloading)

```rust
use libloading::Library;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lib = Library::new("nicyruntime.dll")?;
    
    unsafe {
        let nicy_start: libloading::Symbol<
            unsafe extern "C" fn(*const std::os::raw::c_char)
        > = lib.get(b"nicy_start")?;

        let script = std::ffi::CString::new("myscript.luau")?;
        nicy_start(script.as_ptr());
    }
    
    Ok(())
}
```

> ⚠️ **Important**: Do NOT add `nicyruntime` as a direct Cargo dependency.
> The library uses `crate-type = ["cdylib"]`, which is incompatible with Rust's static linkage.
> Always load it dynamically via `libloading`, `dlopen`, or equivalent.

## License

This project is licensed under the Mozilla Public License 2.0. See the [LICENSE](LICENSE) file for details.
