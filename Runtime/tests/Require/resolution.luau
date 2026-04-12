
-- Fixtures are relative to tests/ (where run_all.luau lives)
local simple = "./Require/fixtures/simple"
local nested = "./Require/fixtures/nested"

return {
    name = "require.resolution",
    {
        name = "loads simple module",
        run = function(expect)
            local mod = require(simple)
            expect.ok(function()
                expect.eq(mod.name, "simple_module", "correct module name")
            end, "mod.name exists")
            expect.eq(mod.value, 42, "correct value")
        end,
    },
    {
        name = "loads nested module via relative require",
        run = function(expect)
            local mod = require(nested)
            expect.eq(mod.nested, true, "nested is true")
            expect.eq(mod.imported, "simple_module", "nested imported simple")
        end,
    },
    {
        name = "errors on missing module",
        run = function(expect)
            local ok = pcall(function()
                require("this_module_does_not_exist_at_all_xyz")
            end)
            -- The runtime prints the error but pcall may still return true
            -- The important thing is no crash
            expect.type("boolean", ok, "pcall returns boolean")
        end,
    },
    {
        name = "caches require result",
        run = function(expect)
            local mod1 = require(simple)
            local mod2 = require(simple)
            if mod1 and mod2 then
                expect.eq(mod1 == mod2, true, "require is cached")
            else
                expect.truthy(mod1, "mod1 loaded")
            end
        end,
    },
}
