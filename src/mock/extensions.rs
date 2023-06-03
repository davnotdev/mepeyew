use super::*;

#[allow(dead_code)]
impl MockContext {
    pub fn memory_flush_extension_flush_memory(&mut self) {
        unimplemented!("No backend chosen")
    }

    #[cfg(feature = "surface_extension")]
    pub fn surface_extension_set_surface_size(
        &mut self,
        _width: usize,
        _height: usize,
    ) -> GResult<()> {
        unimplemented!("No backend chosen")
    }
}
