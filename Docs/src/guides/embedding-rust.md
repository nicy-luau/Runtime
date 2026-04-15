# Embedding in Rust

Embed NicyRuntime in a Rust application using FFI.

> ⚠️ **Important**: `nicyruntime` is a `cdylib`, NOT an `rlib`. You have two options:
> 1. **Dynamic loading** (recommended): Use `libloading` to load the library at runtime
> 2. **Static linking**: Use `#[link(name = "nicyruntime")]` with the library in your library path
>
> Do NOT add `nicyruntime` as a Cargo dependency — this will fail with a crate type error.

## Prerequisites

- Rust 1.75+
- `nicyruntime` shared library in the library search path

## Project Setup

**`Cargo.toml`**:
```toml
[package]
name = "my-host-app"
version = "0.1.0"
edition = "2021"

[dependencies]
libc = "0.2"
```

## Basic Example

**`src/main.rs`**:
```rust
use std::ffi::{CStr, CString};

#[link(name = "nicyruntime")]
extern "C" {
    fn nicy_start(file: *const std::os::raw::c_char);
    fn nicy_eval(code: *const std::os::raw::c_char);
    fn nicy_compile(path: *const std::os::raw::c_char);
    fn nicy_version() -> *const std::os::raw::c_char;
    fn nicy_luau_version() -> *const std::os::raw::c_char;
}

fn main() {
    // Print version info
    let runtime_version = unsafe {
        CStr::from_ptr(nicy_version()).to_string_lossy().into_owned()
    };
    let luau_version = unsafe {
        CStr::from_ptr(nicy_luau_version()).to_string_lossy().into_owned()
    };

    println!("NicyRuntime {}", runtime_version);
    println!("Powered by {}", luau_version);

    // Run a script
    let script = CString::new("myscript.luau").unwrap();
    unsafe { nicy_start(script.as_ptr()) };
}
```

## Safe Wrapper

```rust
use std::ffi::{CStr, CString};

pub fn run_script(path: &str) -> Result<(), String> {
    let c_path = CString::new(path).map_err(|_| "Invalid path")?;
    unsafe { nicy_start(c_path.as_ptr()) };
    Ok(())
}

pub fn eval(code: &str) -> Result<(), String> {
    let c_code = CString::new(code).map_err(|_| "Invalid code")?;
    unsafe { nicy_eval(c_code.as_ptr()) };
    Ok(())
}

pub fn compile(path: &str) -> Result<(), String> {
    let c_path = CString::new(path).map_err(|_| "Invalid path")?;
    unsafe { nicy_compile(c_path.as_ptr()) };
    Ok(())
}

pub fn version() -> String {
    unsafe {
        CStr::from_ptr(nicy_version())
            .to_string_lossy()
            .into_owned()
    }
}
```

## Dynamic Loading

For maximum flexibility, load the runtime dynamically:

```rust
use libloading::Library;
use std::ffi::CString;

fn main() {
    let lib = Library::new("nicyruntime.dll").expect("Failed to load");

    unsafe {
        let nicy_start: libloading::Symbol<
            unsafe extern "C" fn(*const std::os::raw::c_char)
        > = lib.get(b"nicy_start").expect("Symbol not found");

        let script = CString::new("myscript.luau").unwrap();
        nicy_start(script.as_ptr());
    }
}
```

## See Also

- [FFI Reference](ffi-reference/index.md)
- [Embedding in C](embedding-c.md)
