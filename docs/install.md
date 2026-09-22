# Install system

- `launcher-install` crate: Mojang `version_manifest_v2` fetch,
  sha1-verified downloads, `Instance` model with traversal-safe paths.
- Live Mojang manifest verified reachable (2026-09-22).
- `src-tauri/src/launch.rs`: `prepare_client` + `detect_java` + `launch_game`.
  `launch_game` downloads client jar, rule-filtered libraries, and the asset
  index/objects (hash-verified) into app-data, then spawns `java` with the
  classpath + game args for the selected account (msa/legacy userType).
  Play UI: account + instance selectors, status line with pid/errors.
