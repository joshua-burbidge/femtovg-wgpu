use core::f32;

use egui;
use egui_wgpu;
use egui_winit;
use wgpu::{
    CommandEncoderDescriptor, Operations, RenderPassColorAttachment, RenderPassDescriptor,
    TextureViewDescriptor,
};
use winit::{event::WindowEvent, window::Window};

pub struct Ui {
    pub text: String,
    pub panel_width: f32,
}
impl Ui {
    fn new() -> Self {
        Self {
            text: "Initial text".to_owned(),
            panel_width: 200.,
        }
    }
    fn ui(&mut self, ctx: &egui::Context) {
        let panel = egui::SidePanel::left("main-ui-panel")
            .exact_width(self.panel_width)
            .resizable(false);

        panel.show(ctx, |ui| {
            ui.label("Hello, egui!");
            ui.label("Hello, egui!");
            ui.text_edit_multiline(&mut self.text);
            ui.add(egui::TextEdit::multiline(&mut self.text).desired_width(f32::INFINITY));
            if ui.button("Click me").clicked() {
                println!("Button clicked!");
            }
            if ui.button("Click me 2").clicked() {
                println!("Button 2 clicked!");
            }
        });
    }
}

pub struct Egui {
    state: egui_winit::State,
    _context: egui::Context,
    renderer: egui_wgpu::Renderer,
    pub ui: Ui,
}
impl Egui {
    pub fn new(
        window: &Window,
        device: &wgpu::Device,
        output_color_format: wgpu::TextureFormat,
    ) -> Self {
        let egui_context = egui::Context::default();
        let viewport_id = egui_context.viewport_id();
        let egui_winit_state =
            egui_winit::State::new(egui_context.clone(), viewport_id, window, None, None, None);

        let egui_renderer = egui_wgpu::Renderer::new(device, output_color_format, None, 1, false);

        let ui = Ui::new();

        Self {
            state: egui_winit_state,
            _context: egui_context,
            renderer: egui_renderer,
            ui,
        }
    }

    pub fn render_ui(
        &mut self,
        window: &Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_texture: &wgpu::SurfaceTexture,
    ) {
        let state = &mut self.state;

        let raw_input = state.take_egui_input(&window);
        let egui_context = state.egui_ctx();

        let full_output = egui_context.run(raw_input, |ctx| {
            self.ui.ui(ctx);
        });

        let platform_output = full_output.platform_output;
        let clipped_primitives =
            egui_context.tessellate(full_output.shapes, full_output.pixels_per_point);

        state.handle_platform_output(&window, platform_output);

        let egui_renderer = &mut self.renderer;

        for (id, image_delta) in &full_output.textures_delta.set {
            egui_renderer.update_texture(device, &queue, *id, &image_delta);
        }

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("My render encoder"),
        });
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
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

        let texture_view = surface_texture.texture.create_view(&TextureViewDescriptor {
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
    }

    pub fn handle_input(&mut self, window: &Window, event: &WindowEvent) -> bool {
        let egui_winit_state = &mut self.state;
        let event_response = egui_winit_state.on_window_event(window, &event);

        // println!("{:?}", event);
        // println!("{:?}", event_response);

        if event_response.repaint {
            window.request_redraw();
        }

        event_response.consumed
    }
}
