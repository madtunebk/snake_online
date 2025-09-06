use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

/* === protocol (must match server) === */

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Cell(pub i32, pub i32);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSnapshot {
    pub id: String,
    pub name: String,
    pub alive: bool,
    pub score: u32,
    pub lives: u32,
    pub body: Vec<Cell>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum C2S {
    #[serde(rename = "input")]
    Input { dir: Dir },
    #[serde(rename = "ping")]
    Ping { t: u64 },
    #[serde(rename = "respawn")]
    Respawn, // keep if you use R to respawn
    #[serde(rename = "start")]
    Start,
    #[serde(rename = "restart")]
    Restart,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum S2C {
    #[serde(rename = "hello")]
    Hello {
        player_id: String,
        grid: (i32, i32),
        tick_hz: u32,
    },
    #[serde(rename = "state")]
    State {
        seq: u64,
        started: bool,
        food: Cell,
        players: Vec<PlayerSnapshot>,
    },
    #[serde(rename = "pong")]
    Pong { t: u64 },
}

/* === network client === */

pub struct NetClient {
    pub me: Option<String>,
    pub rx_state: mpsc::UnboundedReceiver<S2C>,
    tx_cmd: mpsc::UnboundedSender<C2S>,
}

impl NetClient {
    pub async fn connect(url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let (tx_cmd, mut rx_cmd) = mpsc::unbounded_channel::<C2S>();
        let (tx_state, rx_state) = mpsc::unbounded_channel::<S2C>();

        let (ws_stream, _) = connect_async(url).await?;
        let (mut ws_tx, mut ws_rx) = ws_stream.split();

        // writer task
        tokio::spawn(async move {
            while let Some(cmd) = rx_cmd.recv().await {
                let _ = ws_tx
                    .send(Message::Text(serde_json::to_string(&cmd).unwrap()))
                    .await;
            }
        });

        // reader task
        tokio::spawn(async move {
            while let Some(Ok(msg)) = ws_rx.next().await {
                if let Message::Text(txt) = msg {
                    if let Ok(parsed) = serde_json::from_str::<S2C>(&txt) {
                        let _ = tx_state.send(parsed);
                    }
                }
            }
        });

        // background ping
        let ping_tx = tx_cmd.clone();
        tokio::spawn(async move {
            use std::time::{SystemTime, UNIX_EPOCH};
            loop {
                let t = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;
                let _ = ping_tx.send(C2S::Ping { t });
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        });

        Ok(Self {
            me: None,
            rx_state,
            tx_cmd,
        })
    }

    /// Send a direction input to the server
    pub fn send_dir(&self, d: Dir) {
        let _ = self.tx_cmd.send(C2S::Input { dir: d });
    }

    /// (optional) Respawn request
    pub fn send_respawn(&self) {
        let _ = self.tx_cmd.send(C2S::Respawn);
    }
    /// Start the room (lobby -> active)
    pub fn send_start(&self) {
        let _ = self.tx_cmd.send(C2S::Start);
    }
    /// Restart after game over (reset lives to 3)
    pub fn send_restart(&self) {
        let _ = self.tx_cmd.send(C2S::Restart);
    }
}
