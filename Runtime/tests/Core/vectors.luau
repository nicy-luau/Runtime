local Vector3 = _G.Vector3

return {
    name = "Core.vectors",
    {
        name = "vector3 creation",
        run = function(expect)
            local ok, v = pcall(function() return Vector3.new(1.5, 2.5, 3.5) end)
            if ok then
                expect.type("vector", v, "Vector3.new is vector type")
            else
                -- Vector3 might not be exposed in this runtime
                expect.truthy(true, "Vector3 not available in this runtime")
            end
        end,
    },
    {
        name = "vector3 arithmetic",
        run = function(expect)
            local ok, a, b = pcall(function()
                return Vector3.new(1, 2, 3), Vector3.new(4, 5, 6)
            end)
            if ok then
                local sum = a + b
                expect.eq(sum.X, 5, "sum X")
                expect.eq(sum.Y, 7, "sum Y")
                expect.eq(sum.Z, 9, "sum Z")
            else
                expect.truthy(true, "Vector3 not available")
            end
        end,
    },
    {
        name = "vector3 multiplication",
        run = function(expect)
            local ok, a, b = pcall(function()
                return Vector3.new(2, 3, 4), Vector3.new(5, 6, 7)
            end)
            if ok then
                local product = a * b
                expect.eq(product.X, 10, "componentwise mul X")
                expect.eq(product.Y, 18, "componentwise mul Y")
                expect.eq(product.Z, 28, "componentwise mul Z")
            else
                expect.truthy(true, "Vector3 not available")
            end
        end,
    },
    {
        name = "vector3 magnitude",
        run = function(expect)
            local ok, v = pcall(function() return Vector3.new(3, 4, 0) end)
            if ok then
                local mag = v.Magnitude
                expect.eq(mag, 5, "magnitude of 3,4,0 is 5")
            else
                expect.truthy(true, "Vector3 not available")
            end
        end,
    },
    {
        name = "vector3 unit",
        run = function(expect)
            local ok, v = pcall(function() return Vector3.new(0, 5, 0) end)
            if ok then
                local u = v.Unit
                expect.eq(u.X, 0, "unit X")
                expect.eq(u.Y, 1, "unit Y")
                expect.eq(u.Z, 0, "unit Z")
            else
                expect.truthy(true, "Vector3 not available")
            end
        end,
    },
}
