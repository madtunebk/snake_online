use crate::model::{Cell, Dir, PlayerSnapshot, S2C};
use rand::{thread_rng, Rng};
use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant},
};
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct Player {
    pub id: String,
    pub _name: String, // kept (underscored) to avoid dead_code warnings; wire into snapshots later if you want
    pub snake: Vec<Cell>,
    pub dir: Dir,
    pub alive: bool,
    pub score: u32,
    pub lives: u32,
    pub pending: VecDeque<Dir>,
    pub tx: mpsc::UnboundedSender<S2C>, // outbound to this player
}

pub struct Room {
    pub _name: String,
    pub grid_w: i32,
    pub grid_h: i32,
    pub players: HashMap<String, Player>,
    pub food: Cell,
    pub seq: u64,
    last_tick: Instant,
    tick: Duration,
    pub started: bool,
}

impl Room {
    pub fn new(name: impl Into<String>, grid_w: i32, grid_h: i32, hz: u32) -> Self {
        let mut room = Self {
            _name: name.into(),
            grid_w,
            grid_h,
            players: HashMap::new(),
            food: Cell(0, 0),
            seq: 0,
            last_tick: Instant::now(),
            tick: Duration::from_millis((1000 / hz.max(1)) as u64),
            started: false,
        };
        room.food = room.random_empty();
        room
    }

    fn random_empty(&self) -> Cell {
        let mut rng = thread_rng();
        for _ in 0..1000 {
            let c = Cell(rng.gen_range(0..self.grid_w), rng.gen_range(0..self.grid_h));
            if !self.players.values().any(|p| p.snake.contains(&c)) {
                return c;
            }
        }
        Cell(0, 0)
    }

    pub fn add_player(&mut self, id: String, name: String, tx: mpsc::UnboundedSender<S2C>) {
        let mid = Cell(self.grid_w / 2, self.grid_h / 2);
        let snake = vec![mid, Cell(mid.0 - 1, mid.1), Cell(mid.0 - 2, mid.1)];
        let player = Player {
            id: id.clone(),
            _name: name,
            snake,
            dir: Dir::Right,
            alive: true,
            score: 0,
            lives: 3,
            pending: VecDeque::new(),
            tx,
        };
        self.players.insert(id, player);
    }

    pub fn remove_player(&mut self, id: &str) {
        self.players.remove(id);
    }
    pub fn respawn_player(&mut self, id: &str) {
        if let Some(p) = self.players.get_mut(id) {
            if p.lives > 0 {
                let mid = Cell(self.grid_w / 2, self.grid_h / 2);
                p.snake = vec![mid, Cell(mid.0 - 1, mid.1), Cell(mid.0 - 2, mid.1)];
                p.dir = Dir::Right;
                p.alive = true;
                p.pending.clear();
            }
        }
    }
    pub fn queue_input(&mut self, id: &str, d: Dir) {
        if let Some(p) = self.players.get_mut(id) {
            // prevent 180° reversals
            let illegal = matches!(
                (p.dir, d),
                (Dir::Up, Dir::Down)
                    | (Dir::Down, Dir::Up)
                    | (Dir::Left, Dir::Right)
                    | (Dir::Right, Dir::Left)
            );
            if !illegal {
                p.pending.push_back(d);
            }
        }
    }

    pub fn tick_due(&self) -> bool {
        self.started && self.last_tick.elapsed() >= self.tick
    }

    pub fn step(&mut self) {
        self.last_tick = Instant::now();
        self.seq += 1;
        tracing::debug!("tick seq={} players={}", self.seq, self.players.len());

        // apply one queued dir per player (keeps latency small but stable)
        for p in self.players.values_mut() {
            if let Some(d) = p.pending.pop_front() {
                p.dir = d;
            }
        }

        // compute next heads
        let mut next_heads: HashMap<String, Cell> = HashMap::new();
        for p in self.players.values() {
            if !p.alive {
                continue;
            }
            let mut head = *p.snake.first().unwrap();
            match p.dir {
                Dir::Up => head.1 -= 1,
                Dir::Down => head.1 += 1,
                Dir::Left => head.0 -= 1,
                Dir::Right => head.0 += 1,
            }
            next_heads.insert(p.id.clone(), head);
        }

        // pre-collect all occupied cells for body collisions
        let all_body: Vec<Cell> = self
            .players
            .values()
            .flat_map(|p| if p.alive { p.snake.clone() } else { vec![] })
            .collect();

        // mark deaths: wall, body, head-to-head
        let mut deaths: Vec<String> = vec![];

        // wall & body
        for (id, head) in &next_heads {
            if head.0 < 0 || head.0 >= self.grid_w || head.1 < 0 || head.1 >= self.grid_h {
                deaths.push(id.clone());
                continue;
            }
            if all_body.contains(head) {
                deaths.push(id.clone());
                continue;
            }
        }

        // head-to-head (if two players target the same cell, both die)
        for (id_a, head_a) in &next_heads {
            for (id_b, head_b) in &next_heads {
                if id_a < id_b && head_a == head_b {
                    deaths.push(id_a.clone());
                    deaths.push(id_b.clone());
                }
            }
        }

        // apply moves for survivors
        for (id, head) in next_heads {
            if deaths.contains(&id) {
                continue;
            }
            if let Some(p) = self.players.get_mut(&id) {
                p.snake.insert(0, head);
                if head == self.food {
                    p.score += 1;
                    self.food = self.random_empty();
                } else {
                    p.snake.pop();
                }
            }
        }

        // finalize deaths
        for id in deaths {
            if let Some(p) = self.players.get_mut(&id) {
                p.alive = false;
                if p.lives > 0 {
                    p.lives -= 1;
                }
            }
        }

        // auto-respawn if all remaining-with-lives are dead (solo-friendly)
        let any_with_lives = self.players.values().any(|p| p.lives > 0);
        if any_with_lives
            && self
                .players
                .values()
                .filter(|p| p.lives > 0)
                .all(|p| !p.alive)
        {
            for id in self.players.keys().cloned().collect::<Vec<_>>() {
                if let Some(p) = self.players.get(&id) {
                    if p.lives == 0 {
                        continue;
                    }
                }
                self.respawn_player(&id);
            }
        }

        // broadcast snapshot for this tick
        let msg = self.snapshot();
        for p in self.players.values() {
            let _ = p.tx.send(msg.clone());
        }
    }

    // Build a full-state snapshot message
    pub fn snapshot(&self) -> S2C {
        let players = self
            .players
            .values()
            .map(|p| PlayerSnapshot {
                id: p.id.clone(),
                name: p._name.clone(),
                alive: p.alive,
                score: p.score,
                lives: p.lives,
                body: p.snake.clone(),
            })
            .collect::<Vec<_>>();

        S2C::State {
            seq: self.seq,
            started: self.started,
            food: self.food,
            players,
        }
    }
}
