# Auth (Microsoft via prismarine-auth + offline accounts)

Implemented (2026-09-22):
- `prismarine-auth` `Authflow` (device-code → MSA → Xbox → MC Java token,
  `fetchProfile`) runs in the frontend; profile becomes a `microsoft` account.
- `src-tauri/src/accounts.rs`: `list/add/remove_account` commands, JSON store
  at app-data `accounts.json`. Multiple accounts supported.
- `src/auth.ts`: `addOfflineAccount(name)` (deterministic offline UUID),
  `addMicrosoftAccount(onCode)` showing the device code for the user.
- No Entra app needed: prismarine-auth defaults to the Switch-title flow.
- Follow-ups: persist ms refresh cache via on-disk cache dir (currently
  in-memory), OS-keychain encryption, silent refresh, entitlements check.
