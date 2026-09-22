# Auth (Microsoft via prismarine-auth + offline accounts)

Implemented (2026-09-22):
- `prismarine-auth` `Authflow` (device-code → MSA → Xbox → MC Java token,
  `fetchProfile`) runs in the frontend; profile becomes a `microsoft` account.
- `src-tauri/src/accounts.rs`: `list/add/remove_account` commands, JSON store
  at app-data `accounts.json`. Multiple accounts supported.
- `src/auth.ts`: `addOfflineAccount(name)` (deterministic offline UUID),
  `addMicrosoftAccount(onCode)` showing the device code for the user.
- No Entra app needed: prismarine-auth defaults to the Switch-title flow.
- Tokens in OS keychain (`keyring`, service `dev.remlauncher.app`):
  `accounts.json` never holds secrets; `get_account_token` reads at launch;
  removal wipes the credential. Missing/expired token → clear re-auth prompt.
- Remaining: silent refresh via cached MS refresh token, entitlements check.
