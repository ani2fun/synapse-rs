# Step 37 — The problem page: two panes, four tabs

*(oracle: `ProblemWorkbench` + `SubmissionsFeed` + `ProblemContent.split`, steps 16/23
final form; the parity list's second item.)*

## The branch

A `kind: problem` lesson no longer renders the prose column — `loaded_lesson` returns the
two-pane `ProblemWorkbench` instead, full width, with the TOC/prefs chrome suppressed (the
reader's `is_problem` signal was already there). Breadcrumbs sit above the panes:
`Home › <humanized book> › <lesson title>`.

## The left pane

Title + lede, then the four tabs — **Description · Editorial · Coach · Submissions** —
with the frontmatter `difficulty` badge floated to the row's end (`easy`/`medium`/`hard`
tint classes). All panes MOUNT ONCE and toggle `.hidden` (editor and chat state survive
tab switches); Submissions is additionally lazy (first click mounts it).

- **Description** = the raw markdown BEFORE the first top-level `<details` (the pure
  `problem_content_split`, fence-aware, pinned) — rendered and hydrated as usual, except
  the FIRST workbench placeholder is **extracted**: decoded, removed from the prose, and
  handed to the right pane.
- **Editorial** = the post-`<details>` inline tail, else the co-located
  `<lesson>.editorial.md` sidecar the payload already carries. Spoilers open (this tab IS
  the answer), solution fences hydrate as revealed viewers, and Copy-to-editor routes into
  the right pane's MATCHING language tab — verified live: the Java solution switched the
  workbench to Java across the pane boundary.
- **Coach** = the tutor chat, fed the workbench's live `(source, language)` snapshot.
- **Submissions** = the caller's own list (anonymous → the sign-in note), newest first:
  Current submission + All submissions tables, verdict badges (Accepted / Wrong answer /
  Judge failed), passed/total, the 👁 code card — and it REFETCHES when the workbench's
  submit completes (the `submitted` bump seam on `RunnableBlock`).

## The right pane

The extracted workbench — language tabs, Edit, Submit, Visualise, Run, tests panel —
under the anonymous note bar ("Sign in to edit and submit — you can still Run the
starter"). The 9px splitter drags 28–64%, the same document-level pointer pattern as the
practice widget.

## Verified live

Flip Characters renders the full page: crumbs, EASY badge, four tabs, the extracted
workbench right (Python | Java tabs, Submit, Case 1–6 chips, ARR/EXPECTED grid), the
Editorial's two revealed solutions with the cross-pane copy, the Submissions anonymous
gate. Suite: 361 Rust (+1 split pin) + 44 vitest.

Next: the toolbar icon chrome + the Visualise modal redesign (parity items 3–5).
