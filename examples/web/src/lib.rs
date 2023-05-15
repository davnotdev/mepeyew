use mepeyew::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    run();
}

pub fn run() {
    let context = Context::new(&[(Api::WebGpu, &[])]).unwrap();
}
