use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

fn main() {
    #[cfg(all(feature = "webgpu", target_arch = "wasm32", target_os = "unknown"))]
    wasm::init();

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let window_size = get_window_size(&window);

    //  Setup Code.

    let mut last_window_size = window_size;
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
                    event: WindowEvent::Resized(_),
                    window_id,
                } if window_id == window.id() => {
                    //  Resize Code (for non-Web).
                }
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    window_id,
                } if window_id == window.id() => {
                    let window_size = get_window_size(&window);
                    if last_window_size.0 != window_size.0 || last_window_size.1 != window_size.1 {
                        //  Resize Code (for Web).
                    }
                    last_window_size = window_size;
                    //  Render Code.
                }
                _ => (),
            }
        })
        .unwrap();
}

#[allow(unused_variables)]
fn get_window_size(window: &Window) -> (usize, usize) {
    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    return wasm::get_window_size();

    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    {
        let size = window.inner_size();
        (size.width as usize, size.height as usize)
    }
}

#[cfg(all(feature = "webgpu", target_arch = "wasm32", target_os = "unknown"))]
mod wasm {
    use wasm_bindgen::prelude::*;

    pub fn init() {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }

    pub fn get_window_size() -> (usize, usize) {
        let window = web_sys::window().unwrap();
        let canvas = window
            .document()
            .unwrap()
            .get_element_by_id("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        (
            canvas.client_width() as usize,
            canvas.client_height() as usize,
        )
    }
}
