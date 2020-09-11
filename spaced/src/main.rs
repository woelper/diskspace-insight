#![warn(clippy::all)]

use diskspace_insight;
use diskspace_insight::DirInfo;
use egui::{Slider, Window};
use egui_glium::{storage::FileStorage, RunMode};
use std::sync::mpsc::channel;
use dirs;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
// #[derive(Default, serde::Deserialize, serde::Serialize)]
#[derive(Default)]
struct MyApp {
    my_string: String,
    max_types: i32,
    max_files: i32,
    info: Option<DirInfo>,
    allow_delete: bool
}

impl egui::app::App for MyApp {
    /// This function will be called whenever the Ui needs to be shown,
    /// which may be many times per second.
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut dyn egui::app::Backend) {
        let MyApp {
            my_string,
            max_types,
            max_files,
            info,
            allow_delete
        } = self;

        Window::new("Setup").show(ui.ctx(), |ui| {
            
            ui.horizontal(|ui|{
                
                if ui.button("Home").clicked {
                    if let Some(dir) = dirs::home_dir() {
                        *my_string = dir.to_string_lossy().to_string();
                    }
                }
                if ui.button("Downloads").clicked {
                    if let Some(dir) = dirs::download_dir() {
                        *my_string = dir.to_string_lossy().to_string();
                    }
                }
                if ui.button("Videos").clicked {
                    if let Some(dir) = dirs::video_dir() {
                        *my_string = dir.to_string_lossy().to_string();
                    }
                }
                if ui.button("Cache").clicked {
                    if let Some(dir) = dirs::cache_dir() {
                        *my_string = dir.to_string_lossy().to_string();
                    }
                }
                if ui.button("Temp").clicked {
                    *my_string = std::env::temp_dir().to_string_lossy().to_string();
                }


            });
            
            ui.text_edit(my_string);
            
            ui.checkbox("Allow deletion", allow_delete);
            if ui.button("Scan!").clicked {
                *info = Some(diskspace_insight::scan(&my_string));
            }

        });
        
        
        Window::new("Filetypes").scroll(true).show(ui.ctx(), |ui| {
            ui.label(format!("Files by type, largest first"));
            ui.add(Slider::i32(max_types, 1..=100).text("max results"));

            if let Some(info) = info {
                
                for (i, filetype) in info.types_by_size.iter().enumerate() {
                    if i as i32 >= *max_types {
                        break;
                    }
                    ui.collapsing(format!("{} ({}MB)", filetype.ext, filetype.size/1024/1024), |ui|{
                        for file in &filetype.files {
                            ui.label(format!("{} {}MB", file.path.display(), file.size/1024/1024));
                        }
                    });
                    
                }
            } 
        });

        Window::new("Files").scroll(true).show(ui.ctx(), |ui| {
            ui.label(format!("Files by size, largest first"));
            ui.add(Slider::i32(max_files, 1..=100).text("max results"));

            if let Some(info) = info {
                
                for (i, file) in info.files_by_size.iter().enumerate() {
                    if i as i32 >= *max_files {
                        break;
                    }
                    ui.collapsing(format!("{} ({}MB)", file.path.display(), file.size/1024/1024), |ui|{
           
                    });
                    
                }
            } 
        });

        Window::new("Directories").scroll(true).show(ui.ctx(), |ui| {
            ui.label(format!("Files by type, largest first"));

            if let Some(info) = info {
                
         
            } 
        });
    }

    // fn on_exit(&mut self, storage: &mut dyn egui::app::Storage) {
    //     // egui::app::set_value(storage, egui::app::APP_KEY, self);
    // }
}

fn main() {
    // let i = diskspace_insight::scan("/home/woelper/Downloads");

    let title = "My Egui Window";
    let storage = FileStorage::from_path(".spaced.json".into()); // Where to persist app state
                                                                             // let app: MyApp = egui::app::get_value(&storage, egui::app::APP_KEY).unwrap_or_default(); // Restore `MyApp` from file, or create new `MyApp`.
    let mut app: MyApp = MyApp::default();
    app.my_string = dirs::home_dir().unwrap_or_default().to_string_lossy().to_string();
    app.max_types = 10;
    app.max_files = 40;
    egui_glium::run(title, RunMode::Reactive, storage, app);
}

fn my_save_function() {
    // dummy
}
