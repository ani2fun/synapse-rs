# Step 10 — The go-judge adapter: the run path made real

*(oracle: synapse step 10 — `GoJudgeWire`/`GoJudgeRecipe`/`JavaSourceRewriter`/`GoJudgeRunner`,
`ExecutionRoutes`, `executorUrl` config; `GoJudgeWireSpec` + `JavaSourceRewriterSpec` +
`ExecutionRoutesSpec` + the gated `GoJudgeRunnerIT` ported)*

## The wire (`infrastructure/wire.rs`, golden-tested)

go-judge is a raw command runner — one `POST /run` cmd runs compile+run in a single
`/bin/sh -c`. The load-bearing tricks, all ported exactly: **compile failures are detected via
the `__cf_crc`/`__cf_cerr` marker files** (the compile step exits 0 without running, go-judge
says "Accepted", the parser reads the rc file) — never via HTTP status; `time` is nanoseconds
and `memory` is bytes (mapped to `timeSeconds`/`memoryKb`, absent → `None` not 0);
`"Accepted"` with a nonzero exit is a `RuntimeError`; output streams are raw, capped at the
shared 1 MiB.

## Recipes and the Java rewriter

`Recipe::for_language` — the eleven-language table verbatim (sources, compile/run commands,
the Scala/Kotlin cpu/clock/memory overrides) behind an **exhaustive match**: a new `Language`
without a recipe is a compile error. `java_rewriter::effective_source` renames the first
top-level class (and its word-boundary self-references) to `Main` unless an authored `Main`
exists or the source opens with the tracer sentinel — `SolutionHelper` stays untouched, nested
classes never match (anchored multiline regex).

## The runner (`infrastructure/runner.rs`)

reqwest **forced to HTTP/1.1** (go-judge rejects h2c-upgrade headers as bare 400s), connect
timeout 10 s, request timeout **100 s** (go-judge's own clock limit must fire first — a clean
TLE beats an opaque HTTP timeout), a **semaphore bounding 8 concurrent runs** (rate limits cap
rate; this caps fan-out), and the two-way degradation: connection-level failures →
`BackendUnavailable` (503, with the operator hint), everything else → `BackendFailed` (502).

## The endpoint + config

`POST /api/run` → 200 `RunResult` (bad programs included) · 422 unknown language · 413 over
caps · 503/502 backend, all in the `ApiError` envelope. `executor_url` joins `AppConfig`
(default `http://localhost:5150`); the oracle's bare **`EXECUTOR_URL`** deploy-manifest name is
honored verbatim alongside `SYNAPSE_EXECUTOR_URL`. The identity/rate-limit gate grafts on in
its own step, as in the oracle (step 19).

## Tests

24 new: 9 wire goldens (request shapes incl. the marker-file shell, unit conversions, every
status mapping, measurements-absent, malformed JSON) · 6 rewriter · 5 service (step 09's) held ·
**6 route ITs against a LOCAL go-judge stub** — a real axum server speaking the protocol, so
the whole router → service → runner → wire path runs without a sandbox (200 good, 200 crashed,
422-never-reaches-backend, 413, dead-backend 503 with hint, garbage 502). Plus the **gated
`GOJUDGE_IT` live suite**, run against the real go-judge v1.12.0 in docker: Python stdout, the
full pipeline normalising Java + reading stdin, compile/runtime errors as results — **all green
on first live contact**. Suite: 111 (+3 live).

## Verified

`cargo test --workspace` 111; clippy `-D warnings`; purity/caps/fmt; live sandbox ITs 3/3;
Insomnia grew `POST /api/run` in the same step.
