# profile changelog

## 2.0.0 (2026-08-19)
- Logging contract: service logs now emitted as newline-delimited JSON (NDJSON) to stdout per the platform LXS logging contract (`ts`/`level`/`msg` + optional `service`,`request_id`,`status`,`latency_ms`,`user_id`,`error`). Breaking change — log output format changed.

## 1.1.1 — public content URL (2026-08-16)
- Content URLs returned after avatar/cover upload use `STORAGE_PUBLIC_URL`
  (the estate's public origin) instead of the internal `127.0.0.1` base, so
  `<img>` avatars and header avatars work from browsers.

## 1.1.0 — previous
Avatar ownership, proxied uploads to storage LXS.
