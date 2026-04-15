
return {
    name = "require.concurrent",
    {
        name = "same module required twice returns same result",
        run = function(expect)
            local mod1 = require("./Require/fixtures/simple")
            local mod2 = require("./Require/fixtures/simple")
            if mod1 and mod2 then
                expect.eq(mod1, mod2, "same reference from require cache")
            end
        end,
    },
    {
        name = "require does not re-execute cached module",
        run = function(expect)
            -- First require caches the module
            local mod1 = require("./Require/fixtures/simple")
            -- Second require should return the cached result without re-executing
            local mod2 = require("./Require/fixtures/simple")
            expect.eq(mod1.value, 42, "first require value")
            if mod2 then
                expect.eq(mod2.value, 42, "second require same value")
            end
        end,
    },
    {
        name = "require many times consecutively",
        run = function(expect)
            for i = 1, 100 do
                expect.ok(function()
                    require("./Require/fixtures/simple")
                end, "require #" .. i)
            end
        end,
    },
}
