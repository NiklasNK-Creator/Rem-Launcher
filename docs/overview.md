# Rem Launcher — Project Overview

Local-first, modded-Minecraft launcher. Rust + Tauri.

## Goal
Genuinely useful modded-Minecraft launcher: browse/install/update mods,
resource packs, datapacks, shaders, modpacks from Modrinth behind a provider
abstraction (CurseForge deferred per user decision), manage instances,
launch the game with Microsoft or offline accounts. No backend, no telemetry,
no accounts on our side.

## Status (2026-09-22, 30 commits)
Working end-to-end: multi-accounts (MS via prismarine-auth + offline,
keychain tokens) → instances (vanilla/fabric/quilt) → Modrinth search
(5 content types) → dep-aware install with preview → lockfile updates →
verify/repair → launch with progress + logs → .mrpack import/export.
Superseded notes: Entra registration unneeded (prismarine-auth Switch flow).

## Key facts (researched 2026-09-22)
- Modrinth API v2: `https://api.modrinth.com/v2`, open read access, JSON.
  Search: `GET /v2/search`; project: `GET /v2/project/{id}`; versions:
  `GET /v2/project/{id}/version`; download via version file URLs (CDN).
  Requires a descriptive `User-Agent` header. Rate limit 300 req/min.
- CurseForge: DEFERRED by user (Modrinth-only). Skeleton remains key-gated.
- Auth: prismarine-auth `Authflow` device-code flow, no Entra app needed.
  Tokens in OS keychain. Offline accounts with stable UUIDs.
- Fabric + Quilt via their meta APIs; Forge/NeoForge still open.
- Competitor gaps worth exploiting: Prism is powerful but intimidating;
  Modrinth App is tied to its own ecosystem; ATLauncher/CurseForge app carry
  ads/closed bits; dependency resolution + safe update UX is weak everywhere.

## Docs map
- `docs/architecture.md` — system design + provider abstraction
- `docs/roadmap.md` — feature inventory + build order
- `docs/decisions.md` — ADRs
- `docs/providers/modrinth.md`, `docs/providers/mrpack.md`
- `docs/auth.md` — implemented auth (prismarine-auth + keychain)
- `docs/security.md` — threat model + rules
- `docs/development.md` — build/test workflow
- `docs/dependencies.md` — license review
- `docs/install.md` — install + launch systems
