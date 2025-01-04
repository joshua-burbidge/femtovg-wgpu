#### TODO
- wasm doesn't work with wgpu feature
- try femtovg example with wgpu + wasm
- look at wgpu wasm examples

- crashes when femtovg render is entirely off-screen
    - "trying to destroy SurfaceSemaphores still in use by SurfaceTexture"
    - same error as i got when there were 2 calls to surface.get_current_texture
    - crashes immediately if an off-screen path is initially rendered
    - doesn't crash if i include a canvas.clear_rect call
       - even if the clear_rect call is not the full screen
    - resolved by calling clear_rect, to do that we need to render femtovg first so it is underneath the UI

Egui
- refactor to separate egui/wgpu/femtovg

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
