
return {
    name = "Runtime.traceback",
    {
        name = "debug.traceback produces string",
        run = function(expect)
            local ok, _err = pcall(function()
                error("test error for traceback", 2)
            end)
            -- The runtime's error handler should format the error
            expect.eq(ok, false, "error was caught")
        end,
    },
    {
        name = "nested error traceback",
        run = function(expect)
            local ok, _err = pcall(function()
                local function inner()
                    error("inner error")
                end
                local function outer()
                    inner()
                end
                outer()
            end)
            expect.eq(ok, false, "nested error was caught")
        end,
    },
    {
        name = "pcall protects error handler",
        run = function(expect)
            -- Ensure the error handler doesn't crash on weird input
            expect.ok(function()
                pcall(function()
                    error(123)  -- non-string error
                end)
                pcall(function()
                    error(nil)  -- nil error
                end)
                pcall(function()
                    error({})  -- table error
                end)
            end, "error handler handles weird errors")
        end,
    },
    {
        name = "stack trace depth",
        run = function(expect)
            local ok = pcall(function()
                local function recurse(n)
                    if n > 0 then
                        recurse(n - 1)
                    else
                        error("deep error")
                    end
                end
                recurse(20)
            end)
            expect.eq(ok, false, "deep stack error was caught")
        end,
    },
}
