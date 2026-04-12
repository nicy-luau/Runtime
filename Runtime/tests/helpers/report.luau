--[[
    tests/helpers/report.luau
    Pretty-print test results with ANSI colors.
]]

-- ANSI escape codes
local GREEN   = "\27[32m"
local RED     = "\27[31m"
local BOLD    = "\27[1m"
local DIM     = "\27[2m"
local RESET   = "\27[0m"

local Report = {}
Report.__index = Report

function Report.new()
    return setmetatable({
        passes = {},
        failures = {},
        start_time = os.clock(),
    }, Report)
end

function Report:pass(suite, name)
    table.insert(self.passes, { suite = suite, name = name })
end

function Report:fail(suite, name, reason)
    table.insert(self.failures, { suite = suite, name = name, reason = reason })
end

function Report:print()
    local elapsed = os.clock() - self.start_time
    local total = #self.passes + #self.failures

    print("")
    print(DIM .. string.rep("─", 50) .. RESET)

    local pass_str = GREEN .. tostring(#self.passes) .. RESET .. " passed"
    local fail_str = RED .. tostring(#self.failures) .. RESET .. " failed"
    print(string.format("  %s  |  %s  |  %d total  |  %.2fs",
        pass_str, fail_str, total, elapsed
    ))

    if #self.failures == 0 then
        print("")
        print(GREEN .. BOLD .. "  ✓ All tests passed!" .. RESET)
    else
        print("")
        print(RED .. BOLD .. "  ✗ " .. #self.failures .. " test(s) failed!" .. RESET)
    end
    print(DIM .. string.rep("─", 50) .. RESET)
end

return Report
