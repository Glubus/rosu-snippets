use std::fs;
use std::collections::HashMap;
use egui;
use crate::ui::app_state::{AppState, UnloadedSnippet};
use crate::snippets::structs::Snippets;

#[derive(Default)]
struct SidebarAction {
    select_snippet: Option<usize>,
    load_file: Option<String>,
}

fn render_snippet_entry(snippet: &Snippets, index: usize, is_selected: bool, ui: &mut egui::Ui) -> Option<usize> {
    let mut label = if snippet.is_saved {
        format!("{} 💾", snippet.name)
    } else {
        format!("{} ⚡", snippet.name)
    };

    if !snippet.tags.is_empty() {
        label.push_str("\n");
        label.push_str(&snippet.tags.join(", "));
    }

    if ui.selectable_label(is_selected, label).clicked() {
        Some(index)
    } else {
        None
    }
}

pub fn render_side_panel(app_state: &mut AppState, ctx: &egui::Context) {
    let mut action = SidebarAction::default();

    egui::SidePanel::left("snippets_list").show(ctx, |ui| {
        ui.heading("Snippets");
        
        let mut tagged_snippets: HashMap<String, Vec<(usize, &Snippets)>> = HashMap::new();
        let mut untagged_snippets = Vec::new();
        let selected_idx = app_state.selected_snippet;

        for (index, snippet) in app_state.snippets.iter().enumerate() {
            if snippet.tags.is_empty() {
                untagged_snippets.push((index, snippet));
            } else if let Some(first_tag) = snippet.tags.first() {
                tagged_snippets.entry(first_tag.clone())
                    .or_default()
                    .push((index, snippet));
            }
        }

        // Sort tags alphabetically
        let mut sorted_tags: Vec<_> = tagged_snippets.keys().collect();
        sorted_tags.sort();

        for tag in sorted_tags {
            if let Some(snippets) = tagged_snippets.get(tag) {
                ui.collapsing(tag, |ui| {
                    for &(index, snippet) in snippets {
                        if let Some(idx) = render_snippet_entry(snippet, index, selected_idx == Some(index), ui) {
                            action.select_snippet = Some(idx);
                        }
                    }
                });
            }
        }

        if !untagged_snippets.is_empty() {
            ui.collapsing("Untagged", |ui| {
                for (index, snippet) in untagged_snippets {
                    if let Some(idx) = render_snippet_entry(snippet, index, selected_idx == Some(index), ui) {
                        action.select_snippet = Some(idx);
                    }
                }
            });
        }

        ui.separator();
        ui.heading("Load from file");
        
        let mut unloaded_by_tag: HashMap<String, Vec<&UnloadedSnippet>> = HashMap::new();
        let mut untagged_unloaded = Vec::new();
        
        for unloaded in &app_state.unloaded_snippets {
            let already_loaded = app_state.snippets.iter().any(|s| s.name == unloaded.name);
            if !already_loaded {
                if unloaded.tags.is_empty() {
                    untagged_unloaded.push(unloaded);
                } else if let Some(first_tag) = unloaded.tags.first() {
                    unloaded_by_tag.entry(first_tag.clone())
                        .or_default()
                        .push(unloaded);
                }
            }
        }

        // Sort unloaded tags alphabetically
        let mut sorted_unloaded_tags: Vec<_> = unloaded_by_tag.keys().collect();
        sorted_unloaded_tags.sort();

        for tag in sorted_unloaded_tags {
            if let Some(snippets) = unloaded_by_tag.get(tag) {
                ui.collapsing(tag, |ui| {
                    for unloaded in snippets {
                        if ui.selectable_label(false, &unloaded.name).clicked() {
                            action.load_file = Some(unloaded.name.clone());
                        }
                    }
                });
            }
        }

        if !untagged_unloaded.is_empty() {
            ui.collapsing("Untagged", |ui| {
                for unloaded in untagged_unloaded {
                    if ui.selectable_label(false, &unloaded.name).clicked() {
                        action.load_file = Some(unloaded.name.clone());
                    }
                }
            });
        }
    });

    // Apply actions after all borrows are done
    if let Some(index) = action.select_snippet {
        app_state.selected_snippet = Some(index);
        app_state.start_time = std::time::Instant::now();
    }
    
    if let Some(filename) = action.load_file {
        let mut snippets = Snippets::new();
        if let Ok(_) = snippets.load_snippets(&filename) {
            snippets.name = filename;
            app_state.snippets.push(snippets);
            app_state.selected_snippet = Some(app_state.snippets.len() - 1);
            app_state.start_time = std::time::Instant::now();
        }
    }
} 