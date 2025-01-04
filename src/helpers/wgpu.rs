use egui_wgpu::Renderer;
use egui_winit::{egui::Context, State};
use wgpu::SurfaceTexture;
#[cfg(not(target_arch = "wasm32"))]
use winit::dpi::PhysicalSize;

use femtovg::{renderer::WGPURenderer, Canvas};
use std::sync::Arc;
use winit::{
    event_loop::EventLoop,
    window::{Window, WindowAttributes},
};

use crate::App;

use super::WindowSurface;

pub struct DemoSurface {
    device: Arc<wgpu::Device>,
    surface_config: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface<'static>,
    queue: Arc<wgpu::Queue>,
}

impl WindowSurface for DemoSurface {
    type Renderer = femtovg::renderer::WGPURenderer;

    fn resize(&mut self, width: u32, height: u32) {
        self.surface_config.width = width.max(1);
        self.surface_config.height = height.max(1);
        self.surface.configure(&self.device, &self.surface_config);
    }

    fn present(&self, canvas: &mut Canvas<Self::Renderer>, surface_texture: &SurfaceTexture) {
        // removing this makes the surface error stop
        // let frame = self
        //     .surface
        //     .get_current_texture()
        //     .expect("unable to get next texture from swapchain");

        canvas.flush_to_surface(&surface_texture.texture);

        // surface_texture.present();
    }

    fn get_device(&self) -> &Arc<wgpu::Device> {
        &self.device
    }
    fn get_surface_config(&self) -> &wgpu::SurfaceConfiguration {
        &self.surface_config
    }
    fn get_surface(&self) -> &wgpu::Surface<'static> {
        &self.surface
    }
    fn get_queue(&self) -> &Arc<wgpu::Queue> {
        &self.queue
    }
}

pub fn init_wgpu_app(
    event_loop: EventLoop<()>,
    canvas: Canvas<WGPURenderer>,
    surface: DemoSurface,
    window: Arc<Window>,
) {
    let egui_context = Context::default();

    let viewport_id = egui_context.viewport_id();

    let egui_winit_state = State::new(egui_context, viewport_id, &event_loop, None, None, None);

    let surface_config = surface.get_surface_config();
    let device = surface.get_device();

    let egui_renderer = Renderer::new(device, surface_config.format, None, 1, false);

    let mut app = App::new(canvas, surface, window, egui_winit_state, egui_renderer);

    event_loop.run_app(&mut app).expect("failed to run app");
}

pub async fn start_wgpu(
    #[cfg(not(target_arch = "wasm32"))] width: u32,
    #[cfg(not(target_arch = "wasm32"))] height: u32,
    #[cfg(not(target_arch = "wasm32"))] title: &'static str,
) {
    println!("using Wgpu...");

    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(all(debug_assertions, target_arch = "wasm32"))]
    console_error_panic_hook::set_once();

    let event_loop = EventLoop::new().unwrap();

    #[cfg(not(target_arch = "wasm32"))]
    let window = {
        let window_attrs = WindowAttributes::default()
            .with_inner_size(PhysicalSize::new(1000., 600.))
            .with_title(title);

        #[allow(deprecated)]
        event_loop.create_window(window_attrs).unwrap()
    };

    #[cfg(target_arch = "wasm32")]
    let (window, width, height) = {
        use wasm_bindgen::JsCast;
        use winit::platform::web::WindowAttributesExtWebSys;

        let html_canvas = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();

        let width = html_canvas.width();
        let height = html_canvas.height();

        let window_attrs = WindowAttributes::default().with_canvas(Some(html_canvas.clone()));
        #[allow(deprecated)]
        let window = event_loop.create_window(window_attrs).unwrap();

        let _ = window.request_inner_size(winit::dpi::PhysicalSize::new(width, height));

        (window, width, height)
    };

    let window = Arc::new(window);

    let backends = wgpu::util::backend_bits_from_env().unwrap_or_default();
    let dx12_shader_compiler = wgpu::util::dx12_shader_compiler_from_env().unwrap_or_default();
    let gles_minor_version = wgpu::util::gles_minor_version_from_env().unwrap_or_default();

    let instance = wgpu::util::new_instance_with_webgpu_detection(wgpu::InstanceDescriptor {
        backends,
        flags: wgpu::InstanceFlags::from_build_config().with_env(),
        dx12_shader_compiler,
        gles_minor_version,
    })
    .await;

    let surface = instance.create_surface(window.clone()).unwrap();

    let adapter = wgpu::util::initialize_adapter_from_env_or_default(&instance, Some(&surface))
        .await
        .expect("Failed to find an appropriate adapter");

    // Create the logical device and command queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
            },
            None,
        )
        .await
        .expect("Failed to create device");

    let mut surface_config = surface.get_default_config(&adapter, width, height).unwrap();

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities
        .formats
        .iter()
        .find(|f| !f.is_srgb())
        .copied()
        .unwrap_or_else(|| swapchain_capabilities.formats[0]);
    surface_config.format = swapchain_format;
    surface.configure(&device, &surface_config);

    let device = Arc::new(device);
    let queue = Arc::new(queue);

    let demo_surface = DemoSurface {
        device: device.clone(),
        surface_config,
        surface,
        queue: queue.clone(),
    };

    let renderer = WGPURenderer::new(device, queue);

    let mut canvas = Canvas::new(renderer).expect("Cannot create canvas");
    canvas.set_size(width, height, window.scale_factor() as f32);

    init_wgpu_app(event_loop, canvas, demo_surface, window);
}
