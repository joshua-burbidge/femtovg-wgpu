use std::sync::Arc;

use egui_wgpu::ScreenDescriptor;
use egui_winit::{egui, State};
use femtovg::{Canvas, Color, Paint, Path};
use helpers::WindowSurface;
use wgpu::{
    CommandEncoderDescriptor, Operations, RenderPassColorAttachment, RenderPassDescriptor,
    TextureViewDescriptor,
};
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
    text: String,
    window: Arc<Window>,
    canvas: Canvas<W::Renderer>,
    surface: W,
    egui_winit_state: State,
    egui_renderer: egui_wgpu::Renderer,
}
impl<W: WindowSurface> App<W> {
    fn new(
        canvas: Canvas<W::Renderer>,
        surface: W,
        window: Arc<Window>,
        egui_winit_state: State,
        egui_renderer: egui_wgpu::Renderer,
    ) -> Self {
        App {
            canvas,
            surface,
            window,
            mousex: 0.,
            mousey: 0.,
            dragging: false,
            close_requested: false,
            text: "Initial text".to_owned(),
            egui_winit_state,
            egui_renderer,
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
        let egui_winit_state = &mut self.egui_winit_state;
        let event_response = egui_winit_state.on_window_event(&self.window, &event);

        // println!("{:?}", event);
        // println!("{:?}", event_response);

        if event_response.repaint {
            self.window.request_redraw();
        }
        if event_response.consumed {
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

                let size = window.inner_size();
                let dpi_factor = window.scale_factor();

                canvas.set_size(size.width, size.height, dpi_factor as f32);
                // canvas.clear_rect(0, 0, size.width, size.height, Color::black());

                let egui_winit_state = &mut self.egui_winit_state;

                let raw_input = egui_winit_state.take_egui_input(&window);
                let egui_context = egui_winit_state.egui_ctx();

                let full_output = egui_context.run(raw_input, |ctx| {
                    // Build the UI
                    egui::SidePanel::left("123").show(ctx, |ui| {
                        ui.label("Hello, egui!");
                        ui.label("Hello, egui!");
                        ui.label("Hello, egui!");
                        if ui.text_edit_multiline(&mut self.text).changed() {
                            println!("changed text edit");
                        }
                        if ui.button("Click me").clicked() {
                            println!("Button clicked!");
                        }
                        if ui.button("Click me 2").clicked() {
                            println!("Button 2 clicked!");
                        }
                    });
                });
                let platform_output = full_output.platform_output;
                let clipped_primitives =
                    egui_context.tessellate(full_output.shapes, full_output.pixels_per_point);
                // println!("{:?}", clipped_primitives);

                egui_winit_state.handle_platform_output(&window, platform_output);

                let device = surface.get_device();
                let queue = surface.get_queue();
                let wgpu_surface: &wgpu::Surface<'static> = surface.get_surface();

                let egui_renderer = &mut self.egui_renderer;

                for (id, image_delta) in &full_output.textures_delta.set {
                    egui_renderer.update_texture(&device, &queue, *id, &image_delta);
                }

                // not supposed to create everything on every render - should reuse
                let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("My render encoder"),
                });
                let screen_descriptor = ScreenDescriptor {
                    pixels_per_point: full_output.pixels_per_point,
                    size_in_pixels: [1000, 600],
                };
                egui_renderer.update_buffers(
                    device,
                    queue,
                    &mut encoder,
                    &clipped_primitives,
                    &screen_descriptor,
                );

                let surface_result = wgpu_surface
                    .get_current_texture()
                    .expect(" failed to get current texture");
                let texture_view = surface_result.texture.create_view(&TextureViewDescriptor {
                    label: None,
                    format: None,
                    dimension: None,
                    aspect: wgpu::TextureAspect::All, // this is default
                    base_mip_level: 0,
                    mip_level_count: None,
                    base_array_layer: 0,
                    array_layer_count: None,
                });

                {
                    // wgpu example uses a block like this - maybe it's an alternative to dropping render_pass
                    let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                        label: Some("My render pass"),
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &texture_view,
                            resolve_target: None,
                            ops: Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                    });

                    let mut static_render_pass = render_pass.forget_lifetime();

                    //adding this line is what causes the encode lifetime error
                    // resolved by calling forget_lifetime and using the return from that one here
                    egui_renderer.render(
                        &mut static_render_pass,
                        &clipped_primitives,
                        &screen_descriptor,
                    );
                }
                for x in &full_output.textures_delta.free {
                    egui_renderer.free_texture(x)
                }

                queue.submit(std::iter::once(encoder.finish()));

                // drop(render_pass);
                // drop(surface_result);
                // now use renderer to draw the clipped primitives - how?

                // update textures
                // update buffers
                // render - requires renderpass

                let mut path = Path::new();
                path.move_to(0., 0.);
                path.line_to(100., 100.);
                canvas.stroke_path(&path, &Paint::color(Color::white()));

                // canvas.flush_to_surface(&surface_result.texture);

                // surface_result.present();
                surface.present(canvas, surface_result);
                // surface_result.present();
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
