#[cfg(not(target_arch = "wasm32"))]
use winit::dpi::PhysicalSize;

use femtovg::{renderer::WGPURenderer, Canvas, Color, Paint, Path};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{self, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{self, Window, WindowAttributes},
};

use super::WindowSurface;

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
pub fn init_wgpu_app(
    event_loop: EventLoop<()>,
    canvas: Canvas<WGPURenderer>,
    surface: DemoSurface,
    window: Arc<Window>,
) {
    let mut app = App::new(canvas, surface, window);

    event_loop.run_app(&mut app).expect("failed to run app");
}

pub async fn start_wgpu(
    #[cfg(not(target_arch = "wasm32"))] width: u32,
    #[cfg(not(target_arch = "wasm32"))] height: u32,
    #[cfg(not(target_arch = "wasm32"))] title: &'static str,
    #[cfg(not(target_arch = "wasm32"))] resizeable: bool,
) {
    println!("using Wgpu...");
    let event_loop = EventLoop::new().unwrap();

    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(all(debug_assertions, target_arch = "wasm32"))]
    console_error_panic_hook::set_once();

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

    // it's getting stuck at the first await in wasm, works native
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

    init_wgpu_app(event_loop, canvas, demo_surface, window);
}

pub struct App {
    mousex: f32,
    mousey: f32,
    dragging: bool,
    window: Arc<Window>,
    canvas: Canvas<WGPURenderer>,
    surface: DemoSurface,
}
impl App {
    fn new(canvas: Canvas<WGPURenderer>, surface: DemoSurface, window: Arc<Window>) -> Self {
        App {
            canvas,
            surface,
            window,
            mousex: 0.,
            mousey: 0.,
            dragging: false,
        }
    }
}
impl ApplicationHandler for App {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&format!("initialized").into());
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: window::WindowId,
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
            WindowEvent::MouseWheel {
                device_id: _,
                delta: event::MouseScrollDelta::LineDelta(_, y),
                ..
            } => {
                // it's a PixelDelta in wasm
                let canvas = &mut self.canvas;

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
                button: event::MouseButton::Left,
                state,
                ..
            } => match state {
                event::ElementState::Pressed => self.dragging = true,
                event::ElementState::Released => self.dragging = false,
            },
            WindowEvent::RedrawRequested { .. } => {
                let window = &self.window;
                let mut canvas = &mut self.canvas;
                let surface = &self.surface;

                let size = window.inner_size();
                let dpi_factor = window.scale_factor();

                canvas.set_size(size.width, size.height, dpi_factor as f32);
                canvas.clear_rect(0, 0, size.width, size.height, Color::black());

                let mut path = Path::new();
                path.move_to(0., 0.);
                path.line_to(100., 100.);
                canvas.stroke_path(&path, &Paint::color(Color::white()));

                surface.present(&mut canvas);
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
}
