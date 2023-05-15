use mepeyew::prelude::*;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    //
    //  --- Begin Setup Code ---
    //

    let mut context = Context::new(&[(
        Api::Vulkan,
        &[
            Extension::NativeDebug,
            Extension::Surface(surface::SurfaceConfiguration {
                width: 640,
                height: 480,
                display: window.raw_display_handle(),
                window: window.raw_window_handle(),
            }),
            Extension::ShaderReflection,
        ],
    )])
    .unwrap();

    let vs_pass_1 = include_bytes!("shaders/double_pass/vs_pass_1.spv");
    let vs_pass_2 = include_bytes!("shaders/double_pass/vs_pass_2.spv");
    let fs_pass_1 = include_bytes!("shaders/double_pass/fs_pass_1.spv");
    let fs_pass_2 = include_bytes!("shaders/double_pass/fs_pass_2.spv");

    let vs_pass_1_reflect = context
        .shader_reflection_extension_reflect(
            vs_pass_1,
            shader_reflection::ReflectionShaderTypeHint::Vertex,
        )
        .unwrap();
    let vs_pass_2_reflect = context
        .shader_reflection_extension_reflect(
            vs_pass_2,
            shader_reflection::ReflectionShaderTypeHint::Vertex,
        )
        .unwrap();
    let fs_pass_1_reflect = context
        .shader_reflection_extension_reflect(
            fs_pass_1,
            shader_reflection::ReflectionShaderTypeHint::Fragment,
        )
        .unwrap();
    let fs_pass_2_reflect = context
        .shader_reflection_extension_reflect(
            fs_pass_2,
            shader_reflection::ReflectionShaderTypeHint::Fragment,
        )
        .unwrap();

    let pass_output_attachment_image = context
        .new_attachment_image(
            640,
            480,
            AttachmentImageUsage::ColorAttachment,
            NewAttachmentImageExt::default(),
        )
        .unwrap();

    let pass_output_uniform = ShaderUniform {
        binding: 0,
        frequency: ShaderUniformFrequencyHint::High,
        ty: ShaderUniformType::InputAttachment(pass_output_attachment_image),
    };

    let program_pass_1 = context
        .new_program(
            &ShaderSet::shaders(&[
                (vs_pass_1_reflect, vs_pass_1),
                (fs_pass_1_reflect, fs_pass_1),
            ]),
            &[],
            None,
        )
        .unwrap();

    let program_pass_2 = context
        .new_program(
            &ShaderSet::shaders(&[
                (vs_pass_2_reflect, vs_pass_2),
                (fs_pass_2_reflect, fs_pass_2),
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
        -0.5,  0.5, 0.0, 0.0, 1.0,
        -0.5, -0.5, 0.0, 0.0, 0.0,
         0.5,  0.5, 0.0, 1.0, 1.0,
         0.5, -0.5, 0.0, 1.0, 0.0,
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
        640,
        480,
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
            .set_program(program_pass_1)
            .add_write_color(pass_1_output);

        pass_step.get_step_dependency()
    };
    {
        let pass_step = pass.add_step();
        pass_step
            .add_vertex_buffer(vbo_pass_2)
            .set_index_buffer(ibo)
            .set_program(program_pass_2)
            .add_write_color(surface_attachment)
            .read_local_attachment(pass_1_output)
            .set_wait_for_color_from_step(first_dep, ShaderType::Fragment);
    }

    let compiled_pass = context.compile_pass(&pass, None).unwrap();

    //
    //  --- End Setup Code ---
    //

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
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
                    step_submit.draw_indexed(0, index_data.len());

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
    });
}
