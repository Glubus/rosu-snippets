pub mod mania;
pub mod app_state;
pub mod side_panel;
pub mod central_panel;
pub mod right_panel;
pub mod save_dialog;
pub mod toast;

pub use app_state::AppState;
pub use side_panel::render_side_panel;
pub use central_panel::render_central_panel;
pub use right_panel::render_right_panel;
pub use save_dialog::render_save_dialog;
pub use toast::render_toast; 
