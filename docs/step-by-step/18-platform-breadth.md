# Step 18 — Platform breadth: blog, ⌘K search, rate limiting, SPA + /c4 proxy

*(oracle: synapse step 19 whole — the blog bounded context, `LibrarySearch`/`SearchPalette`,
the identity-aware `RateLimiter` gate on run/submit, and `StaticRoutes` + `LikeC4Proxy`.)*

## The blog context (`server/src/blog/`)

The catalog's flat, chronological cousin, on the same hexagon walk: a domain (`BlogPost::parse`
with a lenient frontmatter fence — a deliberate TWIN of the catalog's, not an import; bounded
contexts own their vocabulary), an application (`BlogService` with the version-gated listing
cache; post bodies re-read per call so live edits show; `prev` = older / `next` = newer from
publish order, undated posts sinking last via `sort_by_key(Reverse(date))`), a filesystem
adapter (`<contentRoot>/blog/<slug>.md`, non-recursive, `_`-prefixed drafts never ship,
traversal-guarded reads, the mtime:count watermark), and the http pair `GET /api/blog` +
`/api/blog/{slug}` (`publishedAt` crosses as an ISO string, empty when undated). `/api/blog`
joins `/api/synapse` under the `ContentCacheControl` stamp — the blog IT pins the exact header.

## ⌘K search (`client/src/search/`)

Entirely client-side over the already-cached index + blog listing — no server round-trip.
`logic/` is pure and natively tested: flatten (every lesson with its `Foundations › DSA ›
Arrays` breadcrumb, every book linked to its first lesson, every post), then rank — prefix 100
> word-start 80 > substring 60 > subsequence 30, +10 for matching the label over the crumb,
lessons > books > posts as the tiebreak. The palette is a singleton mounted in the shell:
`SearchStore` (open/query/selection) lives in app context so the header button and the global
⌘K/Ctrl-K listener drive the same modal; arrows + Enter navigate (`use_navigate`), Escape and
the scrim close, and the library loads lazily on FIRST open.

## The budget gate (`platform/rate_limiter.rs` + the run/submit routes)

An in-memory fixed window, floor-aligned to the epoch, with two ledgers: anonymous meters per
IP (`X-Forwarded-For` first hop → `X-Real-IP` → socket peer via the infallible `Peer`
extractor → `"unknown"`), signed-in per subject — a per-person key survives NAT and earns the
bigger budget (10/60 s vs 100/3600 s, `RATE_LIMIT_*` envs). The gate runs FIRST on
`POST /api/run` and `POST /api/submissions`; a bad bearer is still 401, never silently
anonymous. Over the window → 429 with the retry seconds in the `ApiError` body — no
`Retry-After` header (the oracle's deliberate fork: one uniform envelope). Expired entries
prune opportunistically above 4096 keys; the core takes `now` explicitly, so the unit suite
drives window rollover without a clock. Redis stays deferred — the type is the port.

## The SPA + `/c4` proxy (`platform/static_routes.rs` + `likec4_proxy.rs`)

The fallback ENUMERATES the reserved first segments (`synapse`, `blog`, `account`, `admin` —
kept in step with `Page`) instead of a catch-all: a trailing wildcard is greedy enough to
shadow `/api` (the Cortex-inherited lesson, re-proven here by the IT asserting `/api/nope`
404s while `/blog/x` serves the index). Index + silent-check-sso are `no-cache` (deploys must
show); hashed `/assets/*` are `immutable` for a year; every read is traversal-guarded. An
ABSENT dist mounts nothing — dev keeps Vite and the API-only plain-text root. `LikeC4Proxy`
forwards `GET /c4/*` with the prefix STRIPPED (prod: `LIKEC4_URL=http://synapse-likec4/c4` —
the image serves under `/c4` and the two cancel), copies only `content-type` back, and
degrades to 502 — proven against a real local stub upstream, not a mock.

## Tests + verified live

+36: blog domain 4 · service 4 (fake counting repo proves the cache) · filesystem 4 (real temp
dirs, traversal, watermark) · route ITs 3 · rate limiter 4 (driven clock) + client-ip 2 +
gate ITs 2 (429 envelope + both hints through the real router) · static/proxy ITs 7 · search
logic 5 (native) · page-map growth. Suite: 195 Rust + 40 vitest; critical path 380/700 KiB gz.
Live against the real synapse-content: `/api/blog` lists both real posts newest-first;
`/blog` → card → post renders through the markdown island (leading `<h1>` stripped, Older
pager); ⌘K → "flip" ranks Flip Characters first → Enter lands on the lesson and closes.

Next: security hardening as its own step — baseline headers + the CSP written for the RS
reality (`'wasm-unsafe-eval'` + d2's worker) — then least-privilege account admin.
