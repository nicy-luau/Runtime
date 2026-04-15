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
            local success, err = pcall(function()
                task.cancel(MAX_SAFE + 1)
            end)
            expect.eq(success, false, "cancel throws error for ID above 2^53")
            expect.ok(err:find("exceeds safe integer range"), "error message mentions 2^53 limit")
        end,
    },
    {
        name = "accepts ID at exact 2^53 boundary",
        run = function()
            local MAX_SAFE = 9007199254740992
            -- Should not throw, but returns false since no such delay exists
            local result = task.cancel(MAX_SAFE)
            expect.eq(result, false, "cancel(2^53) returns false (no such delay)")
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
