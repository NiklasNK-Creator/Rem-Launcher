# CurseForge provider

- Base: `https://api.curseforge.com/v1`. Requires `x-api-key` header.
- API key: obtain via CurseForge for Studios / contact form. Approval needed
  for launchers; ToS restrict redistribution and require attribution.
  NEVER commit a key. The client MUST fail with a clear
  `MissingApiKey` error when unset, and read the key from OS
  keychain / env `CURSEFORGE_API_KEY` only.
- Key endpoints: `GET /v1/mods/search`, `GET /v1/mods/{modId}`,
  `GET /v1/mods/{modId}/files`, `GET /v1/mods/{modId}/files/{fileId}`.
- Download URLs may require the key and expire; re-resolve, don't cache URLs.
- Status: skeleton only until key strategy is decided with user.
