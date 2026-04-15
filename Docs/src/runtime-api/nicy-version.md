# nicy_version / nicy_luau_version

Get version information strings.

## C Signatures

```c
const char* nicy_version(void);
const char* nicy_luau_version(void);
```

## Return Value

| Function | Returns |
|----------|---------|
| `nicy_version()` | Engine version string (e.g., `"1.0.0-alpha"`) |
| `nicy_luau_version()` | Luau version string (e.g., `"0.650"`) |

Both return static string pointers. Do not modify or free them.

## Example (C)

```c
#include "NicyRuntime.h"
#include <stdio.h>

int main() {
    printf("Engine: %s\n", nicy_version());
    printf("Luau: %s\n", nicy_luau_version());
    return 0;
}
```

## CLI Usage

```bash
# Engine version only
nicy version
# → nicy 1.0.0-alpha

# Engine + Luau version
nicy runtime-version
# → Engine: 1.0.0-alpha
# → Luau: 0.650
```
