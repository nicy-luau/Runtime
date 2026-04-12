--[[
    tests/helpers/expect.luau
    Minimal assertion helper.
]]

local expect = {}

function expect.eq(a, b, label)
    if a ~= b then
        error(string.format("FAIL [%s]: expected %s, got %s",
            label or "eq", tostring(b), tostring(a)), 2)
    end
end

function expect.neq(a, b, label)
    if a == b then
        error(string.format("FAIL [%s]: expected values to differ", label or "neq"), 2)
    end
end

function expect.ok(fn, label)
    local success, err = pcall(fn)
    if not success then
        error(string.format("FAIL [%s]: %s", label or "ok", tostring(err)), 2)
    end
end

function expect.fails(fn, label)
    local success = pcall(fn)
    if success then
        error(string.format("FAIL [%s]: expected error but succeeded", label or "fails"), 2)
    end
end

function expect.type(expected, value, label)
    if typeof(value) ~= expected then
        error(string.format("FAIL [%s]: expected type %s, got %s",
            label or "type", expected, typeof(value)), 2)
    end
end

function expect.truthy(value, label)
    if not value then
        error(string.format("FAIL [%s]: expected truthy", label or "truthy"), 2)
    end
end

function expect.falsy(value, label)
    if value then
        error(string.format("FAIL [%s]: expected falsy", label or "falsy"), 2)
    end
end

return expect
