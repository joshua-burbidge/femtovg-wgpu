mod helpers;

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    helpers::start(1000, 600, "my demo", true);
    #[cfg(target_arch = "wasm32")]
    helpers::start();
}
