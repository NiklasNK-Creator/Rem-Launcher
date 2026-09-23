# Server management

Rem Launcher stores saved Java multiplayer servers in app data (`servers.json`).
Each entry has a display name, address, and last successful TCP ping latency.

## UI

The **Servers** tab adds, removes, and pings saved servers. The Play dashboard
also exposes saved addresses in an optional server selector. When selected,
the launcher passes `--server`/`--port` and `--quickPlayMultiplayer` to the
Minecraft process after the normal account and instance arguments.

## Safety

Addresses reject whitespace, slashes, newlines, and oversized values. Ping is
a three-second TCP reachability check; it does not parse server status packets
or upload player data. No server directory or backend service is introduced.
