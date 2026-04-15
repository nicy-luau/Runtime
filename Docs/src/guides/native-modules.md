# Native Modules (Bare Metal)

NicyRuntime supports loading **native modules** at runtime via `runtime.loadlib()`. Write performance-critical code in C, C++, Rust, Zig, or any language with C interop, and expose it directly to Luau scripts.

## Overview

When you call `runtime.loadlib("mylib.dll")`, NicyRuntime:

1. Loads the dynamic library (`.dll`, `.so`, `.dylib`)
2. Looks for the symbol `nicydynamic_init` (or `nicydinamic_init` as fallback)
3. Calls the init function, passing the current `lua_State*`
4. The init function registers C functions, types, and data on the Lua stack
5. Returns a table with the exported symbols

> 💡 **Note:** You can use either `NicyRuntime.h` (which bundles everything) or standard `lua.h` + `lauxlib.h`. Both work identically.

---

## C

### Basic Example

**`mylib.c`**:
```c
#include "NicyRuntime.h"

static int my_add(lua_State* L) {
    double a = luaL_checknumber(L, 1);
    double b = luaL_checknumber(L, 2);
    lua_pushnumber(L, a + b);
    return 1;
}

__declspec(dllexport) int nicydynamic_init(lua_State* L) {
    lua_createtable(L, 0, 1);
    lua_pushcfunction(L, my_add);
    lua_setfield(L, -2, "add");
    return 1;
}
```

### Compiling

**Windows (MSVC)**:
```cmd
cl /LD mylib.c /Fe:mylib.dll
```

**Linux/macOS (GCC/Clang)**:
```bash
gcc -shared -fPIC -o mylib.so mylib.c
```

---

## C++

### Example with std::string

**`string_ext.cpp`**:
```cpp
#include "NicyRuntime.h"
#include <string>
#include <algorithm>

static int string_reverse(lua_State* L) {
    const char* input = luaL_checkstring(L, 1);
    std::string s(input);
    std::reverse(s.begin(), s.end());
    lua_pushstring(L, s.c_str());
    return 1;
}

static int string_upper(lua_State* L) {
    const char* input = luaL_checkstring(L, 1);
    std::string s(input);
    std::transform(s.begin(), s.end(), s.begin(), ::toupper);
    lua_pushstring(L, s.c_str());
    return 1;
}

static int string_repeat(lua_State* L) {
    const char* input = luaL_checkstring(L, 1);
    int count = (int)luaL_checkinteger(L, 2);
    std::string result;
    for (int i = 0; i < count; i++) result += input;
    lua_pushstring(L, result.c_str());
    return 1;
}

extern "C" __declspec(dllexport) int nicydynamic_init(lua_State* L) {
    lua_createtable(L, 0, 3);

    lua_pushcfunction(L, string_reverse);
    lua_setfield(L, -2, "reverse");

    lua_pushcfunction(L, string_upper);
    lua_setfield(L, -2, "upper");

    lua_pushcfunction(L, string_repeat);
    lua_setfield(L, -2, "repeat");

    return 1;
}
```

### Compiling

**Windows (MSVC)**:
```cmd
cl /LD /EHsc string_ext.cpp /Fe:string_ext.dll
```

**Linux/macOS (GCC/Clang)**:
```bash
g++ -shared -fPIC -o string_ext.so string_ext.cpp
```

### Usage

```luau
local str = runtime.loadlib("@self/string_ext.dll")

print(str.reverse("hello"))     -- "olleh"
print(str.upper("hello"))       -- "HELLO"
print(str.repeat("ab", 3))      -- "ababab"
```

---

## Rust

### Example with JSON Parsing

**`Cargo.toml`**:
```toml
[package]
name = "json_ext"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
serde_json = "1.0"
libc = "0.2"
```

