# Step 45 — Production readiness

*(an eight-item audit of the Rust workspace, each item shipped and verified separately; the
sandbox release unblocked; and a footer that names the build you are looking at.)*

Steps 36–44 made the product complete and the release automatic. This step asks a different
question: **is this thing operable?** An audit of the workspace — deliberately hunting for
places where the Scala mirroring produced non-idiomatic Rust, and where documented intent had
drifted from actual code — returned less than expected on style and more than expected on
operations.

Worth stating plainly, because it shaped the work: most of what an audit usually finds was
already right. Zero `Box<dyn Error>` in error positions, zero `anyhow` outside `main.rs`, eight
per-context `thiserror` enums, zero `async_trait` (native AFIT throughout), ports as
monomorphised generics rather than `Arc<dyn>`, every `std::fs` call inside `spawn_blocking`, and
not one `cfg(target_arch)` branch in a workspace that targets both native and wasm. The hexagon,
the ports and the DTO layer were re-derived into Rust, not transliterated.

The real findings were narrower and sharper.

## The two that were claims rather than facts

RS001 promised things the code did not do. That is worse than an absent feature, because a
written guarantee stops people looking.

**"`tracing` spans route→service→adapter (ADR-S009 parity)."** The binary had 48 flat events,
zero spans and no `TraceLayer`. A 500 in production could not be tied back to the request that
caused it, because nothing carried an identity across the hops.

`platform/telemetry.rs` now wraps the router in `SetRequestId → TraceLayer → PropagateRequestId`,
emitting one `http` span per request with method, matched path, request id, status and latency.
It sits **outside even compression**: a span that starts inside the header layers cannot report
on them, and a request rejected at the edge is exactly the one worth tracing.

The layer order is load-bearing and easy to get backwards, because `.layer()` applies bottom-up
— the listing reads in reverse of execution. `SetRequestId` must run *before* `TraceLayer`, or
the span is built from a request with no id yet and `request_id` is empty on every span. That
looks fine until the day you try to correlate a failure.

Two `#[instrument]` attributes encode a security decision rather than a formatting one:
`identity.authenticate` is `skip_all`, because `token` is a live bearer credential and
`#[instrument]`'s default is to record **every argument**; the username is recorded only after
verification succeeds. `execution.run` and `submission.submit` skip the source and keep
`source_bytes` — the code is the user's and large, the byte count is the operationally useful
part.

`telemetry_it.rs` holds the claim to actual output by capturing a real subscriber rather than
asserting against a mock, because what matters is what an operator reading logs sees.

**"Integration tests from step 01 … testcontainers Postgres."** `postgres_it.rs` — 252 lines
proving the SQL against a real database — was env-gated and therefore ran nowhere. That matters
more here than it would elsewhere: sqlx is used through runtime `query()` rather than the
compile-time-checked `query!` macros, so with the IT skipped, *nothing* validated the SQL. A
schema-breaking change would have reached production green.

`build-test` now carries a `services: postgres` block. And because a skipped Rust test reports
green — the standard workaround for having no native skip, and precisely why this gap survived
so long — a second step asserts the suite actually **ran**.

## The bug the fix uncovered

Wiring the Postgres IT into CI immediately exposed a defect it had been hiding. Every
`gated_pool()` call deleted *all* `it-rs%` rows, so under default parallelism the tests wiped
each other's fixtures: `listing_is_newest_first` saw 2 rows where it had inserted 3.

It had never surfaced because the suite only ever ran by hand, usually with `--test-threads=1`
as its own header suggested. Shared mutable state under parallelism is a flake waiting for a
faster machine, and CI is a faster machine.

Each test now owns a namespace (`it-rs-listing`, `it-rs-flow`, …) and cleans only that, so the
suite is parallel-safe by construction rather than by convention. Six consecutive parallel runs
green. The first attempt used `lesson_path[1] = $1` and failed on the real database:
`lesson_path` is TEXT, not an array — the repository stores `path.join("/")`.

## CI was not testing the binary it ships

Cargo unifies features across whatever is built together. `cargo test --workspace` builds the
client, which declares `serde_json/preserve_order` for its viz decoder;
`cargo build --release -p synapse-server` — what the Dockerfile runs — builds the server alone
and got no such feature:

```
CI      serde_json  default,indexmap,preserve_order,raw_value,std
Docker  serde_json  default,raw_value,std
```

`Value` was backed by `IndexMap` in CI and `BTreeMap` in production.

Nothing was broken by it, and that was checked rather than assumed: every server-side `Value`
site is order-insensitive — the Ollama and go-judge bodies, `outcome jsonb` (Postgres normalises
key order), and `contract_it` compares through a `BTreeSet`. But it is the worst *shape* of
latent bug. The next order-sensitive line would have been green in CI and wrong in production,
with nothing in the failure pointing at the build configuration.

