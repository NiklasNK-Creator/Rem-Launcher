# Install system

- `launcher-install` crate: Mojang `version_manifest_v2` fetch,
  sha1-verified downloads, `Instance` model with traversal-safe paths.
- Live Mojang manifest verified reachable (2026-09-22).
- `src-tauri/src/launch.rs`: `prepare_client` + `detect_java` + `launch_game`.
  `launch_game` downloads client jar, rule-filtered libraries, and the asset
  index/objects (hash-verified) into app-data, then spawns `java` with the
  classpath + game args for the selected account (msa/legacy userType).
  JVM heap is configurable per instance (`max_memory_mb`, default 4096MB)
  with custom JVM arguments support.
  Play UI: account + instance selectors, status line with pid/errors,
  per-instance Configure (RAM), Folder (Explorer/Finder), Backup, and Restore.
- `launcher-install::forge::parse_install_profile` now parses Forge installer
  ZIPs safely and exposes versioned processors, classpaths, and arguments.
  Forge installers contain `install_profile.json` with processor tasks such as
  `EXTRACT_FILES`, `MCP_DATA`, `DOWNLOAD_MOJMAPS`, `MERGE_MAPPING`,
  `jarsplitter`, `ForgeAutoRenamingTool`, and `binarypatcher`. Full execution
  still requires downloading processor classpaths and running version-specific
  Java tools; launch rejects Forge/NeoForge rather than silently mislaunching.
