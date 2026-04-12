local task = _G.task
return {
    name = "task.defer",
    {
        name = "defers execution",
        run = function(expect)
            local order = {}
            table.insert(order, "before")
            task.defer(function()
                table.insert(order, "deferred")
            end)
            table.insert(order, "after")
            task.wait(0.05)
            expect.eq(order[1], "before", "defer does not block")
            expect.eq(order[2], "after", "defer does not block")
            expect.eq(order[3], "deferred", "deferred function eventually runs")
        end,
    },
    {
        name = "passes arguments",
        run = function(expect)
            local val = nil
            task.defer(function(x)
                val = x
            end, 42)
            task.wait(0.05)
            expect.eq(val, 42, "defer passes arguments")
        end,
    },
}
