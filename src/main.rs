mod app_state;
mod controller;
mod renderer;

use std::time::Instant;
use std::sync::Arc;

use app_state::EditorState;
use controller::Controller;
use pixels::{Pixels, SurfaceTexture};
use renderer::Renderer;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

struct App {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    renderer: Renderer,
    controller: Controller,
    state: EditorState,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            pixels: None,
            renderer: Renderer::new(),
            state: EditorState::new(),
            controller: Controller::new(),
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(Window::default_attributes()).unwrap());
        println!("Window size: {:?}", window.inner_size());
        let size = window.inner_size();
        let surface_texture = SurfaceTexture::new(size.width, size.height, window.clone());
        let pixels = Pixels::new(size.width, size.height, surface_texture).unwrap();

        self.window = Some(window);
        self.pixels = Some(pixels);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Some(window) = self.window.as_ref() else {
            return;
        };

        if window.id() != id {
            return;
        }

        if let Some(pixels) = self.pixels.as_mut() {
            if matches!(event, WindowEvent::RedrawRequested) {
                self.controller.handle_redraw_requested(&mut self.renderer, pixels, &self.state, event_loop);
            } else {
                self.controller.handle_window_event(event, &mut self.state, pixels, event_loop, window);
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            self.controller.handle_main_events_cleared(&mut self.state, window, Instant::now());
        }
    }

}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
    Ok(())
}
