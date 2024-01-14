use crate::state::State;

use winit::event::{Event, WindowEvent, DeviceEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

use anyhow::Result;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// runner
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    result_runner().await.unwrap()
}

// result runner
pub async fn result_runner() -> Result<()> {
    // setup logging
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));

            console_log::init_with_level(log::Level::Info)
                .expect("error occured when initializing logger");
        } else {
            env_logger::init();
        }
    }

    // create event loop
    let event_loop = EventLoop::new()?;

    // create window
    let window = WindowBuilder::new().build(&event_loop)?;

    // setup window
    window.set_title("Blocks");
    window.set_cursor_grab(winit::window::CursorGrabMode::Locked)?;
    window.set_cursor_visible(false);

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;

        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let body = doc.body()?;

                // create canvas
                let canvas = window.canvas()?;
                body.append_child(&canvas).ok()?;

                Some(())
            });
    }

    // app state
    let mut state: Option<State> = None;

    // last render time
    let mut last_render = instant::Instant::now();

    // fps count
    let mut frames = 0;
    let mut last_frames = instant::Instant::now();

    let _ = event_loop.run(move |event, elwt| {
        if let Some(state) = state.as_mut() {
            match event {
                Event::WindowEvent { ref event, .. } => {
                    if !state.event(event) {
                        match event {
                            // quit when requested
                            WindowEvent::CloseRequested => {
                                elwt.exit();
                            }

                            // window resized
                            WindowEvent::Resized(physical_size) => {
                                state.resize(*physical_size);
                                window.request_redraw();
                            }

                            // scale factor changed
                            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                                state.scale(scale_factor);
                                window.request_redraw();
                            }

                            // update screen
                            WindowEvent::RedrawRequested => {
                                let now = instant::Instant::now();

                                // time since last render
                                let dt = now - last_render;
                                last_render = now;

                                frames += 1;

                                // log fps
                                if (now - last_frames).as_secs() >= 1 {
                                    log::info!("fps: {}", frames);

                                    frames = 0;
                                    last_frames = now;
                                }

                                // update and render
                                state.update(dt);
                                state.render().unwrap();

                                window.request_redraw();
                            }

                            #[cfg(target_arch = "wasm32")]
                            WindowEvent::MouseInput { .. } => {
                                // hide and lock cursor
                                window.set_cursor_grab(winit::window::CursorGrabMode::Locked).unwrap();
                                window.set_cursor_visible(false);
                            }

                            _ => {},
                        }
                    }
                }

                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta },
                    ..
                } => {
                    // wasm movement fix
                    #[cfg(target_arch = "wasm32")]
                    if delta.0 == 0.0 && delta.1 == 0.0 {
                        return;
                    }

                    // mouse update
                    state.camera_controller.mouse(delta.0, delta.1);
                }

                _ => {}
            }
        } else if let Event::WindowEvent { event: WindowEvent::Resized(_), .. } = event {
            // create state on first resize
            state = Some(
                pollster::block_on(State::new(&window)).unwrap()
            );
        }
    });

    Ok(())
}
