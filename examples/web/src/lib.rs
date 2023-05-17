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
        @vertex
        fn main(
          @builtin(vertex_index) VertexIndex : u32
        ) -> @builtin(position) vec4<f32> {
          var pos = array<vec3<f32>, 3>(
            vec3(0.0, 0.5),
            vec2(-0.5, -0.5),
            vec2(0.5, -0.5)
          );

          return vec4<f32>(pos[VertexIndex], 0.0, 1.0);
        }"#;

    let fs = r#"
        @fragment
        fn main() -> @location(0) vec4<f32> {
          return vec4(1.0, 0.0, 0.0, 1.0);
        }"#;

    let vs_reflect = ShaderType::Vertex(VertexBufferInput { args: vec![] });
    let fs_reflect = ShaderType::Fragment;

    let program = context
        .new_program(
            &ShaderSet::shaders(&[(vs_reflect, vs.as_bytes()), (fs_reflect, fs.as_bytes())]),
            &[],
            None,
        )
        .unwrap();
}
