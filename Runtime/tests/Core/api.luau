local task = _G.task

return {
    name = "Core.api",

    {
        name = "task.spawn exists",
        run = function(expect)
            expect.type("function", task.spawn, "task.spawn is function")
        end,
    },
    {
        name = "task.wait exists",
        run = function(expect)
            expect.type("function", task.wait, "task.wait is function")
        end,
    },
    {
        name = "task.delay exists",
        run = function(expect)
            expect.type("function", task.delay, "task.delay is function")
        end,
    },
    {
        name = "task.cancel exists",
        run = function(expect)
            expect.type("function", task.cancel, "task.cancel is function")
        end,
    },
    {
        name = "task.defer exists",
        run = function(expect)
            expect.type("function", task.defer, "task.defer is function")
        end,
    },
    {
        name = "require exists",
        run = function(expect)
            expect.type("function", require, "require is function")
        end,
    },
}
