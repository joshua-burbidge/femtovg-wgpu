use femtovg::{renderer::WGPURenderer, Canvas, Color, Paint, Path};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard,
    window::{Window, WindowId},
};

use crate::{egui_ui::Egui, helpers::wgpu::WgpuWindowSurface};

pub struct App {
    mousex: f32,
    mousey: f32,
    dragging: bool,
    close_requested: bool,
    window: Arc<Window>,
    canvas: Canvas<WGPURenderer>,
    surface: WgpuWindowSurface,
    egui: Egui,
}
impl App {
    pub fn new(
        canvas: Canvas<WGPURenderer>,
        surface: WgpuWindowSurface,
        window: Arc<Window>,
        egui: Egui,
    ) -> Self {
        App {
            canvas,
            surface,
            window,
            egui,
            mousex: 0.,
            mousey: 0.,
            dragging: false,
            close_requested: false,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        self.canvas.reset_transform();
        self.canvas.translate(self.egui.ui.panel_width, 0.);

        println!(
            "panel width is {}, text is {}",
            self.egui.ui.panel_width, self.egui.ui.text
        );
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let is_consumed = self.egui.handle_input(&self.window, &event);
        if is_consumed {
            return ();
        }

        match event {
            #[cfg(not(target_arch = "wasm32"))]
            WindowEvent::Resized(physical_size) => {
                let surface = &mut self.surface;

                surface.resize(physical_size.width, physical_size.height);
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position,
                ..
            } => {
                let canvas = &mut self.canvas;

                if self.dragging {
                    let p0 = canvas
                        .transform()
                        .inverse()
                        .transform_point(self.mousex, self.mousey);
                    let p1 = canvas
                        .transform()
                        .inverse()
                        .transform_point(position.x as f32, position.y as f32);

                    canvas.translate(p1.0 - p0.0, p1.1 - p0.1);

                    self.window.request_redraw();
                }

                self.mousex = position.x as f32;
                self.mousey = position.y as f32;
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let canvas = &mut self.canvas;

                let y = match delta {
                    MouseScrollDelta::LineDelta(_x_delta, y_delta) => y_delta,
                    MouseScrollDelta::PixelDelta(delta) => (delta.y * 0.01) as f32,
                };

                let pt = canvas
                    .transform()
                    .inverse()
                    .transform_point(self.mousex, self.mousey);
                canvas.translate(pt.0, pt.1);
                canvas.scale(1.0 + (y / 10.0), 1.0 + (y / 10.0));
                canvas.translate(-pt.0, -pt.1);

                self.window.request_redraw();
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => match state {
                ElementState::Pressed => self.dragging = true,
                ElementState::Released => self.dragging = false,
            },
            WindowEvent::KeyboardInput { event, .. } => {
                let key = event.logical_key;
                match key {
                    keyboard::Key::Named(named_key) => match named_key {
                        keyboard::NamedKey::Escape => {
                            self.close_requested = true;
                        }
                        _ => {}
                    },
                    keyboard::Key::Character(c) => match c.as_str() {
                        "r" => {
                            println!("pressed r");
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            WindowEvent::RedrawRequested { .. } => {
                let window = &self.window;
                let surface = &mut self.surface;
                let surface_texture = surface.get_surface_texture();

                // femtovg
                let canvas = &mut self.canvas;

                let size = window.inner_size();
                let dpi_factor = window.scale_factor();
                canvas.set_size(size.width, size.height, dpi_factor as f32);
                canvas.clear_rect(0, 0, size.width, size.height, Color::black());

                draw_app(canvas);
                surface.present_canvas(canvas, &surface_texture);

                // egui
                let device = surface.get_device();
                let queue = surface.get_queue();
                self.egui.render_ui(window, device, queue, &surface_texture);

                // both
                surface_texture.present();
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            // _ => {}
            _ => {
                println!("{:?}", event);
                #[cfg(target_arch = "wasm32")]
                web_sys::console::log_1(&format!("{:?}", event).into());
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // exiting in wasm just makes it freeze and do nothing
        #[cfg(not(target_arch = "wasm32"))]
        if self.close_requested {
            _event_loop.exit();
        }
    }
}

fn draw_app(canvas: &mut Canvas<WGPURenderer>) {
    let mut path = Path::new();
    path.move_to(0., 0.);
    path.line_to(300., 300.);
    canvas.stroke_path(&path, &Paint::color(Color::white()));
}
