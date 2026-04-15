# Installation

NicyRuntime is distributed as pre-built binaries for every major platform. You can also build from source.

## Download Pre-built Binaries

Visit the [Releases page](https://github.com/nicy-luau/Runtime/releases) and download the archive for your platform:

| Platform | Architecture | Archive | Contents |
|----------|-------------|---------|----------|
| Windows | x64 | `nicy-win-x64.zip` | `nicy.exe`, `nicyruntime.dll` |
| Windows | x86 | `nicy-win-x86.zip` | `nicy.exe`, `nicyruntime.dll` |
| Windows | ARM64 | `nicy-win-arm.zip` | `nicy.exe`, `nicyruntime.dll` |
| macOS | x64 | `nicy-mac-x64.zip` | `nicy`, `libnicyruntime.dylib` |
| macOS | ARM64 | `nicy-mac-arm.zip` | `nicy`, `libnicyruntime.dylib` |
| Linux | x64 | `nicy-linux-x64.zip` | `nicy`, `libnicyruntime.so` |
| Linux | ARM64 | `nicy-linux-arm.zip` | `nicy`, `libnicyruntime.so` |
| Android | ARM64 | `nicy-android-arm.zip` | `nicy`, `libnicyruntime.so` |
| Android | ARMv7 | `nicy-android-v7.zip` | `nicy`, `libnicyruntime.so` |

### Windows

1. Download `nicy-win-x64.zip`
2. Extract to a folder (e.g., `C:\tools\nicy\`)
3. Add the folder to your `PATH`:

```powershell
# Temporary (current session)
$env:PATH += ";C:\tools\nicy"

# Permanent (user-level)
[Environment]::SetEnvironmentVariable(
    "PATH",
    [Environment]::GetEnvironmentVariable("PATH", "User") + ";C:\tools\nicy",
    "User"
)
```

4. Verify installation:

```powershell
nicy --version
```

### macOS

1. Download `nicy-mac-arm.zip` (Apple Silicon) or `nicy-mac-x64.zip` (Intel)
2. Extract and move to `/usr/local/bin/`:

```bash
unzip nicy-mac-arm.zip
sudo mv nicy /usr/local/bin/
sudo mv libnicyruntime.dylib /usr/local/lib/
```

3. Verify:

```bash
nicy --version
```

### Linux

1. Download the appropriate archive for your architecture
2. Extract and move to your preferred location:

```bash
unzip nicy-linux-x64.zip
sudo mv nicy /usr/local/bin/
sudo mv libnicyruntime.so /usr/local/lib/
sudo ldconfig  # Update shared library cache
```

3. Verify:

```bash
nicy --version
```

### Android

The Android builds are primarily intended for embedding. The CLI binary may require a rooted device or Termux environment.

For embedding in Android apps, include `libnicyruntime.so` in your `jniLibs/` folder and use JNI to call the FFI functions.

## Verify Installation

Run the following command to verify everything is working:

```bash
nicy --version
```

You should see output like:

```
nicy 1.0.0-alpha
Luau 0.650 (with CodeGen)
```

> 💡 **Tip:** The `Luau` version number and `with CodeGen` indicator confirm that JIT compilation is available on your platform.

## What's Next?

Now that NicyRuntime is installed, move on to [Quick Start](quick-start.md) to run your first script.
