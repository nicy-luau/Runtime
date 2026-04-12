--[[
    tests/Core/io_files.luau
    ──────────────────────────
    NOTE: Luau does NOT include the standard Lua io library.
    File I/O is not available in Luau.
    This file is intentionally minimal.
]]

return {
    name = "Core.io absent",
    {
        name = "io library is not available",
        run = function(expect)
            -- Luau deliberately excludes io library for safety/sandboxing
            expect.falsy(io, "io library should not exist in Luau")
        end,
    },
}
