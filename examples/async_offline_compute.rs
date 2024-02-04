use mepeyew::*;

fn main() {
    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    {
        use pollster::FutureExt;
        async { run().await }.block_on();
    }

    #[cfg(all(feature = "webgpu", target_arch = "wasm32", target_os = "unknown"))]
    wasm_bindgen_futures::spawn_local(async {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        run().await
    });
}

async fn run() {
    let mut extensions = Extensions::new();
    extensions
        .native_debug(Default::default())
        .naga_translation()
        .webgpu_init(WebGpuInit {
            canvas_id: Some(String::from("canvas")),
        })
        .compute()
        .shader_storage_buffer_object();

    let mut context = Context::async_new(extensions, None).await.unwrap();

    let code = include_bytes!("./shaders/offline_compute/compute.comp");

    let code = context
        .naga_translate_shader_code(
            naga_translation::NagaTranslationStage::Compute,
            naga_translation::NagaTranslationInput::Glsl,
            code,
            Default::default(),
        )
        .unwrap();

    let (ssbo, ssbo_guard) = context
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
        .async_read_synced_shader_storage_buffer(ssbo_guard, None)
        .await
        .unwrap();

    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    eprintln!("{:?}", out);

    #[cfg(all(feature = "webgpu", target_arch = "wasm32", target_os = "unknown"))]
    {
        use js_sys::*;
        use wasm_bindgen::*;
        use web_sys::*;

        let array = Array::new();
        for val in out {
            array.push(&JsValue::from(val));
        }
        console::log_1(&array);
    }
}
