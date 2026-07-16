# Step 33 — The mobile drawer + the LikeC4 chrome

*(oracle: step 38 — `ReaderNavDrawer`, the FAB rules — and commit d8b969a — `C4Blocks` +
`DiagramZoom.openIframe`.)*

## The mobile navigation drawer

Below 1024px the desktop sidebar hides entirely and a bottom-LEFT FAB (the panel-open
glyph) takes its place; tapping it slides in an off-canvas drawer — `Contents`, a close
button, and the SAME `Sidebar` component the desktop reader renders. Three ways out:
the scrim, Escape (guarded per instance), and **any nav-link tap** — a capture click on
the drawer checks `target.closest("a")`, so the link routes and the drawer closes in one
gesture. One breakpoint (1024px) governs both sides of the trade; the old 48rem stacking
block leaves shell.css. The oracle's prod bug is designed in: the drawer mounts OUTSIDE
the reader grid — its in-flow wrapper would otherwise become a phantom third grid item at
desktop width. The FAB sits at `left/bottom: 20px`, level with where the right-side stack
packs from (the prefs FAB keeps the right corner).

## The LikeC4 embed chrome

Every authored `<iframe src="/c4/…">` is wrapped in a `.c4-embed` so an **⤢ Enlarge**
button (top-LEFT — LikeC4's own chrome owns top-right) floats over it. Enlarge opens the
near-fullscreen iframe zoom: a NEW iframe with the same src fills the modal (LikeC4 keeps
its native pan/zoom), with ✕ Close top-LEFT and the parity pill bottom-centre — **− / live
% / +** and the gesture hint ("or pinch / Ctrl+scroll to zoom · scroll or drag to pan").

The ± buttons drive **synthetic ctrl+wheel pinches**: a `WheelEvent` built from the
IFRAME's own constructor (parent-realm events aren't native to the receiving document),
`deltaY ∓16`, dispatched at the centre of `.react-flow__pane` with `ctrlKey: true` — the
flag that makes d3-zoom pinch instead of pan. The live % is parsed from the
`.react-flow__viewport` inline `scale(N)` transform — read via the style ATTRIBUTE, because
iframe elements fail parent-realm `instanceof` (`dyn_into` is a checked cast; the two
cross-realm casts in the zoom path are deliberately unchecked).

**The overlay guard**, both sides: LikeC4's element-details/relationships browser opens a
`<dialog class="likec4-overlay">` whose own ✕ · Share · Export render exactly where our
chrome sits — so while `.likec4-overlay[open]` matches inside the iframe, our chrome steps
aside. Inline: a MutationObserver (`childList+subtree` for the dialog's first insertion,
`attributeFilter: ["open"]` for show/close — the dialog lingers once used) toggles
`.c4-embed--overlay`. Fullscreen: the same 300 ms poll that reads the scale also sets
`.diagram-zoom--c4-overlay` — one timer instead of an observer that would fire every pan
frame. Both iframes also get the scope style (`layerStyle_likec4.panel` hidden — the
merged `/c4` workspace's view picker lists every diagram of every book).

## Verified live

At 420px: FAB visible, sidebar hidden, drawer opened with the real sidebar (4 links),
closed by a nav tap (and routed), reopened, closed by Esc. The C4 embed: Enlarge opened
the fullscreen, the viewer booted, the live % read 80% and one + press moved it to 99%
(≈ +25%), the scope style landed in both iframes, and an injected `.likec4-overlay[open]`
dialog hid the chrome on BOTH the inline embed (observer) and the modal (poll), releasing
on removal. Suite: 349 Rust + 44 vitest; bundle 557/700 KiB gz.

Next: architecture docs + the capstone — LikeC4 click-to-guide and the co-located
`_c4-docs/` panel.
