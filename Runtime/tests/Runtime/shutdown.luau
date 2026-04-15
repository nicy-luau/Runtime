local runtime = _G.runtime

return {
    name = "Runtime.shutdown",
    {
        name = "state intact after operations (H5)",
        run = function(expect)
            for i = 1, 30 do
                local v = runtime.version
                expect.type("string", v, "runtime.version accessible")
            end
        end,
    },
    {
        name = "globals not corrupted (H5)",
        run = function(expect)
            require("./Require/fixtures/simple")
            local v = runtime.entry_file
            expect.type("string", v, "runtime.entry_file still valid")
        end,
    },
}
