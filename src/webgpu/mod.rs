use super::{
    context,
    context::*,
    error::{gpu_api_err, GResult, GpuError},
};
use js_sys::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::*;

use attachment_image::WebGpuAttachmentImage;
use bind_groups::WebGpuBindGroups;
use buffer::{WebGpuBuffer, WebGpuDynamicBuffer};
use flags::{GpuBufferUsageFlags, GpuMapModeFlags, GpuShaderStageFlags, GpuTextureUsageFlags};
use pass::WebGpuCompiledPass;
use program::WebGpuProgram;
use sampler::WebGpuSamplerCache;
use surface::WebGpuSurface;
use texture::{WebGpuMipmapStateCache, WebGpuTexture};

use extensions::compute::{WebGpuCompiledComputePass, WebGpuComputeProgram};

pub const WEBGPU_COLOR_ATTACHMENT_FORMAT: GpuTextureFormat = GpuTextureFormat::Rgba8unorm;
pub const WEBGPU_DEPTH_ATTACHMENT_FORMAT: GpuTextureFormat = GpuTextureFormat::Depth24plusStencil8;

mod attachment_image;
mod bind_groups;
mod buffer;
mod extensions;
mod flags;
mod pass;
mod program;
mod sampler;
mod submit;
mod surface;
mod texture;

pub struct WebGpuContext {
    adapter: GpuAdapter,
    device: GpuDevice,
    surface: Option<WebGpuSurface>,

    vbos: Vec<WebGpuBuffer>,
    ibos: Vec<WebGpuBuffer>,
    ubos: Vec<WebGpuBuffer>,
    dyn_ubos: Vec<WebGpuDynamicBuffer>,
    ssbos: Vec<WebGpuBuffer>,
    programs: Vec<WebGpuProgram>,
    compute_programs: Vec<WebGpuComputeProgram>,
    compiled_passes: Vec<WebGpuCompiledPass>,
    compiled_compute_passes: Vec<WebGpuCompiledComputePass>,
    textures: Vec<WebGpuTexture>,
    attachment_images: Vec<WebGpuAttachmentImage>,
    sampler_cache: WebGpuSamplerCache,
    mipmap_state_cache: WebGpuMipmapStateCache,
}

impl WebGpuContext {
    pub fn new(extensions: Extensions) -> GResult<Self> {
        extensions::check_extensions(&extensions, false)?;

        //  Take adapter, device, and canvas id from WebGpuInitFromWindow extension.
        let (adapter, device, canvas_id) = extensions
            .extensions
            .iter()
            .find_map(|ext| {
                if let Extension::WebGpuInitFromWindow(init) = ext.clone() {
                    Some(Self::init_from_window(init))
                } else {
                    None
                }
            })
            .ok_or(gpu_api_err!(
                "webgpu expected extension WebGpuInitFromWindow (not WebGpuInit) to be used"
            ))??;

        Self::new_with(adapter, device, canvas_id)
    }

    pub async fn async_new(extensions: Extensions) -> GResult<Self> {
        extensions::check_extensions(&extensions, true)?;

        //  Take adapter, device, and canvas id from WebGpuInitFromWindow extension.
        let mut init_extension = None;
        for ext in extensions.extensions.iter() {
            if let Extension::WebGpuInit(init) = ext.clone() {
                init_extension = Some(Self::init(init).await);
                break;
            }
        }
        let (adapter, device, canvas_id) = init_extension.ok_or(gpu_api_err!(
            "webgpu expected extension WebGpuInit (not WebGpuInitFromWindow) to be used"
        ))??;

        Self::new_with(adapter, device, canvas_id)
    }

    fn new_with(
        adapter: GpuAdapter,
        device: GpuDevice,
        canvas_id: Option<String>,
    ) -> GResult<Self> {
        let window = window().unwrap();

        //  Optionally configure canvas.
        let surface = if let Some(canvas_id) = canvas_id {
            let navigator = window.navigator();
            let canvas = window
                .document()
                .unwrap()
                .get_element_by_id(&canvas_id)
                .ok_or(gpu_api_err!(
                    "webgpu canvas element with id `{}` does not exist",
                    canvas_id
                ))?
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .map_err(|_| {
                    gpu_api_err!(
                        "webgpu canvas element with id `{}` is not a canvas",
                        canvas_id
                    )
                })?;
            let context = canvas
                .get_context("webgpu")
                .map_err(|e| gpu_api_err!("webgpu could not get canvas context (1): {:?}", e))?
                .ok_or(gpu_api_err!("webgpu could not get canvas context (2)"))?
                .dyn_into::<GpuCanvasContext>()
                .map_err(|e| gpu_api_err!("webgpu did not get GpuCanvasContext: {:?}", e))?;

            let device_pixel_ratio = window.device_pixel_ratio();
            let device_pixel_ratio = if device_pixel_ratio == 0.0 {
                1.0
            } else {
                device_pixel_ratio
            };
            canvas.set_width((canvas.client_width() as f64 * device_pixel_ratio) as u32);
            canvas.set_height((canvas.client_height() as f64 * device_pixel_ratio) as u32);
            let present_format = navigator.gpu().get_preferred_canvas_format();

            let canvas_config_info = GpuCanvasConfiguration::new(&device, present_format);
            context.configure(&canvas_config_info);

            Some(WebGpuSurface {
                canvas,
                context,
                present_format,
            })
        } else {
            None
        };

        let mipmap_state_cache = WebGpuMipmapStateCache::new(&device);

        Ok(WebGpuContext {
            adapter,
            device,
            surface,

            vbos: vec![],
            ibos: vec![],
            ubos: vec![],
            dyn_ubos: vec![],
            ssbos: vec![],
            programs: vec![],
            compute_programs: vec![],
            compiled_passes: vec![],
            compiled_compute_passes: vec![],
            textures: vec![],
            attachment_images: vec![],
            sampler_cache: WebGpuSamplerCache::new(),
            mipmap_state_cache,
        })
    }
}
