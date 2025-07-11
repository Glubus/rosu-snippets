use egui;
use crate::ui::app_state::AppState;
use crate::ui::save_dialog;
use crate::snippets::structs::{Snippets, NextUpdate};

fn render_snippet_settings(snippet: &mut Snippets, ui: &mut egui::Ui) {
    ui.checkbox(&mut snippet.should_shuffle, "Shuffle columns on insert");
    
    ui.group(|ui| {
        ui.heading("Tags");
        if ui.button("Add Tag").clicked() {
            snippet.tags.push(String::new());
        }
        
        let mut tags_to_remove = Vec::new();
        for (idx, tag) in snippet.tags.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(tag);
                if ui.button("❌").clicked() {
                    tags_to_remove.push(idx);
                }
            });
        }
        
        for &idx in tags_to_remove.iter().rev() {
            snippet.tags.remove(idx);
        }
    });
}

fn render_snippet_info(snippet: &Snippets, speed: f64, ui: &mut egui::Ui) {
    ui.heading(egui::RichText::new(&snippet.name).size(24.0).strong());
    
    if !snippet.tags.is_empty() {
        ui.horizontal(|ui| {
            for tag in &snippet.tags {
                ui.label(
                    egui::RichText::new(tag)
                        .background_color(egui::Color32::from_rgb(70, 70, 70))
                        .color(egui::Color32::WHITE)
                );
                ui.add_space(4.0);
            }
        });
    }
    
    ui.group(|ui| {
        ui.label(format!("Number of hit objects: {}", snippet.hit_objects.len()));
        ui.label(format!("Key count: {}", snippet.keycount));
        ui.label(format!("Bpm: {}", (60000.0/snippet.timing_points.beat_len)*speed));
    });
}

fn render_creation_controls(app_state: &mut AppState, ui: &mut egui::Ui) {
    ui.group(|ui| {
        ui.heading("New Snippet Controls");
        if ui.button("Set Next").clicked() {
            let result = app_state.snippets_maker.set_next(&app_state.process, &mut app_state.state.lock().unwrap());
            let message = match result {
                Ok(NextUpdate::TimeStart) => format!("End at {}", app_state.snippets_maker.time_end),
                Ok(NextUpdate::TimeEnd) => format!("Start at {}", app_state.snippets_maker.time_start),
                Err(e) => format!("Error: {}", e),
            };
            app_state.show_notification(message);
        }
        if ui.button("Create New Snippet").clicked() {
            let snippets_maker = app_state.snippets_maker.clone();
            let mut new_snippets = Snippets::new();
            let creation_result = new_snippets.load_snippets_memory_path(
                &app_state.process, 
                &mut app_state.state.lock().unwrap(), 
                &snippets_maker
            );
            
            match creation_result {
                Ok(_) => {
                    new_snippets.name = format!("New Snippet {}", app_state.snippets.len() + 1);
                    app_state.snippets.push(new_snippets);
                    app_state.selected_snippet = Some(app_state.snippets.len() - 1);
                    app_state.start_time = std::time::Instant::now();
                    app_state.show_notification("New snippet created".to_string());
                }
                Err(_) => {
                    app_state.show_notification("Failed to create snippet".to_string());
                }
            }
        }
    });
}

fn render_playback_controls(app_state: &mut AppState, ui: &mut egui::Ui) {
    ui.group(|ui| {
        ui.heading("Playback Controls");
        ui.add(egui::Slider::new(&mut app_state.snippet_speed, 0.1..=2.0).text("Snippet Speed"));
        ui.add(egui::Slider::new(&mut app_state.scroll_time_ms, 100.0..=1000.0).text("Scroll Time (ms)"));
    });
}

fn render_snippet_controls(app_state: &mut AppState, ui: &mut egui::Ui) {
    ui.group(|ui| {
        ui.heading("Snippet Controls");
        ui.horizontal(|ui| {
            if ui.button("Save Snippet").clicked() {
                app_state.show_save_dialog = true;
            }
            if let Some(selected_idx) = app_state.selected_snippet {
                if let Some(snippet) = app_state.snippets.get(selected_idx) {
                    let is_saved = snippet.is_saved;
                    let name = snippet.name.clone();
                    if is_saved && ui.button("Update").clicked() {
                        if let Some(snippet) = app_state.snippets.get_mut(selected_idx) {
                            if let Ok(_) = snippet.save_snippets_to_file(&name) {
                                app_state.show_notification(format!("Updated {}", name));
                            }
                        }
                    }
                }
            }
            if ui.button("Insert to Beatmap").clicked() {
                if let Some(selected_idx) = app_state.selected_snippet {
                    if let Some(snippet) = app_state.snippets.get_mut(selected_idx) {
                        let name = snippet.name.clone();
                        snippet.insert_snippets_to_beatmap(&app_state.process, &mut app_state.state.lock().unwrap());
                        app_state.show_notification(format!("Successfully inserted {}", name));
                    }
                }
            }
        });
    });
}

pub fn render_right_panel(app_state: &mut AppState, ctx: &egui::Context) {
    egui::SidePanel::right("info_panel").show(ctx, |ui| {
        render_creation_controls(app_state, ui);
        ui.add_space(8.0);

        if let Some(selected_idx) = app_state.selected_snippet {
            render_snippet_controls(app_state, ui);
            ui.add_space(8.0);
            
            if let Some(snippet) = app_state.snippets.get_mut(selected_idx) {
                render_snippet_info(snippet, app_state.snippet_speed as f64, ui);
                ui.add_space(8.0);
                render_snippet_settings(snippet, ui);
                ui.add_space(8.0);
                render_playback_controls(app_state, ui);
            }
        }
    });

    save_dialog::render_save_dialog(app_state, ctx);
} 