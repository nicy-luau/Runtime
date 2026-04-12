--[[
    tests/run_all.luau
    ──────────────────
    Main test runner. Loads all suites via relative requires from this file's
    location, so paths work correctly with nicy's require resolver.
]]

local expect = require("./helpers/expect")
local Report = require("./helpers/report")

local report = Report.new()

-- ANSI escape codes for real-time output
local GREEN   = "\27[32m"
local RED     = "\27[31m"
local BOLD    = "\27[1m"
local DIM     = "\27[2m"
local RESET   = "\27[0m"

-- Suite list — each path is relative to THIS file (tests/).
local suite_paths = {
    -- Core
    { "Core/api",              "./Core/api" },
    { "Core/luaurc",           "./Core/luaurc" },
    { "Core/metatables",       "./Core/metatables" },
    { "Core/vectors",          "./Core/vectors" },
    { "Core/buffers",          "./Core/buffers" },
    { "Core/stdlib",           "./Core/stdlib" },
    { "Core/gc",               "./Core/gc" },
    { "Core/bit32",            "./Core/bit32" },
    { "Core/edge_cases",       "./Core/edge_cases" },
    { "Core/io_files",         "./Core/io_files" },
    { "Core/os_ext",           "./Core/os_ext" },

    -- Require
    { "Require/resolution",    "./Require/resolution" },
    { "Require/relative",      "./Require/relative" },
    { "Require/aliases",       "./Require/aliases" },
    { "Require/circular",      "./Require/circular" },
    { "Require/bytecode",      "./Require/bytecode" },
    { "Require/concurrent",    "./Require/concurrent" },

    -- Runtime
    { "Runtime/error_handler", "./Runtime/error_handler" },
    { "Runtime/shutdown",      "./Runtime/shutdown" },
    { "Runtime/globals",       "./Runtime/globals" },
    { "Runtime/traceback",     "./Runtime/traceback" },
    { "Runtime/debug",         "./Runtime/debug" },
    { "Runtime/loadlib_errors","./Runtime/loadlib_errors" },

    -- Task
    { "Task/wait",             "./Task/wait" },
    { "Task/spawn",            "./Task/spawn" },
    { "Task/delay",            "./Task/delay" },
    { "Task/cancel",           "./Task/cancel" },
    { "Task/defer",            "./Task/defer" },
    { "Task/precision",        "./Task/precision" },
    { "Task/stress",           "./Task/stress" },
    { "Task/stress_extreme",   "./Task/stress_extreme" },
}

local function run_suite(suite_id, require_path)
    local ok, suite_mod = pcall(function()
        return require(require_path)
    end)

    if not ok then
        report:fail(suite_id, "(load)", tostring(suite_mod))
        print(RED .. "  FAIL  " .. RESET .. suite_id .. " > (load)")
        print("        " .. DIM .. tostring(suite_mod) .. RESET)
        return
    end

    if type(suite_mod) ~= "table" then
        report:fail(suite_id, "(load)", "suite did not return a table")
        print(RED .. "  FAIL  " .. RESET .. suite_id .. " > (load)")
        print("        " .. DIM .. "suite did not return a table" .. RESET)
        return
    end

    local suite_name = suite_mod.name or suite_id

    if suite_mod.beforeAll and type(suite_mod.beforeAll) == "function" then
        local ok2, err = pcall(suite_mod.beforeAll)
        if not ok2 then
            report:fail(suite_name, "(beforeAll)", tostring(err))
            print(RED .. "  FAIL  " .. RESET .. suite_name .. " > (beforeAll)")
            print("        " .. DIM .. tostring(err) .. RESET)
            return
        end
    end

    for i = 1, #suite_mod do
        local entry = suite_mod[i]
        if type(entry) ~= "table" or type(entry.run) ~= "function" then
            report:fail(suite_name, "[#" .. i .. "]", "entry missing .run function")
            print(RED .. "  FAIL  " .. RESET .. suite_name .. " > [#" .. i .. "]")
            print("        " .. DIM .. "entry missing .run function" .. RESET)
            continue
        end

        local name = entry.name or "test #" .. i
        local before = entry.before or suite_mod.beforeEach
        local after = entry.after or suite_mod.afterEach

        local ok2, err = pcall(function()
            if before and type(before) == "function" then
                before(expect)
            end
            entry.run(expect)
            if after and type(after) == "function" then
                after(expect)
            end
        end)

        -- Flush scheduler: ensure any spawned tasks complete before next test
        task.wait(0.01)

        if ok2 then
            report:pass(suite_name, name)
            print(GREEN .. "  PASS  " .. RESET .. suite_name .. " > " .. name)
        else
            report:fail(suite_name, name, tostring(err))
            print(RED .. "  FAIL  " .. RESET .. suite_name .. " > " .. name)
            print("        " .. DIM .. tostring(err) .. RESET)
        end
    end

    if suite_mod.afterAll and type(suite_mod.afterAll) == "function" then
        local ok2, err = pcall(suite_mod.afterAll)
        if not ok2 then
            report:fail(suite_name, "(afterAll)", tostring(err))
            print(RED .. "  FAIL  " .. RESET .. suite_name .. " > (afterAll)")
            print("        " .. DIM .. tostring(err) .. RESET)
        end
    end
end

print("")
print("========================================================")
print("         Nicy Test Runner — All Suites")
print("========================================================")
print("")

-- Pre-require fixtures to populate the module cache
-- This prevents "module not found" noise during test execution
require("./Require/fixtures/simple")
require("./Require/fixtures/nested")

for _, entry in ipairs(suite_paths) do
    run_suite(entry[1], entry[2])
end

-- Print summary
report:print()

local total_failures = #report.failures
if total_failures > 0 then
    warn("Some tests failed!")
end
