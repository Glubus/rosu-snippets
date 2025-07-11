use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};
use rosu_mem::process::Process;
use rosu_memory_lib::reader::structs::State;
use crate::snippets::structs::{SnippetsMaker, Snippets};
use crate::ui::mania::ManiaRenderer;

pub struct Notification {
    pub message: String,
    pub created_at: Instant,
    pub duration: Duration,
}

impl Notification {
    pub fn new(message: String) -> Self {
        Self {
            message,
            created_at: Instant::now(),
            duration: Duration::from_secs(3), // 3 secondes par défaut
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() >= self.duration
    }
}

#[derive(Clone, Debug)]
pub struct UnloadedSnippet {
    pub name: String,
    pub tags: Vec<String>,
}

pub struct AppState {
    pub snippets: Vec<Snippets>,
    pub unloaded_snippets: Vec<UnloadedSnippet>,
    pub snippets_maker: SnippetsMaker,
    pub selected_snippet: Option<usize>,
    pub mania_renderer: Option<ManiaRenderer>,
    pub start_time: Instant,
    pub scroll_time_ms: f32,
    pub snippet_speed: f32,
    pub show_save_dialog: bool,
    pub save_filename: String,
    pub process: Arc<Process>,
    pub state: Arc<Mutex<State>>,
    pub notification: Option<Notification>,
}

impl AppState {
    pub fn new(process: Arc<Process>, state: Arc<Mutex<State>>) -> Self {
        let mut app_state = Self {
            snippets: Vec::new(),
            unloaded_snippets: Vec::new(),
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
            notification: None,
        };
        app_state.load_available_snippets();
        app_state
    }

    pub fn load_available_snippets(&mut self) {
        if let Ok(entries) = std::fs::read_dir("snippets") {
            for entry in entries.flatten() {
                if let Some(filename) = entry.file_name().to_str() {
                    if filename.ends_with(".snippets") {
                        let mut snippet = Snippets::new();
                        if let Ok(_) = snippet.load_snippets(filename) {
                            self.unloaded_snippets.push(UnloadedSnippet {
                                name: filename.to_string(),
                                tags: snippet.tags.clone(),
                            });
                        }
                    }
                }
            }
        }
    }

    pub fn show_notification(&mut self, message: String) {
        self.notification = Some(Notification::new(message));
    }

    pub fn update_notification(&mut self) {
        if let Some(notification) = &self.notification {
            if notification.is_expired() {
                self.notification = None;
            }
        }
    }
} 