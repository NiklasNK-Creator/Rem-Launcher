# Security

- Local-first: no backend, no telemetry. Only network peers: Modrinth API/CDN,
  CurseForge API (key-gated), Mojang/Xbox/Microsoft auth + manifests.
- Secrets (CF API key, OAuth tokens) live in OS keychain or env, NEVER in
  source, docs, logs, or commits. `.env` is git-ignored.
- Downloads: hash-verify (sha512/sha1) every artifact before use; HTTPS only.
  Direct mod and modpack downloads are restricted to approved content hosts
  (`cdn.modrinth.com`, `github.com`, `raw.githubusercontent.com`, `gitlab.com`).
- Untrusted content: treat mod JARs as untrusted code; never execute during
  install/scan; parse metadata without running.
- Path safety: confine instance writes to the launcher data dir; reject
  traversal (`..`, absolute paths, Windows-special characters, leading dots)
  via single shared `valid_instance_name` guard across all commands.
- Modpacks: reject zip-slip and unsafe paths on extraction.
- Input validation at provider boundaries; rate-limit respect + backoff.
- Release audit: see release criteria in project brief; check secrets,
  endpoints, costs before every release.
