use egui;
use crate::ui::app_state::AppState;

pub fn render_save_dialog(app_state: &mut AppState, ctx: &egui::Context) {
    if app_state.show_save_dialog {
        egui::Window::new("Save Snippet")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Filename: ");
                    ui.text_edit_singleline(&mut app_state.save_filename);
                });
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() && !app_state.save_filename.is_empty() {
                        let filename = if !app_state.save_filename.ends_with(".snippets") {
                            format!("{}.snippets", app_state.save_filename)
                        } else {
                            app_state.save_filename.clone()
                        };
                        if let Some(selected_idx) = app_state.selected_snippet {
                            if let Some(snippet) = app_state.snippets.get_mut(selected_idx) {
                                snippet.save_snippets_to_file(&filename);
                                app_state.show_notification(format!("Successfully saved {}", filename));
                                app_state.show_save_dialog = false;
                                app_state.save_filename.clear();
                            }
                        }
                    }
                    if ui.button("Cancel").clicked() {
                        app_state.show_save_dialog = false;
                        app_state.save_filename.clear();
                    }
                });
            });
    }
} 