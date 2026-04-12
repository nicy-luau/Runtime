local task = _G.task

return {
    name = "task.spawn",
    {
        name = "executes function",
        run = function(expect)
            local ran = false
            task.spawn(function()
                ran = true
            end)
            task.wait(0.01)
            expect.eq(ran, true, "spawned function ran")
        end,
    },
    {
        name = "passes arguments",
        run = function(expect)
            local result = nil
            task.spawn(function(x, y)
                result = x + y
            end, 3, 4)
            task.wait(0.01)
            expect.eq(result, 7, "arguments passed correctly")
        end,
    },
    {
        name = "handles many spawns",
        run = function(expect)
            local count = 0
            for i = 1, 50 do
                task.spawn(function()
                    count += 1
                end)
            end
            task.wait(0.1)
            expect.eq(count, 50, "all 50 tasks ran")
        end,
    },
    {
        name = "returns thread",
        run = function(expect)
            local thread = task.spawn(function()
                task.wait(0.5)
            end)
            expect.type("thread", thread, "spawn returns thread")
        end,
    },
}
