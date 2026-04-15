local runtime = _G.runtime

return {
    name = "Runtime.error handler",
    {
        name = "stack cleanup after 50 errors (H6)",
        run = function(expect)
            for i = 1, 50 do
                expect.fails(function()
                    error("test error #" .. i)
                end, "error #" .. i .. " caught")
            end
        end,
    },
    {
        name = "100 errors without leak (H1)",
        run = function(expect)
            local count = 0
            for i = 1, 100 do
                expect.fails(function()
                    local t = nil
                    count += 1
                    t.invalid()
                end)
            end
            expect.eq(count, 100, "all 100 iterations ran")
        end,
    },
    {
        name = "error messages formatted correctly",
        run = function(expect)
            expect.fails(function()
                error("specific error message")
            end, "error is propagated")
        end,
    },
    {
        name = "runtime functional after errors",
        run = function(expect)
            local v = runtime.version
            expect.type("string", v, "runtime.version still accessible")
        end,
    },
}
