# Step 57 — The Editorial tab redesign

*(A stepper for the brute → optimal journey, and the phone layout that was never actually
there.)*

## The ask

The problem page's Editorial tab rendered the editorial as one document sliced into
`.pwb-esec` wrappers behind a pill row (step 39) — functional, and flat: every section the
same weight, multi-approach editorials reading as one long scroll, the solution open before
the reader had tried anything. The user redesigned it in Claude Design (`Problem
Workbench.dc.html`, imported via the design MCP) and asked for the Editorial part only.

The design: an **approach stepper** (numbered circles over a connector rail, per-approach
time/space, done/active/future states) when the editorial has multiple approaches; a
**single-approach bar** (★ + complexity pills) when it has one; a sticky **Jump** bar with a
scroll-spy over numbered sections (`01 Intuition` …) in one continuously scrolling document;
the solution behind a dashed **"Reveal the solution"** card (collapsed by default,
re-collapsing on approach switch — the user confirmed this supersedes step 37's
always-revealed rule); and the Complexity section as **Time/Space cards** with big mono
O-values.

## The two formats, verified against the real content

All 68 editorials in synapse-content/dsa were surveyed before writing the parser. Type 1 is
flat (`## Intuition / ## Approach / ## Solution / ## Complexity Analysis`, 35 files); type 2
nests the same set as `###` subsections under one `##` per approach (`## Brute`, `## Better`,
`## Optimal 1` …, 16 approach sets, plus a `### Edge Case` in five). Every file missing a
Solution heading turned out to be one of the 26 *empty* editorials — the formatting is
uniform. Two things only the survey caught:

- **Fence metas carry spaces inside values** — `time=O(log N)`, `time=O(log(min(N1, N2)))`.
  `solution_complexities` split on whitespace and would have truncated these to `O(log`; it
  now pulls tokens until the parentheses balance.
- **Complexity prose separates value from explanation three ways** — dash, period
  (`O(1). As no extra space…`) and comma (`O(K), where K…`). The splitter finds the first
  separator *outside* parentheses after a closed `O(…)` group, so `O(min(N1, N2))` and
  `O(sqrt(N)) + O(K*Log(K)) – …` both survive whole.

## The shape

The pure half is `catalog/logic/editorial.rs`: fence-aware splitting (the
`problem_content_split` idiom), format *detection* (multi iff ≥2 top-level sections and any
carries a canonical `###` — an approach heading is free text, its contents are the
recognisable part), the spoiler-wrapper strip (the inline editorial arrives wearing
`<details>`, and per-section fragment rendering would smear the unbalanced pair across
fragments), solution synthesis for bodies whose fences have no Solution heading, complexity
claims and prose, `pretty_o` (`sqrt(N)`→`√N`, `*`→`·`, `^2`→`²`), and the spy arithmetic.
23 native tests, all shapes from the survey.

The view half is `catalog/view/editorial.rs`. Sections are Leptos-built `<section
data-esec>` wrappers; each body renders through the markdown island per fragment (step-40
memoized, so approach re-visits are cheap) and hydrates the same island set as every other
pane — except solutions mount **gated**: `mount_gated_solutions` is `mount_solutions`'
twin that wraps each `.solution-block` in the reveal card and mounts the existing
`SolutionViewer` only on reveal (visible, so Monaco measures right the first time; Hide
unmounts it; an approach switch re-creates the fragment, so it collapses again). The
practice widget keeps `mount_solutions` and is byte-identical.

The remembered approach got its **own localStorage key**, not a fourth `PanePrefs` field:
an approach label is free text like `section`, and the pipe-delimited record can only let
one field absorb the remainder. `pane::section_index` restores both by label.

The pane region restructured: one shared `.pwb__pane-scroll` became a `.pwb__pane-host`
with per-pane scroll regions, because the stepper must sit *between* the tab bar and the
scrolling content. Side benefit: per-tab scroll positions stop bleeding into each other.

## Three scroll lessons, all verified live

1. **`scrollIntoView` walks every scrollable ancestor.** The first jump implementation
   crept the page itself down ~64px per click. The jump now scrolls the pane directly
   (`ScrollToOptions`), and below 1024px — where the pane stops scrolling — targets the
   window with the same math, via plain `scroll_to_with_x_and_y` like `scroll_to_heading`
   (the page's own `scroll-behavior: smooth` supplies the easing).
2. **Leptos reuses the scroll container's DOM node across the approach re-render**, so its
   old scrollTop survived the switch — switching mid-document landed mid-document. Every
   body after the first now resets to the top explicitly.
3. **The design's typography is not the reader's.** The reading-size preference scales
   `.synapse-prose` via `html[data-reader-size]`, and at "lg" the editorial prose sat level
   with the 19px section titles — the hierarchy vanished (user catch, with screenshots).
   The fragment pins its own 15.5px/1.72; an explicit size on a descendant beats the
   inherited scale without a specificity war.

## The phone layout that was never there

Mobile verification found the real defect: **the standalone problem page had no phone
layout at all.** The ≤1023px block stacked only `.pwb--embedded`; the page kept its
desktop geometry — a fixed-height frame whose overflow-hidden panes *clipped* the tab
content below ~480px, beside a right pane squeezed to **24px**. The whole editor was
invisible on phones, *measured on production* — shipping since step 37 and never noticed
because the mobile passes (46, cutover) checked width overflow and the reader, not a
problem page's depth. The stacking block now covers both frames — and it moved to the END
of practice.css, because the page's fixed height is declared *after* where the block used
to sit and equal specificity made source order the decider (the same class of cascade
accident as step 40's ⌘K bug, caught live instead of shipped).

One more macro footnote: `class:x--done=move || active.get() > i` — the `>` inside an
unbraced attribute closure reads as a tag close and the view macro produces a genuinely
baffling trait error. Braces around the closure end the ambiguity.

## Verified

Desktop: stepper states + `O(√N+K·log(K))` prettified, jump/spy against real positions,
reveal → Monaco lines on first paint, Copy-to-editor into the right pane's matching tab,
switch → top/Intuition/collapsed, reload → approach + section restored. Type-1 page: ★ bar
with `O(N²)` pill. Light theme, 375px (no overflow; jump lands the section at exactly
70px; editor finally visible on a phone). Practice widget unchanged (3 widgets, revealed
viewers, own tabs). 458 native tests + 83 vitest; zero console errors.
