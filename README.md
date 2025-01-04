#### TODO
- wasm doesn't work with wgpu feature
- try femtovg example with wgpu + wasm
- look at wgpu wasm examples

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
