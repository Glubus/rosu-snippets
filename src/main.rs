mod utils;
mod snippets;
mod ui;

use rosu_memory_lib::init_loop;
use std::sync::{Arc, Mutex};
use ui::{AppState, render_side_panel, render_central_panel, render_right_panel, render_toast};

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.mania_renderer.is_none() {
            self.mania_renderer = Some(ui::mania::ManiaRenderer::new());
        }

        render_toast(self, ctx);
        render_side_panel(self, ctx);
        render_central_panel(self, ctx);
        render_right_panel(self, ctx);
        
        ctx.request_repaint();
    }
}

fn main() -> eyre::Result<()> {
    let (mut state, process) = init_loop(500)?;
    println!("Successfully initialized!");
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0]),
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