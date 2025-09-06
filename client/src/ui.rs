use crate::buffer::Grid;
use crate::net::{self, Cell, Dir, PlayerSnapshot, S2C};
use crate::sprites::SpriteAtlas;
use eframe::egui::{self, vec2, Color32, Pos2, Rect, Stroke};
// Keep ui_neon available for overlays module; not used directly here
// use crate::ui_neon::{self, NeonTheme};
use crate::theme::ACCENT;
use crate::{bottombar, topbar};
use std::time::{Duration, Instant};

// Import UI parts modules declared at crate root
use crate::milestones;
use crate::ui_overlays::{self};
use crate::ui_scoreboard;

// Font sizes for overlays
const COUNTDOWN_FONT_SIZE: f32 = 120.0; // change to adjust countdown text
const MILESTONE_FONT_SIZE: f32 = 120.0; // change to adjust milestone toast

pub struct RemoteWorld {
    pub grid: (i32, i32),
    pub food: Cell,
    pub players: Vec<PlayerSnapshot>,
    pub started: bool,
}

pub struct SnakeApp {
    topbar: topbar::TopBar,
    bottombar: bottombar::BottomBar,
    _rt: tokio::runtime::Runtime,
    net: Option<net::NetClient>,
    world: Option<RemoteWorld>,
    sprites: Option<SpriteAtlas>,
    last_latency_ms: Option<u64>,
    gave_up: bool,
    _window_sized: bool,
    countdown_end: Option<Instant>,
    start_sent: bool,
    fancy_font_loaded: bool,
    fancy_font_ready: bool,
    fonts_ready_at: Option<u64>,
    pending_restart: bool,
    last_milestone: Option<usize>,
    milestone_text: Option<String>,
    milestone_until: Option<Instant>,
    suppress_gameover_until: Option<Instant>,
    last_score_seen: Option<u32>,
}

impl SnakeApp {
    pub fn new(url: String) -> Self {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        // run connect on THIS runtime so spawned tasks live on it
        let net = rt.block_on(async { net::NetClient::connect(&url).await.ok() });

        Self {
            topbar: Default::default(),
            bottombar: Default::default(),
            _rt: rt,
            net,
            world: None,
            sprites: None,
            last_latency_ms: None,
            gave_up: false,
            _window_sized: false,
            countdown_end: None,
            start_sent: false,
            fancy_font_loaded: false,
            fancy_font_ready: false,
            fonts_ready_at: None,
            pending_restart: false,
            last_milestone: None,
            milestone_text: None,
            milestone_until: None,
            suppress_gameover_until: None,
            last_score_seen: None,
        }
    }

    pub fn gave_up(&self) -> bool {
        self.gave_up
    }
}

