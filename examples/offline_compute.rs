use mepeyew::prelude::*;

fn main() {
    #[cfg(all(feature = "webgpu", target_arch = "wasm32", target_os = "unknown"))]
    wasm::init();

    let mut extensions = Extensions::new();
    extensions
        .native_debug(NativeDebugConfiguration::default())
        .naga_translation()
        .webgpu_init_from_window(WebGpuInitFromWindow {
            adapter: String::from("mepeyewAdapter"),
            device: String::from("mepeyewDevice"),
            canvas_id: Some(String::from("canvas")),
        });
    let mut context = Context::new(extensions).unwrap();

    let code = include_bytes!("./shaders/offline_compute/compute.comp");

    let code = context
        .naga_translation_extension_translate_shader_code(
            naga_translation::NagaTranslationStage::Compute,
            naga_translation::NagaTranslationInput::Glsl,
            code,
            naga_translation::NagaTranslationExtensionTranslateShaderCodeExt::default(),
        )
        .unwrap();

    let ssbo = context
        .new_shader_storage_buffer(&[1u32; 1024], None)
        .unwrap();
    let program = context
        .new_compute_program(
            &code,
            &[ShaderUniform {
                set: 0,
                binding: 0,
                ty: ShaderUniformType::ShaderStorageBuffer(ssbo),
            }],
            None,
        )
        .unwrap();

    let mut pass = ComputePass::new();
    pass.add_program(program);

    let compiled_pass = context.compile_compute_pass(pass, None).unwrap();

    let mut submit = Submit::new();
    submit.sync_shader_storage_buffer(ssbo);

    let mut compute_submit = ComputePassSubmitData::new(compiled_pass);
    compute_submit.dispatch(program, 1024, 1, 1);

    submit.compute_pass(compute_submit);

    context
        .submit(submit, Some(SubmitExt { sync: Some(()) }))
        .unwrap();

    let out: [u32; 1024] = context
        .read_synced_shader_storage_buffer(ssbo, None)
        .unwrap();

    eprintln!("{:?}", out);

    #[cfg(all(feature = "webgpu", target_arch = "wasm32", target_os = "unknown"))]
    panic!("{:?}", out);
}

#[cfg(all(feature = "webgpu", target_arch = "wasm32", target_os = "unknown"))]
mod wasm {
    pub fn init() {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }
}
