mod model;
mod room;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use model::*;
use room::Room;
use serde::Deserialize;
use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::sync::oneshot;
use tokio::time::interval;
use tracing::*;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    rooms: Arc<DashMap<String, Room>>,
}

#[derive(Deserialize)]
struct WsParams {
    room: Option<String>,
    name: Option<String>,
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let state = AppState {
        rooms: Arc::new(DashMap::new()),
    };
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state.clone());

    tokio::spawn(ticker(state.clone(), 10, Arc::new(AtomicBool::new(true))));

    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

pub async fn run_with_shutdown(
    shutdown: oneshot::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let state = AppState {
        rooms: Arc::new(DashMap::new()),
    };
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state.clone());

    let running = Arc::new(AtomicBool::new(true));
    let running_for_ticker = running.clone();
    tokio::spawn(ticker(state.clone(), 10, running_for_ticker));

    // Flip running to false when shutdown signal arrives
    let running_for_signal = running.clone();
    tokio::spawn(async move {
        let _ = shutdown.await;
        running_for_signal.store(false, Ordering::SeqCst);
    });

    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    let shutdown_future = async move {
        while running.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_future)
        .await?;
    Ok(())
}

async fn ticker(state: AppState, hz: u32, running: Arc<AtomicBool>) {
    let mut tick = interval(Duration::from_millis((1000 / hz.max(1)) as u64));
    while running.load(Ordering::SeqCst) {
        tick.tick().await;
        for mut room in state.rooms.iter_mut() {
            if room.tick_due() {
                room.step();
            }
        }
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(q): Query<WsParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| client_conn(socket, q, state))
}

async fn client_conn(socket: WebSocket, q: WsParams, state: AppState) {
    let room_name = q.room.unwrap_or_else(|| "lobby".into());
    let player_name = q.name.unwrap_or_else(|| "Anon".into());
    let player_id = Uuid::new_v4().to_string();

    // channel from server → this client
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<S2C>();

    // --- limit the mutable guard scope so the ticker can borrow later ---
    {
        // ensure room (mutable!)
        let mut room_entry = state
            .rooms
            .entry(room_name.clone())
            .or_insert_with(|| Room::new(&room_name, 22, 22, 10));

        // register the player
        room_entry.add_player(player_id.clone(), player_name, tx.clone());
        info!("join: room={room_name} id={player_id}");

        // send Hello
        let _ = tx.send(S2C::Hello {
            player_id: player_id.clone(),
            grid: (room_entry.grid_w, room_entry.grid_h),
            tick_hz: 10,
        });

        // send immediate State so the client sees itself right away
        let snap = room_entry.snapshot();
        let players_len = match &snap {
            S2C::State { players, .. } => players.len(),
            _ => 0,
        };
        info!("snapshot_on_join: players={players_len}");
        let _ = tx.send(snap);

        // room_entry guard DROPS here
    }
    // --------------------------------------------------------------------

    // split socket
    let (mut sender, mut receiver) = socket.split();

    // outbound pump: server → client
    let outbound = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender
                .send(Message::Text(serde_json::to_string(&msg).unwrap()))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    // inbound loop: client → server
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(txt) => {
                if let Ok(c2s) = serde_json::from_str::<C2S>(&txt) {
                    match c2s {
                        C2S::Join { .. } => {}
                        C2S::Input { dir } => {
                            // short-lived mutable borrow; fine for the ticker
                            if let Some(mut room) = state.rooms.get_mut(&room_name) {
                                room.queue_input(&player_id, dir);
                            }
                        }
                        C2S::Start => {
                            if let Some(mut room) = state.rooms.get_mut(&room_name) {
                                info!("start: room={room_name}");
                                room.started = true;
                                // optional: snapshot broadcast so clients update started flag immediately
                                let _ = tx.send(room.snapshot());
                            }
                        }
                        C2S::Respawn => {
                            if let Some(mut room) = state.rooms.get_mut(&room_name) {
                                room.respawn_player(&player_id);
                                // send an immediate snapshot so the client sees the new state
                                let _ = tx.send(room.snapshot());
                            }
                        }
                        C2S::Restart => {
                            if let Some(mut room) = state.rooms.get_mut(&room_name) {
                                if let Some(p) = room.players.get_mut(&player_id) {
                                    info!("restart: room={room_name} id={player_id}");
                                    p.lives = 3;
                                }
                                room.respawn_player(&player_id);
                                let _ = tx.send(room.snapshot());
                            }
                        }
                        C2S::Ping { t } => {
                            let _ = tx.send(S2C::Pong { t });
                        }
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    // cleanup
    if let Some(mut room) = state.rooms.get_mut(&room_name) {
        room.remove_player(&player_id);
        info!(
            "leave: room={room_name} id={player_id} now_players={}",
            room.players.len()
        );
    }
    outbound.abort();
}
