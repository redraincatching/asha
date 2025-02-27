use asha::*;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    // log to stderr
    env_logger::init(); 

    launch_app()
}
