A boilerplate for femtovg + wgpu + egui.

#### TODO
- wasm doesn't work with wgpu feature
- try femtovg example with wgpu + wasm
- look at wgpu wasm examples


make egui integration work with GL or WebGPU
- put egui methods into a trait - render, update, etc
- impl trait twice using egui-glow and egui-wgpu
- update opengl helper with changes to Surface trait

#### Credits

References from the [femtovg examples](https://github.com/femtovg/femtovg/tree/master/examples) for femtovg integration and [egui-wgpu-demo](https://github.com/ejb004/egui-wgpu-demo) by [ejb004](https://github.com/ejb004) for the egui-wgpu integration.
