# profile — LXS docs

## Capability

User-profile domain: bio, headline, location, website, WhatsApp number,
province, COD meeting location (`codLocation`), interests, skills,
experience, education, certifications, social links, and platformId — plus
the school / interest / skill reference-data taxonomies. Read endpoints
lazily hydrate a user row from the `auth` domain and keep auth-owned identity
fields (name/username/email/role) fresh on every read. Writes
are gated on a Bearer HS512 JWT and profile-editing roles.

## What it owns / never owns

- **Owns:** everything about a person except credentials: bio, avatar and
  cover URLs, headline,
  location, website, `whatsappNumber`, `province`, `codLocation`, `school`,
  `platformId`, `interests`, `experiences`, `education`, `skills`,
  `certifications`, `socialLinks`; and the `schools` / `interests` / `skills`
  taxonomy collections.
- **Never owns:** credentials, JWT issuance, and identity fields
  (`username`/`email`/`name`/`role`) — those belong to `auth`. Avatar and cover
  uploads are proxied to `storage`, then their URLs are stored here.

## Compose it

```yaml
# ecompose.yml
services:
  profile-backend:
    lxs: profile@1.0.2
    grants:
      secrets: [MONGODB_URI, JWT_SECRET, SERVER_PORT, API_BASE_URL]
```

## Quick usage

```bash
# Health (under /api)
curl -s http://localhost:8080/api/health     # {"status":"UP"}

# Public read (refreshes identity from auth, hydrates if unknown)
curl -s http://localhost:8080/api/users/507f1f77bcf86cd799439011

# Public taxonomy
curl -s http://localhost:8080/api/schools    # ["...", ...]

# Editing requires a Bearer HS512 JWT from auth (roles OWNER/MENTOR/MEMBER)
curl -s -X PUT http://localhost:8080/api/users/507f1f77bcf86cd799439011 \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"bio":"Rustacean since 2024"}'
```

## Docs index

- `api.md` — full endpoint reference with request/response JSON and errors
- `examples.sh` — executable smoke test (golden request→response pairs)
- `openapi.json` — machine-readable OpenAPI 3.0 spec
- `changelog.md` — version history + breaking changes
- `gotchas.md` — production-learned constraints and operational gotchas

## For AI agents

This LXS is distributed as a **binary only** — these docs are the entire
interface. Match `api.md` shapes exactly; run `examples.sh` against a pulled
binary or live estate URL before trusting behavior. See
`docs/gotchas.md` for constraints that are invisible in the binary (notably:
reads fail closed when `auth` is down, and sub-entity writes are role-gated
but not self-ownership-checked).
