# Step 21 — The admin allowlist panel: /admin over admin-gated management verbs

*(oracle: synapse step 35 — `AllowlistAdminRoutes` + `AdminPage` + `ADMIN_USERS` +
`MeDto.admin`.)*

## The management surface (server)

The allowlist port grows its three management verbs — `list` (newest first), `grant` (upsert:
re-granting refreshes the note, `returning` hands back the stored row) and `revoke` (`false`
when nothing matched) — implemented on the same Postgres adapter the submit gate rides.
`/api/admin/allowlist` (GET · POST · DELETE `{username}`) is gated PER CALL: a verified
bearer whose username sits in `ADMIN_USERS` (comma-split, trimmed, LOWERCASE — compared
against the verifier's canonical output, apples to apples; dev default `tester`, prod
`ani2fun`). Anonymous → 401; a perfectly valid stranger → 403 "Admin only" — **the flag is
config, not a token claim**. Grants canonicalise the username server-side (trim + lowercase;
blank → 400), so what the panel stores is exactly what the gate compares. `MeDto.admin` turns
real (`admin_users.contains(username)`) but stays UX-only — the routes re-check every call.
The routes are generic over the allowlist port, so the route ITs drive a FAKE through the
real router while the gated Postgres IT proves the SQL.

## The panel (client)

`/admin` on the account grammar: the grant form (username + optional note), the grants table
(username · note · date · Revoke), one status banner. `MeDto.admin` gates only what renders —
anonymous sees sign-in, a non-admin sees "Admin only", and a non-admin who calls anyway just
sees the API's 403 in the banner. The chip menu grows "Admin panel" for admins (UX only,
same caveat). `Page::Admin` joins the app-map; the SPA fallback already reserved the segment.

## Tests + verified live

+6: the config canonicalisation pin, four admin route ITs over the fake + JWKS stub
(401/403 "Admin only" · the admin lists · grant upserts "  MixedCase  " → `mixedcase`,
blank → 400 · revoke 204/404), and the gated Postgres round-trip (grant → re-grant refreshes
the note → list → revoke → second revoke finds nothing). The step-17 `/api/me` IT updated:
`admin: true` for tester — pinned WITH the comment that it only works because the MiXeD-case
token was canonicalised first. Suite: 216 Rust + 40 vitest; 421/700 KiB gz. Verified live:
`/admin` as tester listed the real seeded grants; granting "  MixedCase  " stored + rendered
`mixedcase` (banner "✓ Granted"); revoke removed it (banner "✓ Revoked"); the chip menu shows
"Admin panel"; curl as test1 → 403 "Admin only", `/api/me` flags test1 `false` / tester
`true`.

Next: the tutoring coach (oracle step 20) — the Ollama adapter + chat UI, off state
"coming soon" — then RS-P7, the viz engine against the cortex-goldens.
