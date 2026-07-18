# Step 40 — The page budget: editors, diagrams, first paint

*(what a rich lesson page costs, and how it is paid: how many Monaco instances may live at
once, when diagrams render, and what the reader sees while all of it is still loading —
plus the client half of the dev-flow logging that made the diagnosis possible.)*

## The question this step answers

A lesson page can carry a dozen runnable fences, five diagrams, a workbench, and a quiz. Each
of those is a real cost — a Monaco instance is megabytes of editor plus a tokenizer; a d2
diagram is a multi-MB WASM layout pass. Built naively, the page mounts all of it eagerly, and
the reader stares at nothing until the slowest thing finishes.

The step splits that into three independent budgets and pays each one differently:

| Budget | What overspends it | How it is paid |
|---|---|---|
| **Editor instances** | one Monaco per `run` fence, all mounted at page load | viewport-lazy mount + a live-editor cap (below) |
| **First paint** | diagrams rendering *inside* the markdown pipeline | diagrams move off the parse path — prose paints first |
| **Repeat visits** | re-running the whole pipeline on every navigation | a rendered-HTML cache keyed by content hash |

`docs/qna.md` Q1 records the Monaco decision in full (four options weighed); Q2 records the
render-performance diagnosis. This chapter is the resulting design.

## Editors: lazy mount, and a cap

A `run` fence renders its shiki-highlighted source and toolbar immediately — that is cheap and
it is what the reader actually looks at. Monaco mounts only when the block scrolls **near** the
viewport, watched by an `IntersectionObserver` with a `600px` margin on both sides
(`execution/view/lazy.rs`), so the editor is warm by the time the reader arrives rather than
popping in under their cursor.

Above that sits a page-level cap: `MAX_LIVE_EDITORS = 3`. When a fourth block mounts, the
oldest **far** instance is disposed — visible blocks are never evicted, so a page that genuinely
shows four editors at once keeps all four. Eviction is safe because a block's state (buffer
edits, run results, verdicts) lives in `BlockStore`, not in Monaco: a disposed editor loses
nothing, and re-approaching re-mounts over the same store with the edits intact.

Two degradation paths are deliberate. If `IntersectionObserver` is unavailable the block sets
`near = true` and mounts eagerly — the pre-lazy behavior, never a dead editor. And `NearWatch`
disconnects its observer on `Drop`, so navigating away cannot leak callbacks.

## The popup codebench

Lazy mounting solves `run` fences. It does nothing for the far more common case: a **plain**
fence that the author never marked runnable, which the reader nevertheless wants to try.

Rather than turn every fence into an editor — which would defeat the budget entirely — every
plain shiki figure in a runnable language grows a **"Try in Editor"** button, and the whole page
shares **one** Monaco behind it. `CodebenchModal` is a singleton mounted once in the shell; its
frame stays in the DOM while closed (`display: none`, not removed) precisely so the instance
survives. Each open swaps value, tokenizer, and read-only state in place — the step-30 seam,
reused. The cost of the feature is therefore one editor for the entire session, no matter how
many snippets a page carries.

The button is deliberately unsubtle: solid `--primary` fill, semibold, always visible, pinned
**top-right** of the figure so it never covers the start of a code line, wearing the same play
icon as Run. Hover-reveal was the wrong instinct — an invitation nobody sees is not an
invitation. `RUNNABLE_FENCES` mirrors the server's `Language::aliases` (21 entries); the server
stays the authority, and an alias added there joins the list in the same step.

Inside the modal: Run, an editable stdin box, and the runnable output panel. Editing gates on
sign-in — anonymous readers get a banner and can still **Run the code as written**, which is the
point of the affordance. Esc closes it like every other popup.

## Prose first: diagrams leave the parse path

The markdown pipeline used to render d2 during parsing: `d2Transform` awaited each diagram's
WASM layout inside a sequential loop, and the client's `set_inner_html` only fired once that
whole promise resolved. On a page with five diagrams, **every word of prose waited on the last
diagram's dagre layout** — and paid the multi-MB WASM download before the first one.

Now `d2Transform` is synchronous. It groups fences and emits source-carrying placeholders
(`.d2-block[data-source]`, `.d2-slideshow[data-slides]`) with no WASM involved, so the pipeline
returns as soon as markdown and shiki finish. Prose paints immediately; `D2Card` renders from
its source at mount, each diagram in its own task, concurrently rather than in a queue.

