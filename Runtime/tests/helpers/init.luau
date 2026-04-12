--[[
    Shared test initialization.
    Provides common setup/teardown used across test suites.
]]

local helpers_dir = "./"
local expect = require(helpers_dir .. "expect")
local Report = require(helpers_dir .. "report")

local Init = {}

function Init.run_suite(suite_table, suite_name)
    local report = Report.new()

    if suite_table.beforeAll and type(suite_table.beforeAll) == "function" then
        suite_table.beforeAll()
    end

    for _, test_fn in ipairs(suite_table) do
        local name = test_fn.name or "unnamed"
        local before = test_fn.before or suite_table.beforeEach
        local after = test_fn.after or suite_table.afterEach

        local success, err = pcall(function()
            if before and type(before) == "function" then
                before(expect)
            end
            test_fn(expect)
            if after and type(after) == "function" then
                after(expect)
            end
        end)

        if success then
            report:pass(suite_name, name)
        else
            report:fail(suite_name, name, tostring(err))
        end
    end

    if suite_table.afterAll and type(suite_table.afterAll) == "function" then
        suite_table.afterAll()
    end

    return report
end

Init.expect = expect
Init.Report = Report

return Init
