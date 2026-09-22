# Rem Launcher

A local-first Minecraft launcher focused on modded Minecraft. Rem Launcher is built with Rust, Tauri, and TypeScript.

## Current capabilities

- Microsoft accounts through [`prismarine-auth`](https://github.com/PrismarineJS/prismarine-auth) and offline/cracked accounts
- Multiple accounts with OS-keychain token storage
- Minecraft instances with configurable memory and JVM arguments
- Modrinth search for mods, resource packs, shaders, datapacks, and modpacks
- Version and loader filters, sorting, pagination, dependency preview, and optional dependencies
- SHA-1/SHA-512 verified downloads, lockfiles, update checks, enable/disable, verify, repair, clone, and delete
- Fabric and Quilt launching; Forge/NeoForge version resolution with installer processors still pending
- Modrinth `.mrpack` import/export
- World backups and restore with zip-slip protection
- Launch progress, Java compatibility checks, and per-instance game logs

## Development

Requirements: Rust stable, Node.js 20+, and Java for launching Minecraft.

```bash
npm install
npm run build
cargo build -p rem-launcher
cargo test -p launcher-core -p launcher-install -p rem-launcher
```

Run the Tauri application with:

```bash
cargo tauri dev
```

Read `docs/` before changing architecture, authentication, providers, security, or installation behavior. Update the relevant documentation in the same change.

## Security and privacy

Rem Launcher is local-first: it has no project backend, telemetry, or launcher account service. Provider, Mojang, Microsoft, and Xbox network calls are documented in `docs/`. Tokens are stored in the operating system keychain, never in `accounts.json` or source control. Downloads are hash-verified and restricted to approved HTTPS hosts where applicable.

## Status

The project is in pre-alpha development. A supervised live-launch test is still required before tagging `v0.1.0-alpha`. See `docs/roadmap.md` for the current roadmap and known limitations.

## License

MIT. See `LICENSE`.
