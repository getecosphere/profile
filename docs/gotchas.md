# Gotchas

Production constraints that are NOT visible in the binary. Source: code
comments across `backend/src/**` and the domain README.md.

- **Reads fail closed when `auth` is down.** Every `GET /users/{id}` and
  `GET /users/username/{x}` calls `sync_from_auth` first
  (`repo/users.rs`), and `AuthClient::fetch` returns `None` on *any* error
  (network, 500, 404). `None` surfaces as **404 RESOURCE_NOT_FOUND** — so
  when auth is unreachable, profile reads 404 for users it already knows.
  There is no stale-cache fallback. A user that exists in auth but is unknown
  here is **lazy-hydrated** on first read instead of 404ing.
- **Only by-id and by-username reads refresh from auth.** `GET /users`,
  `/users/search`, and `/users/platform/{id}` hit MongoDB directly with no
  auth round-trip — freshly-uploaded avatars/covers/roles are NOT reflected
  there until an id/username read re-syncs the row.
- **Sub-entity writes are role-gated but NOT self-ownership-checked.**
  `PUT /users/{id}` additionally enforces `auth.user_id == user_id`
  (403 "You may only edit your own profile"), but experience/education/
  certification/skills/social-links handlers only call
  `auth.require_role(PROFILE_ROLES)` — any valid OWNER/MENTOR/MEMBER token
  can add/update/delete entries on **another** user's profile. Preserved
  from the Java original; don't assume it's safe to expose.
- **Identity fields are read-only here.** Never write
  `username`/`email`/`name`/`role` to a local
  row — they only come from auth. `PUT /users/{id}` with a `name` forwards
  the caller's bearer token to auth (`PUT /auth/me`) and returns 400
  `"Auth could not update this identity"` if auth rejects it.
- **Avatar/cover require Storage.** `POST /users/{id}/avatar` and
  `POST /users/{id}/upload-cover-photo` proxy bytes to the `storage` LXS.
  Set `STORAGE_BASE_URL` (and its public content URL contract) or uploads fail
  with 503. Profile is the writer of `avatarUrl`/`coverPhotoUrl`; Auth never
  receives those fields.
- **Lenient, lossy date parsing.** `startDate`/`endDate`/`issueDate`/
  `expirationDate` accept `YYYY-MM-DD` or `DD/MM/YYYY` only
  (`src/date_parse.rs`, mirrors Java `parseDate`). An unparseable date is
  **dropped with a warning log**, not a validation error — a typo'd date
  silently disappears.
- **`JWT_SECRET` startup guards.** Refuses to boot if `JWT_SECRET` is unset,
  empty, one of `your-secret-key-change-in-production` /
  `change-this-secret` / `secret`, or shorter than **32 bytes**; warns (but
  boots) below the HS512-recommended 64 bytes. Must match the estate's shared
  HS512 secret — this service only validates, never issues.
- **`MONGODB_URI` must include a database name** — `bootstrap()` errors out
  otherwise (`client.default_database()`). Soft-deletes are a `deletedAt`
  field; all list/read queries filter `deletedAt: null`.
- **Rate limit is per-source-IP and shared across every route.** One token
  bucket (`SmartIpKeyExtractor`) covers all endpoints: burst
  `RATE_LIMIT_GENERAL_BURST` (default 120), refill
  `RATE_LIMIT_GENERAL_REPLENISH_SECS` token(s)/sec (default 1). Behind a NAT
  or proxy, all clients share one bucket and can throttle each other. A
  60s `retain_recent()` cleanup keeps the key table bounded. Exceeding →
  **429**.
- **Request bodies capped at 10 MiB** (`RequestBodyLimitLayer`).
- **Two error body shapes.** `AppError` failures return
  `{code,message,details?,timestamp}`; missing/invalid JWTs return the
  different `{"error":"Unauthorized","message":"Unauthorized: …"}` shape
  (from `AuthRejection`). Match on status first, body second.
- **`API_BASE_URL` is unused.** Declared in `lxs.yml` and `.env.example`,
  but `config.rs` never reads it — the API is hardcoded under `/api` and the
  port comes from `SERVER_PORT` (default **8080** in code;
  `.env.example` shows 9010).
- **Observability coupling:** JSON logs + `x-request-id` correlation id
  (reused from inbound header or generated), echoed on the response and
  forwarded to auth on every sync call, so a profile request and its auth
  call share one trace.
- **`codLocation` is new in 1.0.2** (`feat: per-user COD meeting location`).
  Older consumers that strictly deserialize `UserDto`/`User` will see an
  extra field; the model tolerates it on read.
