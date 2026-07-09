# profile

User profile domain — bio, headline, location, website, experience,
education, skills, certifications, social links, interests, platformId, plus
the school/interest/skill reference-data taxonomies. Split out of `rwid/lms`
(the pre-eco monolith) into an independent eco domain, rewritten in Rust
(axum). See `rwid/auth/backend/docs/auth-rewrote-from-java-to-rust.md` for
the origin of this pattern.

## Status

Fully ported and verified. Source: `UserController`/`UserService` (already
Java, already wired to this exact split) plus `SchoolController`/
`InterestController`/`SkillController`, all from `lms-backend`.

## What this domain does and doesn't own

auth owns credentials, JWT, and identity (`username`/`email`/`name`/`role`/
`avatarUrl`/`coverPhotoUrl`). profile owns everything else about a person:
bio, headline, location, website, `whatsappNumber`, `province`, `school`,
`platformId`, `interests`, `experiences`, `education`, `skills`,
`certifications`, `socialLinks`.

profile's local `User` document is **not authoritative** for the
auth-owned fields — it's a cache, refreshed live via `AuthClient` on every
`GET /users/{id}` and `GET /users/username/{username}` (see below). Never
write `username`/`email`/`name`/`role`/`avatarUrl`/`coverPhotoUrl` to a
local row directly; they only ever come from auth.

## Dependencies

`auth` — the only peer call this service makes. `AuthClient`
(`src/auth_client.rs`) hits `GET /auth/users/{id}` and
`GET /auth/users/username/{username}` for two purposes:

1. **Lazy hydration**: the first time this service is asked about a user it
   has no local row for (`require_entity_by_id` in `repo/users.rs`), it
   fetches identity from auth and creates the row rather than 404ing —
   important because a user can register through auth and never have
   touched profile yet.
2. **Freshness on read**: every `GET /users/{id}` / `GET /users/username/{x}`
   calls `sync_from_auth` unconditionally first, so avatar/cover/role
   changes made through auth show up immediately. Profile-*editing*
   operations (add experience, update bio, etc.) only hydrate a missing row
   — they don't force a refresh on every write, since those don't need
   avatar freshness and a network round-trip on every edit would be wasteful.

If `auth` is unreachable, both hydration and freshness-refresh fail closed
(`AuthClient::fetch` returns `None` on any error, which surfaces as 404) —
there's no stale-cache fallback. Acceptable for now; revisit if auth
downtime needs to degrade more gracefully than "profile reads 404 too."

## API

Base path `/api`.

- `GET /users` — all (excludes soft-deleted)
- `GET /users/search?platformId=&query=` — name search scoped to a platform
- `GET /users/platform/{platformId}`
- `GET /users/username/{username}`, `GET /users/{id}` — fresh (see above)
- `PUT /users/{id}` — `OWNER`/`MENTOR`/`MEMBER`; name/headline/bio/location/
  website/interests/whatsappNumber/province
- `POST/PUT/DELETE` on `/users/{id}/experience[/{id}]`,
  `/users/{id}/education[/{id}]`, `/users/{id}/certifications[/{id}]` —
  same role gate
- `POST /users/{id}/skills`, `DELETE /users/{id}/skills/{skill}` — same
  role gate
- `PUT /users/{id}/social-links` — same role gate
- `GET/POST /schools`, `/interests`, `/skills` (top-level, the taxonomy
  lists) — public, one generic implementation
  (`repo::tags::get_all_sorted`/`add_if_missing`) backing all three
  collections instead of three copies of the same code

## Not carried over

`avatarUrl`/`coverPhotoUrl` are **read-only** here (composed from auth on
every fetch). This service has no upload endpoint and no file storage —
uploading an avatar/cover is exclusively `POST /users/{id}/avatar` and
`POST /users/{id}/upload-cover-photo` on `auth`. If you're looking for that
code, it's not missing from this port, it was never meant to be here.

## Date parsing

Experience/education/certification dates accept either `YYYY-MM-DD` or
`DD/MM/YYYY` strings, same as the Java version's `parseDate` — see
`src/date_parse.rs`. An unparseable date is dropped (logged as a warning),
not rejected as a validation error, matching the original's lenient
behavior.

## Observability (added 2026-07-09)

Logs are structured JSON (`tracing_subscriber::fmt().json()`), not the
default human-readable text — prep for centralized log aggregation
(Grafana Loki is the leading candidate, self-hosted alongside the rest of
the estate rather than a SaaS product, in keeping with `eco`'s host-native
philosophy).

Every request gets a correlation id (`src/request_id.rs`): reused from an
incoming `x-request-id` header if present, otherwise a fresh UUID,
recorded on the request's tracing span (so every JSON log line during
that request carries it) and echoed back on the response. `auth_client.rs`'s
`fetch()` now forwards this same header on every outbound call to auth
(both hydration and freshness-refresh), so a single profile request and
the auth call it triggers share one `request_id` and are reconstructable
as one trail once logs are aggregated somewhere queryable.

## Verified

Built, ran against a live `auth` instance and local MongoDB: lazy hydration
for a user registered directly against auth and never seen by profile
before (both by-id and by-username paths), profile field updates,
experience add with date parsing, skills, social links, the schools/
interests/skills taxonomy endpoints, and 401 on an unauthenticated edit
attempt.
