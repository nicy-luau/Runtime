# Cross-Platform

NicyRuntime supports Windows, macOS, Linux, and Android with platform-specific considerations.

## Supported Platforms

| Platform | Architecture | Status | Library Extension |
|----------|-------------|--------|-------------------|
| Windows | x64 | ✅ Stable | `.dll` |
| Windows | x86 | ✅ Stable | `.dll` |
| Windows | ARM64 | ⚠️ Beta | `.dll` |
| macOS | x64 | ✅ Stable | `.dylib` |
| macOS | ARM64 | ✅ Stable | `.dylib` |
| Linux | x64 | ✅ Stable | `.so` |
| Linux | ARM64 | ✅ Stable | `.so` |
| Linux | x86 | ✅ Stable (no vector4) | `.so` |
| Android | ARM64 | ✅ Stable | `.so` |
| Android | ARMv7 | ✅ Stable | `.so` |

## Platform-Specific Features

### Windows

| Feature | Status |
|---------|--------|
| CodeGen/JIT | ✅ Supported |
| High-Resolution Timer | ✅ `NICY_HIRES_TIMER=1` |
| SEH Crash Protection | ✅ `runtime.loadlib()` |
| luau-vector4 | ✅ Supported |

### macOS

| Feature | Status |
|---------|--------|
| CodeGen/JIT | ✅ Supported |
| High-Resolution Timer | ❌ No effect (already high-res) |
| SEH Crash Protection | ❌ Not applicable |
| luau-vector4 | ✅ Supported |

### Linux

| Feature | Status |
|---------|--------|
| CodeGen/JIT | ✅ Supported (x64, ARM64) |
| High-Resolution Timer | ❌ No effect |
| SEH Crash Protection | ❌ Not applicable |
| luau-vector4 | ✅ x64/ARM64 only |

### Android

| Feature | Status |
|---------|--------|
| CodeGen/JIT | ❌ Disabled (stability) |
| High-Resolution Timer | ❌ Not applicable |
| SEH Crash Protection | ❌ Not applicable |
| luau-vector4 | ❌ Disabled |

## Cross-Compilation

### Building from Windows

```powershell
# Build for your current platform
.\build.ps1 -target user

# Build for all platforms
.\build.ps1 -target all
```

### Building from Linux/macOS

```bash
# Install zig and cargo-zigbuild
cargo install cargo-zigbuild --locked

# Build for Windows x64
cargo zigbuild --release --target x86_64-pc-windows-gnu -p nicyruntime

# Build for Linux ARM64
cargo zigbuild --release --target aarch64-unknown-linux-gnu -p nicyruntime
```

### Building for Android

```bash
# Install Android NDK r26d
# Install cargo-ndk
cargo install cargo-ndk --locked

# Build for ARM64
cargo ndk -t arm64-v8a build --release -p nicyruntime

# Build for ARMv7
cargo ndk -t armeabi-v7a build --release -p nicyruntime
```

## Path Handling

NicyRuntime handles path separators correctly on all platforms:

```luau
-- Works on all platforms
local mod = require("modules/myModule")

-- Platform-specific paths
local home = os.getenv("HOME") or os.getenv("USERPROFILE")
```

## Line Endings

Luau source files should use **LF** (`\n`) line endings. CRLF (`\r\n`) files on Windows are handled correctly by the parser.

## See Also

- [Build from Source](../getting-started/build-from-source.md)
- [Performance Tips](performance-tips.md)
