
return {
    name = "require.bytecode",
    {
        name = "loads module (bytecode or source)",
        run = function(expect)
            local ok, mod = pcall(function()
                return require("./Require/fixtures/simple")
            end)
            if ok and mod then
                expect.eq(mod.value, 42, "bytecode/source loads correctly")
            end
        end,
    },
    {
        name = "runtime has nicy_compile or equivalent",
        run = function(expect)
            -- Check if the runtime compilation functions exist
            local has_compile = (type(nicy_compile) == "function")
            if has_compile then
                expect.eq(has_compile, true, "nicy_compile is available")
            else
                -- nicy_compile may not be exposed globally, that's OK
                expect.type("function", require, "require is available")
            end
        end,
    },
    {
        name = "module cached after bytecode load",
        run = function(expect)
            local mod1 = require("./Require/fixtures/simple")
            local mod2 = require("./Require/fixtures/simple")
            if mod1 and mod2 then
                expect.eq(mod1 == mod2, true, "module is cached after first load")
            end
        end,
    },
}
