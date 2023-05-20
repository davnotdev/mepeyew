//  https://webgpu.rocks/reference/typedef/gpubufferusageflags/#idl-gpubufferusageflags
#[allow(unused)]
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
//  https://webgpu.rocks/reference/typedef/gpushaderstageflags/#idl-gpushaderstageflags
#[allow(unused)]
#[repr(u8)]
pub enum GpuShaderStageFlags {
    VERTEX = 0x1,
    FRAGMENT = 0x2,
    COMPUTE = 0x4,
}

//  https://webgpu.rocks/reference/typedef/gputextureusageflags/#idl-gputextureusageflags
#[allow(unused)]
#[repr(u32)]
pub enum GpuTextureUsageFlags {
    CopySrc = 0x01,
    CopyDst = 0x02,
    TextureBinding = 0x04,
    StorageBinding = 0x08,
    RenderAttachment = 0x10,
}
