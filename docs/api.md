# profile API

Base path: `/api`. Auth: all write endpoints require `Authorization: Bearer
<JWT>` where the token is an HS512 JWT issued by `auth` (claims
`sub`/`username`/`role`/`iat`/`exp`). Read endpoints and the taxonomy
endpoints are public. Errors come in two shapes: the structured `AppError`
body `{code, message, details?, timestamp}` for validation/404/403/409/500,
and `{"error":"Unauthorized","message":"Unauthorized: <reason>"}` for
missing/invalid tokens (401).

## Endpoints

### GET /api/health
- **Purpose:** liveness probe.
- **Auth required:** no
- **Success 200:** `{"status":"UP"}`

### GET /api/users
- **Purpose:** list all non-soft-deleted users from the local store. Does
  NOT call auth (no freshness refresh).
- **Auth required:** no
- **Success 200:** JSON array of `UserDto` (see below).

### GET /api/users/search?platformId=&query=
- **Purpose:** case-insensitive `name` regex search scoped to a platform.
- **Auth required:** no
- **Query params:**
  | Param | Type | Required | Notes |
  |---|---|---|---|
  | `platformId` | string | yes | exact match on `platformId` |
  | `query` | string | yes | regex against `name` (options `i`) |
- **Success 200:** JSON array of `UserDto`. Unknown platform → `[]`.
- **Notes:** hits MongoDB directly (`$regex`, `$options: "i"`); no auth call.

### GET /api/users/platform/{platformId}
- **Purpose:** list non-soft-deleted users whose `platformId` matches.
- **Auth required:** no
- **Path params:** `platformId`
- **Success 200:** JSON array of `UserDto`. Unknown platform → `[]`.

### GET /api/users/username/{username}
- **Purpose:** fetch one user by username. Calls auth first
  (`GET /auth/users/username/{username}`) and upserts the local row
  (freshness + lazy hydration), so identity/role changes surface
  immediately. Avatar/cover are profile-owned (written locally via the
  storage LXS), so they are not overwritten by the auth sync.
- **Auth required:** no
- **Path params:** `username`
- **Success 200:** `UserDto`.
- **Errors:**
  - 404 → `{"code":"RESOURCE_NOT_FOUND","message":"User not found with username: {username}","timestamp":"…"}` (auth 404 **or auth unreachable** — fail closed).
  - 500 → `{"code":"INTERNAL_SERVER_ERROR","message":"An unexpected error occurred","timestamp":"…"}`

### GET /api/users/{userId}
- **Purpose:** fetch one user by id. Calls auth first
  (`GET /auth/users/{id}`) and upserts (freshness + lazy hydration).
- **Auth required:** no
- **Path params:** `userId` (ObjectId hex)
- **Success 200:** `UserDto`.
- **Errors:**
  - 404 → `{"code":"RESOURCE_NOT_FOUND","message":"User not found with id: {userId}","timestamp":"…"}`
  - 500 → `INTERNAL_SERVER_ERROR`

### PUT /api/users/{userId}
- **Purpose:** update own profile. Role-gated and self-gated.
- **Auth required:** yes — roles `OWNER` / `MENTOR` / `MEMBER`; token `sub`
  must equal `userId` (else 403 "You may only edit your own profile").
- **Body:** all fields optional (`UpdateUserRequest`):
  | Field | Type | Notes |
  |---|---|---|
  | `name` | string | forwarded to auth (`PUT /auth/me`) — auth owns identity |
  | `headline` | string | |
  | `bio` | string | |
  | `location` | string | |
  | `website` | string | |
  | `interests` | string[] | |
  | `whatsappNumber` | string | |
  | `province` | string | |
  | `codLocation` | string | per-user COD meeting location |
- **Success 200:** `UserDto` (with auth-refreshed identity fields).
- **Errors:**
  - 401 → `{"error":"Unauthorized","message":"Unauthorized: missing bearer token"}`
  - 403 → `{"code":"ACCESS_DENIED","message":"You may only edit your own profile","timestamp":"…"}`
  - 403 → `{"code":"ACCESS_DENIED","message":"Access denied","timestamp":"…"}` (role not permitted)
  - 400 → `{"code":"INVALID_ARGUMENT","message":"Auth could not update this identity","timestamp":"…"}` (auth rejected the name update)

### POST /api/users/{userId}/avatar
- **Purpose:** upload (or replace) the user's avatar. Profile proxies the
  bytes to the **storage LXS** (`POST /storage/objects`, namespace `avatars`)
  and stores the resulting content URL on the local profile row. Profile owns
  avatar/cover — auth does not.
