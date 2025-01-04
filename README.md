#### TODO
- wasm doesn't work with wgpu feature
- try femtovg example with wgpu + wasm
- look at wgpu wasm examples

- crashes when femtovg render is entirely off-screen
    - "trying to destroy SurfaceSemaphores still in use by SurfaceTexture"
    - same error as i got when there were 2 calls to surface.get_current_texture

Egui
- initial render works, next redraw doesn't
    - don't recreate everything on redraw
- need to handle events (on_event)

should exist across multiple renders:
- device
- queue
- window
- EguiRenderer
- egui_winit State
- egui Context

recreate every render:
- encoder
- view
- screen descriptor
- tesselate -> update_texture -> update_buffers -> render_pass -> render process
