local os = _G.os

return {
    name = "Core.os_ext",
    {
        name = "os.exit exists",
        run = function(expect)
            expect.type("function", os.exit, "os.exit is function")
        end,
    },
    {
        name = "os.getenv reads PATH or similar",
        run = function(expect)
            local val = os.getenv("PATH") or os.getenv("USERPROFILE") or os.getenv("HOME")
            if val then
                expect.type("string", val, "os.getenv returns string for known variable")
            else
                expect.truthy(true, "no known env vars available")
            end
        end,
    },
    {
        name = "os.getenv returns nil for unknown",
        run = function(expect)
            local val = os.getenv("__NICY_NONEXISTENT_VAR_XYZ_123")
            expect.eq(val, nil, "unknown env var returns nil")
        end,
    },
    {
        name = "os.tmpname generates unique names",
        run = function(expect)
            local name1 = os.tmpname()
            local name2 = os.tmpname()
            expect.type("string", name1, "tmpname returns string")
            expect.neq(name1, name2, "tmpname generates unique names")
        end,
    },
    {
        name = "os.sleep completes without error",
        run = function(expect)
            local start = os.clock()
            os.sleep(50)
            local elapsed = (os.clock() - start) * 1000
            expect.truthy(elapsed >= 30, "os.sleep(50) took >= 30ms (got " .. math.floor(elapsed) .. "ms)")
        end,
    },
    {
        name = "os.remove returns false for nonexistent file",
        run = function(expect)
            local ok = os.remove("__nicy_nonexistent_file_xyz_123.tmp")
            expect.eq(ok, false, "remove nonexistent returns false")
        end,
    },
    {
        name = "os.rename returns false for nonexistent source",
        run = function(expect)
            local ok = os.rename("__nicy_nonexistent_src.tmp", "__nicy_nonexistent_dst.tmp")
            expect.eq(ok, false, "rename nonexistent source returns false")
        end,
    },
    {
        name = "os.time returns number",
        run = function(expect)
            local t = os.time()
            expect.type("number", t, "os.time returns number")
            expect.truthy(t > 1700000000, "os.time returns reasonable unix timestamp")
        end,
    },
    {
        name = "os.date returns string without format",
        run = function(expect)
            local d = os.date()
            expect.type("string", d, "os.date() returns string")
            expect.truthy(#d > 5, "os.date returns non-empty string")
        end,
    },
    {
        name = "os.date with *t returns table",
        run = function(expect)
            local t = os.date("*t")
            expect.type("table", t, "os.date('*t') returns table")
            expect.truthy(t.year ~= nil, "date table has year")
            expect.truthy(t.month ~= nil, "date table has month")
            expect.truthy(t.day ~= nil, "date table has day")
            expect.truthy(t.hour ~= nil, "date table has hour")
            expect.truthy(t.min ~= nil, "date table has min")
            expect.truthy(t.sec ~= nil, "date table has sec")
        end,
    },
}
