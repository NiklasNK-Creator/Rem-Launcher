# Decisions

## ADR-001 (2026-09-22): Rust + Tauri
Chosen per project direction. Rust for safe concurrent downloads/installs;
Tauri for small local-first binary with no bundled Chromium-scale runtime.
No counter-evidence found in research.

## ADR-002 (2026-09-22): Provider trait with explicit capabilities
CurseForge (key-gated, ToS-restricted) and Modrinth (open) differ too much
for a lowest-common-denominator API. Trait + capability flags; unified models
for shared fields only.

## ADR-003 (2026-09-22): Modrinth first, CurseForge key-gated skeleton
Modrinth works with no key; CurseForge client ships disabled until a key is
configured, with a clear error. Never embed a scraped/shared CF key.

## ADR-004 (2026-09-22): Local-first, no backend
All state on disk; providers + Mojang/Xbox endpoints are the only network
peers. No telemetry, no accounts, no server costs.
