local runtime = _G.runtime
local warn = _G.warn

return {
    name = "Runtime.globals",
    {
        name = "runtime.version exists",
        run = function(expect)
            expect.type("string", runtime.version, "runtime.version is string")
        end,
    },
    {
        name = "runtime.entry_file exists",
        run = function(expect)
            expect.type("string", runtime.entry_file, "runtime.entry_file is string")
        end,
    },
    {
        name = "runtime.entry_dir exists",
        run = function(expect)
            expect.type("string", runtime.entry_dir, "runtime.entry_dir is string")
        end,
    },
    {
        name = "runtime.loadlib exists",
        run = function(expect)
            expect.type("function", runtime.loadlib, "runtime.loadlib is function")
        end,
    },
    {
        name = "runtime.hasJIT exists",
        run = function(expect)
            expect.type("function", runtime.hasJIT, "runtime.hasJIT is function")
        end,
    },
    {
        name = "warn function exists",
        run = function(expect)
            expect.type("function", warn, "warn is function")
        end,
    },
}
