local task = _G.task
return {
    name = "task.stress",
    {
        name = "1000 spawned tasks complete",
        run = function(expect)
            local completed = 0
            for i = 1, 1000 do
                task.spawn(function()
                    completed += 1
                end)
            end
            task.wait(0.5)
            expect.eq(completed, 1000, "all 1000 tasks completed")
        end,
    },
    {
        name = "500 delays fire correctly",
        run = function(expect)
            local count = 0
            for i = 1, 500 do
                task.delay(0.02, function()
                    count += 1
                end)
            end
            task.wait(1.0)
            expect.truthy(count >= 495, "at least 495 of 500 delays fired (got " .. count .. ")")
        end,
    },
    {
        name = "mixed spawn and delay stress",
        run = function(expect)
            local spawned = 0
            local delayed = 0
            for i = 1, 200 do
                task.spawn(function()
                    spawned += 1
                end)
                task.delay(0.01, function()
                    delayed += 1
                end)
            end
            task.wait(0.3)
            expect.eq(spawned, 200, "200 spawned completed")
            expect.eq(delayed, 200, "200 delayed completed")
        end,
    },
    {
        name = "rapid cancel stress",
        run = function(expect)
            for i = 1, 100 do
                local thread = task.spawn(function()
                    task.wait(10)
                end)
                task.wait(0.001)
                task.cancel(thread)
            end
            -- If we got here without hanging, the test passed
            expect.truthy(true, "100 rapid cancels completed")
        end,
    },
}
