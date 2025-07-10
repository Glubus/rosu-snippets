mod utils;
mod snippets;
mod ui;
use crate::snippets::structs::{SnippetsMaker, Snippets};
use crate::ui::mania::ManiaRenderer;
use rosu_memory_lib::{init_loop};
use rdev::{listen, Event, EventType, Key};
use std::sync::{Arc, Mutex};
use rosu_mem::process::Process;
use rosu_memory_lib::reader::structs::State;
use egui;
use std::fs;
use std::time::Instant;
struct AppState {
    snippets: Vec<Snippets>,
        snippets_maker: SnippetsMaker,
    selected_snippet: Option<usize>,
    mania_renderer: Option<ManiaRenderer>,
    start_time: Instant,
    scroll_time_ms: f32,
    snippet_speed: f32,
    show_save_dialog: bool,
    save_filename: String,
    process: Arc<Process>,
    state: Arc<Mutex<State>>,
}

impl AppState {
    fn new(process: Arc<Process>, state: Arc<Mutex<State>>) -> Self {
        Self {
            snippets: Vec::new(),
            snippets_maker: SnippetsMaker::new(),
            selected_snippet: None,
            mania_renderer: None,
            start_time: Instant::now(),
            scroll_time_ms: 1000.0,
            snippet_speed: 1.0,
            show_save_dialog: false,
            save_filename: String::new(),
            process,
            state,
        }
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.mania_renderer.is_none() {
            self.mania_renderer = Some(ManiaRenderer::new());
        }


        egui::SidePanel::left("snippets_list").show(ctx, |ui| {
            ui.heading("Snippets");
            
            // Afficher tous les snippets avec leur statut
            for (index, snippet) in self.snippets.iter().enumerate() {
                let label = if snippet.is_saved {
                    format!("{} 💾", snippet.name)
                } else {
                    format!("{} ⚡", snippet.name)  // ⚡ pour indiquer non sauvegardé
                };
                
                if ui.selectable_label(
                    self.selected_snippet == Some(index),
                    label
                ).clicked() {
                    self.selected_snippet = Some(index);
                    self.start_time = Instant::now();
                }
            }

            ui.separator();
            ui.heading("Load from file");
            // Afficher uniquement les fichiers qui ne sont pas déjà chargés
            if let Ok(entries) = fs::read_dir("snippets") {
                for entry in entries.flatten() {
                    if let Some(filename) = entry.file_name().to_str() {
                        if filename.ends_with(".snippets") {
                            // Vérifier si ce fichier n'est pas déjà chargé
                            let already_loaded = self.snippets.iter().any(|s| s.name == filename);
                            if !already_loaded {
                                if ui.selectable_label(false, filename).clicked() {
                                    let mut snippets = Snippets::new();
                                    if let Ok(snippet) = snippets.load_snippets(filename) {
                                        snippets.name = filename.to_string();
                                        self.snippets.push(snippets);
                                        self.selected_snippet = Some(self.snippets.len() - 1);
                                        self.start_time = Instant::now();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(selected_idx) = self.selected_snippet {
                ui.heading("Selected Snippet");
                ui.horizontal(|ui| {
                    if ui.button("Set Next").clicked() {
                        self.snippets_maker.set_next(&self.process, &mut self.state.lock().unwrap());
                    }
                    if ui.button("Create New Snippet").clicked() {
                        let snippets_maker = self.snippets_maker.clone();
                        let mut new_snippets = Snippets::new();
                        new_snippets.load_snippets_memory_path(&self.process, &mut self.state.lock().unwrap(), &snippets_maker);
                        new_snippets.name = format!("New Snippet {}", self.snippets.len() + 1);
                        println!("New snippet loaded");
                        println!("Snippets length: {}", self.snippets.len());
                        self.snippets.push(new_snippets);
                        println!("Snippets length: {}", self.snippets.len());
                        self.selected_snippet = Some(self.snippets.len() - 1);
                        println!("Selected snippet: {}", self.selected_snippet.unwrap());
                        self.start_time = Instant::now();
                    }
                    if ui.button("Save Snippet").clicked() {
                        self.show_save_dialog = true;
                    }
                    if ui.button("Insert to Beatmap").clicked() {
                        if let Some(snippet) = self.snippets.get_mut(selected_idx) {
                            snippet.insert_snippets_to_beatmap(&self.process, &mut self.state.lock().unwrap());
                        }
                    }
                });

                ui.checkbox(&mut self.snippets[selected_idx].should_shuffle, "Shuffle columns on insert");

                // Dialogue de sauvegarde
                if self.show_save_dialog {
                    egui::Window::new("Save Snippet")
                        .collapsible(false)
                        .resizable(false)
                        .show(ctx, |ui| {
                            ui.horizontal(|ui| {
                                ui.label("Filename: ");
                                ui.text_edit_singleline(&mut self.save_filename);
                            });
                            ui.horizontal(|ui| {
                                if ui.button("Save").clicked() && !self.save_filename.is_empty() {
                                    let filename = if !self.save_filename.ends_with(".snippets") {
                                        format!("{}.snippets", self.save_filename)
                                    } else {
                                        self.save_filename.clone()
                                    };
                                    if let Some(snippet) = self.snippets.get_mut(selected_idx) {
                                        snippet.save_snippets_to_file(&filename);
                                        self.show_save_dialog = false;
                                        self.save_filename.clear();
                                    }
                                }
                                if ui.button("Cancel").clicked() {
                                    self.show_save_dialog = false;
                                    self.save_filename.clear();
                                }
                            });
                        });
                }

                // Affichage des informations du snippet
                if let Some(snippet) = self.snippets.get(selected_idx) {
                    ui.label(format!("Number of hit objects: {}", snippet.hit_objects.len()));
                    ui.label(format!("Bpm: {}", (60000.0/snippet.timing_points.beat_len)*self.snippet_speed as f64));
                    ui.add(egui::Slider::new(&mut self.snippet_speed, 0.1..=2.0).text("Snippet Speed"));
                    ui.add(egui::Slider::new(&mut self.scroll_time_ms, 100.0..=1000.0).text("Scroll Time (ms)"));
                    
                    if let Some(renderer) = &mut self.mania_renderer {
                        let current_time = (self.start_time.elapsed().as_secs_f64() * self.snippet_speed as f64) * 1000.0;
                        renderer.render(ui, &snippet.hit_objects, current_time, self.scroll_time_ms, self.snippet_speed as f64);
                    }
                }
            } else {
                ui.heading("Select a snippet from the list");
            }
        });

        ctx.request_repaint();
    }
}

fn main() -> eyre::Result<()> {
    let (mut state, process) = init_loop(500)?;
    println!("Successfully initialized!");
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Snippets Loader",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(AppState::new(Arc::new(process), Arc::new(Mutex::new(state)))))
        }),
    ).unwrap();
    
    Ok(())
}