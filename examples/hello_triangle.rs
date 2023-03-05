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

    let mut context = Context::new(
        &window.raw_display_handle(),
        &window.raw_window_handle(),
        1024,
        640,
    )
    .unwrap();

    let vs = include_bytes!("shaders/hello_triangle/vs.spv");
    let fs = include_bytes!("shaders/hello_triangle/fs.spv");

    let program = context
        .new_program(&ShaderSet::shaders(&[
            (ShaderType::Vertex, vs),
            (ShaderType::Fragment, fs),
        ]))
        .unwrap();

    #[rustfmt::skip]
    let vertex_data: Vec<VertexBufferElement> = vec![
         0.0,  0.5, 0.0,
        -0.5, -0.5, 0.0,
         0.5, -0.5, 0.0,
    ];

    #[rustfmt::skip]
    let index_data: Vec<IndexBufferElement> = vec![
        0, 1, 2
    ];

    let vbo = context
        .new_vertex_buffer(&vertex_data, BufferStorageType::Static)
        .unwrap();
    let ibo = context
        .new_index_buffer(&index_data, BufferStorageType::Dynamic)
        .unwrap();

    let mut pass = Pass::new(Some(PassInputLoadOpColorType::Clear));
    let output_attachment = pass.get_output_attachment();
    {
        let pass_step = pass.add_step();
        pass_step
            .add_vertex_buffer(vbo)
            .set_index_buffer(ibo)
            .set_program(program)
            .add_write_color(output_attachment);
    }

    let compiled_pass = context.compile_pass(&pass).unwrap();

    //
    //  --- End Setup Code ---
    //

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::RedrawRequested(_) => {
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
                            r: 1.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        },
                    );
                    pass_submit.step(step_submit);
                }

                submit.pass(pass_submit);
                context.submit(submit).unwrap();

                //
                //  --- End Render Code ---
                //
            }
            _ => (),
        }
    });
}
