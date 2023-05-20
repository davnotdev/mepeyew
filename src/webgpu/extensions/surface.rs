use super::*;

impl WebGpuContext {
    pub fn surface_extension_set_surface_size(
        &mut self,
        _width: usize,
        _height: usize,
    ) -> GResult<()> {
        //  Ok, this seems to break more than it fixes.
        // //  Hmmm.
        // let surface_texture = self.surface.as_ref().unwrap().context.get_current_texture();

        // let width = surface_texture.width() as usize;
        // let height = surface_texture.height() as usize;

        // //  Resize Attachment Images.
        // for attachment_image in self.attachment_images.iter_mut() {
        //     attachment_image.recreate_with_new_size(&self.device, width, height);
        // }

        // //  Resize Dependent Passes.
        // for pass_idx in 0..self.compiled_passes.len() {
        //     let pass = &self.compiled_passes[pass_idx];
        //     let new_pass =
        //         WebGpuCompiledPass::new(self, &pass.original_pass, Some(pass.ext.clone()))?;

        //     let pass = &mut self.compiled_passes[pass_idx];
        //     *pass = new_pass;
        // }

        Ok(())
    }
}
