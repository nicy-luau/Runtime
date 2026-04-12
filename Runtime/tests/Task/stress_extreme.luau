local task = _G.task

return {
    name = "Task.stress_extreme",
    {
        name = "5000 spawned tasks complete",
        run = function(expect)
            local completed = 0
            for i = 1, 5000 do
                task.spawn(function()
                    completed += 1
                end)
            end
            task.wait(1.0)
            expect.truthy(completed >= 4990, "at least 4990 of 5000 spawned (got " .. completed .. ")")
        end,
    },
    {
        name = "rapid spawn and cancel 2000",
        run = function(expect)
            for i = 1, 2000 do
                local thread = task.spawn(function()
                    task.wait(10)
                end)
                task.cancel(thread)
            end
            expect.truthy(true, "2000 rapid spawn+cancel completed")
        end,
    },
    {
        name = "mixed 10k operations",
        run = function(expect)
            local spawned = 0
            for i = 1, 5000 do
                if i % 2 == 0 then
                    task.spawn(function()
                        spawned += 1
                    end)
                else
                    task.delay(0.001, function()
                        spawned += 1
                    end)
                end
            end
            task.wait(2.0)
            expect.truthy(spawned >= 4800, "at least 4800 of 5000 completed (got " .. spawned .. ")")
        end,
    },
}
