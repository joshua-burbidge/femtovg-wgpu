#### TODO
- wasm doesn't work with wgpu feature
- try femtovg example with wgpu + wasm
- look at wgpu wasm examples

Egui
- refactor to separate egui/wgpu/femtovg
- maybe separate egui ui state vs egui rendering stuff

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
