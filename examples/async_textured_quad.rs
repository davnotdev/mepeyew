use mepeyew::*;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use stb_image_rust::*;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

fn main() {
    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    {
        use pollster::FutureExt;
        async { run().await }.block_on();
    }

    #[cfg(all(feature = "webgpu", target_arch = "wasm32", target_os = "unknown"))]
    wasm_bindgen_futures::spawn_local(async { wasm::run().await });
}

async fn run() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let window_size = get_window_size(&window);

    //
    //  --- Begin Setup Code ---
    //

    let mut extensions = Extensions::new();
    extensions
        .native_debug(Default::default())
        .naga_translation()
        .surface_extension(SurfaceConfiguration {
            width: window_size.0,
            height: window_size.1,
            display: window.display_handle().unwrap().as_raw(),
            window: window.window_handle().unwrap().as_raw(),
        })
        .webgpu_init(WebGpuInit {
            canvas_id: Some("canvas".to_owned()),
        });

    let mut context = Context::async_new(extensions, None).await.unwrap();

    let vs = include_bytes!("shaders/textured_quad/vs.wgsl");
    let fs = include_bytes!("shaders/textured_quad/fs.wgsl");

    let vs = context
        .naga_translate_shader_code(
            naga_translation::NagaTranslationStage::Vertex,
            naga_translation::NagaTranslationInput::Wgsl,
            vs,
            Default::default(),
        )
        .unwrap();
    let fs = context
        .naga_translate_shader_code(
            naga_translation::NagaTranslationStage::Fragment,
            naga_translation::NagaTranslationInput::Wgsl,
            fs,
            Default::default(),
        )
        .unwrap();

    let sampler = context.get_sampler(None).unwrap();

    let image_bytes = include_bytes!("resources/photo.jpg");
    let mut x: i32 = 0;
    let mut y: i32 = 0;
    let mut comp: i32 = 0;
    let image: *mut u8;
    unsafe {
        image = stbi_load_from_memory(
            image_bytes.as_ptr(),
            image_bytes.len() as i32,
            &mut x as *mut i32,
            &mut y as *mut i32,
            &mut comp as *mut i32,
            STBI_rgb_alpha,
        );
    }

    let texture = context
        .new_texture(x as usize, y as usize, TextureFormat::Rgba, None)
        .unwrap();

    context
        .upload_texture(
            texture,
            unsafe {
                std::slice::from_raw_parts(image, (x * y) as usize * std::mem::size_of::<i32>())
            },
            None,
        )
        .unwrap();

    let texture_uniform = ShaderUniform {
        set: 0,
        binding: 0,
        ty: ShaderUniformType::Texture(texture),
    };
    let sampler_uniform = ShaderUniform {
        set: 0,
        binding: 1,
        ty: ShaderUniformType::Sampler(sampler),
    };

    let program = context
        .new_program(
            &ShaderSet::shaders(&[
                (
                    ShaderType::Vertex(VertexBufferInput { args: vec![3, 2] }),
                    &vs,
                ),
                (ShaderType::Fragment, &fs),
            ]),
            &[texture_uniform, sampler_uniform],
            None,
        )
        .unwrap();

    #[rustfmt::skip]
        let vertex_data: Vec<VertexBufferElement> = vec![
            -0.5,  0.5, 0.0, 0.0, 0.0,
            -0.5, -0.5, 0.0, 0.0, 1.0,
             0.5,  0.5, 0.0, 1.0, 0.0,
             0.5, -0.5, 0.0, 1.0, 1.0,
        ];

    #[rustfmt::skip]
        let index_data: Vec<IndexBufferElement> = vec![
            0, 1, 2,
            2, 1, 3,
        ];

    let vbo = context
        .new_vertex_buffer(&vertex_data, BufferStorageType::Static, None)
        .unwrap();
    let ibo = context
        .new_index_buffer(&index_data, BufferStorageType::Dynamic, None)
        .unwrap();

    let mut pass = Pass::new(
        window_size.0,
        window_size.1,
        Some(NewPassExt {
            depends_on_surface_size: Some(()),
            surface_attachment_load_op: Some(PassInputLoadOpColorType::Clear),
            ..Default::default()
        }),
    );
    let output_attachment = pass.get_surface_local_attachment();
    {
        let pass_step = pass.add_step();
        pass_step
            .add_vertex_buffer(vbo)
            .set_index_buffer(ibo)
            .add_program(program)
            .add_write_color(output_attachment);
    }

    let compiled_pass = context.compile_pass(&pass, None).unwrap();

    //
    //  --- End Setup Code ---
    //

    let mut last_window_size = window_size;
    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::wait_duration(
                std::time::Duration::from_millis(16),
            ));

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                } if window_id == window.id() => elwt.exit(),
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    window_id,
                } if window_id == window.id() => {
                    context
                        .set_surface_size(size.width as usize, size.height as usize)
                        .unwrap();
                }
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    window_id,
                } if window_id == window.id() => {
                    let window_size = get_window_size(&window);
                    if last_window_size.0 != window_size.0 || last_window_size.1 != window_size.1 {
                        context
                            .set_surface_size(window_size.0, window_size.1)
                            .unwrap();
                    }
                    last_window_size = window_size;

                    //
                    //  --- Begin Render Code ---
                    //

                    let mut submit = Submit::new();

                    let mut pass_submit = PassSubmitData::new(compiled_pass);

                    {
                        let mut step_submit = StepSubmitData::new();
                        step_submit
                            .draw_indexed(program, 0, index_data.len())
                            .set_viewport(DrawViewport {
                                x: 0.0,
                                y: 0.0,
                                width: window_size.0 as f32,
                                height: window_size.1 as f32,
                            });

                        pass_submit.set_attachment_clear_color(
                            output_attachment,
                            ClearColor {
                                r: 0.0,
                                g: 0.2,
                                b: 0.2,
                                a: 1.0,
                            },
                        );
                        pass_submit.step(step_submit);
                    }

                    submit.pass(pass_submit);
                    context.submit(submit, None).unwrap();

                    //
                    //  --- End Render Code ---
                    //
                }
                _ => (),
            }
        })
        .unwrap();
}

#[allow(unused_variables)]
fn get_window_size(window: &Window) -> (usize, usize) {
    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    return wasm::get_window_size();

    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    {
        let size = window.inner_size();
        (size.width as usize, size.height as usize)
    }
}

#[cfg(all(feature = "webgpu", target_arch = "wasm32", target_os = "unknown"))]
mod wasm {
    use wasm_bindgen::prelude::*;

    pub async fn run() {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        crate::run().await
    }

    pub fn get_window_size() -> (usize, usize) {
        let window = web_sys::window().unwrap();
        let canvas = window
            .document()
            .unwrap()
            .get_element_by_id("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        (
            canvas.client_width() as usize,
            canvas.client_height() as usize,
        )
    }
}
