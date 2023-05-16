//  https://webgpu.rocks/reference/typedef/gpubufferusageflags/#idl-gpubufferusageflags
#[repr(u32)]
pub enum GpuBufferUsageFlags {
    MapRead = 0x0001,
    MapWrite = 0x0002,
    CopySrc = 0x0004,
    CopyDst = 0x0008,
    Index = 0x0010,
    Vertex = 0x0020,
    Uniform = 0x0040,
    Storage = 0x0080,
    InDirect = 0x0100,
    QueryResolve = 0x0200,
}

