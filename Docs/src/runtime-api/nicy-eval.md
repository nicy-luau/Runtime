# nicy_eval

Evaluate inline Luau code and print the result.

## C Signature

```c
void nicy_eval(const char* code);
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `code` | `const char*` | Luau source code to evaluate |

## Description

`nicy_eval` creates an isolated Luau state, loads the code, and executes it. Results are printed to stdout. Errors are printed to stderr.

Unlike `nicy_start`, it:
- Does **not** run the task scheduler
- Creates a fresh state each time (no shared globals)
- Is intended for quick one-liners and tests

## Example (C)

```c
#include "NicyRuntime.h"

int main() {
    nicy_eval("print(2 + 2)");
    nicy_eval("print('Hello from Luau!')");
    return 0;
}
```

## Notes

- The code is loaded as a chunk — no file-based module resolution
- Each call is independent (no state sharing)
- Errors terminate the program (no recovery)
