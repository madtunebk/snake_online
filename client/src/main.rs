mod bottombar;
mod buffer;
mod milestones;
mod net;
mod sprites;
mod theme;
mod topbar;
mod ui;
mod ui_menu;
mod ui_neon;
mod ui_overlays;
mod ui_scoreboard;
use eframe::egui;

fn parse_args() -> (Option<String>, Option<String>, Option<String>) {
    // Defaults
    let mut server: Option<String> = None; // e.g. "127.0.0.1:8080" or "snakeserver:8080"
    let mut name: Option<String> = None; // e.g. "Groot"
    let mut room: Option<String> = None; // e.g. "lobby"

    // Very small flag parser to support: --server/-s, --name/-n, --room/-r
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--server" | "-s" => {
                server = it.next();
            }
            "--name" | "-n" => {
                name = it.next();
            }
            "--room" | "-r" => {
                room = it.next();
            }
            _ => {
                // ignore unknown args; eframe/winit may add theirs later
            }
        }
    }

    (server, name, room)
}

fn build_ws_url(server: &str, name: &str, room: &str) -> String {
    // Accept server with or without scheme/port. Ensure scheme and path, attach query.
    let base = if server.contains("://") {
        server.to_string()
    } else {
        format!("ws://{}", server)
    };

    let mut url =
        url::Url::parse(&base).unwrap_or_else(|_| url::Url::parse("ws://127.0.0.1:8080").unwrap());
    if url.port().is_none() {
        let _ = url.set_port(Some(8080));
    }
    url.set_path("/ws");
    url.query_pairs_mut()
        .clear()
        .append_pair("room", room)
        .append_pair("name", name);
    url.to_string()
}

fn pick_best_preset() -> (u32, u32) {
    // Simple heuristic: prefer 1280x720; allow override via SNAKE_RES="WxH"
    if let Ok(s) = std::env::var("SNAKE_RES") {
        if let Some((w, h)) = s
            .split_once('x')
            .and_then(|(w, h)| Some((w.parse::<u32>().ok()?, h.parse::<u32>().ok()?)))
        {
            return (w, h);
        }
    }
    (1280, 720)
}

fn main() -> eframe::Result<()> {
    // Parse CLI flags to prefill the menu; if env SNAKE_URL is set, use it to seed fields too.
    let (server_flag, name_flag, room_flag) = parse_args();
    let any_flags = server_flag.is_some() || name_flag.is_some() || room_flag.is_some();
    let (mut server, mut name, mut room) = (
        server_flag.unwrap_or_else(|| "127.0.0.1:8080".to_string()),
        name_flag.unwrap_or_else(|| "Groot".to_string()),
        room_flag.unwrap_or_else(|| "lobby".to_string()),
    );
    if let Ok(env_url) = std::env::var("SNAKE_URL") {
        if !any_flags {
            if let Ok(u) = url::Url::parse(&env_url) {
                if let Some(host) = u.host_str() {
                    server = format!("{}:{}", host, u.port().unwrap_or(8080));
                }
                for (k, v) in u.query_pairs() {
                    match k.as_ref() {
                        "name" => name = v.to_string(),
                        "room" => room = v.to_string(),
                        _ => {}
                    }
                }
            }
        }
    }

    // Lock the app to a reasonable windowed preset (non-resizable)
    let (bw, bh) = pick_best_preset();
    let viewport = egui::ViewportBuilder::default()
        .with_inner_size(egui::vec2(bw as f32, bh as f32))
        .with_resizable(false);
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "🐍 Snake Online (egui)",
        options,
        Box::new(move |cc| {
            // Theme and pixel scale
            theme::apply(cc);
            cc.egui_ctx.set_pixels_per_point(1.0);
            Ok(Box::new(ui_menu::RootApp::new(
                server,
                name,
                room,
                build_ws_url,
            )))
        }),
    )
}
