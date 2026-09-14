use eframe::egui;

pub const BG: egui::Color32 = egui::Color32::from_rgb(8, 12, 23);
pub const PANEL: egui::Color32 = egui::Color32::from_rgb(15, 22, 39);
pub const PANEL_ALT: egui::Color32 = egui::Color32::from_rgb(21, 31, 52);
pub const BLUE: egui::Color32 = egui::Color32::from_rgb(96, 169, 255);
pub const CYAN: egui::Color32 = egui::Color32::from_rgb(113, 220, 239);
pub const VIOLET: egui::Color32 = egui::Color32::from_rgb(177, 150, 255);
pub const TEXT: egui::Color32 = egui::Color32::from_rgb(233, 239, 251);
pub const MUTED: egui::Color32 = egui::Color32::from_rgb(163, 177, 203);
pub const BORDER: egui::Color32 = egui::Color32::from_rgb(42, 56, 79);
pub const GOOD: egui::Color32 = egui::Color32::from_rgb(105, 218, 178);

pub fn install(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BG;
    visuals.window_fill = PANEL;
    visuals.extreme_bg_color = BG;
    visuals.faint_bg_color = PANEL_ALT;
    visuals.selection.bg_fill = egui::Color32::from_rgb(45, 79, 143);
    visuals.hyperlink_color = CYAN;
    visuals.widgets.inactive.bg_fill = PANEL_ALT;
    visuals.widgets.inactive.fg_stroke.color = TEXT;
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(35, 55, 87);
    visuals.widgets.hovered.fg_stroke.color = egui::Color32::WHITE;
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(41, 74, 132);
    visuals.widgets.active.fg_stroke.color = egui::Color32::WHITE;
    visuals.widgets.open.bg_fill = PANEL_ALT;
    ctx.set_visuals(visuals);
}

pub fn card() -> egui::Frame {
    egui::Frame::default()
        .fill(PANEL)
        .stroke(egui::Stroke::new(1.0, BORDER))
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(egui::Margin::same(16))
}

pub fn section_title(ui: &mut egui::Ui, title: &str, subtitle: &str) {
    ui.label(egui::RichText::new(title).size(21.0).strong().color(TEXT));
    ui.label(egui::RichText::new(subtitle).size(13.0).color(MUTED));
}

pub fn primary_button(ui: &mut egui::Ui, label: &str) -> bool {
    ui.add(
        egui::Button::new(egui::RichText::new(label).color(TEXT))
            .fill(egui::Color32::from_rgb(38, 80, 148)),
    )
    .clicked()
}
