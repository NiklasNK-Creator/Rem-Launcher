# Nova Launcher (working title) — Project Overview

Local-first, modded-Minecraft launcher. Rust + Tauri.

## Goal
Genuinely useful modded-Minecraft launcher: browse/install/update mods,
resource packs, datapacks, shaders from Modrinth and CurseForge behind one
provider abstraction, manage instances + modpacks, launch the game with a
Microsoft account. No backend, no telemetry, no accounts on our side.

## Status (2026-09-22)
Greenfield. Scaffold phase: docs + `crates/launcher-core` (provider trait,
Modrinth client, CurseForge client skeleton, models) + Tauri shell planned.

## Key facts (researched 2026-09-22)
- Modrinth API v2: `https://api.modrinth.com/v2`, open read access, JSON.
  Search: `GET /v2/search`; project: `GET /v2/project/{id}`; versions:
  `GET /v2/project/{id}/version`; download via version file URLs (CDN).
  Requires a descriptive `User-Agent` header. Rate limit 300 req/min.
- CurseForge Core API: `https://api.curseforge.com/v1`, requires API key in
  `x-api-key` header (`$2a$10$...` contact-endpoint key is NOT for third-party
  use). Key must come from the user/developer via authorized channel — see
  `docs/providers/curseforge.md`. Key required before CF search/download works.
- Minecraft (Mojang) auth: Microsoft OAuth2 (device-code or browser) → Xbox
  Live XSTS → `https://sessionserver.mojang.com` / Minecraft Services
  entitlements + profile. Must be implemented before launch works.
- Competitor gaps worth exploiting: Prism is powerful but intimidating;
  Modrinth App is tied to its own ecosystem; ATLauncher/CurseForge app carry
  ads/closed bits; dependency resolution + safe update UX is weak everywhere.

## Docs map
- `docs/architecture.md` — system design + provider abstraction
- `docs/roadmap.md` — feature inventory + build order
- `docs/decisions.md` — ADRs
- `docs/providers/modrinth.md`, `docs/providers/curseforge.md`
- `docs/auth.md` — Microsoft/Xbox/Minecraft auth plan
- `docs/security.md` — threat model + rules
- `docs/development.md` — build/test workflow
