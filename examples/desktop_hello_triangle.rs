use mepeyew::*;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let window_size = window.inner_size();

    //
    //  --- Begin Setup Code ---
    //

    let mut extensions = Extensions::new();
    extensions
        .native_debug(Default::default())
        .naga_translation()
        .surface_extension(SurfaceConfiguration {
            width: window_size.width as usize,
            height: window_size.height as usize,
            display: window.display_handle().unwrap().as_raw(),
            window: window.window_handle().unwrap().as_raw(),
        });

    let mut context = Context::new(extensions, None).unwrap();

    let vs = include_bytes!("shaders/hello_triangle/vs.wgsl");
    let fs = include_bytes!("shaders/hello_triangle/fs.wgsl");

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

    let program = context
        .new_program(
            &ShaderSet::shaders(&[
                (
                    ShaderType::Vertex(VertexBufferInput { args: vec![3, 3] }),
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
        window_size.width as usize,
        window_size.height as usize,
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
                    let window_size = window.inner_size();

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
                                width: window_size.width as f32,
                                height: window_size.height as f32,
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
