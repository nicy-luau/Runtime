return {
    name = "Core.buffers",
    {
        name = "buffer creation",
        run = function(expect)
            local b = buffer.create(16)
            expect.type("buffer", b, "buffer.create returns buffer")
        end,
    },
    {
        name = "buffer read and write bytes",
        run = function(expect)
            local b = buffer.create(8)
            buffer.writeu8(b, 0, 42)
            buffer.writeu8(b, 1, 255)
            expect.eq(buffer.readu8(b, 0), 42, "read byte 0")
            expect.eq(buffer.readu8(b, 1), 255, "read byte 1")
        end,
    },
    {
        name = "buffer write 16-bit",
        run = function(expect)
            local b = buffer.create(4)
            buffer.writeu16(b, 0, 65535)
            expect.eq(buffer.readu16(b, 0), 65535, "read 16-bit value")
        end,
    },
    {
        name = "buffer write 32-bit",
        run = function(expect)
            local b = buffer.create(4)
            buffer.writeu32(b, 0, 4294967295)
            expect.eq(buffer.readu32(b, 0), 4294967295, "read 32-bit value")
        end,
    },
    {
        name = "buffer length",
        run = function(expect)
            local b = buffer.create(64)
            local len = buffer.len(b)
            expect.eq(len, 64, "buffer length is 64")
        end,
    },
}
