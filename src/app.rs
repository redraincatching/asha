// ----------------------------------------

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct Disassembly {

}

impl eframe::App for Disassembly {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                ui.monospace("code goes here lmao")
                // put call to disassembly function here
            });
        });
    }
}

// TODO: add cfg and decompiled views

// ----------------------------------------

// Use these to select which view is active
#[derive(Clone, Copy, Debug, Eq, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub enum Tab {
    #[default]
    Disassembly,
    //ContextFlowGraph,
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
#[derive(serde::Deserialize, serde::Serialize)]
pub struct State {
    disassembly: Disassembly,
    //cfg: /* cfg_app */,
    //decompiled: /* decompiled */,

    // use to select which view is open
    current_tab: Tab,

    // path to file we wish to analyse
    #[serde(skip)]
    source_file: Option<String>
    // MAYBE: maintain the analysed file here?
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
#[serde(default)] 
pub struct AshaApp {
    pub state: State,

}

impl AshaApp {
    // mutable iterator of tuple (name, enum_value, app)
    pub fn tabs_iter_mut(&mut self) -> impl Iterator<Item = (&'static str, Tab, &mut dyn eframe::App)> {
        let vec = vec![
            (
                "Disassembly",
                Tab::Disassembly,
                &mut self.state.disassembly as &mut dyn eframe::App
            ),
        ];

        vec.into_iter()
    }

    pub fn tabs_iter(&self) -> impl Iterator<Item = (&'static str, Tab)> {
        let vec = vec![
            (
                "Disassembly",
                Tab::Disassembly
            ),
        ];

        vec.into_iter()
    }

    /// called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // load previous app state (if any).
        // note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    fn show_selected_app(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let selected_tab = self.state.current_tab;
        for (_name, tab, app) in self.tabs_iter_mut() {
            if tab == selected_tab {
                app.update(ctx, frame);
            }
        }
    }
}

impl eframe::App for AshaApp {
    /// called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// called each time the ui needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
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

                ui.add_space(16.0);

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

            // TODO: basic check for correct file type?
            if let Some(file_chosen) = &self.state.source_file {
                ui.label("view for ");
                ui.monospace(file_chosen);
            } else {
                ui.label("please choose a file to analyse");
            }

        });

        if self.state.source_file.is_some() {
            self.show_selected_app(ctx, frame);
        }
    }
}

