# Step 04 — The catalog application: the port and the version-gated cache

*(oracle: synapse step 04's application layer — `ContentRepository`, `ContentError`,
`CatalogService` with the ADR-S010 cache; `CatalogServiceSpec` ported as the spec)*

## The port (`content_repository.rs`)

`ContentRepository` — what the catalog needs from the world, three methods:
`content_version()` (the ADR-S010 watermark: infallible — degraded filesystems report a
constant, they never fail a request), `load_tree()` (the raw tree, metadata pre-decoded), and
`read_lesson(path)` (one file by content-root-relative path, re-read per request).

**Rust shape:** native async-fn-in-trait + a generic service (`CatalogService<R>`), NOT
`Arc<dyn ContentRepository>` — nothing varies at runtime (one production adapter), so static
dispatch is the honest choice and the `async_trait` macro stays banned (RS001). Tests inject an
instrumented stub type; `main` will inject the filesystem adapter in step 05.

`ContentError`: `NotFound` → 404 · `Io` → 500 · `IndexInvalid(SynapseContentError)` → 500
(mapped at `http/`, step 05).

## The service (`service.rs`)

- **The version-gated index cache**: `RwLock<Option<(version, Arc<WalkResult>)>>` — hit iff the
  cached version equals `repo.content_version()`; miss → `load_tree` → `walk` → cache. A
  concurrent double rebuild is harmless (idempotent), exactly like the oracle's plain `Ref`. The
  `Arc` makes a cache hit a pointer bump, not a deep clone.
- **`lesson(path)`**: slug-validate (the first traversal guard) → resolve through the cached
  catalog → look up the REAL file path in `lesson_files` → **re-read the body every request**
  (live edits show; pinned by an instrumented-counter test) → parse frontmatter → join the
  `.editorial.md` sidecar for `kind: problem` (absence is normal, other failures propagate) →
  prev/next from the pre-order reading sequence (empty at book ends, crossing chapter
  boundaries).
- **`component_doc(lesson_path, element_id)`**: validate the id (`[A-Za-z0-9_.-]+`, rejected
  before any read — pinned) → resolve the lesson → sidecar at `<lesson dir>/_c4-docs/<leaf>.md`
  where leaf = the FQN's last dotted segment (a container view's FQN and a sub-view's bare leaf
  resolve the same file — the oracle's step-72 gotcha, designed in).

## Tests

10 service tests over an instrumented in-memory repo (`AtomicUsize` load/read counters): the
cache rebuilds only when the version moves; bodies re-read per call while the index stays
cached; prev/next across chapter boundaries; editorial joins; bad paths (`../etc` included) →
`NotFound`; walker violations → `IndexInvalid`; component docs by leaf id, rejected-before-read
ids, absent sidecars. Suite: 50 unit tests total.

## Verified

`cargo test` 50/50; clippy `-D warnings` (let-chains for the cache guard); purity + caps + fmt
green.