impl eframe::App for SnakeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Neon FX background removed

        // lazy-create sprites
        let sprites = self
            .sprites
            .get_or_insert_with(|| SpriteAtlas::new(ctx, 48));
        // drain network
        if let Some(net) = &mut self.net {
            while let Ok(msg) = net.rx_state.try_recv() {
                match msg {
                    S2C::Hello {
                        player_id, grid, ..
                    } => {
                        self.world = Some(RemoteWorld {
                            grid,
                            food: Cell(0, 0),
                            players: vec![],
                            started: false,
                        });
                        net.me = Some(player_id);
                        // Start a 3-second countdown before spawning into the world
                        self.countdown_end = Some(Instant::now() + Duration::from_secs(3));
                        self.start_sent = false;
                        // Reset milestone and score tracking at the beginning of a run
                        self.last_milestone = None;
                        self.last_score_seen = Some(0);
                    }
                    S2C::State {
                        seq: _,
                        started,
                        food,
                        players,
                    } => {
                        let grid = self.world.as_ref().map(|w| w.grid).unwrap_or((22, 22));
                        self.world = Some(RemoteWorld {
                            grid,
                            food,
                            players,
                            started,
                        });
                        // Check milestone when score increases; show only the highest crossed
                        let me_id = net.me.clone();
                        if let (Some(w), Some(me_id)) = (&self.world, me_id.as_ref()) {
                            if let Some(me) = w.players.iter().find(|p| &p.id == me_id) {
                                let prev_score = self.last_score_seen.unwrap_or(me.score);
                                if me.score > prev_score {
                                    if let Some((idx, label)) =
                                        milestones::milestone_for_score(me.score)
                                    {
                                        if self
                                            .last_milestone
                                            .map_or(true, |prev_idx| idx > prev_idx)
                                        {
                                            self.last_milestone = Some(idx);
                                            self.milestone_text = Some(label.to_string());
                                            self.milestone_until =
                                                Some(Instant::now() + Duration::from_millis(2000));
                                        }
                                    }
                                }
                                self.last_score_seen = Some(me.score);
                            }
                        }
                    }
                    S2C::Pong { t } => {
                        use std::time::{SystemTime, UNIX_EPOCH};
                        let now_ms = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64;
                        if now_ms >= t {
                            self.last_latency_ms = Some(now_ms - t);
                        }
                    }
                }
            }
        }

        // input → send dir (only when started)
        let input = ctx.input(|i| i.clone());
        let started = self.world.as_ref().map(|w| w.started).unwrap_or(false);
        if started {
            if input.key_pressed(egui::Key::ArrowUp) || input.key_pressed(egui::Key::W) {
                if let Some(n) = &self.net {
                    n.send_dir(Dir::Up);
                }
            }
            if input.key_pressed(egui::Key::ArrowDown) || input.key_pressed(egui::Key::S) {
                if let Some(n) = &self.net {
                    n.send_dir(Dir::Down);
                }
            }
            if input.key_pressed(egui::Key::ArrowLeft) || input.key_pressed(egui::Key::A) {
                if let Some(n) = &self.net {
                    n.send_dir(Dir::Left);
                }
            }
            if input.key_pressed(egui::Key::ArrowRight) || input.key_pressed(egui::Key::D) {
                if let Some(n) = &self.net {
                    n.send_dir(Dir::Right);
                }
            }
        }
        if input.key_pressed(egui::Key::R) {
            if let Some(n) = &self.net {
                n.send_respawn();
            }
        }

        // Trigger Start/Restart once countdown ends (independent of world.started)
        if let Some(end) = self.countdown_end {
            if Instant::now() >= end && !self.start_sent {
                if let Some(n) = &self.net {
                    if self.pending_restart {
                        n.send_restart();
                    } else {
                        n.send_start();
                    }
                }
                self.start_sent = true;
                self.pending_restart = false;
                // countdown finished; it will be hidden below on next frames
                self.countdown_end = None;
                // Reset per-run milestone tracking
                self.last_milestone = None;
            }
        }

        ctx.request_repaint_after(Duration::from_millis(16));
        // Top and bottom bars copied from inspiration style
        if let Some(w) = &self.world {
            self.topbar.players = w.players.len().max(1);
        }
        self.topbar.ui(ctx, _frame);
        self.bottombar.ui(ctx);
        // No right side panel: scoreboard will be drawn as an overlay next to the board.

        // Central game area drawn last so it's not overlapped by panels
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.world.is_none() {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    if self.net.is_some() {
                        ui.heading("Connecting…");
                    } else {
                        ui.heading("Could not connect to server");
                        ui.add_space(8.0);
                        if ui
                            .add_sized([200.0, 32.0], egui::Button::new("Back to Menu"))
                            .clicked()
                        {
                            self.gave_up = true;
                        }
                    }
                });
                return;
            }
            let Some(world) = &self.world else {
                return;
            };
            // Central area excludes top/bottom bars. Reserve space for overlay scoreboard
            const SB_WIDTH: f32 = 320.0;
            const GAP: f32 = 64.0;
            let central = ui.max_rect();
            let right_reserved = GAP + SB_WIDTH;
            let right_edge = (central.right() - right_reserved).max(central.left() + 1.0);
            let left_area =
                Rect::from_min_max(central.left_top(), egui::pos2(right_edge, central.bottom()));
            let board_rect = fit_square_in_rect(left_area);
            let painter = ui.painter_at(board_rect);

            // Board frame and grid
            painter.rect_filled(board_rect, 4.0, Color32::from_black_alpha(255));
            painter.rect_stroke(board_rect, 4.0, Stroke::new(2.0, ACCENT));

            let grid = Grid::new(world.grid.0, world.grid.1, board_rect);
            let (cw, ch) = grid.cell_size();
            let grid_stroke = Stroke::new(1.0, Color32::from_gray(60));
            for x in 0..=world.grid.0 {
                let xpx = board_rect.left() + x as f32 * cw;
                painter.line_segment(
                    [
                        Pos2::new(xpx, board_rect.top()),
                        Pos2::new(xpx, board_rect.bottom()),
                    ],
                    grid_stroke,
                );
            }
            for y in 0..=world.grid.1 {
                let ypx = board_rect.top() + y as f32 * ch;
                painter.line_segment(
                    [
                        Pos2::new(board_rect.left(), ypx),
                        Pos2::new(board_rect.right(), ypx),
                    ],
                    grid_stroke,
                );
            }

            // food (apple)
            let food_rect = grid.cell_rect(world.food.0, world.food.1, 1.0);
            painter.image(
                sprites.apple.id(),
                food_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );

            // players (alive only). Hide my own snake until countdown ends.
            let countdown_active = self.countdown_end.map_or(false, |t| Instant::now() < t);
            let my_id = self.net.as_ref().and_then(|n| n.me.as_ref());
            for (pi, p) in world.players.iter().enumerate().filter(|(_, p)| p.alive) {
                if countdown_active {
                    if let Some(mid) = my_id {
                        if &p.id == mid {
                            continue;
                        }
                    }
                }
                for (i, c) in p.body.iter().enumerate() {
                    let head_palette = [
                        Color32::LIGHT_GREEN,
                        Color32::LIGHT_BLUE,
                        Color32::LIGHT_YELLOW,
                        Color32::from_rgb(255, 0, 255),
                        Color32::LIGHT_RED,
                    ];
                    let tint = if i == 0 {
                        head_palette[pi % head_palette.len()]
                    } else {
                        Color32::WHITE
                    };
                    let r = grid.cell_rect(c.0, c.1, 1.0);
                    painter.image(
                        sprites.body.id(),
                        r,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        tint,
                    );
                }
            }

            // Overlays (game over etc.) and scoreboard next to board with fixed gap
            let suppress = self
                .suppress_gameover_until
                .map_or(false, |t| Instant::now() < t);
            let overlay = ui_overlays::show(
                ctx,
                board_rect,
                world,
                self.net.as_ref(),
                &mut self.gave_up,
                suppress,
            );
            if overlay.try_again {
                // Defer the restart until after the countdown ends
                self.pending_restart = true;
                self.countdown_end = Some(Instant::now() + Duration::from_secs(3));
                self.start_sent = false;
                self.suppress_gameover_until = self.countdown_end;
            }
            ui_scoreboard::overlay_next_to(
                ctx,
                central,
                board_rect,
                self.world.as_ref(),
                self.net.as_ref().and_then(|n| n.me.as_deref()),
            );
        });

        // Mark fancy font as ready one frame after installation
        if let Some(target) = self.fonts_ready_at {
            if ctx.frame_nr() >= target {
                self.fancy_font_ready = true;
                self.fonts_ready_at = None;
            }
        }

        // Pre-start countdown overlay on top of everything (even if world.started already true)
        if let Some(end) = self.countdown_end {
            if Instant::now() < end {
                self.ensure_fancy_font(ctx);
                let now = Instant::now();
                let rem = if end > now {
                    end - now
                } else {
                    Duration::from_secs(0)
                };
                let secs = (rem.as_secs_f32().ceil() as i32).max(0);
                let text = if secs > 0 {
                    format!("{}", secs)
                } else {
                    "Go!".to_string()
                };
                let center = ctx.screen_rect().center();
                egui::Area::new("countdown_overlay".into())
                    .order(egui::Order::Foreground)
                    .fixed_pos(center - egui::vec2(18.0, 42.0))
                    .show(ctx, |ui| {
                        use eframe::egui::text::LayoutJob;
                        use eframe::egui::{FontFamily, FontId};
                        let family = if self.fancy_font_ready {
                            FontFamily::Name("CoabaFancy".into())
                        } else {
                            FontFamily::Proportional
                        };
                        let font = FontId::new(COUNTDOWN_FONT_SIZE, family);
                        let color = crate::theme::ACCENT;
                        let mut job = LayoutJob::simple_singleline(text.into(), font, color);
                        job.wrap.max_width = f32::INFINITY;
                        ui.label(job);
                    });
            }
        }

        // Milestone toast overlay (brief, top-center)
        if let Some(until) = self.milestone_until {
            if Instant::now() < until {
                let text_opt = self.milestone_text.clone();
                if let Some(text) = text_opt {
                    self.ensure_fancy_font(ctx);
                    let top_center =
                        egui::pos2(ctx.screen_rect().center().x, ctx.screen_rect().top() + 60.0);
                    egui::Area::new("milestone_toast".into())
                        .order(egui::Order::Foreground)
                        .pivot(egui::Align2::CENTER_CENTER)
                        .fixed_pos(top_center)
                        .show(ctx, |ui| {
                            use eframe::egui::text::LayoutJob;
                            use eframe::egui::{FontFamily, FontId};
                            let family = if self.fancy_font_ready {
                                FontFamily::Name("CoabaFancy".into())
                            } else {
                                FontFamily::Proportional
                            };
                            let font = FontId::new(MILESTONE_FONT_SIZE, family);
                            let color = crate::theme::ACCENT;
                            let mut job = LayoutJob::simple_singleline(text.into(), font, color);
                            job.wrap.max_width = f32::INFINITY;
                            ui.label(job);
                        });
                }
            } else {
                self.milestone_until = None;
            }
        }

        // (bottom bar already added above)
    }
}

