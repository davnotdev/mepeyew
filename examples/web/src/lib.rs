use mepeyew::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn main(adapter: JsValue, device: JsValue) {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    run(adapter, device);
}

pub fn run(adapter: JsValue, device: JsValue) {
    let context = Context::new(&[(
        Api::WebGpu,
        &[
            Extension::WebGpuInit(webgpu_init::WebGpuInit {
                adapter,
                device,
                canvas_id: Some(String::from("canvas")),
            }),
        ],
    )])
    .unwrap();
}
