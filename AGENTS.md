# Repository Guidelines

## Project Structure & Module Organization
- Root is a Rust workspace (`Cargo.toml`) with members: `server/` and `client/`.
- `server/`: Axum + Tokio WebSocket backend. Key modules: `main.rs`, `model.rs` (protocol), `room.rs` (game logic).
- `client/`: egui desktop app. Entry in `main.rs`; UI and networking in `ui*.rs`, `net.rs`, `sprites.rs`, etc.
- `inspiration/`: reference-only examples; not built by the workspace.
- Runtime details: server listens on `0.0.0.0:8080`, WebSocket path `/ws`.

## Build, Test, and Development Commands
- Build all: `cargo build --workspace` (add `--release` for optimized binaries).
- Run server: `RUST_LOG=info cargo run -p snake-server`.
- Run client: `cargo run -p snake-client -- --server 127.0.0.1:8080 --name Alice --room lobby`.
- Format: `cargo fmt --all`.
- Lint: `cargo clippy --workspace -D warnings`.
- Test (when added): `cargo test --workspace`.

## Coding Style & Naming Conventions
- Rust 2021, 4-space indentation, `rustfmt` defaults. Keep diffs clean via `cargo fmt`.
- Naming: types/enums `UpperCamelCase`, functions/vars `snake_case`, constants `SCREAMING_SNAKE_CASE`, modules/files `snake_case.rs`.
- Protocol: keep `server/src/model.rs` and `client/src/net.rs` message types in sync. When changing C2S/S2C, update both sides and note in PR.

## Testing Guidelines
- Current tree has no tests. Add unit tests beside modules using `#[cfg(test)]` blocks.
- Suggested server tests: `Room::step` ticks, collision rules, input queueing, respawn logic.
- Suggested client tests: pure helpers (e.g., URL building). Avoid GUI snapshot tests unless needed.
- Aim for fast, deterministic tests; run with `cargo test --workspace`.

## Commit & Pull Request Guidelines
- Use Conventional Commits (e.g., `feat(server): head-to-head collision`, `fix(client): egui panic on resize`).
- Keep commits focused and buildable. Include short rationale when changing protocol or timing.
- PRs must include: clear description, steps to run (`server` + `client`), screenshots for UI changes, and linked issues.

## Security & Configuration Tips
- Server bind/port: defaults to `0.0.0.0:8080`; use a reverse proxy for TLS if deploying.
- Client config: env `SNAKE_URL` (full ws URL) and `SNAKE_RES=1280x720`; CLI flags `--server/-s`, `--name/-n`, `--room/-r`.
- Logging: prefer `tracing` with `RUST_LOG` over `println!`.

