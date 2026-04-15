--[[
    tests/Core/gc.luau
    ────────────────────
    Tests for Luau garbage collection via collectgarbage().
    Luau supports: count, collect, isrunning, stop, restart, step
]]

return {
    name = "Core.gc",
    {
        name = "collectgarbage is a global function",
        run = function(expect)
            expect.type("function", collectgarbage, "collectgarbage exists")
        end,
    },
    {
        name = "collectgarbage(\"count\") returns number",
        run = function(expect)
            local kb = collectgarbage("count")
            expect.type("number", kb, "count returns number")
            expect.truthy(kb >= 0, "count is non-negative")
        end,
    },
    {
        name = "collectgarbage(\"collect\") runs full cycle",
        run = function(expect)
            -- Allocate some memory
            for i = 1, 500 do
                local _ = {data = string.rep("x", 200)}
            end
            local before = collectgarbage("count")
            collectgarbage("collect")
            local after = collectgarbage("count")
            expect.truthy(after <= before, "collect reclaims memory")
        end,
    },
    {
        name = "collectgarbage(\"isrunning\") returns boolean",
        run = function(expect)
            local running = collectgarbage("isrunning")
            expect.type("boolean", running, "isrunning returns boolean")
            expect.truthy(running, "gc should be running by default")
        end,
    },
    {
        name = "collectgarbage(\"stop\") and \"restart\"",
        run = function(expect)
            -- Stop GC
            collectgarbage("stop")
            local running = collectgarbage("isrunning")
            expect.falsy(running, "gc should be stopped")

            -- Restart GC
            collectgarbage("restart")
            running = collectgarbage("isrunning")
            expect.truthy(running, "gc should be running after restart")
        end,
    },
    {
        name = "collectgarbage(\"step\") executes incremental GC",
        run = function(expect)
            -- Allocate memory to give step something to do
            for i = 1, 200 do
                local _ = {data = string.rep("x", 100)}
            end
            local result = collectgarbage("step")
            expect.type("boolean", result, "step returns boolean")
        end,
    },
    {
        name = "table garbage collection reclaims memory",
        run = function(expect)
            local count_before = collectgarbage("count")
            for i = 1, 1000 do
                local _ = {data = string.rep("x", 100)}
            end
            collectgarbage("collect")
            local count_after = collectgarbage("count")
            -- After GC, memory should be close to or less than before
            expect.truthy(count_after <= count_before + 10, "GC reclaimed memory")
        end,
    },
    {
        name = "weak table values are collected",
        run = function(expect)
            local weak = setmetatable({}, { __mode = "v" })
            local t = {key = "value"}
            weak[1] = t
            t = nil
            collectgarbage("collect")
            -- weak[1] should be nil after gc
            expect.falsy(weak[1], "weak table value should be collected")
        end,
    },
    {
        name = "weak table keys are collected with __mode = 'k'",
        run = function(expect)
            local weak = setmetatable({}, { __mode = "k" })
            local key = {id = "mykey"}
            weak[key] = "value"
            key = nil
            collectgarbage("collect")
            -- Table should be empty after GC
            local count = 0
            for _ in pairs(weak) do
                count = count + 1
            end
            expect.eq(count, 0, "weak table should be empty after GC")
        end,
    },
    {
        name = "memory grows and shrinks with GC cycles",
        run = function(expect)
            local baseline = collectgarbage("count")

            -- Allocate
            local big = {}
            for i = 1, 2000 do
                big[i] = string.rep("abcdefghij", 100)
            end
            local during = collectgarbage("count")
            expect.truthy(during > baseline, "memory should grow during allocation")

            -- Release and collect
            big = nil
            collectgarbage("collect")
            local after = collectgarbage("count")
            expect.truthy(after < during, "memory should shrink after GC")
        end,
    },
}
