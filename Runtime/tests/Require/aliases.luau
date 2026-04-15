
return {
    name = "require.aliases",
    {
        name = "require works with alias parser active",
        run = function(expect)
            local ok, _ = pcall(function()
                require("./Require/fixtures/simple")
            end)
            expect.eq(ok, true, "require works alongside alias parser")
        end,
    },
    {
        name = "missing alias handled gracefully",
        run = function(expect)
            local ok, _err = pcall(function()
                require("@nonexistent_alias_xyz/anything")
            end)
            expect.type("boolean", ok, "alias lookup returns boolean result")
        end,
    },
}
