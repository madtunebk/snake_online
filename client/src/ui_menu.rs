use crate::ui_neon::{self, NeonTheme};
use eframe::egui::{self, Rounding};

use crate::ui::SnakeApp;

pub struct RootApp<F>
where
    F: Fn(&str, &str, &str) -> String + Send + Sync + 'static,
{
    state: AppState,
    server: String,
    name: String,
    room: String,
    build_url: F,
    did_auto_resize: bool,
    hosted_server: bool,
    host_shutdown: Option<tokio::sync::oneshot::Sender<()>>,
}

enum AppState {
    Menu(MenuState),
    Game(SnakeApp),
}

struct MenuState {
    server: String,
    name: String,
    room: String,
}

impl<F> RootApp<F>
where
    F: Fn(&str, &str, &str) -> String + Send + Sync + 'static,
{
    pub fn new(server: String, name: String, room: String, build_url: F) -> Self {
        Self {
            state: AppState::Menu(MenuState {
                server: server.clone(),
                name: name.clone(),
                room: room.clone(),
            }),
            server,
            name,
            room,
            build_url,
            did_auto_resize: false,
            hosted_server: false,
            host_shutdown: None,
        }
    }
}

impl<F> eframe::App for RootApp<F>
where
    F: Fn(&str, &str, &str) -> String + Send + Sync + 'static,
{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Auto-pick a window size on first frame that fits the monitor nicely
        if !self.did_auto_resize {
            if let Some(monitor) = ctx.input(|i| i.viewport().monitor_size) {
                let new_size = auto_size_for_monitor(monitor);
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(new_size));
                ctx.send_viewport_cmd(egui::ViewportCommand::Resizable(false));
                if let Some(center_cmd) = egui::ViewportCommand::center_on_screen(ctx) {
                    ctx.send_viewport_cmd(center_cmd);
                }
                self.did_auto_resize = true;
            }
        }
        match &mut self.state {
            AppState::Menu(menu) => {
                let theme = NeonTheme::default();
                let mut start = false;
                enum HostAction {
                    Start,
                    Stop,
                }
                let mut host_action: Option<HostAction> = None;
                egui::CentralPanel::default().show(ctx, |ui| {
                    // Subtle background tint
                    let bg = ui.max_rect();
                    ui.painter().rect_filled(
                        bg,
                        0.0,
                        egui::Color32::from_rgba_unmultiplied(10, 14, 16, 255),
                    );

                    // Centered neon card
                    egui::Area::new("menu_card".into())
                        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, -8.0))
                        .show(ui.ctx(), |ui| {
                            ui_neon::neon_popup_frame(ui, theme).show(ui, |ui| {
                                ui.set_min_width(420.0);
                                ui.vertical_centered(|ui| {
                                    ui.add_space(6.0);
                                    ui.label(
                                        egui::RichText::new("SNAKE ONLINE")
                                            .size(34.0)
                                            .color(theme.panel_stroke)
                                            .strong(),
                                    );
                                    ui.add_space(2.0);
                                    ui.label(
                                        egui::RichText::new("Fast. Simple. Snacky.")
                                            .italics()
                                            .color(egui::Color32::from_gray(200)),
                                    );
                                    ui.add_space(12.0);

                                    egui::Grid::new("quick_join_grid")
                                        .num_columns(2)
                                        .spacing([10.0, 8.0])
                                        .min_col_width(80.0)
                                        .show(ui, |ui| {
                                            ui.label("Name:");
                                            ui.text_edit_singleline(&mut menu.name);
                                            ui.end_row();
                                            ui.label("Server:");
                                            ui.text_edit_singleline(&mut menu.server);
                                            ui.end_row();
                                            ui.label("Room:");
                                            ui.text_edit_singleline(&mut menu.room);
                                            ui.end_row();
                                        });

                                    ui.add_space(14.0);
                                    let primary = |label: &str| {
                                        egui::Button::new(
                                            egui::RichText::new(label).size(18.0).strong(),
                                        )
                                        .rounding(Rounding::same(12.0))
                                        .stroke(egui::Stroke::new(1.8, theme.panel_stroke))
                                        .fill(egui::Color32::from_rgb(40, 50, 58))
                                    };

                                    if ui.add_sized([320.0, 44.0], primary("Play")).clicked() {
                                        start = true;
                                    }
                                    ui.add_space(8.0);
                                    if ui
                                        .add_sized(
                                            [320.0, 40.0],
                                            primary(if self.hosted_server {
                                                "Stop Local Server"
                                            } else {
                                                "Host Local Server"
                                            }),
                                        )
                                        .clicked()
                                    {
                                        host_action = Some(if self.hosted_server {
                                            HostAction::Stop
                                        } else {
                                            HostAction::Start
                                        });
                                    }
                                    ui.add_space(4.0);
                                    if ui
                                        .add_sized([320.0, 36.0], egui::Button::new("Exit"))
                                        .clicked()
                                    {
                                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                    }
                                    ui.add_space(2.0);
                                });
                            });
                        });

                    // Bottom-right corner credit
                    egui::Area::new("menu_credit".into())
                        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-12.0, -10.0))
                        .order(egui::Order::Foreground)
                        .show(ui.ctx(), |ui| {
                            ui.label(
                                egui::RichText::new("@Coaba")
                                    .size(12.0)
                                    .color(egui::Color32::from_gray(190))
                                    .italics(),
                            );
                        });
                });
                // Process host start/stop after UI borrows end
                if let Some(action) = host_action {
                    match action {
                        HostAction::Stop => {
                            if let Some(tx) = self.host_shutdown.take() {
                                let _ = tx.send(());
                            }
                            self.hosted_server = false;
                        }
                        HostAction::Start => {
                            self.server = "127.0.0.1:8080".to_string();
                            let (tx, rx) = tokio::sync::oneshot::channel::<()>();
                            self.host_shutdown = Some(tx);
                            std::thread::spawn(move || {
                                let rt = tokio::runtime::Builder::new_multi_thread()
                                    .enable_all()
                                    .build()
                                    .unwrap();
                                rt.block_on(async move {
                                    let _ = snake_server::run_with_shutdown(rx).await;
                                });
                            });
                            self.hosted_server = true;
                            // reflect local server in quick field
                            menu.server = "127.0.0.1:8080".to_string();
                        }
                    }
                }
                if start {
                    self.server = menu.server.clone();
                    self.name = menu.name.clone();
                    self.room = menu.room.clone();
                    let url = (self.build_url)(&self.server, &self.name, &self.room);
                    self.state = AppState::Game(SnakeApp::new(url));
                }
            }
            AppState::Game(game) => {
                game.update(ctx, _frame);
                if game.gave_up() {
                    self.state = AppState::Menu(MenuState {
                        server: self.server.clone(),
                        name: self.name.clone(),
                        room: self.room.clone(),
                    });
                }
            }
        }
    }
}

fn auto_size_for_monitor(monitor_size: egui::Vec2) -> egui::Vec2 {
    let scale = 0.77;
    let candidates = [
        egui::vec2(1280.0, 720.0) * scale,
        egui::vec2(1920.0, 1080.0) * scale,
        egui::vec2(2560.0, 1440.0) * scale,
    ];
    let margin = 32.0;
    let max_w = (monitor_size.x - margin).max(0.0);
    let max_h = (monitor_size.y - margin).max(0.0);
    for &size in candidates.iter().rev() {
        if size.x <= max_w && size.y <= max_h {
            return size;
        }
    }
    monitor_size * 0.90
}
