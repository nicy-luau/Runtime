return {
    name = "Core.metatables",
    {
        name = "__index metamethod works",
        run = function(expect)
            local mt = { __index = function(_, key)
                return "got_" .. tostring(key)
            end }
            local t = setmetatable({}, mt)
            expect.eq(t.foo, "got_foo", "__index returns dynamic value")
        end,
    },
    {
        name = "__newindex metamethod works",
        run = function(expect)
            local captured = {}
            local mt = { __newindex = function(_, key, value)
                captured[key] = value
            end }
            local t = setmetatable({}, mt)
            t.hello = "world"
            expect.eq(captured.hello, "world", "__newindex captured assignment")
        end,
    },
    {
        name = "__call metamethod works",
        run = function(expect)
            local mt = { __call = function(_, ...)
                return "called", ...
            end }
            local t = setmetatable({}, mt)
            local a, b, c = t(1, 2)
            expect.eq(a, "called", "__call first return value")
            expect.eq(b, 1, "__call arg1")
            expect.eq(c, 2, "__call arg2")
        end,
    },
    {
        name = "rawget bypasses __index",
        run = function(expect)
            local mt = { __index = function() return "fake" end }
            local t = setmetatable({ real = 42 }, mt)
            expect.eq(rawget(t, "real"), 42, "rawget returns actual value")
            expect.eq(rawget(t, "fake"), nil, "rawget returns nil for missing")
        end,
    },
    {
        name = "rawset bypasses __newindex",
        run = function(expect)
            local mt = { __newindex = function() end }
            local t = setmetatable({}, mt)
            rawset(t, "key", "value")
            -- rawset stores directly; accessing t.key reads the raw value
            -- __newindex is only called for NEW assignments, not reads
            expect.eq(rawget(t, "key"), "value", "rawset set the value directly")
        end,
    },
}
