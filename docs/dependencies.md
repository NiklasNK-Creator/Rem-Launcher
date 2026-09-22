# Dependency & license notes

Reviewed 2026-09-22 for the alpha gate.

## Direct Rust deps (all permissive, standard ecosystem)
- `tauri 2` (Apache-2.0/MIT) — app shell.
- `tokio 1` (MIT), `reqwest 0.12` (MIT/Apache-2.0) — async HTTP.
- `serde 1` + `serde_json 1` (MIT/Apache-2.0) — models/config.
- `thiserror 2` (MIT/Apache-2.0), `async-trait 0.1` (MIT/Apache-2.0).
- `sha1 0.10`, `sha2 0.10` (MIT/Apache-2.0) — artifact/instance verification.
- `zip 2` (MIT/Apache-2.0) — natives extraction.
- `urlencoding 2` (MIT) — Modrinth query building.
- Transitive tree is the standard tokio/reqwest/tauri closure; no GPL,
  no copyleft, no native shims beyond Tauri's own WebView2 linkage.

## Direct JS deps
- `@tauri-apps/api 2` (Apache-2.0/MIT).
- `prismarine-auth 3.1.1` (MIT) → `@azure/msal-node` (MIT),
  `@xboxreplay/xboxlive-auth` (MIT), `jsonwebtoken` (MIT), small utils (MIT).
  No GPL anywhere in the subtree.

## Notes
- prismarine-auth defaults to the Switch-title `live` flow (community-used
  client IDs under its `Titles`); no Entra registration needed. If Microsoft
  ever revokes those IDs, sign-in breaks until we register our own app ID —
  tracked as a known risk, not a blocker.
- Modrinth API: open, proper UA sent, rate limits respected.
- CurseForge: deferred (no key, no calls). No ToS exposure.
- No telemetry, backend, or recurring-cost service anywhere.
