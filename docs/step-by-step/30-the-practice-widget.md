# Step 30 — The "Your Turn" practice widget: a problem inside the lesson

*(oracle: the post-33 embedded-practice arc — `PracticeBlocks`/`PracticeProblem`,
docs/embedded-practice-problems.md, FINAL design — grown here with approach tabs, the
language switch, and language-exact Copy-to-editor.)*

## The authoring contract

A practice problem is a group of adjacent fences promoted by ` ```problem `: the statement
markdown, one or more ` ```lang run ` starters (language variants), an optional
` ```testcases ` judge set, and now **one or more** ` ```editorial ` fences. An editorial
fence may carry an approach tag — ` ```editorial approach-brute-force-1 ` /
` approach-optimal-1 ` — and each tagged fence becomes its own tab inside the Editorial
pane ("Brute Force" · "Optimal"; a repeated kind numbers itself). A bare editorial stays a
single "Editorial" tab; without ` ```problem ` the group renders as a plain workbench —
fully backward compatible. `render.ts` consumes the group into one `.practice-problem`
placeholder (`data-problem`/`data-variants`/`data-spec`/`data-editorials`), vitest-pinned.
The 4-backtick rule applies: a problem/editorial body carrying fences opens with four.

## The widget

`hydrate_practices` mounts a `PracticeProblem` per placeholder (title = the nearest
preceding heading's "Practice: <Topic>" tail). The shell is the two-pane `.pwb--embedded`
card, inline at the reading-column width: left — the PRACTICE badge + title, the
Description tab and one tab per approach (panes mount once and toggle `.hidden`;
editorials are LAZY — their Monaco solution viewers load on first open); right — the
reused `RunnableBlock` in practice mode (**Run only** — the Submit verb never renders);
between them the 9px draggable splitter (28–64%, document-level pointer listeners). The
⤢ Enlarge toggle CSS-promotes the SAME live `.pwb__panes` to a near-fullscreen modal —
Monaco and every buffer survive — with scrim/Esc close, per instance. Modal chrome sits
top-LEFT (the house rule; LikeC4 owns top-right).

## The language switch

`RunnableBlock` grew multi-variant: adjacent run fences are now **language tabs** over ONE
Monaco. Each variant keeps its own `BlockStore` (buffer, run state, edit unlock); switching
swaps the editor's value + tokenizer (`setModelLanguage` via the island's new
`setLanguage`) + read-only state in place. Run/Submit/Visualise/⌘-keymap all act on the
active tab; the coach's `code_sink` follows. This closes step 28's seam — the gallery's
Java variants are now reachable (verified: the Java array example ran Accepted through its
tab).

## Language-exact Copy-to-editor

Every `.solution-block` inside an editorial hydrates as a revealed read-only viewer
(language pill, `time=`/`space=` complexity chips, Copy-to-editor). The copy seam is
`(tick, language, code)`: the workbench finds the tab MATCHING the solution's language,
switches to it, and overwrites THAT buffer — the oracle's copy-to-wrong-tab bug, fixed by
design here. Verified live: copying the Java optimal solution switched the workbench to
Java and ran Accepted (`5`), while the Python tab still held its untouched starter
(Wrong answer, `0`).

## Verified live

The smoke lesson (deleted after): title "Sum Two Numbers" from the Practice heading;
Description | Brute Force | Optimal tabs; Python | Java workbench tabs; no Submit; tests
panel live; both approaches' viewers with chips; language-exact copy + judged runs both
ways; Enlarge → `.pwb--expanded` → Esc closed. The real `low-level-design` book renders
its three authored practice problems (bare editorials → single Editorial tab). Suite:
347 Rust + 44 vitest; bundle 557/700 KiB gz.

Next: the diagram slice — the mermaid island, diagram cards + the zoom modal (Enlarge and
Close both LEFT), and the LikeC4 embed check.