fn fit_square_in_rect(outer: Rect) -> Rect {
    let padding = 8.0;
    let inner = outer.shrink2(egui::vec2(padding, padding));
    let size = inner.size();
    let d = size.x.min(size.y);
    Rect::from_center_size(inner.center(), vec2(d, d))
}

impl SnakeApp {
    fn ensure_fancy_font(&mut self, ctx: &egui::Context) {
        if self.fancy_font_loaded {
            return;
        }
        // Hardcoded search paths (no env var)
        let path = {
            let candidates = ["assets/fonts/Coaba.ttf", "client/assets/fonts/Coaba.ttf"];
            candidates
                .iter()
                .map(std::path::Path::new)
                .find(|p| p.exists())
                .map(|p| p.to_string_lossy().to_string())
        };
        let mut installed = false;
        if let Some(p) = path {
            if let Ok(bytes) = std::fs::read(&p) {
                let mut defs = egui::FontDefinitions::default();
                defs.font_data
                    .insert("coaba_fancy".into(), egui::FontData::from_owned(bytes));
                use eframe::egui::FontFamily;
                let mut chain: Vec<String> = vec!["coaba_fancy".into()];
                if let Some(prop) = defs.families.get(&FontFamily::Proportional).cloned() {
                    chain.extend(prop);
                }
                defs.families
                    .insert(FontFamily::Name("CoabaFancy".into()), chain);
                ctx.set_fonts(defs);
                installed = true;
                // fonts apply on next frame
                self.fonts_ready_at = Some(ctx.frame_nr() + 1);
            }
        }
        // Wait until next frame to mark ready to avoid binding errors
        if installed {
            self.fancy_font_ready = false;
        }
        self.fancy_font_loaded = true;
    }
}
