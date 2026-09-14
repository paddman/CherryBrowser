use eframe::egui;

use super::{Action, model::{SearchProvider, Workspace}, theme};

pub fn show(ui: &mut egui::Ui, state: &mut Workspace) -> Option<Action> {
    let mut action = None;
    theme::card().show(ui, |ui| {
        theme::section_title(ui, "Make Cherry yours", "Appearance and workspace controls that actually change the app.");
        ui.add_space(14.0);
        if ui.checkbox(&mut state.data.focus_mode, "Focus mode — hide decorative hero and companion").changed() { state.dirty = true; }
        if ui.checkbox(&mut state.data.show_companion, "Show Cherry companion on wide windows").changed() { state.dirty = true; }
        ui.add_space(12.0);
        ui.label("Search provider");
        ui.horizontal_wrapped(|ui| {
            for provider in [SearchProvider::DuckDuckGo, SearchProvider::Bing] {
                if ui.selectable_value(&mut state.data.search_provider, provider, provider.label()).changed() { state.dirty = true; }
            }
        });
        ui.label("Only submitted search terms are sent to the selected provider. No live suggestions or background AI requests.");
        ui.add_space(18.0);
        ui.separator();
        ui.add_space(14.0);
        theme::section_title(ui, "Local workspace", "Bookmarks, notes and these preferences — not your browsing history.");
        ui.add(egui::Label::new(state.storage_label()).wrap());
        ui.label("Storage is plain text, not encrypted. Click Save workspace before closing to retain edits. No account or sync service is connected.");
        if ui.add_enabled(state.can_save() && state.dirty, egui::Button::new("Save workspace")).clicked() { action = Some(Action::SaveWorkspace); }
    });
    ui.add_space(16.0);
    theme::card().show(ui, |ui| {
        theme::section_title(ui, "Engine capabilities", "Milestone 0.3.4 — transparent, not a security score.");
        for (label, detail) in [
            ("Implemented", "HTTP/HTTPS, HTML/CSS subset, images, native font metrics, navigation cancellation and bounded resource loading."),
            ("Not implemented", "JavaScript, forms, browser origin isolation, cookies, web storage, multi-process sandbox, Flexbox and Grid."),
            ("Not connected", "AI provider, tracker-blocking engine, secure-DNS controls, download manager and browser tabs."),
        ] {
            ui.add_space(10.0);
            ui.label(egui::RichText::new(label).strong().color(theme::CYAN));
            ui.label(detail);
        }
        ui.add_space(12.0);
        ui.label("HTTPS is transport encryption, not proof that a website is safe. Modern sites may render only partially.");
        ui.small("Workspace sections are not independent browser tabs. Web content still uses Cherry's own renderer, never a WebView.");
    });
    action
}