The server now declares the feature too, so every invocation produces an identical `serde_json`.
`build_config_it.rs` asserts the *behaviour* the feature provides, so deleting the line fails a
test with a message naming the fix.

The alternative — adding `cargo test -p synapse-server` to CI — tests both shapes but pays a
second compile on every push and leaves two shapes to reason about. Making them the same shape
is cheaper and removes the question.

## "Shared" described the folder, not the code

`shared/` was 4,670 lines, of which `viz/` was **4,037 — 86%** — and the server referenced it
**zero** times. It sat there because the structure was mirrored from Synapse's Scala `shared/`
crossproject.

The usual justification does not hold, and it was checked rather than assumed: the reason to
keep a wasm-targeted engine in a dual-target crate is native testability for the cortex goldens,
but the client is already an `rlib` with zero `wasm_bindgen_test`, compiled and tested natively
by the same `cargo test --workspace` that runs everything else. The goldens lose nothing by
moving.

```
shared/src/viz/        → client/src/viz/engine/     (33 files)
shared/tests/*.rs      → client/tests/              (goldens + adapt stages)
```

The move was mechanically clean because the coupling already was: `viz` had no `use crate::`
outside itself, and nothing else in `shared` referenced it beyond the `pub mod` line.

What it bought, measured rather than asserted:

```
shared rebuild   1.70s → 0.88s
server rebuild  12.04s → 6.36s     (47% faster on any shared change)
```

`shared`'s dependency list is now exactly `serde`. `thiserror` went with `viz` — it had been used
at a single site, all of it inside the engine. What remains is a genuine kernel: ~630 lines of
wire contract both sides actually use.

## The supply chain had no gate at all

407 crates reach production and, since step 44, no human is in the loop. `cargo test` proves the
code we wrote; nothing proved the code we did not.

`cargo deny check` now gates the release, and every one of its four checks earns its
configuration — the gate was run first, all seven failures read, and each decided on its merits
rather than reaching for `ignore`:

- **advisories** — vulnerabilities deny for the whole graph, always. `unmaintained` is scoped to
  **workspace** deps. The two hits (`proc-macro-error2`, `paste`) are build-time proc-macro
  crates pulled in by `leptos_macro`, both carrying "no safe upgrade is available", neither
  shipping in the binary or the bundle. Denying them means a permanently red gate we cannot
  clear by any action of our own, and **a gate that is always red is a gate nobody reads**.
  Scoped to workspace it still fails the day *we* depend on something abandoned.
- **licenses** — `private.ignore`, because our own crates are `publish = false` and correctly
  carry no licence field. `CDLA-Permissive-2.0` allowed for `webpki-roots`: a data licence on
  the Mozilla CA bundle rustls ships, which is the crate that fixed the production sign-in 503.
  **MPL-2.0 is deliberately not allowed** even though adding it costs nothing — nothing uses it,
  and pre-allowing weak copyleft is permission granted for a decision no one has made.
- **bans** — `openssl` denied outright: we forbid unsafe and drive TLS through rustls with
  bundled roots. `allow-wildcard-paths`, because `{ path = "../shared" }` reads as a wildcard to
  cargo-deny and is simply how a workspace spells an internal dependency.
- **sources** — crates.io only.

Dependabot covers cargo, npm and github-actions. Actions matter most: they run with
`packages: write` in the release job. It opened ten PRs within minutes, and `jsonwebtoken 9→10`
failed `build-test` immediately — the system working on its first day.

## The sandbox could not be rebuilt

`go-judge-build-push-promote.yml` had run exactly once and failed:

```
denied: permission_denied: write_package → ghcr.io/ani2fun/synapse-go-judge
```

Not the `packages: write` escalation that broke the app release — that block was always correct,
and the app image pushed from the same repo with the same token throughout. The difference was
**package-level**: `synapse-rs` was created by this repository so this repository could write it,
while `synapse-go-judge` was created by the Scala repo and its GHCR access list did not follow
the move in `aaf8e06`. The production code-execution sandbox was unbuildable — precisely the
scenario moving it here was meant to prevent.

One initial overstatement is worth recording: this was also reported as "there is no image for CI
to pull", which was wrong. `:latest` existed and was **public** — production was running the
Scala repo's 2026-07-13 build the whole time. Nothing was down; new builds simply could not ship.

Granting the repository access on the package fixed it. Run `29678821643` pushed a new image
(digest `30857ba7…`, replacing `f4196f8e…`) and promoted it. With a real image published, the
`GOJUDGE_IT` suites are wired in too: `docker run --privileged` in a step, because go-judge
sandboxes with namespaces and cgroups and Actions **service containers cannot be privileged**.
It runs on every push rather than behind a paths filter, and costs no wall-clock — it starts
alongside `build-test`, which takes minutes on its own, so a 2 GB pull and four tests finish
well inside that window.

