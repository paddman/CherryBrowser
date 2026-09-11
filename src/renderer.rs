use std::collections::HashMap;

use eframe::egui;

use crate::{
    dom::NodeId,
    image_data::DecodedImage,
    layout::{LayoutDocument, PaintItem, RectF, Rgba},
};

#[derive(Debug, Clone, Default)]
pub struct RenderOutcome {
    pub clicked_href: Option<String>,
    pub hovered_href: Option<String>,
}

pub fn show_document(
    ui: &mut egui::Ui,
    document: &LayoutDocument,
    images: &HashMap<NodeId, DecodedImage>,
    textures: &mut HashMap<NodeId, egui::TextureHandle>,
) -> RenderOutcome {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_min_size(egui::vec2(
                document.width.max(ui.available_width()),
                document.height.max(ui.available_height()),
            ));

            let origin = ui.min_rect().min;
            let painter = ui.painter().clone();
            let context = ui.ctx().clone();
            let mut outcome = RenderOutcome::default();

            // Block backgrounds are currently emitted after their descendants once
            // their final height is known. Paint rectangles in reverse emission order
            // so outer backgrounds land below nested backgrounds and image placeholders.
            for item in document.items.iter().rev() {
                if let PaintItem::Rect(rect) = item {
                    painter.rect_filled(
                        to_egui_rect(origin, rect.rect),
                        egui::CornerRadius::ZERO,
                        to_color32(rect.color),
                    );
                }
            }

            for item in &document.items {
                let PaintItem::Image(image_item) = item else {
                    continue;
                };
                let Some(decoded) = images.get(&image_item.node) else {
                    continue;
                };

                let texture = textures.entry(image_item.node).or_insert_with(|| {
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                        [decoded.width as usize, decoded.height as usize],
                        &decoded.rgba,
                    );
                    context.load_texture(
                        format!("cherry-image-{}", image_item.node),
                        color_image,
                        egui::TextureOptions::LINEAR,
                    )
                });

                painter.image(
                    texture.id(),
                    to_egui_rect(origin, image_item.rect),
                    egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            }

            for (index, item) in document.items.iter().enumerate() {
                let PaintItem::Text(text) = item else {
                    continue;
                };

                let position = origin + egui::vec2(text.rect.x, text.rect.y);
                let font_family = if text.monospace {
                    egui::FontFamily::Monospace
                } else {
                    egui::FontFamily::Proportional
                };
                let font = egui::FontId::new(text.font_size, font_family);
                let color = to_color32(text.color);

                painter.text(
                    position,
                    egui::Align2::LEFT_TOP,
                    &text.text,
                    font.clone(),
                    color,
                );

                if text.bold {
                    painter.text(
                        position + egui::vec2(0.45, 0.0),
                        egui::Align2::LEFT_TOP,
                        &text.text,
                        font,
                        color,
                    );
                }

                if text.underline {
                    let underline_y = position.y + text.font_size + 2.0;
                    painter.line_segment(
                        [
                            egui::pos2(position.x, underline_y),
                            egui::pos2(position.x + text.rect.width, underline_y),
                        ],
                        egui::Stroke::new(1.0, color),
                    );
                }

                if let Some(href) = &text.href {
                    let response = ui
                        .interact(
                            to_egui_rect(origin, text.rect),
                            ui.make_persistent_id(("cherry-link", index)),
                            egui::Sense::click(),
                        )
                        .on_hover_cursor(egui::CursorIcon::PointingHand);

                    if response.hovered() {
                        outcome.hovered_href = Some(href.clone());
                    }
                    if response.clicked() {
                        outcome.clicked_href = Some(href.clone());
                    }
                }
            }

            outcome
        })
        .inner
}

fn to_egui_rect(origin: egui::Pos2, rect: RectF) -> egui::Rect {
    egui::Rect::from_min_size(
        origin + egui::vec2(rect.x, rect.y),
        egui::vec2(rect.width.max(0.0), rect.height.max(0.0)),
    )
}

fn to_color32(color: Rgba) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(color.r, color.g, color.b, color.a)
}
