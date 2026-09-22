# Architecture

> Read this before changing the system. Update it when behavior changes.

## Stack
- **Backend:** Rust workspace (`crates/*`). All business logic lives here.
- **Shell:** Tauri v2 — Rust commands + webview frontend. No backend server.
- **Frontend:** TypeScript + Vite (planned). Thin: calls Tauri commands only.

## Crates
- `launcher-core` — pure logic, no I/O globals: provider abstraction,
  search models, version resolution, instance model, hash utilities.
  MUST stay UI- and Tauri-independent so it is unit-testable.
- `providers/modrinth` (inside core for now, split later) — Modrinth v2 client.
- `providers/curseforge` — CurseForge Core v1 client (key-gated).
- Planned: `launcher-auth` (Microsoft/Xbox/MC flow + token store),
  `launcher-install` (version manifests, downloads, Java detection, launch),
  `launcher-app` (Tauri command layer).

## Provider abstraction (core idea)
```rust
trait ContentProvider {
    fn id(&self) -> ProviderId;               // modrinth | curseforge | ...
    fn search(&self, query: SearchQuery) ...;
    fn project(&self, id: &str) ...;
    fn versions(&self, id: &str, filters: VersionFilter) ...;
    fn resolve(&self, req: DependencyRequest) ...; // dep resolution per provider
}
```
Unified models (`Project`, `ProjectVersion`, `SearchQuery`) are provider-neutral;
provider-specific fields live behind `ProviderMeta`. Capabilities are explicit
(`ProviderCapabilities { search, version_files, dependencies, ... }`) because
CurseForge and Modrinth do NOT expose identical functionality — never paper
over differences with lossy unification.

## Rules for future agents
- Read `docs/` before architectural changes.
- Business logic goes in crates, never in Tauri command handlers.
- No new external service, backend, telemetry, or recurring cost without docs
  + explicit user approval. Local-first is non-negotiable.
- Secrets (CF API key, OAuth client secrets) never in source/docs/commits.
