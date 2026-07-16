# Step 34 — C4 click-to-guide: the diagram becomes the table of contents

*(oracle: step 32 / ADR-S032 — `C4NodeResolver` + `attachNodeBridge` + `C4DocsPanel`, the
co-located `_c4-docs/` design.)*

## The resolver

`resolve_c4_node` is pure and pinned: a click's composed path (target-first
`(tag, class, data-id)` hops) resolves to a LikeC4 element FQN iff a hop carries the EXACT
`react-flow__node` class token (split-on-whitespace — edges carry random-hash ids but not
the token, and `react-flow__node-toolbar` is a substring, not a match) with a non-empty
`data-id`. A `<button>` BEFORE the node is one of LikeC4's own per-node controls
(relationships / details) — resolve to `None` and let the viewer keep its native overlay.

## The bridge

Both iframes — the inline embed (wired in the same `load` hook as the overlay guard and the
scope style) and the fullscreen zoom — get a CAPTURE-phase click listener on the
same-origin document. The hop extraction is Reflect-only: **nothing in an iframe's composed
path passes a parent-realm `instanceof`**, so `dyn_ref`/`dyn_into` silently drop every hop
(the second cross-realm trap this arc; the first was step 33's WheelEvent). `tagName`,
`class`, and `data-id` are read via `Reflect::get`/`getAttribute.call` — window/document
hops have no `tagName` and drop out naturally. On a hit the click is swallowed and the
selection signal set.

## The panel

`C4DocsPanel` slides in from the right: COMPONENT GUIDE eyebrow, the doc's title,
kind/technology chips, and the markdown body rendered through the same TS pipeline as
lessons. The doc comes from the EXISTING `GET /api/synapse/c4-doc/{id}?lesson=…` (built in
step 06 — the server has carried the co-located `_c4-docs/` lookup since the catalog
hexagon): fetch per selection, stale replies for superseded selections dropped, a missing
doc renders the honest "has no guide here yet" card. ✕ and Esc close; clicking another
component switches context in place. RS deviation, on purpose: a fixed right-side panel
instead of the oracle's JS grid-collapse (`!important` inline-style surgery) — the reader
column stays put.

## Verified live

On the Architecture Docs lesson: clicking the `sfClient` node body opened "Client"
(Container · Browser SPA) with the rendered guide; clicking `sfServer` switched to
"Server & DB" in place; clicking a LikeC4 per-node `<button>` changed NOTHING (the viewer
keeps its own overlay); Esc closed the panel. Suite: 352 Rust (+3 resolver pins) +
44 vitest; bundle 557/700 KiB gz.

Next: RS-P9 — the production build (Dockerfile, GitOps), then the parity gate + cutover.
