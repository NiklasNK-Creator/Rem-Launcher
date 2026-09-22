# Install system

- `launcher-install` crate: Mojang `version_manifest_v2` fetch,
  sha1-verified downloads, `Instance` model with traversal-safe paths.
- Live Mojang manifest verified reachable (2026-09-22).
- `src-tauri/src/launch.rs`: `prepare_client(version)` resolves the Mojang
  version package, sha1-downloads `client.jar` to app-data `versions/<v>/`;
  `detect_java()` finds a `java` on PATH. Actual process launch is next.
