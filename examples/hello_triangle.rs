use mepeyew::prelude::*;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

fn main() {
    #[cfg(all(feature = "webgpu", target_arch = "wasm32", target_os = "unknown"))]
    wasm::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    //
    //  --- Begin Setup Code ---
    //

    let mut context = Context::new(&[
        (
            Api::Vulkan,
            &[
                Extension::NativeDebug,
                Extension::Surface(surface::SurfaceConfiguration {
                    width: 640,
                    height: 480,
                    display: window.raw_display_handle(),
                    window: window.raw_window_handle(),
                }),
                Extension::NagaTranslation,
            ],
        ),
        #[cfg(all(feature = "webgpu", target_arch = "wasm32", target_os = "unknown"))]
        (
            Api::WebGpu,
            &[
                Extension::WebGpuInit(webgpu_init::WebGpuInit {
                    adapter: String::from("mepeyewAdapter"),
                    device: String::from("mepeyewDevice"),
                    canvas_id: Some(String::from("canvas")),
                }),
                Extension::NagaTranslation,
            ],
        ),
    ])
    .unwrap();

    let vs = include_bytes!("shaders/hello_triangle/vs.wgsl");
    let fs = include_bytes!("shaders/hello_triangle/fs.wgsl");

    let vs = context
        .naga_translation_extension_translate_shader_code(
            naga_translation::NagaTranslationStage::Vertex,
            naga_translation::NagaTranslationInput::Wgsl,
            vs,
            naga_translation::NagaTranslationExtensionTranslateShaderCodeExt::default(),
        )
        .unwrap();
    let fs = context
        .naga_translation_extension_translate_shader_code(
            naga_translation::NagaTranslationStage::Fragment,
            naga_translation::NagaTranslationInput::Wgsl,
            fs,
            naga_translation::NagaTranslationExtensionTranslateShaderCodeExt::default(),
        )
        .unwrap();

    let program = context
        .new_program(
            &ShaderSet::shaders(&[
                (
                    ShaderType::Vertex(VertexBufferInput {
                        args: vec![VertexInputArgCount(3), VertexInputArgCount(3)],
                    }),
                    &vs,
                ),
                (ShaderType::Fragment, &fs),
            ]),
            &[],
            None,
        )
        .unwrap();

    #[rustfmt::skip]
    let vertex_data: Vec<VertexBufferElement> = vec![
         0.0,  0.5, 0.0, 1.0, 0.0, 0.0,
        -0.5, -0.5, 0.0, 0.0, 1.0, 0.0,
         0.5, -0.5, 0.0, 0.0, 0.0, 1.0,
    ];

    #[rustfmt::skip]
    let index_data: Vec<IndexBufferElement> = vec![
        0, 1, 2
    ];

    let vbo = context
        .new_vertex_buffer(&vertex_data, BufferStorageType::Static, None)
        .unwrap();
    let ibo = context
        .new_index_buffer(&index_data, BufferStorageType::Static, None)
        .unwrap();

    let mut pass = Pass::new(
        640,
        480,
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
            .set_program(program)
            .add_write_color(output_attachment);
    }

    let compiled_pass = context.compile_pass(&pass, None).unwrap();

    //
    //  --- End Setup Code ---
    //

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => control_flow.set_exit(),
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                window_id,
            } if window_id == window.id() => {
                context
                    .surface_extension_set_surface_size(size.width as usize, size.height as usize)
                    .unwrap();
            }
            Event::MainEventsCleared => {
                //
                //  --- Begin Render Code ---
                //

                let mut submit = Submit::new();

                let mut pass_submit = PassSubmitData::new(compiled_pass);

                {
                    let mut step_submit = StepSubmitData::new();
                    step_submit.draw_indexed(0, index_data.len());

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
                window.request_redraw();
            }
            _ => (),
        }
    });
}

#[cfg(all(feature = "webgpu", target_arch = "wasm32", target_os = "unknown"))]
mod wasm {
    pub fn init() {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }
}
