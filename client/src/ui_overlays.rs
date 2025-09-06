use crate::ui_neon::{self, NeonTheme};
use eframe::egui::{self, Color32};

use crate::{net, ui::RemoteWorld};

pub struct OverlayAction {
    pub try_again: bool,
}

pub fn show(
    ctx: &egui::Context,
    rect: egui::Rect,
    world: &RemoteWorld,
    net: Option<&net::NetClient>,
    gave_up: &mut bool,
    suppress_gameover: bool,
) -> OverlayAction {
    let mut action = OverlayAction { try_again: false };
    // Pre-start overlay removed; client auto-sends Start on connect.
    if !world.started {
        return action;
    }
    if suppress_gameover {
        return action;
    }

    // Game over when no lives left and dead
    if let Some(n) = net {
        // Find me
        if let Some(me_id) = &n.me {
            if let Some(me) = world.players.iter().find(|p| &p.id == me_id) {
                if me.lives == 0 && !me.alive {
                    egui::Area::new("gameover_overlay".into())
                        .order(egui::Order::Foreground)
                        .fixed_pos(rect.center() - egui::vec2(160.0, 60.0))
                        .show(ctx, |ui| {
                            ui_neon::neon_popup_frame(ui, NeonTheme::default()).show(ui, |ui| {
                                ui.vertical_centered(|ui| {
                                    ui.heading("Game Over");
                                    ui.label("No lives left");
                                    ui.add_space(8.0);
                                    ui.horizontal(|ui| {
                                        if ui
                                            .add_sized(
                                                [120.0, 28.0],
                                                egui::Button::new("Try Again"),
                                            )
                                            .clicked()
                                        {
                                            action.try_again = true;
                                        }
                                        if ui
                                            .add_sized(
                                                [160.0, 28.0],
                                                egui::Button::new("Give up and cry"),
                                            )
                                            .clicked()
                                        {
                                            *gave_up = true;
                                        }
                                    });
                                    if *gave_up {
                                        ui.add_space(6.0);
                                        ui.colored_label(
                                            Color32::LIGHT_RED,
                                            "😭 See you next round!",
                                        );
                                    }
                                });
                            });
                        });
                }
            }
        }
    }
    action
}
