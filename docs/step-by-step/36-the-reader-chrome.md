# Step 36 — The reader chrome: the page that keeps you oriented

*(oracle: the steps 12–13 reader arc in its post-33 final form — `ReaderSidebar`/
`SidebarMode`, `ReaderMiniMap`, `ReaderToc`, `ReaderFocus`, `ReaderStickyBar`,
`ReadingProgress`; the first item of the user's parity list.)*

## One scroll handler, five consumers

`ChromeState` (headings · title · progress · active-id · show-top · past-title) is created
once per `LessonPage` and fed by one scroll recompute (oracle thresholds: scroll-top FAB at
600px, sticky bar at 160px, active heading = last with `rect.top ≤ 120`). Headings are
harvested from the rendered prose (`h2[id]/h3[id]` — rehype-slug has been minting the ids
since step 08). The consumers: the 2px top progress bar, the sticky "title / active
section" wayfinding bar, the right-edge minimap, the TOC popover, and the Compact rail's
progress ring.

## The sidebar's three faces

Expanded: the toprow (**← Learn** flips to the catalog browse; « collapses to the rail; the
panel icon hides entirely), the italic display book title + the mono-uppercase description,
the **Filter this book…** box (pure `prune_entries` — case-insensitive substring on titles;
a matching chapter keeps all its lessons; pinned natively), and the chapter tree
(`<details>` chevrons, active-lesson highlight with the primary left edge). Compact: the
numbered rail — `01…09` tiles linking each chapter's first lesson, the ACTIVE tile wearing
a conic **progress ring** driven by the same page-scroll fraction. Hidden: the grid column
collapses to 0 and a floating expand affordance appears top-left. The mode persists in
localStorage (`reader-sidebar`), and the mobile drawer keeps reusing the same component,
pinned Expanded. The Learn browse lists every book (Dashboard link + categories; only the
current book's category starts open; the current book highlighted).

## The floating chrome

The minimap (≥1200px): one horizontal bar per heading at its true document fraction,
de-overlapped by the pure `spread_fractions` (min gap 0.05, forward/backward passes —
pinned), l3 bars shorter, hover reveals the label pill, click jumps (−80px header offset),
and the teal fill grows with progress. The FAB stack packs upward from 20px: TOC (list
icon, popover with active-row ticks), focus (76px), reading prefs (moved to 132px), and the
transient scroll-top (188px) that never leaves a hole. Focus mode: `F` toggles, `Esc`
exits, `.syn-focus` hides the header/sidebar/FABs/minimap/sticky and leaves the prose; the
hint pill dims after 2.6s. The prose header becomes the oracle's: **← Library**, the italic
display title, the lede; prev/next become pager cards with humanized titles.

## Also fixed on the way

The Run button's label was INVISIBLE — `.runnable__run` had `color: hsl(var(--primary) /
0.08)` on a primary background (the user's screenshot-3 catch); now `--primary-foreground`.
And the widget host's scale layer gained flex centering, so modal widgets sit centered
instead of hugging the left edge (screenshot-5's first item). Two more of the known Leptos
traps resurfaced and are named here for the next reader: plain elements take `data-x=`
(never `attr:data-x=`), and `use_context` inside a `spawn_local` body runs ownerless —
capture the context BEFORE the async block.

## Verified live

On Storage Engines: the sidebar head/filter/tree (filter "storage" → 2 lessons + clear
button), Learn browse (6 books, current active, Dashboard link), Compact rail (9 tiles,
`--progress: 9.3` on the active ring) and back, Hidden + expand round-trip via
localStorage; 12 minimap ticks with the correct active; the TOC popover (12 rows, active
row, jump-and-close); sticky bar `Storage Engines / The problem (why this exists)` past
160px; progress 9.27% at depth; scroll-top appearing past 600; focus mode on `f`, hint
pill, off on `Esc`. Suite: 360 Rust (+2 chrome-logic pins) + 44 vitest.

Next: the two-pane problem page (the parity list's second item).
