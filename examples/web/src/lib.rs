use mepeyew::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn main(adapter: JsValue, device: JsValue) {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    run(adapter, device);
}

pub fn run(adapter: JsValue, device: JsValue) {
    let mut context = Context::new(&[(
        Api::WebGpu,
        &[Extension::WebGpuInit(webgpu_init::WebGpuInit {
            adapter,
            device,
            canvas_id: Some(String::from("canvas")),
        })],
    )])
    .unwrap();

    let vs = r#"
        struct VertexOutput {
            @builtin(position) position : vec4<f32>,
            @location(0) color : vec3<f32>,
        }

        @vertex
        fn main(
            @location(0) position : vec3<f32>,
            @location(1) color : vec3<f32>
        ) -> VertexOutput {
            var output: VertexOutput;
            output.position = vec4<f32>(position, 1.0);
            output.color = color;
            return output;
        }"#;

    let fs = r#"
        @fragment
        fn main(
            @location(0) color: vec3<f32>
        ) -> @location(0) vec4<f32> {
            return vec4(color, 1.0);
        }"#;

    let vs_reflect = ShaderType::Vertex(VertexBufferInput {
        args: vec![VertexInputArgCount(3), VertexInputArgCount(3)],
    });
    let fs_reflect = ShaderType::Fragment;

    let program = context
        .new_program(
            &ShaderSet::shaders(&[(vs_reflect, vs.as_bytes()), (fs_reflect, fs.as_bytes())]),
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
}
