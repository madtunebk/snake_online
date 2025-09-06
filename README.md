# 🐍 Snake Online

Fast, lightweight, and **online** multiplayer Snake — built in Rust.
Create a room, invite friends, and see who can keep their snake alive the longest.

> Status: early project scaffolding with `client/` and `server/` crates and shared assets. Perfect base to iterate on.

---

## ✨ Features

* **Classic Snake** gameplay with smooth movement and food spawns
* **Multiplayer-ready** design (separate `server` and `client`)
* **Modular workspace** layout for clean separation of concerns
* **Cross-platform** (Rust) and easy to build with `cargo`

---

## 🗂️ Project Structure

```
snake_online/
├─ assets/
│  └─ fonts/              # In-game fonts and shared assets
├─ client/                # Game client crate (UI, input, rendering, net)
├─ server/                # Game server crate (rooms, state, matchmaking)
├─ Cargo.toml             # Workspace manifest
├─ Cargo.lock
└─ README.md
```

---

## 🚀 Quick Start

### Prerequisites

* [Rust](https://www.rust-lang.org/tools/install) (stable).
  Recommended: `rustup update` to the latest stable.

### 1) Clone

```bash
git clone https://github.com/madtunebk/snake_online
cd snake_online
```

### 2) Build

```bash
cargo build --release
```

### 3) Run the Server

Depending on how the binaries are named in `server/Cargo.toml`, one of these will work:

```bash
# if the server is a workspace package
cargo run -p server

# or, if there's a bin target named 'server'
cargo run --bin server
```

**Environment variables** (optional):

```bash
# defaults are just examples—adjust to your code
export SNAKE_BIND_ADDR=0.0.0.0
export SNAKE_PORT=4000
```

### 4) Run the Client

In a second terminal:

```bash
# if the client is a workspace package
cargo run -p client

# or, if there's a bin target named 'client'
cargo run --bin client
```

If the client needs to know where the server is:

```bash
export SNAKE_SERVER_URL=ws://127.0.0.1:4000
```

---

## 🎮 Controls (defaults)

* **Move:** Arrow Keys / WASD
* **Pause / Menu:** `P` / `Esc`
* **Quit:** `Esc` / `Ctrl+C` in the terminal

> If your client uses different bindings, adjust this section to match.

---

## ⚙️ Configuration

Create a `.env` file at the repo root (if you prefer env files):

```env
# Server
SNAKE_BIND_ADDR=0.0.0.0
SNAKE_PORT=4000

# Client
SNAKE_SERVER_URL=ws://localhost:4000
```

You can load these via your preferred env loader (e.g., `dotenvy`) if the project uses one.

---

## 🧪 Development

* **Run checks**

  ```bash
  cargo check
  cargo clippy --all-targets -- -D warnings
  cargo fmt --all
  ```
* **Run with logs**

  ```bash
  RUST_LOG=info cargo run -p server
  RUST_LOG=debug cargo run -p client
  ```

---

## 🐳 (Optional) Docker

If you want to containerize the server, drop this `Dockerfile` into `server/`:

```dockerfile
# server/Dockerfile
FROM rust:1 as builder
WORKDIR /app
COPY ../.. /app
# build only the server binary to speed up subsequent builds
RUN cargo build --release -p server

FROM debian:stable-slim
WORKDIR /app
COPY --from=builder /app/target/release/server /usr/local/bin/snake-server
ENV SNAKE_BIND_ADDR=0.0.0.0
ENV SNAKE_PORT=4000
EXPOSE 4000
CMD ["snake-server"]
```

Build & run:

```bash
docker build -t snake-server ./server
docker run --rm -p 4000:4000 snake-server
```

---

## 🛣️ Roadmap

* ✅ Workspace layout (`client/`, `server/`, `assets/`)
* ⏳ Lobby/rooms and player matchmaking
* ⏳ Power-ups and obstacles
* ⏳ Spectator mode
* ⏳ Persistent leaderboards
* ⏳ Replays

> Open an issue or PR to suggest features!

---

## 📦 Releases

TBD — once the gameplay loop is finalized, tagged releases will be published.

---

## 🤝 Contributing

1. Fork the repo
2. Create a feature branch: `git checkout -b feat/amazing-thing`
3. Commit: `git commit -m "feat: add amazing thing"`
4. Push: `git push origin feat/amazing-thing`
5. Open a Pull Request

Please run `cargo fmt` and `cargo clippy` before submitting.

---

## 📝 License

This project currently doesn’t declare a license.
Consider adding one (MIT/Apache-2.0 are common in Rust). If/when a `LICENSE` file is added, reference it here.

---

## 🙏 Acknowledgements

* Rust community & ecosystem
* Everyone who grew up dodging their own tail 🐍
