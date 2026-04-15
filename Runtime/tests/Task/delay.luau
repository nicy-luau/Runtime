local task = _G.task

return {
    name = "task.delay",
    {
        name = "executes after delay",
        run = function(expect)
            local ran = false
            task.delay(0.05, function()
                ran = true
            end)
            task.wait(0.1)
            expect.eq(ran, true, "delayed function ran")
        end,
    },
    {
        name = "respects order by time",
        run = function(expect)
            local order = {}
            task.delay(0.1, function() table.insert(order, "b") end)
            task.delay(0.01, function() table.insert(order, "a") end)
            task.wait(0.15)
            expect.eq(order[1], "a", "shorter delay runs first")
            expect.eq(order[2], "b", "longer delay runs second")
        end,
    },
    {
        name = "passes arguments",
        run = function(expect)
            local val = nil
            task.delay(0.02, function(x, y)
                val = x * y
            end, 3, 5)
            task.wait(0.05)
            expect.eq(val, 15, "arguments passed to delayed")
        end,
    },
}
