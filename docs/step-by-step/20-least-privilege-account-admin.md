# Step 20 — Least-privilege account admin: the allowlist, erase/delete, and /account

*(oracle: synapse step 21 fused with step 37's final design — the account surface is born on
the SCOPED service-account client; the master-realm admin it replaced never exists here.)*

## The submit allowlist (server)

Migration `0002`: `submission_allowlist(username pk, note, granted_at)` — keyed by the IdP
USERNAME (human-grantable live; `submissions.user_id` still stores the opaque `sub`), stored
lowercase to match the verifier's canonicalisation. The gate is an application precondition
(`SubmitSolution::authorize`, FIRST in `submit` — rejects never touch the suite or the store)
behind `SUBMISSION_ALLOWLIST_ENFORCED` (default false: dev/personal instances stay open; prod
flips it on). Enforced: anonymous → 401 "Sign in to submit"; unlisted → 403 "Submitting is
allow-listed on this deployment — '<user>' isn't on the allowlist yet…" with the operator
hint. The port carries `is_allowed` today; list/grant/revoke join with the admin panel.

## Account deletion via the scoped client (server)

`DELETE /api/me` does exactly one thing: `KeycloakAdmin::delete_user(sub)`. The adapter
authenticates as **`synapse-admin`** — a confidential service-account client in OUR OWN realm
(`client_credentials`, `realm-management:manage-users` only). The audit finding this design
answers: the oracle's first cut used the master-realm bootstrap admin, whose leaked env would
have been a full IdP takeover; the scoped client's blast radius is "delete users in one
realm". Two hops per delete — token, then `DELETE /admin/realms/{realm}/users/{sub}` (204 =
deleted, **404 = already gone, both success**); every other outcome is 503, never a swallowed
success. base+realm split from the issuer; a malformed issuer degrades to `master` and fails
loudly at call time. Config: `KEYCLOAK_ADMIN_CLIENT_ID`/`_SECRET` (dev realm file seeds
`synapse-admin`/`dev-admin-secret` with exactly that role).

## The /account page (client)

The account grammar the admin panel will reuse: the identity card (avatar initial, @handle,
email, "Signed in with Keycloak"), then the danger zone — three permanent actions behind a
styled confirm modal, reporting through one inline status banner (`ActionStatus`:
Busy/Ok/Error). **Erase my submissions** · **Erase all my data** (also clears this browser's
reading preferences, reloads) · **Delete my account** — orchestrated erase → delete →
sign-out ON THE CLIENT, so the server's identity context never depends on submissions. The
chip menu's "Manage account & data" routes here; the workbench Submit gains the auth gate
(disabled while anonymous with the full why-copy; the ⇧⌘⏎ path checks the same signal).

## Tests + verified live

+8: the gate pair (off → anyone saves; on → 401/403/ok with nothing written on reject), the
issuer split, config default pins, three `DELETE /api/me` ITs against a stub that ASSERTS the
`client_credentials` grant + client id + bearer (401 anonymous · 200 through the scoped flow ·
admin-down → 503), and the gated Postgres seed check. Suite: 210 Rust + 40 vitest; 401/700
KiB gz. Verified live: enforcement ON → curl anonymous 401 / unlisted `test1` 403 (exact
copy) / listed `tester` 202; **the real Keycloak user `test1` deleted through the real
`synapse-admin` client** (fresh direct grant → `invalid_grant`, then recreated via the same
scoped client); in-browser — anonymous Submit disabled with the hint, tester's /account
rendered, confirm-modal Erase → "✓ Deleted 2 submission(s)".

Next: the admin allowlist panel — `/admin` over admin-gated `/api/admin/allowlist`
(list · grant · revoke), `ADMIN_USERS`, `MeDto.admin`.
