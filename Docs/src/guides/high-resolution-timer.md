# High-Resolution Timer

On Windows, the default timer resolution is ~15.6ms. NicyRuntime can enable a high-resolution timer for sub-millisecond precision.

## Enabling

Set the `NICY_HIRES_TIMER` environment variable:

```bash
NICY_HIRES_TIMER=1 nicy run timing_test.luau
```

## Implementation

When enabled, NicyRuntime calls:
- `timeBeginPeriod(1)` at startup
- `timeEndPeriod(1)` at shutdown

This sets the system timer resolution to 1ms, improving the precision of:
- `os.clock()`
- `os.sleep()`
- `task.wait()`

## ⚠️ Important Warnings

### System-Wide Effect

`timeBeginPeriod` affects the **entire system**, not just your process. While active:
- All applications benefit from (or are affected by) the higher timer resolution
- Power consumption may increase (CPU wakes up more frequently)
- Battery life may decrease on laptops

### Only Use When Needed

Only enable the high-resolution timer when you actually need sub-millisecond precision:
- ✅ Performance profiling
- ✅ Real-time simulations
- ✅ Audio processing
- ❌ General scripting

### Automatic Cleanup

NicyRuntime automatically calls `timeEndPeriod` when the runtime shuts down, even if an error occurs. This ensures the system timer resolution is restored.

## Platform Support

| Platform | Supported |
|----------|-----------|
| Windows | ✅ Yes |
| macOS | ❌ No effect (already high-resolution) |
| Linux | ❌ No effect (use `clock_gettime`) |
| Android | ❌ No effect |

## Example

```luau
-- timing_test.luau
local start = os.clock()

-- Some computation
local sum = 0
for i = 1, 1000000 do
    sum = sum + i
end

local elapsed = os.clock() - start
print(string.format("Sum: %d", sum))
print(string.format("Time: %.6f seconds", elapsed))
```

Run with high-resolution timer:
```bash
NICY_HIRES_TIMER=1 nicy run timing_test.luau
```

## Alternatives

For cross-platform high-resolution timing, consider using `os.clock()` without the flag — on non-Windows platforms, it already uses high-resolution timers.
