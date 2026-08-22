# Profile LXS integration guide

`profile` owns person-facing data: bio, avatar URL, headline, and other
profile fields. `auth` owns credentials and identity. Do not merge those
responsibilities in an estate core.

## Compose safely

Use `eco lxs add profile@<version>` and, when users can upload avatars,
compose `storage` too. The profile service requires the estate-wide JWT and
Mongo database; Eco/configgen supplies them through the LXS contract.

Expose one stable browser prefix and rewrite it to Profile's `/api` root:

```yaml
profile-backend:
  lxs: profile@<version>
  access:
    routes:
      - { path: /api/profile/*, level: auth, strip: /api/profile, rewrite: /api }
```

`profile-ui` uses `/api/profile` by default, so an update becomes
`PUT /api/profile/users/<subject-id>`. Do not expose `/api/users/*` in one
estate and `/api/profile/*` in another unless a legacy consumer forces it.

## Ownership and authorization

Auth token `sub` must match the profile being edited. Profile forwards a name
change to Auth, stores bio locally, and uploads avatar bytes to Storage. A core
may render the avatar menu, but it links to the independently composed
`profile-ui` settings page. Never let a browser choose another user id.

## Release discipline

Update this guide and `docs/` with any API, ownership, or route-prefix change.
Run checks, build/publish the LXS, and push both source and registry releases.
