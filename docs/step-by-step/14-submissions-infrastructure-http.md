# Step 14 — Submissions: Postgres, the suite resolver, and the 202/poll surface

*(oracle: synapse step 15 — `PostgresSubmissionRepository` + Liquibase changeset 001,
`FileSystemProblemTests` with the fence fallback folded in as final design, `SubmissionApi` +
`SubmissionRoutes`; `PostgresSubmissionRepositoryIT` ported and extended)*

## The Postgres edge (`infrastructure/postgres.rs` + `migrations/0001_submissions.sql`)

The state ADT flattens to `(status, outcome jsonb, completed_at)` at THIS edge only, with the
oracle's schema verbatim — including the `completed_shape` check constraint (completed ⟺
outcome AND completed_at present) and the `(lesson_path, created_at desc)` recency index. The
JSONB codecs are **adapter-owned** (storage format ≠ wire contract) and replicate circe's
derived shape exactly — the externally-tagged wrapper object
`{"Rejected":{"passed":8,…,"firstFailure":{…}}}` with camelCase fields and case-name `RunStatus`
— pinned by a unit test, so a Rust deployment can read rows the Scala oracle wrote. sqlx uses
runtime-bound queries (compile-time `query!` checking needs a DB or a committed `.sqlx` cache in
CI; the gated ITs are the SQL's proof instead).

## The suite resolver (`FsProblemTests`)

Resolved THROUGH the catalog walker's lesson-file map — naive path joining is impossible with
`NN-` order prefixes. Two tiers, oracle-final: the `.tests.json` sidecar is authoritative; a
testcases fence inside the lesson is the fallback (the fence-only-problems fix, folded in as
day-one design). Absent both → not a problem; a suite that won't decode is a LOUD
`InvalidSuite`; re-read per lookup → hot reload. Five hermetic behaviors over an in-memory
content repo with real numbered-dir shapes.

## The wire (`shared/submission.rs` + `http/`)

`POST /api/submissions` → **202** `{id}` (the whole point: judging is a detached task) · public
`GET /api/submissions/{id}` (poll until `"completed"`; 400 on a non-UUID before any lookup) ·
`GET ?path=` newest-first (identity later makes it private, as the oracle staged it).
`SubmissionDto` flattens the outcome: verdict `"accepted" | "rejected" | "judge-failed"`, the
8/118 counts, the ONE revealed failing case, ISO instants. Insomnia grew all three requests.

## Dev/IT learns the databases must not share

The compose Postgres already carries the ORACLE's Liquibase-managed `submissions` table — sqlx's
migrator collided instantly ("relation already exists"). Dev and ITs now use their own
`synapse_rs` database on the same instance; adopting the prod schema stays a cutover-step
concern (baseline the migration as applied), exactly as the plan wrote it. The server FAILS FAST
on a dead Postgres at boot with migrations applied there (Liquibase-on-acquire parity); test
plumbing uses a LAZY pool so store-free routes stay testable without a database.

## Verified

150 Rust tests + 40 vitest; clippy `-D warnings`; purity/caps/fmt. Non-gated route ITs: 404
not-a-problem and 400 bad-id (store never touched), 500 loud invalid suite. **Gated
`POSTGRES_IT` (real database): the ADT flattens/reassembles through JSONB byte-for-byte; listing
is newest-first with `by_user` narrowing; and the crown piece — the FULL `POST` 202 → detached
judge (against a local go-judge stub) → poll flips to `completed/accepted 2/2`, through the real
router, real Postgres, and the real adapter chain.**
