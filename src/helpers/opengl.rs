#[cfg(not(target_arch = "wasm32"))]
mod non_wasm_imports {
    pub use glutin::{
        config::ConfigTemplateBuilder,
        context::{ContextApi, ContextAttributesBuilder},
        display::GetGlDisplay,
        prelude::*,
        surface::SurfaceAttributesBuilder,
    };
    pub use glutin_winit::DisplayBuilder;
    #[allow(deprecated)]
    pub use raw_window_handle::HasRawWindowHandle;
    pub use std::num::NonZeroU32;
}
#[cfg(not(target_arch = "wasm32"))]
use non_wasm_imports::*;

use super::WindowSurface;

use femtovg::{renderer::OpenGl, Canvas, Color, Paint, Path};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

pub struct DemoSurface {
    #[cfg(not(target_arch = "wasm32"))]
    context: glutin::context::PossiblyCurrentContext,
    #[cfg(not(target_arch = "wasm32"))]
    surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
}

impl WindowSurface for DemoSurface {
    type Renderer = OpenGl;

    fn resize(&mut self, width: u32, height: u32) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.surface.resize(
                &self.context,
                width.try_into().unwrap(),
                height.try_into().unwrap(),
            );
        }
    }
    fn present(&self, canvas: &mut femtovg::Canvas<Self::Renderer>) {
        canvas.flush_to_surface(&());
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.surface.swap_buffers(&self.context).unwrap();
        }
    }
}

// fn test<W: WindowSurface>() -> W {
//     let demo_surface = DemoSurface {
//         #[cfg(not(target_arch = "wasm32"))]
//         context,
//         #[cfg(not(target_arch = "wasm32"))]
//         surface,
//     };
// }

// this gets called on start
pub fn init_opengl_app() {
    let event_loop = EventLoop::new().unwrap();

    let mut app = App::default();

    event_loop.run_app(&mut app).expect("failed to run app");
}

// this needs to get called on resume, not start
// remove resizable arg
pub fn start_opengl(
    event_loop: &ActiveEventLoop,
    #[cfg(not(target_arch = "wasm32"))] width: u32,
    #[cfg(not(target_arch = "wasm32"))] height: u32,
    #[cfg(not(target_arch = "wasm32"))] title: &'static str,
    #[cfg(not(target_arch = "wasm32"))] resizeable: bool,
) -> (
    // Canvas<<DemoSurface as WindowSurface>::Renderer>,
    Canvas<OpenGl>,
    DemoSurface,
    Arc<Window>,
) {
    println!("using openGL...");
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(all(debug_assertions, target_arch = "wasm32"))]
    console_error_panic_hook::set_once();

    // let event_loop = EventLoop::new().unwrap();

    #[cfg(not(target_arch = "wasm32"))]
    let (canvas, window, context, surface) = {
        let window_attr = WindowAttributes::default()
            .with_inner_size(PhysicalSize::new(width, height))
            .with_title(title)
            .with_resizable(resizeable);

        let template = ConfigTemplateBuilder::new().with_alpha_size(8);

        let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attr));

        let (window, gl_config) = display_builder
            .build(event_loop, template, |mut configs| configs.next().unwrap())
            .unwrap();

        let window = window.unwrap();

        let gl_display = gl_config.display();

        #[allow(deprecated)]
        let raw_window_handle = window
            .raw_window_handle()
            .expect("raw window handle failed");

        let context_attributes = ContextAttributesBuilder::new().build(Some(raw_window_handle));
        let fallback_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::Gles(None))
            .build(Some(raw_window_handle));

        let mut not_current_gl_context = Some(unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .unwrap_or_else(|_| {
                    gl_display
                        .create_context(&gl_config, &fallback_context_attributes)
                        .expect("failed to create context")
                })
        });

        let (width, height): (u32, u32) = window.inner_size().into();

        let attrs = SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new().build(
            raw_window_handle,
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        );

        let surface = unsafe {
            gl_config
                .display()
                .create_window_surface(&gl_config, &attrs)
                .unwrap()
        };

        let gl_context = not_current_gl_context
            .take()
            .unwrap()
            .make_current(&surface)
            .unwrap();

        let renderer =
            unsafe { OpenGl::new_from_function_cstr(|s| gl_display.get_proc_address(s).cast()) }
                .expect("Cannot create renderer");

        let mut canvas = Canvas::new(renderer).expect("Cannot create canvas");
        canvas.set_size(width, height, window.scale_factor() as f32);

        (canvas, window, gl_context, surface)
    };

    #[cfg(target_arch = "wasm32")]
    let (canvas, window) = {
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

        let renderer = OpenGl::new_from_html_canvas(&html_canvas).expect("Cannot create renderer");

        let window_attrs = WindowAttributes::default().with_canvas(Some(html_canvas));
        let window = event_loop.create_window(window_attrs).unwrap();

        let _ = window.request_inner_size(PhysicalSize::new(width, height));

        let canvas = Canvas::new(renderer).expect("Cannot create canvas");

        (canvas, window)
    };

    let demo_surface = DemoSurface {
        #[cfg(not(target_arch = "wasm32"))]
        context,
        #[cfg(not(target_arch = "wasm32"))]
        surface,
    };

    // run(canvas, event_loop, demo_surface, Arc::new(window));
    (canvas, demo_surface, Arc::new(window))
}

pub struct App {
    mousex: f32,
    mousey: f32,
    dragging: bool,
    window: Option<Arc<Window>>,
    canvas: Option<Canvas<OpenGl>>,
    surface: Option<DemoSurface>,
}
impl Default for App {
    fn default() -> Self {
        App {
            mousex: 0.,
            mousey: 0.,
            dragging: false,
            window: None,
            canvas: None,
            surface: None,
        }
    }
}
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let (canvas, surface, window) = start_opengl(event_loop, 1000, 600, "my demo", true);
        self.canvas = Some(canvas);
        self.surface = Some(surface);
        self.window = Some(window);
        // #[cfg(not(target_arch = "wasm32"))]
        // helpers::start(event_loop, 1000, 600, "my demo", true);
        // #[cfg(target_arch = "wasm32")]
        // helpers::start(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            #[cfg(not(target_arch = "wasm32"))]
            WindowEvent::Resized(physical_size) => {
                let surface = self.surface.as_mut().unwrap();

                surface.resize(physical_size.width, physical_size.height);
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position,
                ..
            } => {
                let canvas = self.canvas.as_mut().unwrap();

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

                    self.window.as_ref().unwrap().request_redraw();
                }

                self.mousex = position.x as f32;
                self.mousey = position.y as f32;
            }
            WindowEvent::MouseWheel {
                device_id: _,
                delta: winit::event::MouseScrollDelta::LineDelta(_, y),
                ..
            } => {
                let canvas = self.canvas.as_mut().unwrap();

                let pt = canvas
                    .transform()
                    .inverse()
                    .transform_point(self.mousex, self.mousey);
                canvas.translate(pt.0, pt.1);
                canvas.scale(1.0 + (y / 10.0), 1.0 + (y / 10.0));
                canvas.translate(-pt.0, -pt.1);

                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => match state {
                ElementState::Pressed => self.dragging = true,
                ElementState::Released => self.dragging = false,
            },
            WindowEvent::RedrawRequested { .. } => {
                let window = self.window.as_ref().unwrap();
                let canvas = self.canvas.as_mut().unwrap();
                let surface = self.surface.as_ref().unwrap();

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
            _ => println!("{:?}", event),
            // _ => {}
        }
    }
}
