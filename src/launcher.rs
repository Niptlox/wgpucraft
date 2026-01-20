use log::{error, info};
#[cfg(feature = "tracy")]
use tracy_client::span;
use winit::{event::Event, event_loop::ControlFlow};

use winit::{event_loop::EventLoop, window::WindowBuilder};

use crate::{State, core::config::AppConfig};

pub fn run() {
    env_logger::init();

    info!("Booting WGPUCraft");

    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let config = AppConfig::load_or_default("config.json").unwrap_or_else(|err| {
        error!("Falling back to default config: {:?}", err);
        AppConfig::default()
    });

    let mut state = State::new(&window, config);
    state.initialize();

    event_loop
        .run(
            move |event, elwt: &winit::event_loop::EventLoopWindowTarget<()>| match event {
                Event::WindowEvent { window_id, event } if window_id == state.window.id() => {
                    state.handle_window_event(event, elwt)
                }
                Event::DeviceEvent { ref event, .. } => {
                    #[cfg(feature = "tracy")]
                    let _span = span!("handling device input");

                    state.handle_device_input(event, elwt);
                }
                Event::AboutToWait => {
                    state.handle_wait(elwt);
                }
                _ => (),
            },
        )
        .unwrap();
}
