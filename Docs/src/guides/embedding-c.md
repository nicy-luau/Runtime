# Embedding in C

This guide shows how to embed NicyRuntime in a C application.

## Prerequisites

- A C compiler (MSVC, GCC, Clang)
- `NicyRuntime.h` header file
- `nicyruntime` shared library (`.dll`, `.so`, or `.dylib`)

## Setup

```
my_app/
├── main.c
├── NicyRuntime.h
└── nicyruntime.dll   # or .so / .dylib
```

## Basic Example

```c
#include <stdio.h>
#include "NicyRuntime.h"

int main(int argc, char* argv[]) {
    if (argc < 2) {
        fprintf(stderr, "Usage: %s <script.luau>\n", argv[0]);
        return 1;
    }

    // Print version info
    printf("NicyRuntime %s\n", nicy_version());
    printf("Powered by %s\n\n", nicy_luau_version());

    // Execute the script
    nicy_start(argv[1]);

    return 0;
}
```

## Compiling

### Windows (MSVC)

```cmd
cl /I. main.c nicyruntime.lib /Fe:my_app.exe
```

### Linux (GCC)

```bash
gcc -o my_app main.c -L. -lnicyruntime -Wl,-rpath,.
```

### macOS (Clang)

```bash
clang -o my_app main.c -L. -lnicyruntime -Wl,-rpath,@executable_path
```

## Using the Lua C API

For fine-grained control, use the exported FFI functions:

```c
#include <stdio.h>
#include "NicyRuntime.h"

int main() {
    // nicy_eval prints results directly
    nicy_eval("print('hello from luau')");

    // For more control, get a lua_State through your own initialization
    // or use nicy_start() for full scripts
    return 0;
}
```

## Dynamic Loading (No Linking)

Load the library at runtime with `dlopen`/`LoadLibrary`:

```c
#include <stdio.h>
#include <stdlib.h>

#ifdef _WIN32
#include <windows.h>
#define LIB_HANDLE HMODULE
#define LIB_LOAD(path) LoadLibraryA(path)
#define LIB_SYM(lib, name) GetProcAddress(lib, name)
#else
#include <dlfcn.h>
#define LIB_HANDLE void*
#define LIB_LOAD(path) dlopen(path, RTLD_NOW)
#define LIB_SYM(lib, name) dlsym(lib, name)
#endif

typedef void (*nicy_start_fn)(const char*);

int main() {
    LIB_HANDLE lib = LIB_LOAD(
#ifdef _WIN32
        "nicyruntime.dll"
#elif __APPLE__
        "libnicyruntime.dylib"
#else
        "libnicyruntime.so"
#endif
    );

    if (!lib) {
        fprintf(stderr, "Failed to load runtime library\n");
        return 1;
    }

    nicy_start_fn start = (nicy_start_fn)LIB_SYM(lib, "nicy_start");
    if (!start) {
        fprintf(stderr, "Failed to find nicy_start\n");
        return 1;
    }

    start("myscript.luau");
    return 0;
}
```

## Error Handling

Errors from `nicy_start` are printed to stderr and the function returns. To control error output, use environment variables:

```c
// Enable verbose errors
#ifdef _WIN32
    _putenv("NICY_VERBOSE_ERRORS=1");
#else
    setenv("NICY_VERBOSE_ERRORS", "1", 1);
#endif

nicy_start("myscript.luau");
```

## See Also

- [FFI Reference](ffi-reference/index.md) — Complete C API
- [Embedding in Rust](embedding-rust.md)
