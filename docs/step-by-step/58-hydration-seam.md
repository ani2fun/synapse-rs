# Step 58 — The hydration seam

*(One bundle for the store caravan, one kit for the mount loop — the first step of the
deepening loop.)*

## The ask

A `/codebase-design` assessment of the whole workspace (three parallel surveys: server
ports/seams, client layering/stores, shared+platform) ranked seven deepening opportunities;
the user approved executing all seven as a loop. Items 1+2 — this step — both live at the
same place: the client's hydration seam, where the markdown pipeline's planted placeholders
become live components.

Two smells, both named in the code itself:

- **The out-of-tree store caravan.** Hydrated islands mount via `leptos::mount::mount_to`,
  which starts a fresh root owner — App context is unreachable (the step-17 lesson). The fix
  has always been capture-then-carry, but the carry had grown to four stores threaded
  individually through every level: reader → `hydrate_workbenches`/`hydrate_practices` →
  `PracticeProblem` → `RunnableBlock` (11 props), with
  `#[allow(clippy::too_many_arguments)] // the out-of-tree store caravan` standing guard in
  two files. Adding a fifth store meant touching every signature in the chain.
- **Eight hand-rolled mount loops.** Every hydrator repeated the same mechanics —
  `query_selector_all` → `dyn_into::<HtmlElement>` → decode the URI-encoded `data-*` attr →
  `mount_to` → collect `Vec<Box<dyn Any>>` — so the decode-or-skip rule and the
  handle-is-the-lifetime rule lived as eight implied copies, and a new widget author had to
  learn them by imitation. (The editorial once shipped WITHOUT its mounts and rendered empty
  boxes under captions — the cost of the seam being convention, not structure.)

## The shape

One new flat module, `client/src/hydration.rs` (118 lines) — view-side infrastructure, the
client's `platform/`-equivalent. Deliberately NOT under `logic/` (it touches leptos/web-sys;
the purity gate would — rightly — reject it) and NOT in `islands/` (that folder is FFI
externs only). Top-level placement also keeps `quiz` and `viz` from conceptually importing
execution's view layer.

**`IslandStores`** — `#[derive(Clone, Copy)]` over the four context stores (`auth`, `theme`,
`viz_modal`, `codebench`) plus `capture()`, which is documented IN-TREE ONLY. The canonical
explanation of the out-of-tree rule moved here from `RunnableBlock`'s prop comments; every
prop site now carries one pointer line. Two boundaries drawn in its doc comment:

- Only App-level context stores ride in the bundle. Per-render signals (`code_sink`,
  `load_code`, `c4_selected`, `wb_spec`, `submitted`) are minted by the page that owns them
  and stay separate props — bundling them would break the "context snapshot" meaning.
- Carriers take the whole bundle even when they read part of it (`RunnableBlock` never
  touches `codebench`) — a second, narrower bundle type would be more total interface than
  it saves on a `Copy` struct. Leaves that need ONE store keep their narrow prop:
  `SolutionViewer`/`GatedSolution` take `theme`, `FenceBar` takes `codebench`.

**The mount kit** — four helpers, deliberately not a framework: `elements` (selector →
`Vec<HtmlElement>`), `decoded_attr` (read + URI-decode, because the pipeline plants every
payload encoded), `mount` (mount_to + boxed unmount handle = the island's lifetime), and
`mount_each` (the whole discovery loop; a closure returning `None` skips a malformed
placeholder — the decode-or-skip rule, stated once). Six of eight hydrators collapsed onto
`mount_each`; two stayed honestly bespoke: `hydrate_c4_embeds` (re-parents iframes into a
created host — forcing that through the kit would hide the wrap dance) and the
`description_pane` first-workbench extraction (works on `Element`, not `HtmlElement`). Both
still use `mount`.

## What moved

- Carriers now take `stores: IslandStores`: `hydrate_workbenches` (6→4 params),
  `hydrate_practices` (7→4), `PracticeProblem` (8→5 props), `RunnableBlock` (11→9),
  `description_pane` (8→5), `editorial_pane`/`approach_body`/`section_block`/
  `markdown_fragment`/`markdown_pane` (theme+codebench → stores at every hop).
- **Both `too_many_arguments` allows are gone** — the refactor's proof of work; clippy
  pedantic agrees nothing needs them.
- `RunnableBlock` destructures the bundle in its first line, so all ~10 interior uses of
  `auth`/`theme`/`viz_modal` compiled unchanged; its local `stores: Vec<BlockStore>` was
  renamed `block_stores` (4 lines) to free the name.
- Both capture hubs (reader, problem page) became `let stores = IslandStores::capture();`.
- Every converted file shrank: `hydrate.rs` 57→33, `practice.rs` 520→473, `problem.rs`
  587→580, `editorial.rs` 623→602, `quiz/mod.rs` 151→138, `diagrams.rs` 377→354,
  `viz/blocks.rs` 75→68, `fence_group.rs` 207→198, `reader.rs` 369→361. `runnable.rs`
  784→786 (rustfmt splits the destructure) — still under the 800 cap.

What a new widget author must know is now: plant a placeholder, write a `mount_each` closure,
append it to the hydrate chain — and if the widget needs App stores, take `IslandStores`.
The disposed-owner subtlety lives in one module instead of eight imitations.

## Verified

Gates: conventions (purity + caps), fmt, clippy pedantic `-D warnings`, `cargo test
--workspace` 458, vitest 83 — counts unchanged, as a behaviour-identical refactor should
leave them. Release bundle 660/700 KiB gz (was 659 — noise).

Live, one page per hydrator family, zero console errors across six navigations:

- **Gallery lesson**: 18/18 workbenches; Run → Accepted through real go-judge; Visualise →
  the modal traced 15 steps (`stores.viz_modal`); Monaco mounted dark and **re-themed live**
  on the header toggle (`stores.theme`, rgb(24,27,33) → light).
- **java-basics**: 3 practice widgets (editorial tabs lazy-mount, `✓ SOLUTION` viewer,
  Copy-to-editor logged "solution copied into the java tab"), 32/32 fence bars, 7 mermaid
  SVGs; Try in Editor opened the Codebench modal (`stores.codebench`).
- **flip-characters**: two-pane problem page; first workbench EXTRACTED to the right pane
  (0 left in the description); anonymous sign-in bar (`stores.auth`); Editorial = 9 sections
  + jump bar; Reveal → live read-only Monaco; the editorial's standalone `viz` fence mounted
  and played (two-pointer array, left/right cursors) — `mount_widgets` inside
  `markdown_fragment`.
- **delivery-framework**: 4 quiz cards, option → Check → verdict rendered.
- **google-docs**: 2 C4 iframes wrapped in `.c4-embed` with the Enlarge chrome.

One non-bug understood along the way: Copy-to-editor into a single-language block updates the
`BlockStore` but not the shiki preview — `switch_to(target)` early-returns when the target
tab is already active, so `wants_editor` never fires and lazy Monaco stays unmounted. That is
step-40's economy working as designed (the buffer is correct the moment the editor mounts),
and it predates this step.
