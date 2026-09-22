# Roadmap

## Done (2026-09-22/23)
- [x] Provider trait + Modrinth v2 client (search/project/versions/deps)
- [x] Tauri shell + tabbed dark-blue frontend (accounts/instances/search/play/log)
- [x] Multi-accounts: Microsoft (prismarine-auth device code) + offline,
      keychain tokens, last-used tracking, entitlements, skin/cape display
- [x] Instances: create/list/delete/clone, per-type dirs, verify/repair,
      configurable RAM/JVM args, open folder, world backups/restore
- [x] Search all 5 Modrinth content types; version/loader/sort filters,
      pagination, dependency preview, optional dependencies
- [x] Lockfile + update badges + one-click updates + enable/disable toggles
- [x] Launch: client/libs/natives/assets, Fabric + Quilt, modern args,
      Java gate, progress bar, log capture, usage memory
- [x] `.mrpack` import/export round-trip with path/hash/host validation
- [x] Dependency + license review (all MIT/Apache)
- [x] README + MIT license + UI smoke verification

## Next (in order)
1. Supervised live-launch test with user → tag `v0.1.0-alpha`.
2. Forge/NeoForge installer processor support (version resolution exists).
3. Silent MS token refresh (persist prismarine-auth cache to disk).
4. Server list and multiplayer quick-connect.
5. CurseForge — only if user reverses deferral (needs API key).

## Decisions (user, 2026-09-22)
- Name: **Rem Launcher** (blue theme, Rem-inspired).
- CurseForge: deferred (Modrinth-only).
- Auth: prismarine-auth Switch-title flow — no Entra registration needed
  (supersedes earlier "register our own client ID" plan).
- Offline/cracked accounts: fully supported alongside Microsoft.
