# Troubleshooting

Common issues and their solutions.

## Installation Issues

### "nicy: command not found"

The CLI is not in your `PATH`. Add the directory containing `nicy` to your `PATH`:

```powershell
# Windows
$env:PATH += ";C:\tools\nicy"

# Linux/macOS
export PATH="/usr/local/bin:$PATH"
```

### "Library not found" (macOS)

macOS may block unsigned libraries. Allow the library:

```bash
xattr -d com.apple.quarantine libnicyruntime.dylib
```

### "libnicyruntime.so: cannot open shared object file" (Linux)

Update the shared library cache:

```bash
sudo ldconfig
```

Or set `LD_LIBRARY_PATH`:

```bash
export LD_LIBRARY_PATH="/path/to/nicy:$LD_LIBRARY_PATH"
```

## Runtime Issues

### Module Not Found

```
Error: module 'mymodule' not found
```

**Causes**:
- File doesn't exist at the expected path
- Typo in the module name
- Wrong file extension

**Solutions**:
- Check the searched paths in the error message
- Use `@self` for relative paths: `require("@self/mymodule")`
- Verify the file exists: `ls mymodule.luau`

### Circular Require

```
Error: Cyclic require detected: a -> b -> a
```

**Cause**: Two modules require each other.

**Solution**: Restructure your code to break the cycle:

```luau
-- Before (circular):
-- a.luau: local B = require("b")
-- b.luau: local A = require("a")

-- After (no cycle):
-- common.luau: shared code
-- a.luau: local Common = require("common")
-- b.luau: local Common = require("common")
```

### Task Scheduler Not Running

Tasks spawned with `task.spawn` don't execute.

**Cause**: The scheduler only runs when using `nicy_start()`. It does not run with `nicy_eval()`.

**Solution**: Use `nicy_start()` for scripts that use async tasks.

### Native Code Not Available

```luau
print(runtime.hasJIT)  -- false
```

**Causes**:
- Running on Android (CodeGen disabled)
- Built without `luau-codegen` feature
- Using a pre-built binary for a platform without CodeGen support

**Solution**: Rebuild from source with CodeGen enabled (not applicable for Android).

## Compilation Issues

### "zig not found"

Install Zig 0.14.0:

```bash
# Download from https://ziglang.org/download/
# Or use a package manager
brew install zig        # macOS
choco install zig       # Windows
```

### "NDK not found" (Android)

Set the `ANDROID_NDK_HOME` environment variable:

```bash
export ANDROID_NDK_HOME="/path/to/android-ndk-r26d"
```

### Static Assertion Failed (Linux x86)

```
error: static assertion failed: size mismatch for value
```

This is a known issue with Luau on 32-bit Linux. The `luau-vector4` feature is automatically disabled for `linux-x86` targets. If you're building from source, ensure you have the latest `Cargo.toml` configuration.

## Performance Issues

### Scripts Running Slowly

1. **Enable CodeGen**: Add `--!native` and `--!optimize 2` to your scripts
2. **Use local variables**: Avoid global lookups in hot loops
3. **Pre-allocate tables**: Use `table.create()` for known sizes
4. **Profile your code**: Use `os.clock()` to find bottlenecks

See [Performance Tips](../advanced/performance-tips) for more.

### High Memory Usage

1. **Force garbage collection**: `collectgarbage("collect")`
2. **Avoid global caches**: Don't store large data in global tables
3. **Clear references**: Set variables to `nil` when done

See [Memory Management](../advanced/memory-management) for more.

## Error Reporting

### Enable Verbose Errors

```bash
NICY_VERBOSE_ERRORS=1 nicy run script.luau
```

### Disable Colors

```bash
NICY_NO_COLOR=1 nicy run script.luau
```

## Getting Help

If you're still stuck:

1. **Check the docs**: Search this documentation site
2. **Check the issues**: [GitHub Issues](https://github.com/nicy-luau/Runtime/issues)
3. **Report a bug**: [New Issue](https://github.com/nicy-luau/Runtime/issues/new)

When reporting bugs, include:
- NicyRuntime version (`nicy --version`)
- Platform and architecture
- Error output (with `NICY_VERBOSE_ERRORS=1`)
- Minimal reproducible example
