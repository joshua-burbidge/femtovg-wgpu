use std::sync::Arc;

use femtovg::{renderer::OpenGl, Canvas};
use winit::{event_loop::ActiveEventLoop, window::Window};

// use super::run;

pub trait WindowSurface {
    type Renderer: femtovg::Renderer + 'static;
    fn resize(&mut self, width: u32, height: u32);
    fn present(&self, canvas: &mut femtovg::Canvas<Self::Renderer>);
}

#[cfg(not(feature = "wgpu"))]
mod opengl;

#[cfg(feature = "wgpu")]
mod wgpu;

pub fn init_event_loop() {
    #[cfg(not(feature = "wgpu"))]
    opengl::init_opengl_app();
    #[cfg(feature = "wgpu")]
    wgpu::init_wgpu_app();
}

// pub fn start_opengl<W: WindowSurface>(
//     event_loop: &ActiveEventLoop,
//     width: u32,
//     height: u32,
//     title: &'static str,
//     resizeable: bool,
// ) -> (Canvas<OpenGl>, OpenGlSurface, Arc<Window>) {
//     let result = spin_on::spin_on(opengl::start_opengl(
//         event_loop, width, height, title, resizeable,
//     ));

//     result
// }

// pub fn start<W: WindowSurface>(
//     event_loop: &ActiveEventLoop,
//     #[cfg(not(target_arch = "wasm32"))] width: u32,
//     #[cfg(not(target_arch = "wasm32"))] height: u32,
//     #[cfg(not(target_arch = "wasm32"))] title: &'static str,
//     #[cfg(not(target_arch = "wasm32"))] resizeable: bool,
// ) -> (Canvas<OpenGl>, W, Arc<Window>) {
//     #[cfg(not(feature = "wgpu"))]
//     use opengl::start_opengl as async_start;
//     #[cfg(feature = "wgpu")]
//     use wgpu::start_wgpu as async_start;
//     #[cfg(not(target_arch = "wasm32"))]
//     let result = spin_on::spin_on(async_start(event_loop, width, height, title, resizeable));
//     #[cfg(target_arch = "wasm32")]
//     wasm_bindgen_futures::spawn_local(async_start(event_loop));

//     // println!("{:?}", result);
//     result
// }
