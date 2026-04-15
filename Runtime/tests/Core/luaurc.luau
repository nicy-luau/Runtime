local simple = "./Require/fixtures/simple"
local task = _G.task

return {
    name = "Core.luaurc parser (H4)",
    {
        name = "parser stable under repeated use",
        run = function(expect)
            for i = 1, 20 do
                expect.ok(function()
                    require(simple)
                end, "require #" .. i .. " succeeded")
            end
        end,
    },
    {
        name = "parser works after task operations",
        run = function(expect)
            task.spawn(function() end)
            task.wait(0.01)
            expect.ok(function()
                require(simple)
            end, "require after tasks works")
        end,
    },
}
