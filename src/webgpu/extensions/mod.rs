use super::*;

pub fn supported_extensions() -> &'static [ExtensionType] {
    &[
        // ExtensionType::Surface,
        // ExtensionType::FlightFramesCount,
        // ExtensionType::GpuPowerLevel,
        // ExtensionType::NativeDebug,
        // ExtensionType::MemoryFlush,
        // ExtensionType::ShaderReflection,
        ExtensionType::WebGpuInit,
    ]
}

impl WebGpuContext {
    pub fn extension_is_enabled(&self, ty: ExtensionType) -> bool {
        self.enabled_extensions.contains(&ty)
    }
}
