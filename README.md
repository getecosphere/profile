# profile

User profile domain -- bio, experience, education, skills, certifications, social links

Split out of `rwid/lms` (the original pre-eco monolith) as an independent,
eco-managed domain — see `rwid/auth`'s
`backend/docs/auth-rewrote-from-java-to-rust.md` for the reasoning and
pattern this split follows (explicit dependencies instead of direct
cross-domain database access, security hardening baseline, etc.).

## Status

Scaffold only. Boots, connects to MongoDB, validates JWTs issued by `auth`,
has the estate's standard security baseline (rate limiting, security
headers, body size limits, CORS) wired up. The actual domain logic has not
been ported from `lms-backend` yet.

## Split from (lms-backend)

UserController/UserService (bio, experience, education, skills, certifications, social links, interests), School, Interest, Skill

## Depends on

auth

## Structure

- `backend/` — Rust (axum) service
