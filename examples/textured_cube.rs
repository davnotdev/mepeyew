use mepeyew::prelude::*;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use stb_image_rust::*;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    #[cfg(feature = "webgpu")]
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
            ],
        ),
        (
            Api::WebGpu,
            &[Extension::WebGpuInit(webgpu_init::WebGpuInit {
                adapter: String::from("mepeyewAdapter"),
                device: String::from("mepeyewDevice"),
                canvas_id: Some(String::from("canvas")),
            })],
        ),
    ])
    .unwrap();

    // let vs = include_bytes!("shaders/textured_cube/vs.spv");
    // let fs = include_bytes!("shaders/textured_cube/fs.spv");
    let vs_code = r#"
        struct VertexOutput {
            @builtin(position) position : vec4<f32>,
            @location(1) texture_coord : vec2<f32>,
        }

        @vertex
        fn main(
            @location(0) position : vec3<f32>,
            @location(1) texture_coord : vec2<f32>
        ) -> VertexOutput {
            var output: VertexOutput;
            output.position = vec4<f32>(position, 1.0);
            output.color = color;
            output.texture_coord = texture_coord;
            return output;
        }"#;

    let fs_code = r#"
        @group(0) @binding(0) var my_sampler: sampler;
        @group(0) @binding(1) var my_texture: texture_2d<f32>;

        @fragment
        fn main(
            @location(0) texture_coord: vec2<f32>
        ) -> @location(0) vec4<f32> {
            return textureSample(my_texture, my_sampler, texture_coord);
        }"#;
    let vs = vs_code.as_bytes();
    let fs = fs_code.as_bytes();

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
        .new_texture(x as usize, y as usize, sampler, TextureFormat::Rgba, None)
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

    let uniform = ShaderUniform {
        ty: ShaderUniformType::Texture(texture),
        binding: 0,
        frequency: ShaderUniformFrequencyHint::High,
    };

    let program = context
        .new_program(
            &ShaderSet::shaders(&[
                (
                    ShaderType::Vertex(VertexBufferInput {
                        args: vec![VertexInputArgCount(3), VertexInputArgCount(2)],
                    }),
                    vs,
                ),
                (ShaderType::Fragment, fs),
            ]),
            &[uniform],
            None,
        )
        .unwrap();

    #[rustfmt::skip]
    let vertex_data: Vec<VertexBufferElement> = vec![
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

    let vbo = context
        .new_vertex_buffer(&vertex_data, BufferStorageType::Static, None)
        .unwrap();
    let ibo = context
        .new_index_buffer(&index_data, BufferStorageType::Dynamic, None)
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
    });
}

#[cfg(feature = "webgpu")]
mod wasm {
    pub fn init() {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }
}
