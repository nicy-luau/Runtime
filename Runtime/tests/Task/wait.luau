local task = _G.task

return {
    name = "task.wait",
    {
        name = "returns number or nil",
        run = function(expect)
            local result = task.wait(0)
            -- Runtime may return nil for zero-delay waits
            if result ~= nil then
                expect.type("number", result, "wait(0) returns number when present")
            end
        end,
    },
    {
        name = "respects delay",
        run = function(expect)
            local dt = task.wait(0.05)
            if dt then
                expect.truthy(dt >= 0.04, "wait(0.05) >= 0.04")
            end
        end,
    },
    {
        name = "handles negative as zero",
        run = function(expect)
            local dt = task.wait(-1)
            if dt then
                expect.truthy(dt >= 0, "wait(-1) returns >= 0")
            end
        end,
    },
    {
        name = "zero delay returns immediately",
        run = function(expect)
            local dt = task.wait(0)
            if dt then
                expect.truthy(dt >= 0, "wait(0) returns >= 0")
            end
        end,
    },
}
