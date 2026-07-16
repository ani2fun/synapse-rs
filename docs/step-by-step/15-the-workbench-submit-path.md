# Step 15 — The workbench submit path

*(oracle: the step-13/16 workbench slices — `WorkbenchTests`, `SubmitState` + the 1.2 s poll
loop, `verdictPanel`, the Run-with-stdin seam — reduced to the pre-identity scope exactly as the
oracle staged it)*

## The tests panel (`view/workbench.rs`)

The placeholder's `data-spec` (the authored `TestSpec`) hydrates into: **case chips** that seed
the values grid from the authored case, editable **value fields** per declared arg, and the
expected-output card. The Run seam changes meaning when a suite is present: stdin is the active
case's values through the SHARED `stdin_for` shape — the same bytes the server-side judge feeds.

## Judged output

With an expected output the run is JUDGED in the view via the shared `judge`: the badge becomes
"Accepted ✓ / Wrong answer ✗" and the stdout slab takes the ok/err legend tint. Without a suite
the output panel renders plain — the oracle's exact split.

## Submit → 202 → poll → verdict

`SubmitStore` (state layer): POST `/api/submissions` → hold the id → poll every **1.2 s, ≤ 100
tries**, gated by an `alive` flag flipped in `on_cleanup` so an unmounted block stops polling
(the oracle's `alive` Var). The verdict panel renders the lifecycle: judging (with the id),
**accepted** (n/n), **rejected** (counts + the ONE revealed first failure: expected/stdout/
stderr), judge-failed (the machinery detail), failed (transport). ⇧⌘⏎ reaches monaco as the
third `addAction` — wired only when the block HAS a suite (the verb exists only where it means
something). Identity later gates Submit/Edit on sign-in.

## Plumbing notes

The lesson path threads LessonPage → LessonBody → `hydrate_workbenches` (a submission needs its
problem's directory-mirror path). The editor island's callback bundle grew a struct
(`EditorCallbacks`) once the fourth verb arrived — flat args stop scaling at four. `SubmitState::
Done` boxes its DTO (clippy's large-variant catch).

## Verified

150 Rust + 40 vitest; clippy `-D warnings`; purity/caps/fmt; bundle 318/700 KiB gz (the wasm
grew with the panels). **Live, full stack (server + real Postgres + real go-judge): the
flip-characters starter runs against case 1's stdin and is judged "Wrong answer ✗" with the err
tint (it echoes unreversed); Submit walks 202 → judging → the verdict panel lands "Wrong answer
✗ — 0/11 cases passed" with the first failure revealed; a CORRECT solution through the same API
lands `accepted 11/11`.** Zero console errors.
