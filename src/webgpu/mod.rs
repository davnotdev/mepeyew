use super::{
    context::*,
    error::{gpu_api_err, GResult, GpuError},
};
use js_sys::*;
use std::collections::HashSet;
use wasm_bindgen::prelude::*;
use web_sys::*;

use attachment_image::WebGpuAttachmentImage;
use buffer::WebGpuBuffer;
use flags::{GpuBufferUsageFlags, GpuShaderStageFlags, GpuTextureUsageFlags};
use pass::WebGpuCompiledPass;
use program::WebGpuProgram;
use sampler::WebGpuSamplerCache;
use surface::WebGpuSurface;
use texture::WebGpuTexture;

pub const WEBGPU_COLOR_ATTACHMENT_FORMAT: GpuTextureFormat = GpuTextureFormat::Rgba8unorm;
pub const WEBGPU_DEPTH_ATTACHMENT_FORMAT: GpuTextureFormat = GpuTextureFormat::Depth24plusStencil8;

mod attachment_image;
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
    enabled_extensions: HashSet<ExtensionType>,

    vbos: Vec<WebGpuBuffer>,
    ibos: Vec<WebGpuBuffer>,
    ubos: Vec<WebGpuBuffer>,
    ssbos: Vec<WebGpuBuffer>,
    programs: Vec<WebGpuProgram>,
    compiled_passes: Vec<WebGpuCompiledPass>,
    textures: Vec<WebGpuTexture>,
    attachment_images: Vec<WebGpuAttachmentImage>,
    sampler_cache: WebGpuSamplerCache,
}

impl WebGpuContext {
    pub fn new(extensions: &[Extension]) -> GResult<Self> {
        let supported_extensions = extensions::supported_extensions();
        let (enabled_extensions, unsupported_extensions): (Vec<_>, Vec<_>) = extensions
            .iter()
            .map(|ext| ext.get_type())
            .partition(|ty| supported_extensions.contains(ty));
        let enabled_extensions = enabled_extensions.into_iter().collect::<HashSet<_>>();
        if !unsupported_extensions.is_empty() {
            Err(gpu_api_err!(
                "webgpu these extensions not supported: {:?}",
                unsupported_extensions
            ))?;
        }

        //  Take adapter, device, and canvas id from WebGpuInit extension.
        let (adapter_str, device_str, canvas_id) = extensions
            .iter()
            .find_map(|ext| {
                if let Extension::WebGpuInitFromWindow(init) = ext.clone() {
                    Some((init.adapter, init.device, init.canvas_id))
                } else {
                    None
                }
            })
            .ok_or(gpu_api_err!(
                "webgpu expected extension WebGpuInit to be used."
            ))?;

        let window = window().unwrap();

        let window_flabby: &JsValue = &window;

        let adapter_key = JsValue::from_str(&adapter_str);
        let device_key = JsValue::from_str(&device_str);

        let adapter = Reflect::get(window_flabby, &adapter_key)
            .map_err(|e| gpu_api_err!("webgpu window.{} does not exist: {:?}", adapter_str, e))?
            .dyn_into::<GpuAdapter>()
            .map_err(|e| {
                gpu_api_err!("webgpu window.{} is not a GPUAdapter: {:?}", adapter_str, e)
            })?;
        let device = Reflect::get(window_flabby, &device_key)
            .map_err(|e| gpu_api_err!("webgpu window.{} does not exist: {:?}", device_str, e))?
            .dyn_into::<GpuDevice>()
            .map_err(|e| {
                gpu_api_err!("webgpu window.{} is not a GPUDevice: {:?}", device_str, e)
            })?;

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

        Ok(WebGpuContext {
            adapter,
            device,
            surface,
            enabled_extensions,

            vbos: vec![],
            ibos: vec![],
            ubos: vec![],
            ssbos: vec![],
            programs: vec![],
            compiled_passes: vec![],
            textures: vec![],
            attachment_images: vec![],
            sampler_cache: WebGpuSamplerCache::new(),
        })
    }
}
