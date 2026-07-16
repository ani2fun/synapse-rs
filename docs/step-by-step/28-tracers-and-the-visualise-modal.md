# Step 28 — The tracers + the Visualise modal: live code becomes a playing widget

*(oracle: chapters 28–31's tracer/session/modal arc — `python-harness.py`, `java-harness.java`,
`TraceDecoder`, `TraceSession`, `VisualiseModal` + `SourcePane` + `FramesPanel`, ADR-S029/S031.)*

## The tracer islands, verbatim

The two harnesses are the oracle's, byte-for-byte, in `client/islands/tracer/` behind a new
`@tracer` alias: `python-harness.py` (a `sys.settrace` line tracer with the
`__SYNAPSE_USER_SOURCE_B64__` template slot; budgets: 600 steps · 400 objects · depth 60 ·
512 KB with quarter-drop-tail) and `java-harness.java` (recompiles the user's class INSIDE the
sandbox behind the `// __SYNAPSE_TRACER__` sentinel). `python.ts` / `java.ts` wrap a raw source
string into a traced program; the Rust side (`islands/tracer.rs`) is two thin async externs.
There is **no new endpoint** — a traced run is an ordinary `POST /api/run` whose stdout carries
the heap JSON between `__SYNAPSE_HEAP_BEGIN__`/`__SYNAPSE_HEAP_END__` markers.

## Decode → adapt → the same host

`viz::decoder` splits program output from trace (the **last** BEGIN wins — a user printing the
marker earlier only shifts the split left; `TruncatedOutput` when END is missing because the
budget clipped stdout) and walks the JSON into the shared `HeapTrace` shapes: list/tuple/array →
`Arr`, dict → `Dict`, anything else an `Instance`; `{"ref": id}` becomes a heap pointer.
`serde_json`'s `preserve_order` keeps locals/fields in tracer order — the frames panel shows
variables in the order the program bound them, like the oracle. From there the trace flows into
the **same** `adapt::adapt` the cortex-goldens pin and the **same** `WidgetHost` the authored
fences use — the modal is just a bigger room for the identical spine.

`viz::session` caches one `Session` per `(language, source, structure, root, stdin)` key: the
same code re-opens instantly, editing the buffer or switching the active test case re-traces,
and ↻ Re-trace forces a fresh run. Every failure lands as a `Failed` card that shows stderr,
else the compiler output, else whatever the program printed — never a blank modal.

## The button and the modal

A Python/Java variant whose fence carries a `viz=` hint (`render.ts` has forwarded it since
step 08; `Variant.viz` + `logic::can_visualise` gate it) grows a **Visualise** button between
Edit and Run. It snapshots the CURRENT buffer + the tests panel's stdin — you can edit the code
and trace YOUR version. `VizModalStore` is provided in `App` and threaded to the out-of-tree
blocks as a prop, the same rule as `auth` and `theme` (out-of-tree `mount_to` roots cannot reach
App context).

The modal: bar (◆ VISUALISE · the structure token · ↻ Re-trace · ✕), the case strip when the
trace segmented into several cases, then the two-column body — `WidgetHost` (driven by an
**external** playback signal, with the data-driven legend the inline widgets never show) beside
the read-only Monaco source pane (current/next line decorations following the stepper) and the
frames panel (per-frame fn + locals, the active frame carrying the `L<n>` chip, changed locals
tinted). Program output folds under a `<details>`. Esc closes, Space toggles play, ←/→ step.
`WidgetHost` grew the two optional props (`external`, `legend`) for this — the inline path is
untouched. Diff-mode stops, the timeline drawer, deep links, and the guide overlay join with the
wrap step, exactly as the oracle staged them.

## Verified live

On the Visualisation Gallery lesson (18 hinted blocks): the array trace ran 15 steps —
transport `1 / 15`, six cells appearing at step 2, `arr list [5, 2, 8, …]` in the frames panel,
the legend showing exactly the cues the trace uses (no "removed" row for a reverse); the BST
insert traced 82 steps and ended at 7 nodes / 6 edges with `<module> L17` active; the BFS graph
traced 39 steps to 8 nodes / 8 edges on the seeded force layout. Keyboard: ←/→ stepped, Space
played at the 900 ms cadence, Esc closed; re-opening the same block hit the session cache
instantly. The Java harness round-tripped through `/api/run` (in-sandbox recompile, Accepted,
markers present) — its UI seat arrives with the language switch. Suite: 329 Rust + 40 vitest;
557/700 KiB gz.

Next: the bespoke widget gallery — the six `None` families get their HTML renderers.
