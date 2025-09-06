use eframe::egui::{self, Color32, FontId, TextStyle, Visuals};

// Accent color used across frames and strokes
pub const ACCENT: Color32 = Color32::from_rgb(200, 100, 150);

pub fn apply(cc: &eframe::CreationContext<'_>) {
    let mut style = (*cc.egui_ctx.style()).clone();
    style.visuals = Visuals::dark();

    // Accented strokes and selections
    style.visuals.widgets.inactive.bg_stroke.color = ACCENT;
    style.visuals.selection.bg_fill = ACCENT.linear_multiply(0.30);
    style.visuals.widgets.hovered.bg_stroke.color = ACCENT;
    style.visuals.widgets.active.bg_stroke.color = ACCENT;

    // Slightly larger, readable defaults
    style
        .text_styles
        .insert(TextStyle::Body, FontId::proportional(18.0));
    style
        .text_styles
        .insert(TextStyle::Button, FontId::proportional(18.0));
    style
        .text_styles
        .insert(TextStyle::Monospace, FontId::monospace(16.0));
    style
        .text_styles
        .insert(TextStyle::Heading, FontId::proportional(22.0));

    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 8.0);
    style.spacing.window_margin = egui::Margin::same(8.0);
    style.visuals.window_rounding = 8.0.into();
    style.visuals.widgets.inactive.rounding = 6.0.into();

    cc.egui_ctx.set_style(style);
}
