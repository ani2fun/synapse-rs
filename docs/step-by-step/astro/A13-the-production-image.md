# A13 — The production image: two processes, one env-flip rollback

The image becomes the topology the whole branch has been rehearsing: axum fronting the Node
sidecar — and the rollback that every step promised ("unset one env var") is now a property of
the IMAGE, not of the git history.

## The builder

Three artifacts now build in one stage, and the ORDER is load-bearing twice over: the Astro app
builds FIRST because the old client's Vite build single-sources its islands from `web/src` and
resolving `web/tsconfig.json`'s `extends: astro/tsconfigs/strict` needs `web/node_modules` to
exist (the first build died exactly there — the stale-Dockerfile debt A12's chapter deferred to
this step, paid). And the monaco/mermaid/@terrastruct prune runs LAST, after the old-client
build, because that same single-sourcing resolves those bare specifiers against
`web/node_modules` at build time (found by breaking it). The old client's build and its 700 KiB
gate stay — the rollback artifact earns its keep at 672/700.

## Self-containment, answered empirically

`web/dist/server/entry.mjs` is NOT self-contained: the standalone adapter externalizes
framework deps (preact, shiki, the remark ecosystem, …) as bare specifiers — probed by running
the dist from a bare temp dir (`ERR_MODULE_NOT_FOUND: preact`), not read off a docs page. The
runtime therefore carries a prod `node_modules`, produced by `npm prune --omit=dev` in the
builder (the lockfile install stays the single source of truth; the runtime stage stays
offline). Client-only packages that exist solely as inputs to `dist/client`'s hashed chunks —
monaco, mermaid, d2, ~238 MB — are removed after their last build-time use; the SSR bundle's
externalized imports were enumerated and name none of them, and the lesson-page boot check is
the tripwire if a future Astro starts externalizing one. Image: 185 MB → 953 MB (Node 22 + the
SSR dependency tree — the price of server rendering; down from 1.44 GB before the prune).

## start.sh

Empty/unset `SYNAPSE_ASTRO_URL` → `unset` it, then `exec /app/synapse-server`: ONE process,
the old client from STATIC_ROOT, byte-identical to the pre-A13 image. The `unset` is
load-bearing: figment reads an empty env string as `Some("")` and enables the proxy at a dead
upstream — the first rollback test 502'd every page (the same trap `dev-tools/e2e`'s legacy
branch already dodges). Otherwise: export `SYNAPSE_API_URL` at the server's own port (the
sidecar's SSR fetches), start both processes, `wait -n` — either death kills the container,
because half-alive is the worst state an orchestrator can be handed. Bash, not dash: `wait -n`
is a bashism. Node runs fine as uid 65532 with no HOME (tested, not guessed).

## Verified (the step's whole value)

Astro boot: health 200 · `/` serves `_astro/` assets (zero `assets/`, zero wasm) · a real
lesson SSR'd (`<h1>` + 7 prose `<p>` — the shiki/remark path, which is what the prune must not
break) · CSP present · gzip negotiated · silent-check-sso 200. Kill test BOTH directions:
pkill the sidecar → container exits ~1s; pkill the server → same. Env-flip rollback: old
client serves, API healthy, zero node processes. All re-verified after the prune; 3/3 pruned
packages confirmed absent.

## Numbers

Image 953 MB (was 1.44 GB pre-prune; 185 MB pre-A13) · prod node_modules 321 MB · old client
672/700 KiB gz · viz wasm 288 KiB gz · kill-to-exit ~1s both ways.

## The lesson

**A rollback is an artifact plus a rehearsal, and this step needed both to be honest.** The
old client in the image is inert weight until the env flip actually serves it — and the flip's
first rehearsal failed on an empty-string-vs-unset distinction no design review would have
caught. The same discipline caught the other two traps (the tsconfig ordering, the prune
ordering): every one surfaced by running the thing, none by reading it.
