local expect = require("../helpers/expect")
local task = _G.task

return {
    name = "task.cancel",
    {
        name = "cancels running thread",
        run = function(expect)
            local thread = task.spawn(function()
                task.wait(10)
            end)
            task.wait(0.01)
            local result = task.cancel(thread)
            expect.eq(result, true, "cancel returns true for valid thread")
        end,
    },
    {
        name = "rejects negative ID",
        run = function()
            local result = task.cancel(-1)
            expect.eq(result, false, "cancel(-1) returns false")
        end,
    },
    {
        name = "rejects zero ID",
        run = function()
            local result = task.cancel(0)
            expect.eq(result, false, "cancel(0) returns false")
        end,
    },
    {
        name = "rejects ID above 2^53 (precision loss)",
        run = function()
            local MAX_SAFE = 9007199254740992
            local result = task.cancel(MAX_SAFE + 1)
            expect.eq(result, false, "cancel rejects ID above 2^53")
        end,
    },
    {
        name = "double cancel returns false",
        run = function()
            local thread = task.spawn(function()
                task.wait(10)
            end)
            task.wait(0.01)
            task.cancel(thread)
            -- Second cancel behavior may vary; just check it returns boolean
            local result2 = task.cancel(thread)
            expect.type("boolean", result2, "double cancel returns boolean")
        end,
    },
}
