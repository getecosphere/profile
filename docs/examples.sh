#!/usr/bin/env bash
# profile LXS smoke test — golden request→response pairs.
# Usage: BASE_URL=<http://host:port> ./examples.sh
# Runs against a pulled binary or a live estate URL; every curl must succeed
# and return the documented shape or the script exits non-zero.
# Public endpoints are always verified. Authenticated/user-id checks run only
# when JWT_SECRET (+ optional USER_ID) are provided, since hydration reads
# need a live auth service.
set -euo pipefail

BASE_URL="${BASE_URL:-http://127.0.0.1:8080}"
JWT_SECRET="${JWT_SECRET:-}"
USER_ID="${USER_ID:-507f1f77bcf86cd799439011}"

# Mint an HS512 JWT like auth does (claims sub/username/role/iat/exp).
mint_token() {
  local secret="$1" sub="$2" role="$3"
  local now exp hdr payload sig signing_input
  now=$(date +%s)
  exp=$((now + 3600))
  hdr=$(printf '%s' '{"alg":"HS512","typ":"JWT"}' | openssl base64 -A | tr '+/' '-_' | tr -d '=')
  payload=$(printf '{"sub":"%s","username":"u_%s","role":"%s","iat":%s,"exp":%s}' "$sub" "$sub" "$role" "$now" "$exp" | openssl base64 -A | tr '+/' '-_' | tr -d '=')
  signing_input="${hdr}.${payload}"
  sig=$(printf '%s' "$signing_input" | openssl dgst -sha512 -hmac "$secret" -binary | openssl base64 -A | tr '+/' '-_' | tr -d '=')
  echo "${signing_input}.${sig}"
}

# 1) health
code=$(curl -s -o /tmp/profile-health.out -w '%{http_code}' "$BASE_URL/api/health")
test "$code" = "200"
grep -q '"UP"' /tmp/profile-health.out
echo "OK /api/health -> 200"

# 2) public read endpoints
code=$(curl -s -o /tmp/profile-users.out -w '%{http_code}' "$BASE_URL/api/users")
test "$code" = "200"
echo "OK GET /api/users -> 200"

code=$(curl -s -o /tmp/profile-search.out -w '%{http_code}' \
  "$BASE_URL/api/users/search?platformId=no-such-platform&query=zzz")
test "$code" = "200"
echo "OK GET /api/users/search -> 200"

code=$(curl -s -o /tmp/profile-platform.out -w '%{http_code}' \
  "$BASE_URL/api/users/platform/no-such-platform")
test "$code" = "200"
echo "OK GET /api/users/platform/{id} -> 200"

# 3) taxonomy (public, idempotent)
code=$(curl -s -o /tmp/profile-schools.out -w '%{http_code}' "$BASE_URL/api/schools")
test "$code" = "200"
echo "OK GET /api/schools -> 200"

name="smoke-$(date +%s)"
code=$(curl -s -o /tmp/profile-addschool.out -w '%{http_code}' \
  -X POST "$BASE_URL/api/schools?name=$name")
test "$code" = "201"
echo "OK POST /api/schools -> 201"

code=$(curl -s -o /tmp/profile-interests.out -w '%{http_code}' "$BASE_URL/api/interests")
test "$code" = "200"
code=$(curl -s -o /tmp/profile-skills.out -w '%{http_code}' "$BASE_URL/api/skills")
test "$code" = "200"
echo "OK GET /api/interests + /api/skills -> 200"

# 4) writes require auth -> 401 without a token
code=$(curl -s -o /tmp/profile-unauth.out -w '%{http_code}' -X PUT \
  "$BASE_URL/api/users/$USER_ID" -H "Content-Type: application/json" -d '{"bio":"hacked"}')
test "$code" = "401"
echo "OK unauthenticated PUT -> 401"

# 5) authenticated profile edit (needs live auth for hydration)
if [[ -n "$JWT_SECRET" ]]; then
  TOKEN=$(mint_token "$JWT_SECRET" "$USER_ID" "MEMBER")
  code=$(curl -s -o /tmp/profile-put.out -w '%{http_code}' -X PUT \
    "$BASE_URL/api/users/$USER_ID" \
    -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
    -d '{"bio":"Rustacean since 2024"}')
  test "$code" = "200"
  echo "OK authenticated PUT -> 200"

  code=$(curl -s -o /tmp/profile-get.out -w '%{http_code}' "$BASE_URL/api/users/$USER_ID")
  test "$code" = "200"
  grep -q '"Rustacean since 2024"' /tmp/profile-get.out
  echo "OK GET /api/users/{id} -> 200 (fresh from auth)"

  # role not permitted -> 403
  BADTOKEN=$(mint_token "$JWT_SECRET" "$USER_ID" "GUEST")
  code=$(curl -s -o /tmp/profile-forbidden.out -w '%{http_code}' -X PUT \
    "$BASE_URL/api/users/$USER_ID" \
    -H "Authorization: Bearer $BADTOKEN" -H "Content-Type: application/json" \
    -d '{"bio":"x"}')
  test "$code" = "403"
  echo "OK role-denied PUT -> 403"
else
  echo "SKIP authenticated checks (set JWT_SECRET and a live auth service)"
fi

echo "ALL OK"
