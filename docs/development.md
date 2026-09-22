# Development

- Toolchain: stable Rust (1.98+), Tauri v2, Node 20+ for frontend.
- Layout: `crates/*` (Rust), `src-tauri/` (Tauri shell), `src/` (frontend).
- `launcher-core` has no Tauri/UI deps — `cargo test -p launcher-core` must
  pass offline except tests marked `#[ignore]` (live API tests).
- Before architectural changes: read `docs/`. After behavior changes: update
  `docs/` in the same commit.
