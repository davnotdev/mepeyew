use mepeyew::*;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

fn main() {
    #[cfg(all(feature = "webgpu", target_arch = "wasm32", target_os = "unknown"))]
    wasm::init();

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
        .webgpu_init_from_window(WebGpuInitFromWindow {
            adapter: String::from("mepeyewAdapter"),
            device: String::from("mepeyewDevice"),
            canvas_id: Some(String::from("canvas")),
        });

    let mut context = Context::new(extensions, None).unwrap();

    let vs_pass_1 = include_bytes!("shaders/double_pass/vs_pass_1.wgsl");
    let vs_pass_2 = include_bytes!("shaders/double_pass/vs_pass_2.wgsl");
    let fs_pass_1 = include_bytes!("shaders/double_pass/fs_pass_1.wgsl");
    let fs_pass_2 = include_bytes!("shaders/double_pass/fs_pass_2.wgsl");

    let vs_pass_1 = context
        .naga_translate_shader_code(
            naga_translation::NagaTranslationStage::Vertex,
            naga_translation::NagaTranslationInput::Wgsl,
            vs_pass_1,
            Default::default(),
        )
        .unwrap();
    let fs_pass_1 = context
        .naga_translate_shader_code(
            naga_translation::NagaTranslationStage::Fragment,
            naga_translation::NagaTranslationInput::Wgsl,
            fs_pass_1,
            Default::default(),
        )
        .unwrap();
    let vs_pass_2 = context
        .naga_translate_shader_code(
            naga_translation::NagaTranslationStage::Vertex,
            naga_translation::NagaTranslationInput::Wgsl,
            vs_pass_2,
            Default::default(),
        )
        .unwrap();
    let fs_pass_2 = context
        .naga_translate_shader_code(
            naga_translation::NagaTranslationStage::Fragment,
            naga_translation::NagaTranslationInput::Wgsl,
            fs_pass_2,
            Default::default(),
        )
        .unwrap();

    let pass_output_attachment_image = context
        .new_attachment_image(
            window_size.0,
            window_size.1,
            AttachmentImageUsage::ColorAttachment,
            None,
        )
        .unwrap();

    let pass_output_uniform = ShaderUniform {
        set: 0,
        binding: 0,
        ty: ShaderUniformType::InputAttachment(pass_output_attachment_image),
    };

    let program_pass_1 = context
        .new_program(
            &ShaderSet::shaders(&[
                (
                    ShaderType::Vertex(VertexBufferInput { args: vec![3] }),
                    &vs_pass_1,
                ),
                (ShaderType::Fragment, &fs_pass_1),
            ]),
            &[],
            None,
        )
        .unwrap();

    let program_pass_2 = context
        .new_program(
            &ShaderSet::shaders(&[
                (
                    ShaderType::Vertex(VertexBufferInput { args: vec![3] }),
                    &vs_pass_2,
                ),
                (ShaderType::Fragment, &fs_pass_2),
            ]),
            &[pass_output_uniform],
            None,
        )
        .unwrap();

    #[rustfmt::skip]
    let vertex_data_pass_1: Vec<VertexBufferElement> = vec![
        -1.0,  1.0, 0.0,
        -1.0,  0.0, 0.0,
         0.0,  1.0, 0.0,
         0.0,  0.0, 0.0,
    ];

    #[rustfmt::skip]
    let vertex_data_pass_2: Vec<VertexBufferElement> = vec![
        -0.5,  0.5, 0.0,
        -0.5, -0.5, 0.0,
         0.5,  0.5, 0.0,
         0.5, -0.5, 0.0,
    ];

    #[rustfmt::skip]
    let index_data: Vec<IndexBufferElement> = vec![
        0, 1, 2, 
        2, 1, 3,
    ];

    let vbo_pass_1 = context
        .new_vertex_buffer(&vertex_data_pass_1, BufferStorageType::Static, None)
        .unwrap();
    let vbo_pass_2 = context
        .new_vertex_buffer(&vertex_data_pass_2, BufferStorageType::Static, None)
        .unwrap();
    let ibo = context
        .new_index_buffer(&index_data, BufferStorageType::Static, None)
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
    let pass_1_output = pass.add_attachment_color_image(
        pass_output_attachment_image,
        PassInputLoadOpColorType::Clear,
    );
    let surface_attachment = pass.get_surface_local_attachment();

    let first_dep = {
        let pass_step = pass.add_step();
        pass_step
            .add_vertex_buffer(vbo_pass_1)
            .set_index_buffer(ibo)
            .add_program(program_pass_1)
            .add_write_color(pass_1_output);

        pass_step.get_step_dependency()
    };
    {
        let pass_step = pass.add_step();
        pass_step
            .add_vertex_buffer(vbo_pass_2)
            .set_index_buffer(ibo)
            .add_program(program_pass_2)
            .add_write_color(surface_attachment)
            .read_local_attachment(pass_1_output)
            .set_wait_for_color_from_step(first_dep, ShaderStage::Fragment);
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
                            .draw_indexed(program_pass_1, 0, index_data.len())
                            .set_viewport(DrawViewport {
                                x: 0.0,
                                y: 0.0,
                                width: window_size.0 as f32,
                                height: window_size.1 as f32,
                            });

                        pass_submit.set_attachment_clear_color(
                            pass_1_output,
                            ClearColor {
                                r: 0.0,
                                g: 0.4,
                                b: 0.0,
                                a: 1.0,
                            },
                        );
                        pass_submit.step(step_submit);
                    }
                    {
                        let mut step_submit = StepSubmitData::new();
                        step_submit
                            .draw_indexed(program_pass_2, 0, index_data.len())
                            .set_viewport(DrawViewport {
                                x: 0.0,
                                y: 0.0,
                                width: window_size.0 as f32,
                                height: window_size.1 as f32,
                            });

                        pass_submit.set_attachment_clear_color(
                            surface_attachment,
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

    pub fn init() {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
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
