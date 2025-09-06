use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
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
    #[serde(rename = "join")]
    Join { room: String, name: String },
    #[serde(rename = "input")]
    Input { dir: Dir },
    #[serde(rename = "ping")]
    Ping { t: u64 },
    #[serde(rename = "respawn")]
    Respawn,
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
