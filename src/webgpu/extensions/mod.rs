mod surface;

use super::*;

pub fn supported_extensions() -> &'static [ExtensionType] {
    &[
        ExtensionType::Surface,
        ExtensionType::WebGpuInitFromWindow,
        #[cfg(feature = "naga_translation")]
        ExtensionType::NagaTranslation,
    ]
}

impl WebGpuContext {
    pub fn extension_is_enabled(&self, ty: ExtensionType) -> bool {
        self.enabled_extensions.contains(&ty)
    }
}
