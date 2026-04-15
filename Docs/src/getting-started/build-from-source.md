# Build from Source

Building NicyRuntime from source gives you full control over the build configuration and enables contributions.

## Prerequisites

### Required

- **Rust** 1.75+ — [Install via rustup](https://rustup.rs/)
- **Git** — For cloning the repository

### Optional (for cross-compilation)

- **Zig** 0.14.0 — For cross-compiling Linux/macOS binaries
- **Android NDK** r26d — For Android builds
- **cargo-zigbuild** — `cargo install cargo-zigbuild --locked`
- **cargo-ndk** — `cargo install cargo-ndk --locked`

## Clone the Repository

```bash
git clone https://github.com/nicy-luau/Runtime.git
cd Runtime
```

## Quick Build (Native Target)

The simplest build — compiles for your current platform:

```bash
# Build both CLI and Runtime
cargo build --release --workspace

# Or build individually
cargo build --release -p nicy
cargo build --release -p nicyruntime
```

Output:
- `target/release/nicy` (or `nicy.exe` on Windows)
- `target/release/libnicyruntime.so` (or `.dll` / `.dylib`)

## Using the Build Script

The included `build.ps1` script simplifies cross-platform builds:

```powershell
# Build for your current platform (auto-detected)
.\build.ps1 -target user

# Build for all supported platforms
.\build.ps1 -target all

# Build for a specific platform
.\build.ps1 -target win-x64
.\build.ps1 -target linux-arm
.\build.ps1 -target mac-arm

# Force clean rebuild
.\build.ps1 -target user -force
```

### Supported Targets

| Target | Platform | Architecture | Toolchain |
|--------|----------|-------------|-----------|
| `win-x64` | Windows | x86_64 | MSVC (native) |
| `win-x86` | Windows | x86 | MSVC (native) |
| `win-arm` | Windows | ARM64 | MSVC (native) |
| `mac-x64` | macOS | x86_64 | Zig |
| `mac-arm` | macOS | ARM64 | Zig |
| `linux-x64` | Linux | x86_64 | Zig |
| `linux-arm` | Linux | ARM64 | Zig |
| `linux-x86` | Linux | x86 | Zig |
| `android-arm` | Android | ARM64 | cargo-ndk |
| `android-v7` | Android | ARMv7 | cargo-ndk |

## Cross-Compilation Setup

### Linux/macOS (via Zig)

```bash
# Install Zig 0.14.0
# https://ziglang.org/download/

# Install cargo-zigbuild
cargo install cargo-zigbuild --locked

# Build for Linux x64
cargo zigbuild --release --target x86_64-unknown-linux-gnu -p nicyruntime

# Build for macOS ARM64
cargo zigbuild --release --target aarch64-apple-darwin -p nicyruntime
```

### Android

```bash
# Install Android NDK r26d
# https://developer.android.com/ndk/downloads

# Install cargo-ndk
cargo install cargo-ndk --locked

# Build for ARM64
cargo ndk -t arm64-v8a build --release -p nicyruntime

# Build for ARMv7
cargo ndk -t armeabi-v7a build --release -p nicyruntime
```

## Build Configuration

### Release Profile

The default release profile is optimized for **minimum binary size**:

```toml
[profile.release]
strip = "symbols"      # Remove debug symbols
lto = true             # Link-time optimization
codegen-units = 1      # Single codegen unit for best optimization
opt-level = "z"        # Optimize for size (use "3" for speed)
```

### Customizing the Build

To optimize for **speed** instead of size, create `.cargo/config.toml`:

```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
```

### Feature Flags

The Runtime crate doesn't expose optional features currently, but you can modify `Runtime/Cargo.toml`:

```toml
# Disable CodeGen on platforms where it's unstable
[target.'cfg(target_os = "android")'.dependencies]
mlua-sys = { version = "0.10.0", features = ["luau", "vendored"] }
# Note: no "luau-codegen" feature for Android
```

## Verify the Build

```bash
# Run the built CLI
./target/release/nicy --version

# Run the test suite
./target/release/nicy run Runtime/tests/run_all.luau
```

## Build Artifacts

After a successful build:

```
target/
├── release/
│   ├── nicy                          # CLI binary
│   ├── libnicyruntime.so             # Runtime library
│   │   ├── deps/                     # Dependency libraries
│   │   └── .fingerprint/             # Build fingerprints
│   └── build/                        # Build scripts output
```

## Troubleshooting

### "zig not found"

Install Zig and ensure it's in your `PATH`:

```bash
zig version  # Should output 0.14.0
```

### "NDK not found" (Android)

Set the `ANDROID_NDK_HOME` environment variable:

```bash
export ANDROID_NDK_HOME=/path/to/android-ndk-r26d
```

### Build fails on Windows x86

Some Luau features may not be fully compatible with 32-bit Windows. Try building for x64 instead.

## What's Next?

- [Run your first script](quick-start.md)
- [Explore the CLI commands](../cli/commands)
- [Embed the runtime in your app](../guides/embedding-c)
