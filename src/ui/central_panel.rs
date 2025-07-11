use egui;
use crate::ui::app_state::AppState;

pub fn render_central_panel(app_state: &mut AppState, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        if let Some(selected_idx) = app_state.selected_snippet {
            if let Some(snippet) = app_state.snippets.get(selected_idx) {
                let speed = app_state.snippet_speed;
                let scroll_time = app_state.scroll_time_ms;
                let start_time = app_state.start_time;

                if let Some(renderer) = &mut app_state.mania_renderer {
                    let current_time = (start_time.elapsed().as_secs_f64() * speed as f64) * 1000.0;
                    renderer.render(ui, &snippet.hit_objects, current_time, scroll_time, speed as f64, snippet.keycount);
                }
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.heading("Select a snippet from the list");
            });
        }
    });
} 