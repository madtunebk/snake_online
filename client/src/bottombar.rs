use crate::theme::ACCENT;
use eframe::egui::{self, Align, Frame, Layout, RichText, TopBottomPanel};

#[derive(Default)]
pub struct BottomBar;

impl BottomBar {
    pub fn ui(&mut self, ctx: &egui::Context) {
        TopBottomPanel::bottom("bottombar")
            .exact_height(40.0)
            .frame(Self::frame())
            .show(ctx, |ui| {
                let (ms, fps) = Self::frame_stats(ctx);
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("frame: {ms:.1} ms  •  {fps:.0} fps")).monospace(),
                    );
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(RichText::new("client: connected").monospace());
                });
            });
    }

    fn frame() -> Frame {
        Frame {
            inner_margin: egui::Margin::symmetric(10.0, 6.0),
            fill: egui::Color32::from_black_alpha(255),
            stroke: egui::Stroke::new(1.0, ACCENT),
            ..Default::default()
        }
    }

    fn frame_stats(ctx: &egui::Context) -> (f32, f32) {
        let dt = ctx.input(|i| i.stable_dt).max(1.0 / 240.0);
        (dt as f32 * 1000.0, (1.0 / dt) as f32)
    }
}
