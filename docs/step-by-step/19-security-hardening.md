# Step 19 — Security hardening: baseline headers + the CSP for the RS reality

*(oracle: synapse step 36's headers half, with the step-38 prod CSP incidents folded in as
final design and turned into fixtures. The other step-36 half — canonical lowercase
usernames — landed with our identity step, where it belongs.)*

## The stamp (`platform/security_headers.rs`)

Five headers on EVERY response — API, `/c4` proxy, static SPA, and error paths alike (defence
in depth is not just the happy path): `X-Content-Type-Options: nosniff`,
`X-Frame-Options: SAMEORIGIN`, `Referrer-Policy: strict-origin-when-cross-origin`, the CSP,
and unconditional HSTS (`max-age=31536000; includeSubDomains` — Cloudflare terminates TLS,
but stating it at the origin keeps the guarantee if the edge is ever bypassed). The set is
precomputed ONCE from the issuer at wiring time and applied as the OUTERMOST layer in `app()`,
so every sub-tree — including the proxy's 502 degrade — comes back stamped.

## The CSP, written for what this app actually is

Parameterised by the OIDC issuer: `origin_of` derives `scheme://host[:port]` and ONLY that
origin joins `'self'` in `connect-src` (keycloak-js token calls) and `frame-src` (silent SSO).
An unparseable issuer fails OPEN with a warning — sign-in breaking loudly beats no policy.
Every other allowance exists for a named reason, most of them paid for in prod:

- `'wasm-unsafe-eval'` — load-bearing for the **Leptos app itself** (in the oracle it only
  carried d2's WASM; here the whole client is WASM).
- `'unsafe-eval'` — d2's blob render worker calls `new Function(elkJs)` at init, even under
  dagre; a `blob:` worker INHERITS the page CSP and no directive scopes eval to one worker
  (the step-38 incident, verbatim).
- `'unsafe-inline'` script + style — the index theme-bootstrap and runtime-injected styles;
  `worker-src 'self' blob:` — Monaco/d2/mermaid/tracer workers; Google Fonts css+files;
  `img-src 'self' data: https:` for prose images; Cloudflare Insights origins (the step-36
  incident: the first CSP was validated against the lightest page and silently broke fonts +
  Monaco in prod).
- `frame-ancestors 'self'` · `base-uri 'self'` · `object-src 'none'`.

## Tests

+7: three unit (origin parsing incl. ports, the allowance inventory, the fail-open policy
staying single-spaced) and four ITs asserting EXACT values through the real router per route
class — a 200, the Keycloak origin pair, the app-resource allowances, and the stamp on a 404,
the proxy's 502, and the static SPA index. Suite: 202 Rust + 40 vitest (client untouched).
Verified live: `curl -I` against the running dev stack shows all five headers with
`http://localhost:8181` correctly derived into `connect-src`/`frame-src`.

The standing lesson both incidents teach: **dev never reproduces CSP breakage** — Vite serves
the client without the origin's headers. Validate policy changes under prod-shaped serving
against the heaviest pages (Monaco + d2).

Next: least-privilege account admin — `DELETE /api/me` via the scoped `synapse-admin`
service-account client, the allowlist, and the `/account` page.
