use egui::{self, Rect, Vec2, pos2};
use rosu_map::section::hit_objects::{HitObject, HitObjectKind};

pub struct ManiaRenderer {
    column_width: f32,
    note_size: f32,
    snippet_speed: f64,
}

impl ManiaRenderer {
    pub fn new() -> Self {
        Self {
            column_width: 65.0,
            note_size: 65.0,
            snippet_speed: 1.0,
        }
    }

    fn render_hold(&self, ui: &mut egui::Ui, x_pos: f32, start_y: f32, end_y: f32,  judgment_line_y: f32) {
        let note_width = self.note_size * 0.8;
        let x_center = x_pos + (self.column_width - note_width) / 2.0;

        // Couleurs
        let body_color = egui::Color32::from_rgb(200, 200, 200); // Blanc gris pour le body
        let cap_color = egui::Color32::from_rgb(0, 174, 255);    // Bleu pour le cap
        
        // Hold body (rectangle)
        // On utilise directement start_y et end_y, mais on limite à la judgment line
        let height = (end_y - start_y).abs();
        let y_start = start_y.min(end_y);
        let y_end = (start_y.max(end_y)).min(judgment_line_y);
        let visible_height = (y_end - y_start).abs();
        
        // On dessine toujours le body, le clipping s'occupera de la visibilité
        ui.painter().rect_filled(
            Rect::from_min_size(
                pos2(x_center, y_start),
                Vec2::new(note_width, visible_height),
            ),
            0.0,
            body_color,
        );

        // Hold end (petit rectangle)
        let cap_height = note_width * 0.3;
        if end_y <= judgment_line_y {
            ui.painter().rect_filled(
                Rect::from_min_size(
                    pos2(x_center, end_y),
                    Vec2::new(note_width, cap_height),
                ),
                0.0,
                cap_color,
            );
        }
    }

    pub fn render(&mut self, ui: &mut egui::Ui, hit_objects: &[HitObject], mut current_time: f64, scroll_time_ms: f32, snippet_speed: f64, keycount: usize) {
        self.snippet_speed = snippet_speed;
        egui::Frame::dark_canvas(ui.style())
            .show(ui, |ui| {
                let available_rect = ui.available_rect_before_wrap();
                let height = available_rect.height();
                
                // Zone de jeu (colonnes)
                let play_area = Rect::from_min_size(
                    pos2(available_rect.min.x, available_rect.min.y),
                    Vec2::new(self.column_width * keycount as f32, height),
                );

                // Définir la zone de clipping
                let clip_rect = ui.clip_rect().intersect(play_area);
                ui.set_clip_rect(clip_rect);
                
                // Draw columns
                for i in 0..keycount {
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
                        pos2(available_rect.min.x + self.column_width * keycount as f32, judgment_line_y)
                    ],
                    egui::Stroke::new(2.0, egui::Color32::WHITE),
                );

                if let Some(last) = hit_objects.last() {
                    // Calculate total duration including the last note's hold duration if it's a hold note
                    let mut pattern_duration = last.start_time;
                    if let HitObjectKind::Hold(h) = &last.kind {
                        pattern_duration += h.duration;
                    }
                    pattern_duration += scroll_time_ms as f64;
                    current_time %= pattern_duration;

                    let note_image = egui::Image::new(egui::include_image!("../../assets/note.png"));
                    
                    // Draw hold notes first so they appear behind regular notes
                    for hit_object in hit_objects.iter().filter(|h| matches!(h.kind, HitObjectKind::Hold(_))) {
                        if let HitObjectKind::Hold(h) = &hit_object.kind {
                            let column = (h.pos_x / 512.0 * keycount as f32) as usize % keycount;
                            let x_pos = available_rect.min.x + (column as f32 * self.column_width);
                            
                            let note_time = hit_object.start_time + scroll_time_ms as f64;
                            let end_time = hit_object.start_time + h.duration + scroll_time_ms as f64;
                            
                            let time_diff = note_time - current_time;
                            let end_time_diff = end_time - current_time;
                            
                            let adjusted_time_diff = time_diff / self.snippet_speed;
                            let adjusted_end_diff = end_time_diff / self.snippet_speed;
                            
                            let y_pos = judgment_line_y - ((adjusted_time_diff as f32) / scroll_time_ms) * height;
                            let end_y_pos = judgment_line_y - ((adjusted_end_diff as f32) / scroll_time_ms) * height;
                            
                            // Draw hold if the end hasn't passed the judgment line yet
                            if end_y_pos <= judgment_line_y {
                                self.render_hold(ui, x_pos, y_pos, end_y_pos, judgment_line_y);
                            }
                        }
                    }

                    // Then draw regular notes and hold heads
                    for hit_object in hit_objects {
                        let note_time = hit_object.start_time + scroll_time_ms as f64;
                        let time_diff = note_time - current_time;
                        let adjusted_time_diff = time_diff / self.snippet_speed;
                        let y_pos = judgment_line_y - ((adjusted_time_diff as f32) / scroll_time_ms) * height;

                        // Check if note is visible (above the bottom of the screen)
                        if y_pos <= judgment_line_y {
                            let x_pos = match &hit_object.kind {
                                HitObjectKind::Circle(h) => {
                                    let column = (h.pos.x / 512.0 * keycount as f32) as usize % keycount;
                                    available_rect.min.x + (column as f32 * self.column_width)
                                }
                                HitObjectKind::Hold(h) => {
                                    let column = (h.pos_x / 512.0 * keycount as f32) as usize % keycount;
                                    available_rect.min.x + (column as f32 * self.column_width)
                                }
                                _ => continue,
                            };
                            
                            // Only draw if note is within visible area
                            if y_pos >= available_rect.min.y {
                                note_image.paint_at(
                                    ui,
                                    Rect::from_min_size(
                                        pos2(x_pos + (self.column_width - self.note_size) / 2.0, y_pos - self.note_size/2.0),
                                        Vec2::new(self.note_size, self.note_size),
                                    )
                                );
                            }
                        }
                    }
                }
            });
    }
}