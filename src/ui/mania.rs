use egui::{self, Rect, Vec2, pos2};
use rosu_map::section::hit_objects::{HitObject, HitObjectKind};
pub struct ManiaRenderer {
    column_width: f32,
    note_size: f32,  // On utilise une seule taille pour avoir un carré parfait
    snippet_speed: f64,
}
impl ManiaRenderer {
    pub fn new() -> Self {
        Self {
            column_width: 80.0,
            note_size: 80.0,  // Taille unique pour le carré
            snippet_speed: 1.0,
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, hit_objects: &[HitObject], mut current_time: f64, scroll_time_ms: f32, snippet_speed: f64) {
        self.snippet_speed = snippet_speed;
        egui::Frame::dark_canvas(ui.style())
            .show(ui, |ui| {
                let available_rect = ui.available_rect_before_wrap();
                let height = available_rect.height();
                
                // Zone de jeu (les 4 colonnes)
                let play_area = Rect::from_min_size(
                    pos2(available_rect.min.x, available_rect.min.y),
                    Vec2::new(self.column_width * 4.0, height),
                );

                // Définir la zone de clipping
                let clip_rect = ui.clip_rect().intersect(play_area);
                ui.set_clip_rect(clip_rect);
                
                // Draw columns
                for i in 0..4 {
                    let column_rect = Rect::from_min_size(
                        pos2(
                            available_rect.min.x + (i as f32 * self.column_width),
                            available_rect.min.y,
                        ),
                        Vec2::new(self.column_width, height),
                    );
                    ui.painter().rect_filled(
                        column_rect,
                        0.0,
                        egui::Color32::from_gray(20),
                    );
                }

                let judgment_line_y = available_rect.max.y - 100.0;
                ui.painter().line_segment(
                    [
                        pos2(available_rect.min.x, judgment_line_y),
                        pos2(available_rect.min.x + self.column_width * 4.0, judgment_line_y)
                    ],
                    egui::Stroke::new(2.0, egui::Color32::WHITE),
                );

                if let Some(last) = hit_objects.last() {
                    // La durée du pattern inclut maintenant le scroll_time_ms à la fin
                    let pattern_duration = last.start_time + scroll_time_ms as f64;
                    current_time %= pattern_duration;

                    let note_image = egui::Image::new(egui::include_image!("../../assets/note.png"));
                    
                    for hit_object in hit_objects {
                        // On ajoute scroll_time_ms au temps de la note pour qu'elle commence à apparaître plus tôt
                        let note_time = hit_object.start_time + scroll_time_ms as f64;
                        let time_diff = note_time - current_time;
                        // On divise time_diff par la même vitesse pour annuler son effet sur la vitesse de défilement
                        let adjusted_time_diff = time_diff / self.snippet_speed;
                        let y_pos = judgment_line_y - ((adjusted_time_diff as f32) / scroll_time_ms) * height;

                        if y_pos > available_rect.min.y && y_pos < height {
                            let mut x_pos = 0.0;
                            match hit_object.kind {
                                HitObjectKind::Circle(h) => {
                                    let column = (h.pos.x / 512.0 * 4.0) as usize % 4;
                                    x_pos = available_rect.min.x + (column as f32 * self.column_width);
                                }
                                HitObjectKind::Hold(h) => {
                                    let column = (h.pos_x / 512.0 * 4.0) as usize % 4;
                                    x_pos = available_rect.min.x + (column as f32 * self.column_width);
                                }
                                _ => continue,
                            }
                            
                            note_image.paint_at(
                                ui,
                                Rect::from_min_size(
                                    pos2(x_pos + (self.column_width - self.note_size) / 2.0, y_pos - self.note_size/2.0),  // Centrer dans la colonne
                                    Vec2::new(self.note_size, self.note_size),  // Carré parfait
                                )
                            );
                        }
                    }
                }
            });
    }
}