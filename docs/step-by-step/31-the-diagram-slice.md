# Step 31 — The diagram slice: mermaid, d2, LikeC4 — all rendering, one zoom

*(oracle: the step 24–25 diagram arc — `mermaid.ts`, `DiagramBlocks`, `MermaidView`/`D2View`,
`DiagramZoom`, diagrams.css.)*

## What was broken

The markdown pipeline (step 08, oracle-verbatim) has been emitting diagram placeholders all
along — `.mermaid-block[data-source]`, `.d2-block[data-svg]`, `.d2-slideshow[data-slides]` —
but nothing hydrated them: authored diagrams rendered as empty divs (two visible in the
`low-level-design` book alone), and the LikeC4 lesson iframe had no `/c4` route in dev.

## The mermaid island

`islands/diagram/mermaid.ts` is the oracle's, verbatim: mermaid@11, `securityLevel:
"strict"`, always the LIGHT `"default"` theme (authored diagrams colour nodes with light
pastel fills and never set text colour — the dark theme would paint light-on-light),
`fontFamily: "inherit"`, a monotonic render id. The loader is the RS flat-FFI shape
(`renderMermaid(target, src)` folds the lazy import in); the multi-hundred-KB chunk lands
only on lessons that actually carry a diagram.

## Cards + hydration

`catalog::view::diagrams::hydrate_diagrams` mounts per placeholder: `MermaidCard` (island
render; a malformed diagram becomes the loud `.diagram-error` card with the raw source —
never a blank figure), `SvgCard` (d2's parse-time SVG injected), and `D2Slideshow` (a run of
adjacent d2 fences steps through one figure with the ‹ i / n › transport). Every figure sits
on a FIXED-LIGHT card — the authored palettes assume light — capped at `min(70vh, 32rem)` so
inline diagrams stay glanceable.

## The zoom overlay — chrome on the LEFT

A rendered figure grows the ⤢ Enlarge pill (top-LEFT, hover-revealed); it opens the
near-fullscreen `.diagram-zoom` overlay: light paper card over a blurred scrim, wheel zoom +
drag pan on the viewport, the − % + ⟲ pill bottom-centre, ✕ Close **top-LEFT**, Esc/scrim
close. The house rule is deliberate and now uniform: OUR chrome — the card Enlarge, the
overlay Close, the practice widget's Enlarge — sits top-left, because LikeC4's own chrome
(✕ · Share · Export, per-node tools) owns the top-right corner.

## LikeC4

The lesson embed is an authored `<iframe src="/c4/view/…">`; the dev seam was missing on
both ends: vite now proxies `/c4` to the server (which fronts the compose `likec4` service —
the oracle's opt-in `--profile c4` builder over the merged synapse-content workspace), and
the iframe gets a rounded dark frame. Verified: the viewer SPA boots inside the iframe
(react-flow mounted) on the Architecture Docs lesson.

## Verified live

`java-basics`: both mermaid diagrams render (0 errors), Enlarge → overlay with the SVG,
+ → 125%, Close at (14, 12) top-left, Esc closes. `storage-engines`: the 3-slide d2
slideshow steps 1/3 → 2/3 plus a single diagram card. `architecture-docs`: the LikeC4
viewer live through `/c4`. Suite: 347 Rust + 44 vitest; bundle 557/700 KiB gz (mermaid is
a lazy chunk, off the critical path).

Next: RS-P8 continues — the landing tour + hero, then the mobile drawer + LikeC4 fullscreen
chrome, then architecture docs + capstone.
