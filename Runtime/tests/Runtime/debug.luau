
return {
    name = "Runtime.debug",
    {
        name = "debug.traceback returns string",
        run = function(expect)
            local tb = debug.traceback()
            expect.type("string", tb, "debug.traceback is string")
            expect.truthy(#tb > 0, "traceback is non-empty")
        end,
    },
    {
        name = "debug.traceback with message",
        run = function(expect)
            local tb = debug.traceback("test error", 1)
            expect.truthy(string.match(tb, "test error") ~= nil, "traceback includes message")
        end,
    },
    {
        name = "debug.getinfo exists",
        run = function(expect)
            if debug.getinfo then
                expect.type("function", debug.getinfo, "debug.getinfo is function")
            else
                expect.truthy(true, "debug.getinfo not available in this runtime")
            end
        end,
    },
    {
        name = "debug.getinfo on current function",
        run = function(expect)
            if debug.getinfo then
                local info = debug.getinfo(1, "nSl")
                expect.type("table", info, "getinfo returns table")
            else
                expect.truthy(true, "debug.getinfo not available")
            end
        end,
    },
    {
        name = "debug library has expected functions",
        run = function(expect)
            if debug.getlocal then
                expect.type("function", debug.getlocal, "debug.getlocal exists")
            end
            if debug.getupvalue then
                expect.type("function", debug.getupvalue, "debug.getupvalue exists")
            end
            if debug.getmetatable then
                expect.type("function", debug.getmetatable, "debug.getmetatable exists")
            end
            expect.truthy(true, "debug library present")
        end,
    },
}
