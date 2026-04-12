return {
    name = "Core.bit32",
    {
        name = "bit32.band",
        run = function(expect)
            expect.eq(bit32.band(0xFF, 0x0F), 0x0F, "bitwise AND")
            expect.eq(bit32.band(0b1100, 0b1010), 0b1000, "bitwise AND binary")
        end,
    },
    {
        name = "bit32.bor",
        run = function(expect)
            expect.eq(bit32.bor(0xF0, 0x0F), 0xFF, "bitwise OR")
        end,
    },
    {
        name = "bit32.bxor",
        run = function(expect)
            expect.eq(bit32.bxor(0xFF, 0xFF), 0x00, "bitwise XOR same")
            expect.eq(bit32.bxor(0xFF, 0x00), 0xFF, "bitwise XOR zero")
        end,
    },
    {
        name = "bit32.bnot",
        run = function(expect)
            expect.eq(bit32.bnot(0), 0xFFFFFFFF, "bitwise NOT zero")
            expect.eq(bit32.band(bit32.bnot(0xFF), 0xFF), 0, "NOT + AND = 0")
        end,
    },
    {
        name = "bit32.lshift and rshift",
        run = function(expect)
            expect.eq(bit32.lshift(1, 8), 256, "left shift by 8")
            expect.eq(bit32.rshift(256, 8), 1, "right shift by 8")
        end,
    },
    {
        name = "bit32.arshift",
        run = function(expect)
            expect.eq(bit32.arshift(0x80000000, 1), 0xC0000000, "arshift preserves sign")
        end,
    },
    {
        name = "bit32.extract",
        run = function(expect)
            expect.eq(bit32.extract(0xFF, 0, 4), 0xF, "extract lower nibble")
            expect.eq(bit32.extract(0xFF00, 8, 8), 0xFF, "extract byte")
        end,
    },
    {
        name = "bit32.replace",
        run = function(expect)
            expect.eq(bit32.replace(0x0000, 0xFF, 0, 8), 0xFF, "replace lower byte")
        end,
    },
}
