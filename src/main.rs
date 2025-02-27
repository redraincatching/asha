//use asha::{self, output_assembly, read_compiled};

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    //let bytes = read_compiled("./executables/hello");
    //output_assembly(bytes).expect("error reading object file");

    env_logger::init(); // log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Ok(Box::new(asha::AshaApp::new(cc)))),
    )
}
