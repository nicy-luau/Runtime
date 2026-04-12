
local simple = "./fixtures/simple"
local nested = "./fixtures/nested"

return {
    name = "require.relative paths",
    {
        name = "relative require works",
        run = function(expect)
            local mod = require(simple)
            if mod then
                expect.eq(mod.value, 42, "relative require works")
            end
        end,
    },
    {
        name = "nested relative paths",
        run = function(expect)
            expect.ok(function()
                require(nested)
            end, "nested relative require")
        end,
    },
}
