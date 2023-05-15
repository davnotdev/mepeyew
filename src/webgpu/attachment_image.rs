use super::*;

impl WebGpuContext {
    pub fn new_attachment_image(
        &mut self,
        _initial_width: usize,
        _initial_height: usize,
        _attachment_usage: AttachmentImageUsage,
        _ext: Option<NewAttachmentImageExt>,
    ) -> GResult<AttachmentImageId> {
        todo!()
    }
}