Two constraints shape that renderer, both load-bearing:

- **A fresh `D2()` per render.** A single module-level instance cannot serve concurrent
  compiles — several diagrams rendering at once deadlock it (reproduced with three). Only the
  multi-MB dynamic import is cached, which is where the real cost is anyway; `new D2()` is
  cheap.
- **Eager at mount, not viewport-lazy.** Diagrams render as soon as their card exists. Prose-first
  comes from d2 being off the *parse* path, not from deferring the render, so laziness buys
  nothing here and risks an unrendered card.

`mermaid.initialize(config)` now runs behind a module-level latch — once per session instead of
once per diagram. Malformed diagrams surface an error card at mount rather than failing the
whole pipeline at parse time.

## The rendered-HTML cache

With d2 client-rendered, the pipeline's output is small — placeholders, not baked SVG — so it is
worth keeping. `islands::markdown::render` memoizes by a hash of the raw markdown in a
thread-local map, and back/forward plus sidebar re-clicks skip fetch, parse, shiki, and all.
Keying by content hash rather than by path means an edit during authoring misses cleanly instead
of serving a stale page.

## The client's dev-flow logging

ADR-S009's layered trace had a server half only. The client now has the matching port
(`client/src/log.rs`): a coloured `SYNAPSE` badge with per-level emoji, `debug` gated to
localhost so production consoles stay quiet. This is what made the render diagnosis tractable —
`lazy workbench: near = …` and the codebench's open trace are ordinary debug lines.

## Chrome, in passing

- **The editor/tests resize strip** — the problem workbench's two panes are draggable, with a
  double-click to fill.
- **The problem page's real width and height** — the shell's cap is off and the editor fills its
  pane rather than sitting in it.
- **Tooltip clipping** — the Edit tooltip escaped its clipping ancestor; the strip above the
  editor is reclaimed for the editor once signed in.
- **The frames panel** — long values were truncated to `[a, e, i, …]` while whitespace sat to
  their left; the preview widened to twelve elements, which the cortex goldens then flagged. The
  goldens gained a fourth documented delta (`normalize()` erases frame `value` *and* `changed`,
  since a wider preview un-masks mutations past index 2) plus a test pinning the width.
- **The ⌘K palette** rendered at the page's bottom-left. The cause was a genuinely broken
  stylesheet: step-25 deleted the `.header__search*` selector lines but left their declaration
  bodies orphaned at file scope, and CSS error recovery swallowed the rule that followed —
  `.cmdk-scrim`, which supplies `position: fixed` and the centering. With the orphans removed the
  palette is an overlay again, at `z-index: 300` (the oracle's ⌘K rung: above the C4 docs panel
  at 250 and the diagram scrim at 70). A postcss scan confirmed the other fourteen stylesheets
  carry no file-scope declarations — this file was the only casualty.

## Where it lands

Suites after this step: **363 native · 45 vitest**; fmt, conventions, and clippy (native +
wasm) clean; critical path **628/700 KiB gz**.

Verified live at :5373 — prose and headings present while diagrams are still filling in, all
five d2 diagrams rendering with real viewBoxes and no hangs, the codebench opening from a plain
fence and running, re-navigation served from the HTML cache, and the palette centered over a
dimmed scrim.

## Postscript — the stylesheet gate (same day)

The ⌘K bug is worth a gate, not just a fix. Broken CSS is uniquely quiet: the browser discards
the damaged region *and the rule after it*, logs nothing, and the page merely looks wrong — this
one survived fifteen steps that way. So `client/styles/stylesheets.test.ts` now parses every
stylesheet with postcss and fails on the two shapes the damage takes: a **parse error** (a stray
`}` or an unclosed block — what actually happened) and **declarations orphaned at file scope**
(the same wound when the braces happen to balance).

It lives in vitest rather than `check-conventions.sh` on purpose: the convention gate is
deliberately dependency-free (grep/find/wc, so CI can run it before the toolchain), and CSS is
not a thing to parse with grep. The suite carries fixtures for both shapes so it cannot pass
vacuously, plus a count assertion so an empty glob can't silently disarm it — and it was checked
against the real damaged file from history, which it rejects at `search.css:8:110`. postcss
becomes an explicit devDependency (it was already present transitively, via vite). 45 → 63
vitest.
