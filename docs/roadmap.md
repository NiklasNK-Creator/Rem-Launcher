# Roadmap

## Done (2026-09-22)
- [x] Provider trait + Modrinth v2 client (search/project/versions/deps)
- [x] Tauri shell + blue frontend (accounts/instances/search/play/log)
- [x] Multi-accounts: Microsoft (prismarine-auth device code) + offline,
      keychain tokens, last-used tracking
- [x] Instances: create/list/delete, per-type dirs, verify/repair
- [x] Search all 5 Modrinth content types; install with dep preview
- [x] Lockfile + update badges + one-click updates
- [x] Launch: client/libs/natives/assets, Fabric + Quilt, modern args,
      Java gate, progress bar, log capture, usage memory
- [x] .mrpack import/export round-trip
- [x] Dependency + license review (all MIT/Apache)

## Next (in order)
1. Supervised live-launch test with user → tag `v0.1.0-alpha`.
2. Forge/NeoForge loader support (different installer model).
3. Silent MS token refresh (persist prismarine-auth cache to disk).
4. Entitlements check at sign-in (owned vs Game Pass).
5. CurseForge — only if user reverses deferral (needs API key).
6. Skin/cape management, server list, world backups.

## Decisions (user, 2026-09-22)
- Name: **Rem Launcher** (blue theme, Rem-inspired).
- CurseForge: deferred (Modrinth-only).
- Auth: prismarine-auth Switch-title flow — no Entra registration needed
  (supersedes earlier "register our own client ID" plan).
- Offline/cracked accounts: fully supported alongside Microsoft.
