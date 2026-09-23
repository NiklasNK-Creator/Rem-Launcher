# Auth (Microsoft via prismarine-auth + offline accounts)

Implemented (2026-09-22/23):
- `prismarine-auth` `Authflow` (device-code → MSA → Xbox → MC Java token,
  `fetchProfile` and entitlements) runs in the frontend; profile becomes a
  `microsoft` account.
- `src-tauri/src/accounts.rs`: list/add/remove commands, JSON profile store,
  OS-keychain token storage, and persistent prismarine cache commands.
- `src/auth.ts`: offline accounts with genuine vanilla MD5 UUIDs, Microsoft
  device-code sign-in, persistent `CacheFactory`, and silent refresh helper.
- `accounts.json` never contains secrets. Cache files live under app data and
  are used by prismarine-auth for MSA/Xbox/Minecraft token refresh.
- Missing/expired credentials produce a clear re-auth prompt.
- No Entra app needed: prismarine-auth defaults to the Switch-title flow.

Remaining: improve cache encryption at rest and test a full live launch with a
real Microsoft account. Tokens are never logged or uploaded by Rem Launcher.
