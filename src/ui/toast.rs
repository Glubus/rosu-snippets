use egui;
use crate::ui::app_state::AppState;

pub fn render_toast(app_state: &mut AppState, ctx: &egui::Context) {
    app_state.update_notification();
    if let Some(notification) = &app_state.notification {
        egui::Window::new("Notification")
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 10.0))
            .collapsible(false)
            .resizable(false)
            .title_bar(false)
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new(&notification.message)
                        .color(egui::Color32::WHITE)
                        .background_color(egui::Color32::from_rgb(70, 70, 70))
                );
            });
    }
} 