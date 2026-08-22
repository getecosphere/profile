# profile changelog

## 2.0.1 (2026-08-22)
- Corrected ownership documentation: Profile + Storage own avatar and cover
  uploads; Auth remains pure credentials and identity.
- Declare `STORAGE_PUBLIC_URL` so Eco injects browser-reachable content URLs
  through the gateway instead of persisting an internal Storage address.
- Added the canonical `/api/profile/*` gateway composition guidance in
  `AGENTS.md` so profile-ui and core estates use one browser prefix.

## 2.0.0 (2026-08-19)
- Logging contract: service logs now emitted as newline-delimited JSON (NDJSON) to stdout per the platform LXS logging contract (`ts`/`level`/`msg` + optional `service`,`request_id`,`status`,`latency_ms`,`user_id`,`error`). Breaking change — log output format changed.

## 1.1.1 — public content URL (2026-08-16)
- Content URLs returned after avatar/cover upload use `STORAGE_PUBLIC_URL`
  (the estate's public origin) instead of the internal `127.0.0.1` base, so
  `<img>` avatars and header avatars work from browsers.

## 1.1.0 — previous
Avatar ownership, proxied uploads to storage LXS.
