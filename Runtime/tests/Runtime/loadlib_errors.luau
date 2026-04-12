local runtime = _G.runtime

return {
    name = "Runtime.loadlib_errors",
    {
        name = "loadlib returns nil for nonexistent file",
        run = function(expect)
            local ok, err = runtime.loadlib("nao_existe.dll")
            expect.falsy(ok, "loadlib returns falsy for missing file")
            expect.type("string", err, "error message is string")
        end,
    },
    {
        name = "loadlib returns nil for non-library file",
        run = function(expect)
            local ok, err = runtime.loadlib("../helpers/expect.luau")
            expect.falsy(ok, "loadlib returns falsy for non-DLL file")
            expect.type("string", err, "error message is string")
        end,
    },
    {
        name = "loadlib with @self alias resolves correctly",
        run = function(expect)
            -- @self/ should resolve to the directory of the main script
            -- This should fail gracefully (file doesn't exist) but NOT crash
            local ok, err = runtime.loadlib("@self/nonexistent.dll")
            expect.falsy(ok, "loadlib returns falsy for missing @self file")
            expect.truthy(err, "error message is present")
        end,
    },
    {
        name = "loadlib with relative path resolves correctly",
        run = function(expect)
            -- ./helpers/ should resolve relative to entry_dir
            local ok, err = runtime.loadlib("./helpers/nao_existe.dll")
            expect.falsy(ok, "loadlib returns falsy for missing relative path")
            expect.type("string", err, "error message is string")
        end,
    },
    {
        name = "loadlib continues runtime after failure",
        run = function(expect)
            -- First call should fail
            runtime.loadlib("invalid.dll")
            -- runtime should still work
            expect.type("string", runtime.version, "runtime still works after loadlib failure")
            expect.type("function", runtime.loadlib, "loadlib still callable")
        end,
    },
    {
        name = "loadlib with empty path fails gracefully",
        run = function(expect)
            -- Empty path should either return error or fail to load
            -- The important thing is runtime continues working
            local ok, err = runtime.loadlib("")
            -- Either it returns nil with error, or throws an error
            if ok then
                -- If it succeeded somehow, verify runtime still works
                expect.type("string", runtime.version, "runtime still works")
            else
                expect.type("string", err, "error message is string")
            end
            -- Verify runtime is still functional
            expect.type("string", runtime.version, "runtime works after empty path")
        end,
    },
}
