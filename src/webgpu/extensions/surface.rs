use super::*;

impl WebGpuContext {
    pub fn surface_extension_set_surface_size(
        &mut self,
        _width: usize,
        _height: usize,
    ) -> GResult<()> {
        //  The width and height values are typically wrong when using winit.
        let surface = self.surface.as_ref().unwrap();
        let window = window().unwrap();

        let device_pixel_ratio = window.device_pixel_ratio();
        let device_pixel_ratio = if device_pixel_ratio == 0.0 {
            1.0
        } else {
            device_pixel_ratio
        };
        let width = (surface.canvas.client_width() as f64 * device_pixel_ratio) as usize;
        let height = (surface.canvas.client_height() as f64 * device_pixel_ratio) as usize;

        surface.canvas.set_width(width as u32);
        surface.canvas.set_height(height as u32);

        //  Resize Attachment Images.
        for attachment_image in self.attachment_images.iter_mut() {
            attachment_image.recreate_with_new_size(&self.device, width, height);
        }

        //  Resize Dependent Passes.
        for pass_idx in 0..self.compiled_passes.len() {
            let pass = &self.compiled_passes[pass_idx];
            let new_pass =
                WebGpuCompiledPass::new(self, &pass.original_pass, Some(pass.ext.clone()))?;

            let pass = &mut self.compiled_passes[pass_idx];
            *pass = new_pass;
        }

        Ok(())
    }
}
