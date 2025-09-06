use crate::theme::ACCENT;
use eframe::egui::{
    self, Align, Area, Color32, Frame, Layout, Pos2, RichText, Rounding, ScrollArea, Stroke, Ui,
    Vec2,
};

use crate::ui::RemoteWorld;

// SidePanel version removed to avoid overlapping edge-cases — using overlay aligned to board instead.

/// Overlay scoreboard positioned to the right of the given board rectangle.
pub fn overlay_next_to(
    ctx: &egui::Context,
    bounds: egui::Rect,
    board_rect: egui::Rect,
    world: Option<&RemoteWorld>,
    me_id: Option<&str>,
) {
    let gap = 64.0; // distance between board and scoreboard
    let width = 320.0;
    // Match the board height exactly so they align visually
    let mut height = board_rect.height();
    // Clamp to central bounds in case of rounding
    height = height.min(bounds.height());

    // Anchor to the right of the board, aligned to board top
    let mut pos = Pos2::new(board_rect.right() + gap, board_rect.top());
    let max_x = bounds.right() - width - 20.0;
    pos.x = pos.x.min(max_x);

    Area::new("scoreboard_overlay_bounds".into())
        .fixed_pos(pos)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            ui.allocate_ui_with_layout(
                Vec2::new(width, height),
                egui::Layout::top_down(Align::LEFT),
                |ui| {
                    // Match the game area's black background for a cohesive look
                    Frame {
                        inner_margin: egui::Margin::symmetric(10.0, 10.0),
                        rounding: Rounding::same(6.0),
                        stroke: Stroke::new(1.0, ACCENT),
                        fill: Color32::from_black_alpha(255),
                        ..Default::default()
                    }
                    .show(ui, |ui| {
                        ui.set_min_size(Vec2::new(width, height));
                        ui.set_max_size(Vec2::new(width, height));
                        ui.vertical(|ui| {
                            Frame {
                                inner_margin: egui::Margin::symmetric(10.0, 4.0),
                                rounding: Rounding::same(12.0),
                                stroke: Stroke::new(1.0, ACCENT),
                                fill: ACCENT.linear_multiply(0.20),
                                ..Default::default()
                            }
                            .show(ui, |ui| {
                                ui.label(RichText::new("Scores").strong());
                            });
                            ui.add_space(10.0);

                            ScrollArea::vertical()
                                .id_source("scores_scroll_overlay")
                                .show(ui, |ui| {
                                    if let Some(world) = world {
                                        // rank by score desc
                                        let mut entries: Vec<_> = world
                                            .players
                                            .iter()
                                            .map(|p| {
                                                (
                                                    p.id.clone(),
                                                    p.name.clone(),
                                                    p.score,
                                                    p.alive,
                                                    p.lives,
                                                )
                                            })
                                            .collect();
                                        entries.sort_by_key(|(_, _, score, _, _)| {
                                            std::cmp::Reverse(*score)
                                        });

                                        for (rank, (id, name, score, alive, lives)) in
                                            entries.into_iter().enumerate()
                                        {
                                            player_row(
                                                ui,
                                                rank + 1,
                                                &id,
                                                &name,
                                                score,
                                                alive,
                                                lives,
                                                me_id,
                                            );
                                        }
                                    } else {
                                        ui.label("Waiting for state…");
                                    }
                                });
                        });
                    });
                },
            );
        });
}

fn player_row(
    ui: &mut Ui,
    rank: usize,
    id: &str,
    name: &str,
    score: u32,
    alive: bool,
    lives: u32,
    me_id: Option<&str>,
) {
    let row_fill = Color32::TRANSPARENT;
    Frame {
        inner_margin: egui::Margin::symmetric(10.0, 6.0),
        outer_margin: egui::Margin::symmetric(0.0, 6.0),
        rounding: Rounding::same(8.0),
        stroke: Stroke::new(1.0, ACCENT),
        fill: row_fill,
        ..Default::default()
    }
    .show(ui, |ui| {
        ui.horizontal(|ui| {
            pill(ui, format!("#{rank}"));
            let is_me = me_id.map(|m| m == id).unwrap_or(false);
            let label = if !name.is_empty() {
                name.to_string()
            } else {
                id.chars().take(6).collect()
            };
            if is_me {
                ui.label(RichText::new(label).strong());
            } else {
                ui.label(label);
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                pill(ui, format!("⭐ {}", score));
                ui.label(
                    RichText::new(format!("{}{}", if alive { "❤" } else { "♡" }, lives))
                        .monospace(),
                );
            });
        });
    });
}

fn pill(ui: &mut Ui, text: String) {
    let f = Frame {
        inner_margin: egui::Margin::symmetric(8.0, 4.0),
        rounding: Rounding::same(10.0),
        fill: Color32::TRANSPARENT,
        stroke: Stroke::new(1.0, ACCENT),
        ..Default::default()
    };
    f.show(ui, |ui| {
        ui.monospace(text);
    });
}
