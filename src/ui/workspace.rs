use eframe::egui;

use super::{Action, ShellPage, model::Workspace, theme};

pub fn quick_links(ui: &mut egui::Ui) -> Option<Action> {
    let mut action = None;
    ui.horizontal_wrapped(|ui| {
        for (title, url) in [
            ("Example page", "https://example.com/"),
            ("Rust", "https://www.rust-lang.org/"),
            ("IANA", "https://www.iana.org/"),
            ("GitHub", "https://github.com/paddman/CherryBrowser"),
        ] {
            if ui.add(egui::Button::new(title).min_size(egui::vec2(110.0, 38.0))).on_hover_text(url).clicked() {
                action = Some(Action::Open(url.into()));
            }
        }
    });
    action
}

pub fn bookmarks(ui: &mut egui::Ui, state: &mut Workspace, full: bool) -> Option<Action> {
    let mut action = None;
    theme::card().show(ui, |ui| {
        theme::section_title(ui, "Bookmarks", "Keep useful pages close. Opened inside Cherry's own engine.");
        ui.add_space(12.0);
        if full {
            ui.add(egui::TextEdit::singleline(&mut state.filter).hint_text("Filter by title or URL").desired_width(f32::INFINITY).char_limit(256));
            ui.add_space(10.0);
        }
        let query = state.filter.to_lowercase();
        let mut remove = None;
        let mut edit = None;
        let mut count = 0;
        for (index, item) in state.data.bookmarks.iter().enumerate() {
            if full && !format!("{} {}", item.title, item.url).to_lowercase().contains(&query) {
                continue;
            }
            if !full && count >= 6 {
                break;
            }
            count += 1;
            ui.push_id(index, |ui| {
                if ui.add_sized([ui.available_width(), 34.0], egui::Button::new(&item.title)).on_hover_text(&item.url).clicked() {
                    action = Some(Action::Open(item.url.clone()));
                }
                if full {
                    ui.add(egui::Label::new(egui::RichText::new(&item.url).size(12.0).color(theme::MUTED)).wrap());
                    ui.horizontal(|ui| {
                        if ui.small_button("Edit").clicked() { edit = Some(item.clone()); }
                        if ui.small_button("Remove").clicked() { remove = Some(index); }
                    });
                }
                ui.add_space(8.0);
            });
        }
        if let Some(index) = remove {
            let removed = state.data.bookmarks.remove(index);
            if state.editing_url.as_deref() == Some(&removed.url) {
                state.editing_url = None;
                state.draft_title.clear();
                state.draft_url.clear();
            }
            state.dirty = true;
        }
        if let Some(item) = edit {
            state.draft_title = item.title;
            state.draft_url = item.url.clone();
            state.editing_url = Some(item.url);
        }
        if count == 0 {
            ui.label(if state.data.bookmarks.is_empty() { "No bookmarks yet. Add a link below or save a loaded page with Ctrl/Cmd+D." } else { "No matching bookmarks." });
        }
        if full {
            ui.separator();
            ui.add_space(10.0);
            ui.label(if state.editing_url.is_some() { "Edit bookmark" } else { "Add a bookmark" });
            ui.add(egui::TextEdit::singleline(&mut state.draft_title).hint_text("Title").desired_width(f32::INFINITY).char_limit(120));
            ui.add(egui::TextEdit::singleline(&mut state.draft_url).hint_text("https://example.com").desired_width(f32::INFINITY).char_limit(4096));
            ui.horizontal_wrapped(|ui| {
                if theme::primary_button(ui, "Save link") { state.save_draft(); }
                if state.editing_url.is_some() && ui.button("Cancel edit").clicked() {
                    state.editing_url = None;
                    state.draft_title.clear();
                    state.draft_url.clear();
                }
            });
            ui.small("HTTP/HTTPS only. Up to 128 bookmarks. Save workspace to persist edits.");
        } else if ui.button("Manage bookmarks").clicked() {
            action = Some(Action::Section(ShellPage::Bookmarks));
        }
    });
    action
}

pub fn history(ui: &mut egui::Ui, urls: &[String], filter: &mut String) -> Option<Action> {
    let mut action = None;
    theme::card().show(ui, |ui| {
        theme::section_title(ui, "History", "Completed navigation in this window. Never saved to the workspace file.");
        ui.add_space(12.0);
        ui.add(egui::TextEdit::singleline(filter).hint_text("Filter URLs").desired_width(f32::INFINITY).char_limit(256));
        let query = filter.to_lowercase();
        let mut matches = 0;
        for (index, url) in urls.iter().enumerate().rev() {
            if !url.to_lowercase().contains(&query) { continue; }
            matches += 1;
            ui.push_id(index, |ui| {
                if ui.add(egui::Button::new(url)).clicked() { action = Some(Action::Open(url.clone())); }
            });
        }
        if matches == 0 { ui.label(if urls.is_empty() { "No completed navigation yet." } else { "No matching history." }); }
        ui.add_space(12.0);
        if ui.add_enabled(!urls.is_empty(), egui::Button::new("Clear this window's history")).clicked() {
            action = Some(Action::ClearHistory);
        }
        ui.small("Clearing also resets Back/Forward. The currently displayed page stays open.");
    });
    action
}
