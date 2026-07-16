# Step 29 — The bespoke widget gallery: six structures get their own chrome

*(oracle: chapter 33 — `WidgetShapes`, `DomKit`, the flow-layout renderers, the cue legend,
the frame line chips.)*

## Why flow layout

The SVG families place every node on a computed canvas — right for trees and graphs, wasteful
for structures whose shape IS a layout the browser already knows: a hashmap is rows of chains,
a queue is a strip, a linked list is boxes and arrows. The six families the registry had been
answering with the honest "isn't available yet" card now render as **HTML flow widgets**: they
size to content, wrap, and inherit the design tokens for free. No layout pass — each widget
re-derives its little shape model per step off the same `step_index` signal.

## The pure shape layer

`viz::shapes` (oracle `WidgetShapes`, natively tested case for case) projects a `VizStep` into
what each renderer draws:

- **`buckets`** — each `kind == "entry"` node is a bucket; a ref entry (`·`) walks
  entry → cells → instances into `key: value` pills (a cell with no out-edge is itself the
  value); a scalar entry is one keyless pill. Buckets sort numeric-first-ascending, then text.
- **`chain`** — start at the head cursor (`head`/`h`/`first`), else the node with no incoming
  `next`; walk `next`/`nxt` edges cycle-guarded; append unreached stragglers in wire order.
  Any `prev`/`previous` edge anywhere marks the list doubly.
- **`forest`/`forest_graph`** — the parent array read structurally: a cell is a root iff its
  label is unparseable OR equals its own slot. The drawable forest relabels nodes by ELEMENT
  INDEX, draws parent→child edges, and badges every root with a synthetic `root` cursor.
- **`grid_cells`** — nested rows follow the outer ref-cells (columns by slot, holes stay
  `None`); a flat row falls back to √n columns, mirroring `GridLayouts`.
- **`heap_tree`** — a bare-array step synthesizes `i → 2i+1 (left) · 2i+2 (right)` edges; a
  step that already carries edges (an object heap) passes through untouched.

## The renderers

`render::dom` is the shared kit: `diff_mods` (each cue appended independently), the floating
`cursor_badge` (stacked names + one ▾ caret, absolutely positioned above the node), and the
∅ / → glyphs. On it sit five renderers — `buckets` (index chip + pill chain), `strip` (the
queue/deque strip with head·tail / front·back end markers coloured through the role tokens,
and the vertical stack column, top-first with the TOP marker), `list_chain` (value boxes with
NEXT/PREV compartments joined by coloured SVG arrows, closed by ∅), `grid_table` (index
gutters, shared borders via negative margin, dashed holes), and `dual` — the heap and
union-find **dual views**: the derived tree above, the raw backing array below, both driven by
the SAME step signal. Identical node ids make the diff cues light up in both panes at once;
for union-find the tree shows indices while the array shows parent pointers — same id,
deliberately different text. Registry quirks live where the oracle put them: callstack keeps
the SVG frame boxes, and deque flips the queue strip's vocabulary.

## Legend + chips, finished

The modal legend is now the oracle's: data-driven items (cursor, new, changed, removed,
pointer) with inline themed colours — diff swatches wear the diff TOKENS the renderers
actually tint, cursor items wear the marker palette — and a doubly list adds the next/prev
arrow lines. The frames panel's active frame now carries both chips: `L<current>` (primary)
and `→ L<next>` (muted), the same pair the source pane highlights.

## Verified live

Real traces on the gallery lesson, per family: the queue strip ended `[30, 40]` with the
popped tail re-emitted as a dashed removed cell (the adapt diff made visible); the deque strip
spoke front/back; the stack column ran top-first (`9,1,7,3` at slots `3,2,1,0`) with TOP on
top; the singly list drew 4 boxes + arrows + ∅ with no PREV compartments, the doubly list
grew them plus the return arrows and the next/prev legend lines; the hashmap showed the
collision chain `apple → grape` in bucket 1, buckets in numeric order; union-find's forest
badged 2 roots over `parent[i] = [1,3,3,3,5,5]`; the heap drew 7 tree nodes over 7 array
cells; the 3×4 grid carried its index gutters. Frame chips read `L7 → L8`. Suite: 341 Rust
(+12 shape pins) + 40 vitest; bundle 557/700 KiB gz.

Next: RS-P8 — the "Your Turn" practice widget, then landing tour + hero, then the mobile
drawer + LikeC4 chrome.
