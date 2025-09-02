use eframe::{egui::{*}, Frame};
use log::{debug, info};
use crate::steam::{write_steam_shortcuts, DesiredShortcut};

use super::ui::*;
use super::instances::*;
use super::export_page::*;
use super::settings_page::*;

pub const APP_NAME : &str = "Opal";

const APP_SIDEBAR_WIDTH : f32 = 220.0;
const APP_LOGO_PADDING : f32 = 12.0;
pub const APP_HEADER_PADDING : f32 = 20.0;

const INSTANCES_JSON_PATH : &str = "instances\\instgroups.json";
const PRISMLAUNCHER_EXE_PATH : &str = "prismlauncher.exe";

#[derive(Default)]
pub struct AppModel {
    pub config: Config,
    pub instances: Vec<Instance>,

    // For logging (TODO)
    pub log_printout : String,
}
impl AppModel {
    pub fn update_instances(&mut self) {
        let mut instance_path = self.config.prism_main_path.clone();
        instance_path.push(INSTANCES_JSON_PATH);

        match get_instances_from_path(instance_path, self.config.include_hidden) {
            Ok(i) => self.instances = i,
            Err(e) => {
                self.log_printout
                    .push_str(&format!("\nERROR: Couldn't Update unstances! {}", e))
            }
        }
    }

    pub fn update_steam_shortcuts(&mut self) {


        // Build desired shortcuts as owned and upsert by app_id.
        // We also re-number "order" later, so the `order` we put here is temporary.
        
        let exe_path_string = self.config.prism_main_path.to_string_lossy().to_string()
            + "\\" + PRISMLAUNCHER_EXE_PATH;
        let mut desired_shortcuts = Vec::new();

        for inst in self.instances.iter() {
            if inst.checked {
                let app_name = inst.folder_name.clone();
                let launch_options = format!("-l \"{}\"", app_name);

                desired_shortcuts.push( DesiredShortcut {
                    // These are the arguments that go into Shortcut::new() as well
                    app_name : app_name.clone(),
                    exe : exe_path_string.clone(),
                    shortcut_path : String::new(),
                    start_dir : String::from(self.config.prism_main_path.to_string_lossy()),
                    launch_options : launch_options,

                    // TODO
                    icon : String::new(),
                    tags : vec![String::from("Installed"), String::from("Ready to play")],
                });
            }
        }

        if let Err(e) = write_steam_shortcuts(&self.config.steam_shortcuts_path, desired_shortcuts) {
            self.log_printout.push_str(&format!("\nERROR: Couldn't update instances! {}", e));
        } else {
            return
        }
    }
}

/// Application root: holds the tabs, the active tab index, and the logo texture.
#[derive(Default)]
pub struct App {
    pages: Vec<Box<dyn TabPage>>,
    active: usize,

    // To make it so the tabs have global information access
    model : AppModel,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {

        // Register your tabs here. Adding tabs = push another `Box::new(MyPage { ... })`.
        let pages: Vec<Box<dyn TabPage>> = vec![
            Box::new(ExportPage::default()),
            Box::new(SettingsPage::default()),
        ];

        let mut model = AppModel {
            ..Default::default()
        };
        model.update_instances();

        Self {
            pages,
            active: 0,
            model : model,
            ..Default::default()
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        ctx.set_pixels_per_point(1.2);

        // LEFT SIDEBAR
        SidePanel::left("sidebar")
            .exact_width(APP_SIDEBAR_WIDTH)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    // Show logo (scaled to sidebar width) if it loaded
                    ui.image("File://assets/icon.png");
                });

                ui.add_space(APP_LOGO_PADDING);

                // Tab list (scrollable in case you add many)
                ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    for i in 0..self.pages.len() {
                        // We only need the label here (immutable borrow)
                        let label = self.pages[i].label();
                        // Selectable tab button
                        let resp = ui.selectable_label(self.active == i, label);
                        if resp.clicked() {
                            self.active = i;
                        }
                    }
                });
            });

        // RIGHT CONTENT
        CentralPanel::default().show(ctx, |ui| {
            ui.add_space(8.0);

            // Borrow the active page mutably to render its UI
            if let Some(page) = self.pages.get_mut(self.active) {
                page.ui(ui, &mut self.model);
            } else {
                ui.label("No page selected.");
            }
        });
    }
}