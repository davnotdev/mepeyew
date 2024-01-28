use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let _window_size = window.inner_size();

    //  Setup Code.

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
                    //  Resize Code.
                }
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    window_id,
                } if window_id == window.id() => {
                    let _window_size = window.inner_size();
                    //  Render Code.
                }
                _ => (),
            }
        })
        .unwrap();
}
