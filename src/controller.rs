use crate::app_state::EditorState;
use crate::renderer::Renderer;
use pixels::Pixels;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::Key;
use winit::window::Window;

pub struct Controller {}

impl Controller {
    pub fn new() -> Self {
        Self {}
    }

    pub fn handle_window_event(
        &self,
        event: WindowEvent,
        state: &mut EditorState,
        pixels: &mut Pixels,
        event_loop: &ActiveEventLoop,
        window: &Window,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Window close requested");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                println!("Window resized: {:?}", size);
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    eprintln!("Failed to resize pixel surface: {err}");
                    event_loop.exit();
                }
                if let Err(err) = pixels.resize_buffer(size.width, size.height) {
                    eprintln!("Failed to resize pixel buffer: {err}");
                    event_loop.exit();
                }
                window.request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    let _ = state.handle_command_key(&event);
                    if matches!(event.logical_key, Key::Character(_)) {
                        if let Some(text) = event.text.as_ref() {
                            for ch in text.chars() {
                                state.push_character(ch);
                            }
                        }
                    }

                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {}
            _ => {}
        }
    }

    pub fn handle_main_events_cleared(
        &self,
        state: &mut EditorState,
        window: &Window,
        now: std::time::Instant,
    ) {
        if state.tick_blink(now) {
            window.request_redraw();
        }
    }

    pub fn handle_redraw_requested(
        &self,
        renderer: &mut Renderer,
        pixels: &mut Pixels,
        state: &EditorState,
        event_loop: &ActiveEventLoop,
    ) {
        renderer.draw(pixels, state);
        if let Err(err) = pixels.render() {
            eprintln!("Render error: {err}");
            event_loop.exit();
        }
    }
}
