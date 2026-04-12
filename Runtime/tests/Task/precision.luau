
local task = _G.task

return {
    name = "task.timing precision",
    {
        name = "wait timing is reasonable",
        run = function(expect)
            local start = os.clock()
            task.wait(0.1)
            local elapsed = os.clock() - start
            expect.truthy(elapsed >= 0.05, "wait(0.1) took >= 0.05")
            expect.truthy(elapsed < 0.5, "wait(0.1) took < 0.5")
        end,
    },
    {
        name = "multiple delays complete correctly",
        run = function(expect)
            local completed = 0
            for i = 1, 10 do
                task.delay(0.05, function() completed += 1 end)
            end
            task.wait(0.15)
            expect.eq(completed, 10, "all 10 delays completed")
        end,
    },
}
