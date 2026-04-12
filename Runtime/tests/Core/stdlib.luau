return {
    name = "Core.stdlib",

    -- String tests
    {
        name = "string.format works",
        run = function(expect)
            local s = string.format("%s = %d", "answer", 42)
            expect.eq(s, "answer = 42", "string.format output")
        end,
    },
    {
        name = "string.match and gsub",
        run = function(expect)
            local m = string.match("hello world", "(%w+)%s+(%w+)")
            expect.truthy(m ~= nil, "string.match captures groups")
            local replaced, count = string.gsub("a.b.c", "%.", "-")
            expect.eq(replaced, "a-b-c", "gsub replaces dots")
            expect.eq(count, 2, "gsub reports 2 replacements")
        end,
    },
    {
        name = "string.sub and string.len",
        run = function(expect)
            local s = "hello"
            expect.eq(string.sub(s, 1, 3), "hel", "substring 1-3")
            expect.eq(#s, 5, "string length")
            expect.eq(string.len(s), 5, "string.len")
        end,
    },

    -- Math tests
    {
        name = "math library basic",
        run = function(expect)
            expect.eq(math.floor(3.7), 3, "floor")
            expect.eq(math.ceil(3.2), 4, "ceil")
            expect.eq(math.abs(-5), 5, "abs")
            expect.eq(math.max(1, 5, 3), 5, "max")
            expect.eq(math.min(1, 5, 3), 1, "min")
        end,
    },
    {
        name = "math.random",
        run = function(expect)
            math.randomseed(12345)
            local r1 = math.random()
            expect.type("number", r1, "random returns number")
            expect.truthy(r1 >= 0 and r1 < 1, "random in [0, 1)")
            local r2 = math.random(1, 10)
            expect.truthy(r2 >= 1 and r2 <= 10, "random in range [1, 10]")
        end,
    },
    {
        name = "math constants",
        run = function(expect)
            expect.truthy(math.pi > 3.14 and math.pi < 3.15, "pi value")
            expect.truthy(math.huge > 1e300, "math.huge is large")
        end,
    },

    -- Table tests
    {
        name = "table.insert and table.remove",
        run = function(expect)
            local t = {}
            table.insert(t, "a")
            table.insert(t, "b")
            table.insert(t, "c")
            expect.eq(#t, 3, "table has 3 elements")
            expect.eq(t[1], "a", "first element")
            local removed = table.remove(t, 2)
            expect.eq(removed, "b", "removed b")
            expect.eq(#t, 2, "table has 2 elements after remove")
        end,
    },
    {
        name = "table.concat",
        run = function(expect)
            local t = {"a", "b", "c"}
            expect.eq(table.concat(t, "-"), "a-b-c", "concat with separator")
            expect.eq(table.concat(t), "abc", "concat no separator")
        end,
    },
    {
        name = "table.sort",
        run = function(expect)
            local t = {3, 1, 4, 1, 5, 9}
            table.sort(t)
            expect.eq(t[1], 1, "sorted first")
            expect.eq(t[6], 9, "sorted last")
            table.sort(t, function(a, b) return a > b end)
            expect.eq(t[1], 9, "reverse sorted first")
        end,
    },
    {
        name = "table.unpack",
        run = function(expect)
            local t = {1, 2, 3}
            local a, b, c = table.unpack(t)
            expect.eq(a, 1, "unpack first")
            expect.eq(b, 2, "unpack second")
            expect.eq(c, 3, "unpack third")
        end,
    },
}
