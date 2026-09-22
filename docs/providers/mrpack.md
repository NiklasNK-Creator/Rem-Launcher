# Modrinth modpack format (.mrpack)

Sourced from Modrinth help center (2026-09-22). Read before changing
`src-tauri/src/modpacks.rs`.

- ZIP with `modrinth.index.json` at root (UTF-8). `formatVersion: 1`,
  `game: "minecraft"`, `versionId`, `name`, `dependencies` map
  (`minecraft`, `forge`/`neoforge`/`fabric-loader`/`quilt-loader`),
  `files[]` with `path` (instance-relative), `hashes` (sha1+sha512),
  optional `env` (`client`/`server`: required/optional/unsupported),
  `downloads[]` (HTTPS URLs), `fileSize`.
- Storage: `overrides/` → copied to instance root; `client-overrides/`
  same (client); `server-overrides/` server-only (we skip it).
- Security (spec warns explicitly): `path` must not escape the instance
  (`..`, drive names, leading `/`/`\`). Allowed download domains:
  `cdn.modrinth.com`, `github.com`, `raw.githubusercontent.com`,
  `gitlab.com`. Follow redirects.
- Our import: validates paths + hosts, skips `client: unsupported` files,
  verifies sha512/sha1, creates an instance from pack name + deps.
