
return {
    name = "require.circular dependencies",
    {
        name = "basic require works",
        run = function(expect)
            local ok = pcall(function()
                require("./Require/fixtures/simple")
            end)
            expect.eq(ok, true, "basic require works")
        end,
    },
}
