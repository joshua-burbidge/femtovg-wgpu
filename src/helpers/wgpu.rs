#[cfg(not(target_arch = "wasm32"))]
use winit::dpi::PhysicalSize;

use femtovg::{renderer::WGPURenderer, Canvas};
use std::sync::Arc;
use winit::{event_loop::EventLoop, window::WindowAttributes};

use super::{run, WindowSurface};

pub struct DemoSurface {
    device: Arc<wgpu::Device>,
    surface_config: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface<'static>,
}

impl WindowSurface for DemoSurface {
    type Renderer = femtovg::renderer::WGPURenderer;

    fn resize(&mut self, width: u32, height: u32) {
        self.surface_config.width = width.max(1);
        self.surface_config.height = height.max(1);
        self.surface.configure(&self.device, &self.surface_config);
    }

    fn present(&self, canvas: &mut Canvas<Self::Renderer>) {
        let frame = self
            .surface
            .get_current_texture()
            .expect("unable to get next texture from swapchain");

        canvas.flush_to_surface(&frame.texture);

        frame.present();
    }
}

// this gets called on start
pub fn init_event_loop() {
    let event_loop = EventLoop::new().unwrap();

    let mut app = App::<DemoSurface>::default();

    event_loop.run_app(&mut app).expect("failed to run app");
}

pub async fn start_wgpu(
    #[cfg(not(target_arch = "wasm32"))] width: u32,
    #[cfg(not(target_arch = "wasm32"))] height: u32,
    #[cfg(not(target_arch = "wasm32"))] title: &'static str,
    #[cfg(not(target_arch = "wasm32"))] resizeable: bool,
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
            .with_title(title)
            .with_resizable(resizeable);

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

    web_sys::console::log_1(&format!("{:?}", gles_minor_version).into());
    web_sys::console::log_1(&format!("{:?}", dx12_shader_compiler).into());

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

    let demo_surface = DemoSurface {
        device: device.clone(),
        surface_config,
        surface,
    };

    let renderer = WGPURenderer::new(device, Arc::new(queue));

    let mut canvas = Canvas::new(renderer).expect("Cannot create canvas");
    canvas.set_size(width, height, window.scale_factor() as f32);

    web_sys::console::log_1(&format!("running").into());

    run(canvas, event_loop, demo_surface, window);
}