- **Auth required:** yes — roles `OWNER`/`MENTOR`/`MEMBER`
- **Multipart:** field `file` (image bytes).
- **Success 201:** `{ "avatarUrl": "<storage content URL>" }`
- **Errors:**
  - 400 → `INVALID_ARGUMENT` `"Missing 'file' field"` / multipart failure
  - 401 unauthenticated; 403 role denied; 404 user not found
  - 503 → `SERVICE_UNAVAILABLE` `"Avatar upload requires the storage LXS (STORAGE_BASE_URL is not set)."`
  - 502/500 → storage LXS unreachable or rejected the upload

### POST /api/users/{userId}/upload-cover-photo
- **Purpose:** upload (or replace) the user's cover photo. Same mechanics as
  avatar (namespace `cover-photos`).
- **Auth required:** yes — roles `OWNER`/`MENTOR`/`MEMBER`
- **Multipart:** field `file` (image bytes).
- **Success 201:** `{ "coverPhotoUrl": "<storage content URL>" }`
- **Errors:** same as avatar upload.

### POST /api/users/{userId}/experience
- **Purpose:** add an experience entry. Role-gated (no self-ownership check).
- **Auth required:** yes — roles `OWNER`/`MENTOR`/`MEMBER`
- **Body** (`ExperienceRequest`, all optional):
  `title`, `company`, `location`, `description`, `startDate`, `endDate`
  (string, `YYYY-MM-DD` or `DD/MM/YYYY`), `currentlyWorking` (bool)
- **Success 201:** `UserDto` with the new entry appended (id server-generated UUID).

### PUT /api/users/{userId}/experience/{experienceId}
- **Purpose:** update an experience entry (fields present are merged).
- **Auth required:** yes — same roles
- **Success 200:** `UserDto`.

### DELETE /api/users/{userId}/experience/{experienceId}
- **Purpose:** remove an experience entry.
- **Auth required:** yes — same roles
- **Success 200:** `UserDto`.

### POST /api/users/{userId}/education
- **Purpose:** add an education entry.
- **Auth required:** yes — same roles
- **Body** (`EducationRequest`, all optional): `school`, `degree`,
  `fieldOfStudy`, `startDate`, `endDate`, `description`
- **Success 201:** `UserDto`.

### PUT /api/users/{userId}/education/{educationId}
- **Purpose:** update an education entry.
- **Auth required:** yes — same roles
- **Success 200:** `UserDto`.

### DELETE /api/users/{userId}/education/{educationId}
- **Purpose:** remove an education entry.
- **Auth required:** yes — same roles
- **Success 200:** `UserDto`.

### POST /api/users/{userId}/skills?skill=
- **Purpose:** add a skill to the user's `skills` list (idempotent — already
  present skills are not duplicated).
- **Auth required:** yes — same roles
- **Query params:** `skill` (required)
- **Success 201:** `UserDto`.

### DELETE /api/users/{userId}/skills/{skill}
- **Purpose:** remove a skill (exact string match; no-op if absent).
- **Auth required:** yes — same roles
- **Path params:** `skill`
- **Success 200:** `UserDto`.

### POST /api/users/{userId}/certifications
- **Purpose:** add a certification.
- **Auth required:** yes — same roles
- **Body** (`CertificationRequest`, all optional): `name`, `issuer`,
  `issueDate`, `expirationDate` (lenient date strings), `credentialUrl`
- **Success 201:** `UserDto`.

### PUT /api/users/{userId}/certifications/{certificationId}
- **Purpose:** update a certification.
- **Auth required:** yes — same roles
- **Success 200:** `UserDto`.

### DELETE /api/users/{userId}/certifications/{certificationId}
- **Purpose:** remove a certification.
- **Auth required:** yes — same roles
- **Success 200:** `UserDto`.

### PUT /api/users/{userId}/social-links
- **Purpose:** merge social links (present fields overwrite).
- **Auth required:** yes — same roles
- **Body** (`SocialLinksRequest`, all optional): `linkedin`, `twitter`,
  `github`, `portfolio`
- **Success 200:** `UserDto`.

### GET /api/schools
- **Purpose:** list the school taxonomy, sorted alphabetically.
- **Auth required:** no
- **Success 200:** `["...", ...]` (JSON array of strings).

### POST /api/schools?name=
- **Purpose:** add a school to the taxonomy if missing (idempotent).
- **Auth required:** no
- **Query params:** `name` (required)
- **Success 201:** plain-text `name` (not JSON).

### GET /api/interests
- **Purpose:** list the interest taxonomy, sorted.
- **Auth required:** no
- **Success 200:** JSON array of strings.

