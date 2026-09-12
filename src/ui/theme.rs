use eframe::egui;

pub const BG: egui::Color32 = egui::Color32::from_rgb(4, 9, 21);
pub const PANEL: egui::Color32 = egui::Color32::from_rgb(7, 15, 34);
pub const PANEL_ALT: egui::Color32 = egui::Color32::from_rgb(10, 23, 49);
pub const BLUE: egui::Color32 = egui::Color32::from_rgb(62, 148, 255);
pub const CYAN: egui::Color32 = egui::Color32::from_rgb(75, 224, 255);
pub const VIOLET: egui::Color32 = egui::Color32::from_rgb(147, 92, 255);
pub const TEXT: egui::Color32 = egui::Color32::from_rgb(229, 238, 255);
pub const MUTED: egui::Color32 = egui::Color32::from_rgb(147, 165, 205);
pub const BORDER: egui::Color32 = egui::Color32::from_rgb(35, 75, 139);
pub const GOOD: egui::Color32 = egui::Color32::from_rgb(55, 210, 142);

pub fn install(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BG;
    visuals.window_fill = PANEL;
    visuals.extreme_bg_color = egui::Color32::from_rgb(2, 7, 18);
    visuals.faint_bg_color = egui::Color32::from_rgb(8, 17, 38);
    visuals.selection.bg_fill = egui::Color32::from_rgb(69, 92, 226);
    visuals.hyperlink_color = CYAN;
    visuals.widgets.inactive.bg_fill = PANEL_ALT;
    visuals.widgets.inactive.fg_stroke.color = TEXT;
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(24, 48, 94);
    visuals.widgets.hovered.fg_stroke.color = egui::Color32::WHITE;
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(53, 68, 148);
    visuals.widgets.active.fg_stroke.color = egui::Color32::WHITE;
    visuals.widgets.open.bg_fill = egui::Color32::from_rgb(25, 49, 98);
    ctx.set_visuals(visuals);
}

pub fn card() -> egui::Frame {
    egui::Frame::default()
        .fill(PANEL)
        .stroke(egui::Stroke::new(1.0, BORDER))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::same(12))
}

pub fn section_title(ui: &mut egui::Ui, title: &str, subtitle: &str) {
    ui.label(egui::RichText::new(title).size(17.0).strong().color(TEXT));
    ui.label(egui::RichText::new(subtitle).size(11.0).color(MUTED));
}