That closes the last of the stub-only coverage: the adapter that turns our wire format into
go-judge calls — the recipes, the Java rewriter, ns→s and bytes→KiB, "Accepted with a nonzero
exit is a RuntimeError" — is now checked against the real sandbox rather than a stub that agrees
with whatever we believed when we wrote it.

## The edge, the profile, the toolchain

Three small items in the same family: things decided for the wasm bundle but never for the binary
serving production.

**Edge limits.** Axum applies no request timeout, so a hung handler held a connection
indefinitely. The number is the interesting part: it must **exceed** the go-judge runner's 100 s,
which is itself deliberate — the runner waits that long so the sandbox's own clock fires first
and the user gets a clean TLE instead of an opaque HTTP timeout. A shorter edge timeout would
kill legitimate slow runs at the door, and would look like a flaky sandbox rather than a
misconfigured router. The runner's constant is re-exported and a test locks the relationship, so
neither number can be edited alone. 504 rather than tower-http's historical 408: anything
reaching a two-minute bound has almost certainly been waiting on an upstream, and 408 would claim
the client was slow to send.

**`[profile.release]`.** Step 35 hand-tuned the wasm profile while the thing actually serving
production shipped on cargo defaults — settings chosen for compile speed, the wrong trade for an
artifact built once and run for weeks. thin LTO, not fat: most of the cross-crate inlining at a
fraction of the link time, and the server is not size-constrained the way a 700 KiB budget is.
Debug symbols **kept** — a production backtrace beats the megabytes. Image 190 MB → 182 MB.

**Toolchain pinned to 1.97.0.** CI runs clippy pedantic with `-D warnings`, so a floating
"stable" meant a new release every six weeks could turn CI red on code nobody touched. The change
paid for itself immediately: pinning installed the exact toolchain and clippy promptly failed on
two lints in the code written minutes earlier. Pinning does not hide new lints; it makes their
arrival a commit instead of a surprise. It also closes a CI≠prod gap of the same family as the
`serde_json` one — the Dockerfile's `rust:1-bookworm` now resolves to the same compiler CI uses.

## The footer

Last, the visible half of all this: the landing page gained a footer, and on the right, the
commit the reader is running.

```
© 2026 Aniket Kakde · Synapse — read it, run it, understand it.
Built with Rust, Leptos and WebAssembly. Source on GitHub    ⟨ VERSION 94195ba ⟩
```

The version is a **compile-time constant baked into the wasm**, not a value fetched from the
server, and the distinction matters: the footer should describe the bundle in the reader's
browser, not the server answering it. Those can diverge — `index.html` is `no-cache` but assets
are immutable and hashed, so a browser holding a cached bundle across a deploy would report a
version it is not running. A constant cannot lie about itself.

`client/build.rs` resolves `SYNAPSE_VERSION` (the release build-arg, `github.sha`) → `git
rev-parse HEAD` → `"dev"`. `rerun-if-env-changed` is load-bearing: Cargo does not track `env!()`
reads, so without it a rebuild after a new commit reuses the previously baked string and the
footer confidently shows the wrong SHA. The build-arg is required rather than a nicety —
`.dockerignore` excludes `.git`, so the fallback cannot reach the image.

Seven characters, mono, in the same register the site already uses for metadata — and it links to
the commit, which is the difference between a string you squint at and one you can act on.

## Verified live

Post-deploy, against production: `x-request-id` generated and a supplied one preserved; a 2 MB
body rejected 413 at the edge; a junk bearer answered 401 (JWKS reachable — the discriminator
that would have caught the TLS bug); python and java both `Accepted` through the real sandbox;
anonymous submit 401 with the allowlist copy; CSP intact on the heaviest page with zero console
errors; and the footer's version matching the promoted image tag exactly.

386 rust + 74 vitest. Critical path 636/700 KiB gz.

The rust count is unchanged by the IT wiring, and that is the point worth noticing: the four
sandbox tests and the five Postgres tests were *always* in the total. They passed by returning
early. What changed is not how many tests exist but how many of them assert anything — which is
exactly why a count is a poor gate and the explicit "prove it RAN" steps are not.

## The lesson

**A documented guarantee is where nobody looks.** Two of the three highest findings were not
missing features but written claims — spans and testcontainers — that had drifted from the code.
An absent feature advertises itself; a claim in an ADR actively stops the next person checking.
The audit's most valuable habit was reading every "we do X" as a question rather than a
statement, and the two that turned out false had been false for many steps.