**`src/lib.rs`**:
```rust
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use serde_json::{Value, json};

// Type alias for the Lua state pointer
type LuaState = *mut c_void;

// Lua C API extern declarations
extern "C" {
    fn luaL_checkstring(l: LuaState, narg: c_int) -> *const c_char;
    fn luaL_checkinteger(l: LuaState, narg: c_int) -> i64;
    fn lua_pushstring(l: LuaState, s: *const c_char);
    fn lua_pushinteger(l: LuaState, n: i64);
    fn lua_createtable(l: LuaState, narr: c_int, nrec: c_int);
    fn lua_setfield(l: LuaState, idx: c_int, k: *const c_char);
    fn lua_pushcfunction(l: LuaState, f: extern "C" fn(LuaState) -> c_int);
}

static JSON_PARSE_SYMBOL: &[u8] = b"json_parse\0";
static JSON_STRINGIFY_SYMBOL: &[u8] = b"json_stringify\0";
static JSON_VERSION_SYMBOL: &[u8] = b"version\0";

extern "C" fn json_parse(l: LuaState) -> c_int {
    let input = unsafe { CStr::from_ptr(luaL_checkstring(l, 1)) };
    let input_str = match input.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    match serde_json::from_str::<Value>(input_str) {
        Ok(value) => {
            let result = serde_json::to_string(&value).unwrap_or_default();
            let c_result = CString::new(result).unwrap();
            unsafe { lua_pushstring(l, c_result.as_ptr()) };
            1
        }
        Err(e) => {
            let err = CString::new(format!("parse error: {}", e)).unwrap();
            unsafe { lua_pushstring(l, err.as_ptr()) };
            1
        }
    }
}

extern "C" fn json_stringify(l: LuaState) -> c_int {
    let input = unsafe { CStr::from_ptr(luaL_checkstring(l, 1)) };
    let input_str = match input.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    match serde_json::from_str::<Value>(input_str) {
        Ok(value) => {
            let result = serde_json::to_string_pretty(&value).unwrap_or_default();
            let c_result = CString::new(result).unwrap();
            unsafe { lua_pushstring(l, c_result.as_ptr()) };
            1
        }
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn nicydynamic_init(l: LuaState) -> c_int {
    unsafe {
        lua_createtable(l, 0, 3);

        lua_pushcfunction(l, json_parse);
        lua_setfield(l, -2, JSON_PARSE_SYMBOL.as_ptr() as *const c_char);

        lua_pushcfunction(l, json_stringify);
        lua_setfield(l, -2, JSON_STRINGIFY_SYMBOL.as_ptr() as *const c_char);

        let version = CString::new("1.0.0 (serde_json)").unwrap();
        lua_pushstring(l, version.as_ptr());
        lua_setfield(l, -2, JSON_VERSION_SYMBOL.as_ptr() as *const c_char);
    }
    1
}
```

### Compiling

```bash
cargo build --release
# Output: target/release/json_ext.dll (or .so / .dylib)
```

### Usage

```luau
local json = runtime.loadlib("@self/json_ext.dll")

-- Parse JSON
local parsed = json.parse('{"name": "Luau", "age": 5}')
print(parsed)  -- {"age":5,"name":"Luau"}

-- Pretty print
local pretty = json.stringify('{"a":1,"b":2}')
print(pretty)
-- {
--   "a": 1,
--   "b": 2
-- }
```

---

## Zig

### Example with Hashing

**`hash_ext.zig`**:
```zig
const std = @import("std");
const c = @cImport({
    @cInclude("lua.h");
    @cInclude("lauxlib.h");
});

export fn nicydynamic_init(L: *c.lua_State) c_int {
    c.lua_createtable(L, 0, 3);

    c.lua_pushcfunction(L, hash_md5);
    c.lua_setfield(L, -2, "md5");

    c.lua_pushcfunction(L, hash_sha256);
    c.lua_setfield(L, -2, "sha256");

    c.lua_pushstring(L, "1.0.0 (Zig)");
    c.lua_setfield(L, -2, "version");

    return 1;
}

fn hash_md5(L: *c.lua_State) callconv(.C) c_int {
    const input = c.luaL_checkstring(L, 1);
    const input_slice = std.mem.span(@as([*:0]const u8, @ptrCast(input)));

    var hash: [16]u8 = undefined;
    // Note: In real code, use a proper crypto library
    // This is a simplified example
    std.mem.set(u8, &hash, 0);

    const hex = std.fmt.bytesToHex(&hash, .lower);
    const c_str = std.fmt.allocPrint(std.heap.c_allocator, "{s}", .{hex}) catch return 0;
    c.lua_pushstring(L, c_str.ptr);
    return 1;
}

fn hash_sha256(L: *c.lua_State) callconv(.C) c_int {
    const input = c.luaL_checkstring(L, 1);
    const input_slice = std.mem.span(@as([*:0]const u8, @ptrCast(input)));

    var hash: [32]u8 = undefined;
    std.mem.set(u8, &hash, 0);

    const hex = std.fmt.bytesToHex(&hash, .lower);
    const c_str = std.fmt.allocPrint(std.heap.c_allocator, "{s}", .{hex}) catch return 0;
    c.lua_pushstring(L, c_str.ptr);
    return 1;
}
```

### Compiling

**With Zig build system**:
```bash
# Build as shared library
zig build-lib hash_ext.zig -dynamic -lc -llua -O ReleaseFast
# Output: hash_ext.dll (or .so / .dylib)
```

**With `build.zig`**:
```zig
const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const lib = b.addSharedLibrary(.{
        .name = "hash_ext",
        .root_source_file = b.path("hash_ext.zig"),
        .target = target,
        .optimize = optimize,
    });

    lib.linkLibC();
    lib.linkSystemLibrary("lua");
    b.installArtifact(lib);
}
```

