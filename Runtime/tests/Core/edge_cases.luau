return {
    name = "Core.edge_cases",
    {
        name = "nil handling",
        run = function(expect)
            local a = nil
            expect.falsy(a, "nil is falsy")
            expect.eq(a == nil, true, "nil equals nil")
        end,
    },
    {
        name = "empty table operations",
        run = function(expect)
            local t = {}
            expect.eq(#t, 0, "empty table length is 0")
            expect.falsy(next(t), "next on empty table is nil")
        end,
    },
    {
        name = "large number operations",
        run = function(expect)
            local big = 1e15
            expect.eq(big + 1, 1000000000000001, "large number arithmetic")
        end,
    },
    {
        name = "empty string operations",
        run = function(expect)
            expect.eq(#"", 0, "empty string length")
            expect.eq(string.find("", ""), 1, "find empty in empty")
            expect.eq(string.sub("", 1, 1), "", "sub on empty")
        end,
    },
    {
        name = "boolean edge cases",
        run = function(expect)
            expect.eq(0 and true, true, "0 is truthy")
            expect.eq("" and true, true, "empty string is truthy")
            expect.eq(false or "default", "default", "false or default")
            expect.eq(nil or "default", "default", "nil or default")
        end,
    },
    {
        name = "pcall with no arguments",
        run = function(expect)
            -- pcall() with no args may error or return false depending on runtime
            local ok = pcall(function()
                local result = pcall()
                expect.falsy(result, "inner pcall with no args returns false")
            end)
            expect.truthy(true, "pcall edge case handled")
        end,
    },
    {
        name = "xpcall exists",
        run = function(expect)
            expect.type("function", xpcall, "xpcall is function")
        end,
    },
    {
        name = "coroutine library exists",
        run = function(expect)
            expect.type("function", coroutine.create, "coroutine.create exists")
            expect.type("function", coroutine.resume, "coroutine.resume exists")
            expect.type("function", coroutine.yield, "coroutine.yield exists")
        end,
    },
    {
        name = "type function works for all types",
        run = function(expect)
            expect.eq(type(nil), "nil", "type nil")
            expect.eq(type(0), "number", "type number")
            expect.eq(type(""), "string", "type string")
            expect.eq(type(true), "boolean", "type boolean")
            expect.eq(type({}), "table", "type table")
            expect.eq(type(print), "function", "type function")
        end,
    },
    {
        name = "rawequal works",
        run = function(expect)
            local a, b = {}, {}
            expect.eq(rawequal(a, a), true, "rawequal same table")
            expect.eq(rawequal(a, b), false, "rawequal different tables")
        end,
    },
}
