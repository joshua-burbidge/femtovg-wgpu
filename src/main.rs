use std::sync::Arc;

use egui_wgpu::Renderer;
use egui_winit::{egui::CentralPanel, State};
use femtovg::{Canvas, Color, Paint, Path};
use helpers::WindowSurface;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard,
    window::{Window, WindowId},
};

mod helpers;

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    helpers::start(1000, 600, "femtovg app");
    #[cfg(target_arch = "wasm32")]
    helpers::start();
}

pub struct App<W: WindowSurface> {
    mousex: f32,
    mousey: f32,
    dragging: bool,
    close_requested: bool,
    window: Arc<Window>,
    canvas: Canvas<W::Renderer>,
    surface: W,
    egui_winit_state: State,
    // egui_ctx: Context,
}
impl<W: WindowSurface> App<W> {
    fn new(
        canvas: Canvas<W::Renderer>,
        surface: W,
        window: Arc<Window>,
        egui_winit_state: State,
        // egui_ctx: &Context,
    ) -> Self {
        App {
            canvas,
            surface,
            window,
            mousex: 0.,
            mousey: 0.,
            dragging: false,
            close_requested: false,
            egui_winit_state,
            // egui_ctx,
        }
    }
}
impl<W: WindowSurface> ApplicationHandler for App<W> {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
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
                    keyboard::Key::Named(keyboard::NamedKey::Escape) => {
                        self.close_requested = true;
                    }
                    _ => {}
                }
            }
            WindowEvent::RedrawRequested { .. } => {
                let window = &self.window;
                let canvas = &mut self.canvas;
                let surface = &mut self.surface;

                let egui_winit_state = &mut self.egui_winit_state;

                let raw_input = egui_winit_state.take_egui_input(&window);
                let egui_context = egui_winit_state.egui_ctx();

                let full_output = egui_context.run(raw_input, |ctx| {
                    // Build the UI
                    CentralPanel::default().show(ctx, |ui| {
                        ui.label("Hello, egui!");
                        if ui.button("Click me").clicked() {
                            println!("Button clicked!");
                        }
                    });
                });
                let platform_output = full_output.platform_output;
                let clipped_primitives =
                    egui_context.tessellate(full_output.shapes, full_output.pixels_per_point);
                println!("{:?}", clipped_primitives);

                egui_winit_state.handle_platform_output(&window, platform_output);

                // let painter = Painter::new(egui_context);
                let surface_config = surface.get_surface_config();
                let device = surface.get_device();
                let queue = surface.get_queue();

                let mut egui_renderer =
                    Renderer::new(device, surface_config.format, None, 1, false);

                for (id, image_delta) in &full_output.textures_delta.set {
                    egui_renderer.update_texture(&device, &queue, *id, &image_delta);
                }
                // now use renderer to draw the clipped primitives - how?

                // update textures
                // update buffers
                // render - requires renderpass

                let size = window.inner_size();
                let dpi_factor = window.scale_factor();

                canvas.set_size(size.width, size.height, dpi_factor as f32);
                canvas.clear_rect(0, 0, size.width, size.height, Color::black());

                let mut path = Path::new();
                path.move_to(0., 0.);
                path.line_to(100., 100.);
                canvas.stroke_path(&path, &Paint::color(Color::white()));

                surface.present(canvas);
                // this is calling flush_to_surface and swap_buffers
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
