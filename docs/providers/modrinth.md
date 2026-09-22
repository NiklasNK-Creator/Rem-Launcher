# Modrinth provider

- Base: `https://api.modrinth.com/v2`. Open read access, JSON.
- Required header: descriptive `User-Agent`
  (e.g. `nova-launcher/0.1 (+https://github.com/...)`).
- Rate limit: ~300 requests/min. Respect `Retry-After`.
- Key endpoints:
  - `GET /v2/search?q=&facets=&limit=&offset=` — project search.
  - `GET /v2/project/{id|slug}` — project details.
  - `GET /v2/project/{id}/version?game_versions=&loaders=` — versions.
  - File download: `url` field on version files (CDN, hash-verified sha512).
- Content types: `mod`, `resourcepack`, `datapack`, `shader`, `modpack`.
- Loaders: `fabric`, `forge`, `neoforge`, `quilt`, `vanilla`, ...
- License: API ToS require attribution + proper UA. No key needed.
