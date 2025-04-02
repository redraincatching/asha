use crate::{decompilation::{generate_sections, InstructionSection}, disassemble_file, output_assembly, read_compiled};

// ----------------------------------------

// TODO: add cfg and decompiled views

type ViewFunction = fn(&egui::Context, &State);

fn no_view_selected(ctx: &egui::Context, _state: &State) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.label("please choose a file to analyse");
    });
}

fn disassembly_view(ctx: &egui::Context, state: &State) {
    egui::CentralPanel::default().show(ctx, |ui| {
        if let Some(file_chosen) = state.get_source_file() {
            let path = std::path::Path::new(file_chosen);
            let filename: String = path.file_name().unwrap().to_str().unwrap().to_string();
            // wow that's ugly

            ui.label("disassembly view for ");
            ui.monospace(filename);

            egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                let bytes = read_compiled(file_chosen);
                ui.monospace(output_assembly(bytes).expect("error reading object file"));
            });
        }     
    });
}

// ----------------------------------------

type ISWrapper = (InstructionSection, egui::Pos2);

// TODO: cache the disassembly and pass it from here
// ALSO TODO: clean this all up and make it work
// NOTE: this is totally broken, i hate guis
fn cfg_view(ctx: &egui::Context, state: &State) {
    egui::CentralPanel::default().show(ctx, |ui| {
        if let Some(file_chosen) = state.get_source_file() {
            let path = std::path::Path::new(file_chosen);
            let filename: String = path.file_name().unwrap().to_str().unwrap().to_string();

            ui.label("control flow graph view for ");
            ui.monospace(filename);

            egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                let bytes = read_compiled(file_chosen);
                let disassembly = disassemble_file(bytes).expect("whoops");

                let block_map = generate_sections(disassembly);
                
                // calculate positions for blocks
                let mut y_offset = 0.0;

                let mut wrapped_blocks: Vec<ISWrapper> = Vec::new();
                for block in &mut block_map.values() {
                    let pos = egui::Pos2::new(100.0, y_offset);
                    wrapped_blocks.push((block.clone(), pos)); // wrap block and position in ISWrapper

                    y_offset += 100.0; // adjust the vertical space between blocks
                }

                // now render each block and add labels for branches
                // TODO: (maybe, eventually) actually make this draw a graph
                // if i can get petgraph to work and output graphviz then maybe
                for (block, position) in &wrapped_blocks {
                    ui.group(|ui| {
                        // draw block (using its position and a rectangle to represent it)
                        ui.painter().rect_filled(
                            egui::Rect::from_min_size(*position, egui::vec2(0.0, 10.0)),
                            0.0,
                            egui::Color32::LIGHT_GRAY,
                        );
                        ui.monospace(format!("{}", &block)); // show block data
                    });
                }
            });
        }     
    });
}

// ----------------------------------------

// Use these to select which view is active
#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum Tab {
    #[default]
    Disassembly,
    ContextFlowGraph,
    //Decompiled
}

impl core::fmt::Display for Tab {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut name = format!("{self:?}");
        name.make_ascii_lowercase();
        f.write_str(&name)
    }
}

// ----------------------------------------

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Default)]
pub struct State {
    // use to select which view is open
    current_tab: Tab,

    // path to file we wish to analyse
    source_file: Option<String>
    // MAYBE: maintain the analysed file here?
}

impl State {
    fn get_source_file(&self) -> Option<&String> {
        return self.source_file.as_ref()
    }
}

#[derive(Default)]
pub struct AshaApp {
    pub state: State,
}

impl AshaApp {
    pub fn tabs_iter(&self) -> impl Iterator<Item = (&'static str, Tab)> {
        let vec = vec![
            (
                "Disassembly",
                Tab::Disassembly
            ),
            (
                "Control Flow Graph",
                Tab::ContextFlowGraph
            )
        ];

        vec.into_iter()
    }

    /// called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }

    fn show_selected_view(&self, ctx: &egui::Context, state: &State) {
        let selected_tab = self.state.current_tab;
        let view_function: ViewFunction = match selected_tab {
            Tab::Disassembly => disassembly_view,
            Tab::ContextFlowGraph => cfg_view,
            // TODO: the others
            _ => no_view_selected
        };

        view_function(ctx, state);
    }
}

impl eframe::App for AshaApp {
    /// called each time the ui needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {

            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    
                    if ui.button("Open file…").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.state.source_file = Some(path.display().to_string());
                        }
                    }

                    egui::widgets::global_theme_preference_buttons(ui);
                });

                ui.add_space(8.0);

                ui.separator();

                // buttons to switch views
                ui.horizontal(|ui| {
                    for (name, tab) in self.tabs_iter() {
                        if ui.button(name).clicked() {
                            self.state.current_tab = tab;
                        }
                    }
                })
            });
        });

        if self.state.source_file.is_some() {
            self.show_selected_view(ctx, &self.state);
        }
    }
}