### Usage

```luau
local hash = runtime.loadlib("@self/hash_ext.dll")

print(hash.md5("hello"))      -- 5d41402abc4b2a76b9719d911017c592 (example)
print(hash.sha256("hello"))   -- 2cf24dba5fb0a30e26e83b2ac5b9e29e... (example)
```

---

## CPython (Embedding Python in Luau)

### Example with Python Execution

**`python_ext.c`**:
```c
#include "NicyRuntime.h"
#include <Python.h>

static int py_eval(lua_State* L) {
    const char* code = luaL_checkstring(L, 1);

    // Initialize Python (once)
    if (!Py_IsInitialized()) {
        Py_Initialize();
    }

    // Run the code
    PyRun_SimpleString(code);

    return 0;
}

static int py_exec(lua_State* L) {
    const char* code = luaL_checkstring(L, 1);

    if (!Py_IsInitialized()) {
        Py_Initialize();
    }

    PyObject* result = PyRun_String(code, Py_file_input,
                                     PyEval_GetGlobals(), PyEval_GetLocals());

    if (result) {
        lua_pushboolean(L, 1);
        Py_DECREF(result);
    } else {
        PyErr_Print();
        lua_pushboolean(L, 0);
    }
    return 1;
}

static int py_import(lua_State* L) {
    const char* module_name = luaL_checkstring(L, 1);

    if (!Py_IsInitialized()) {
        Py_Initialize();
    }

    PyObject* module = PyImport_ImportModule(module_name);
    if (module) {
        lua_pushboolean(L, 1);
        Py_DECREF(module);
    } else {
        PyErr_Print();
        lua_pushboolean(L, 0);
    }
    return 1;
}

__declspec(dllexport) int nicydynamic_init(lua_State* L) {
    lua_createtable(L, 0, 3);

    lua_pushcfunction(L, py_eval);
    lua_setfield(L, -2, "eval");

    lua_pushcfunction(L, py_exec);
    lua_setfield(L, -2, "exec");

    lua_pushcfunction(L, py_import);
    lua_setfield(L, -2, "import");

    return 1;
}
```

### Compiling

**Windows (MSVC)**:
```cmd
cl /LD python_ext.c /Fe:python_ext.dll /I"C:\Python312\include" /link /LIBPATH:"C:\Python312\libs" python312.lib
```

**Linux (GCC)**:
```bash
gcc -shared -fPIC -o python_ext.so python_ext.c $(python3-config --cflags --ldflags)
```

### Usage

```luau
local py = runtime.loadlib("@self/python_ext.dll")

-- Run Python code
py.eval("print('Hello from Python!')")
py.eval("import math; print(math.pi)")

-- Import modules
local success = py.import("requests")
if success then
    print("requests module available")
end
```

---

## Calling Convention Notes

### Symbol Export

| Platform | Export Macro |
|----------|-------------|
| Windows (MSVC) | `__declspec(dllexport)` |
| Windows (MinGW) | `__declspec(dllexport)` |
| Linux/macOS | (none needed, default visibility) |

### Symbol Name

The runtime looks for:
1. `nicydynamic_init` (primary)
2. `nicydinamic_init` (fallback, common typo)

### Return Value

The init function **must return 1** — the module table on the Lua stack. Returning anything else causes an error.

---

## Error Handling

If your module fails to initialize (missing dependency, crash during init), `runtime.loadlib` returns `nil` + error message:

```luau
local lib, err = runtime.loadlib("@self/missing.dll")
if not lib then
    print("Failed to load: " .. tostring(err))
end
```

### SEH Crash Protection (Windows)

On Windows, `runtime.loadlib()` is wrapped in SEH (Structured Exception Handling). If the native library crashes during load, the error is caught and returned as a string instead of crashing the process.

---

## Caching

Libraries are cached by their resolved path. Subsequent calls with the same path return the cached module table:

```luau
local a = runtime.loadlib("@self/mylib.dll")
local b = runtime.loadlib("@self/mylib.dll")
print(a == b)  -- true (same cached instance)
```

---

## Path Resolution

| Format | Description |
|--------|-------------|
| `@self/lib.so` | Relative to the entry script's directory |
| `./relative/lib.so` | Relative to current working directory |
| `/absolute/path/lib.so` | Absolute path |

---

## Unloading

Libraries are automatically unloaded when the runtime shuts down.

---

## See Also

- [runtime Table API](../runtime-api/runtime-table.md)
- [FFI Reference](../ffi-reference/index.md)
- [Embedding in C](embedding-c.md)
- [Embedding in Rust](embedding-rust.md)
