# Step 27 — The widget spine: one host, six SVG families, honest cards

*(oracle: chapter 26 part 2's client half + chapter 27's renderers — `WidgetHost`,
`RendererRegistry`, `RenderKit`, the transport bar, and the fence hydration.)*

## The host and the dispatch

A ` ```viz widget=<structure> ` fence's JSON body already crosses the markdown island as a
`div.viz-widget[data-widget][data-payload]` placeholder (planted in step 08, vitest-pinned).
`blocks::discover` finds them; `decode` resolves the structure and the payload INDEPENDENTLY,
so `WidgetHost` renders honest cards — "The “X” widget isn't available yet." for an unknown
structure or a family without a renderer, "Couldn't read the “X” widget payload." for bad
JSON — never a blank box. The host chrome: the optional title, the scrollable canvas, the
transport bar ONLY when there's more than one step, and the reactive caption reading the
current step's annotation. Dispatch is the shared pure `RenderFamily::of`; the registry maps
a family to a Leptos renderer — Cells (array/bitset/fenwick), Stack, Tree (tree/segment-tree),
Chain (skiplist), Force (graph, on the seeded deterministic layout), and Trie (the tidy tree
layout on the generic canvas). The six step-33 bespoke HTML families return `None` for now.

## The render kit and the families

`RenderKit`: `diff_class` (`--new`/`--changed`/`--removed`), `themed` (wire hex →
`var(--viz-role-*, hex)` so pointer colours brighten in dark mode), `cursor_stack` (several
pointers on one node stack UPWARD one line apart — never overlap), `top_margin` (viewBox
headroom so stacked labels never clip — the "root label truncated" bug, pre-fixed), and
`fitted_text` (`textLength` squeeze). Every family lays out ONCE over `geometry::union`; the
step signal only toggles presence + diff classes (plain redraw at stable positions — keyed
move animations are the later refinement, exactly as the oracle staged it). The transport
bar owns the ONE interval timer (900 ms), started on play, dropped on pause/unmount; the
`Playback` tick self-stops at the end.

## Verified live

A throwaway smoke lesson (created in the content checkout, deleted after): the two-pointer
array rendered with the index row, `left`/`right` carets in their role colours, the title and
the 2-step transport; stepping showed "left moves to 8", the changed cell tinted primary, and
`2 / 2`; the BST drew with its root cursor stack; the skiplist chain drew `next` arrows + the
`∅` terminator; the cyclic graph fell to the seeded force layout; hashmap and `frobnicate`
both showed the honest unavailable card — all in dark mode on the role tokens. Suite: 329
Rust + 40 vitest; 557/700 KiB gz.

Next: the tracers + the Visualise modal (live traces through the SAME adapt pipeline and the
SAME host), then the bespoke widget gallery fills the six `None` families.
