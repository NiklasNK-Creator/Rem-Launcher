# Roadmap

## Done
- [x] Initial research (Modrinth v2, CurseForge Core, MS auth, competitors)
- [ ] Scaffold: workspace + launcher-core + docs (in progress)

## Next (in order)
1. `launcher-core`: unified models + provider trait + Modrinth client
   (search/project/versions) + CurseForge client skeleton (key-gated).
2. Tauri shell: `src-tauri` + minimal frontend calling `search` command.
3. `launcher-auth`: Microsoft device-code flow → Xbox XSTS → MC profile;
   encrypted token store (OS keychain).
4. `launcher-install`: Mojang version manifests, artifact download with
   hash verification, Java discovery, instance dirs, launch.
5. Mod management: install/update/remove, dependency resolution across
   providers, lockfile per instance.
6. Modpack import/export (Modrinth `.mrpack` first, CurseForge second).
7. Offline mode, repair/verify, update channels.

## Decisions (user, 2026-09-22)
- Name: **Rem Launcher** (blue theme, Rem-inspired).
- CurseForge: deferred (Modrinth-only for now).
- Auth: register our own Entra public-client ID (pending Azure-portal step).
