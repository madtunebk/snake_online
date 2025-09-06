use crate::theme::ACCENT;
use eframe::egui::{self, Align, Frame, Layout, RichText, TopBottomPanel};

pub struct TopBar {
    pub title: String,
    pub players: usize,
}

impl Default for TopBar {
    fn default() -> Self {
        Self {
            title: String::new(),
            players: 1,
        }
    }
}

impl TopBar {
    pub fn ui(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.title.is_empty() {
            self.title = "Snake Online".to_string();
        }
        TopBottomPanel::top("topbar")
            .frame(Self::frame())
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("🐍 ").size(22.0));
                    ui.label(RichText::new(&self.title).strong().size(22.0));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.add(egui::Button::new("❤️ 0"))
                            .on_hover_text("Likes / Kudos");
                        ui.add(egui::Button::new(format!(
                            "players {}",
                            self.players.max(1)
                        )))
                        .on_hover_text("Connected players");
                    });
                });
            });
    }

    fn frame() -> Frame {
        Frame {
            inner_margin: egui::Margin::symmetric(10.0, 8.0),
            outer_margin: egui::Margin::symmetric(8.0, 6.0),
            fill: egui::Color32::from_black_alpha(255),
            stroke: egui::Stroke::new(1.0, ACCENT),
            rounding: 6.0.into(),
            ..Frame::default()
        }
    }
}

// (No auto-size helper here; window sizing handled in ui_menu)