### POST /api/interests?name=
- **Purpose:** add an interest if missing (idempotent).
- **Auth required:** no
- **Query params:** `name` (required)
- **Success 201:** plain-text `name`.

### GET /api/skills
- **Purpose:** list the skill taxonomy, sorted.
- **Auth required:** no
- **Success 200:** JSON array of strings.

### POST /api/skills?name=
- **Purpose:** add a skill if missing (idempotent).
- **Auth required:** no
- **Query params:** `name` (required)
- **Success 201:** plain-text `name`.

### UserDto (all reads/writes return this shape)
```json
{
  "id": "507f1f77bcf86cd799439011",
  "name": "Alice",
  "username": "alice",
  "email": "alice@example.com",
  "headline": "Backend engineer",
  "avatarUrl": "https://cdn.example.com/alice.png",
  "coverPhotoUrl": null,
  "bio": "Rustacean since 2024",
  "location": "Jakarta",
  "website": "https://example.com",
  "school": "ITB",
  "whatsappNumber": "+6281234567890",
  "province": "DKI Jakarta",
  "codLocation": "Monas",
  "role": "MEMBER",
  "platformId": "pl_abc",
  "interests": ["rust", "climbing"],
  "experiences": [
    {
      "id": "8f14e45f-ceea-467a-9f18-1f0b6c9e9c3b",
      "title": "Software Engineer",
      "company": "Acme",
      "location": "Remote",
      "description": "...",
      "startDate": "2022-03-01T00:00:00Z",
      "endDate": "2023-06-15T00:00:00Z",
      "currentlyWorking": false
    }
  ],
  "education": [],
  "skills": ["rust"],
  "certifications": [],
  "socialLinks": { "linkedin": "...", "twitter": "...", "github": "...", "portfolio": "..." },
  "createdAt": "2026-07-09T10:00:00Z",
  "updatedAt": "2026-08-12T10:00:00Z"
}
```

## Error reference

| Status | Code | Body shape | When |
|---|---|---|---|
| 400 | `VALIDATION_ERROR` | `{code,message,details,timestamp}` | blank required field (e.g. taxonomy `name`) |
| 400 | `INVALID_ARGUMENT` | `{code,message,timestamp}` | auth rejected an identity name update |
| 401 | — | `{"error":"Unauthorized","message":"Unauthorized: …"}` | missing/invalid/expired bearer token |
| 403 | `ACCESS_DENIED` | `{code,message,timestamp}` | role not permitted, or editing another user's profile |
| 404 | `RESOURCE_NOT_FOUND` | `{code,message,timestamp}` | user unknown to auth (or auth unreachable — fail closed) |
| 409 | `ALREADY_EXISTS` | `{code,message,timestamp}` | reserved for conflicts (not currently raised) |
| 500 | `INTERNAL_SERVER_ERROR` | `{code,message,timestamp}` | unexpected / Mongo / auth-client error |
| 429 | — | tower-governor body | per-source-IP rate limit exceeded |

## Rate limiting / limits

All routes share one per-source-IP token bucket
(`tower_governor`, `SmartIpKeyExtractor`): burst capacity
`RATE_LIMIT_GENERAL_BURST` (default `120`), refill
`RATE_LIMIT_GENERAL_REPLENISH_SECS` token(s)/sec (default `1`). Exceeding it
returns `429`. A background task prunes stale keys every 60s. Request bodies
are capped at **10 MiB** (`RequestBodyLimitLayer`). No per-route limits.

## Env vars (code-read in `src/config.rs` / `main.rs` / `lib.rs`)

| Var | Default | Notes |
|---|---|---|
| `MONGODB_URI` | `mongodb://localhost:27017/profile_dev` | must include a database name (bootstrap errors otherwise) |
| `JWT_SECRET` | — (required) | HS512 shared secret; **refuses to boot** if empty, a known placeholder, or < 32 bytes; warns < 64 bytes |
| `SERVER_PORT` | `8080` | listen port |
| `CORS_ALLOWED_ORIGINS` | `http://localhost:3000` | comma-separated list |
| `AUTH_BASE_URL` | `http://localhost:9001/api` | peer auth base (auth endpoints appended: `/auth/users/…`, `/auth/me`) |
| `RATE_LIMIT_GENERAL_BURST` | `120` | token bucket burst |
| `RATE_LIMIT_GENERAL_REPLENISH_SECS` | `1` | token(s) refill per second |

`API_BASE_URL` appears in `lxs.yml` and `.env.example` but is **not read** by
the code (the API is hardcoded under `/api`).
