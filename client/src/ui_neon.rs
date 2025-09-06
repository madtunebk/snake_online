use eframe::egui::{self, Color32, Frame, Margin, Rounding, Stroke};
use eframe::epaint::Shadow;

#[derive(Clone, Copy)]
pub struct NeonTheme {
    pub panel_fill: Color32,
    pub panel_stroke: Color32,
    pub rounding: f32,
}

impl Default for NeonTheme {
    fn default() -> Self {
        Self {
            panel_fill: Color32::from_rgba_unmultiplied(16, 22, 26, 235),
            // Vibrant neon pink
            panel_stroke: Color32::from_rgb(255, 96, 180),
            rounding: 10.0,
        }
    }
}

pub fn neon_popup_frame(ui: &egui::Ui, theme: NeonTheme) -> Frame {
    // Base on popup but with our colors
    let mut f = Frame::popup(ui.style());
    f = f
        .rounding(Rounding::same(theme.rounding))
        .stroke(Stroke::new(1.5, theme.panel_stroke))
        .fill(theme.panel_fill)
        .inner_margin(Margin::same(12.0))
        .outer_margin(Margin::same(4.0))
        .shadow(Shadow {
            offset: egui::vec2(0.0, 6.0),
            blur: 24.0,
            spread: 0.0,
            color: theme.panel_stroke.linear_multiply(0.15),
        });
    f
}

// Leave only the popup frame, used by overlays/menus.
