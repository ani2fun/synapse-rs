# Step 23 — The viz contract spine: VizGraph, the vocabulary, dispatch, colours, playback

*(oracle: synapse step 26 part 1 + step 25's `Playback` — the ADR-S026/S027 foundation the
whole RS-P7 phase builds on. Pure shared code: no IO, no DOM, no consumer yet.)*

## The render contract (`shared/src/viz/graph.rs`)

`VizCases` is the ubiquitous language every producer speaks: the live-trace adapter (the
adapt step) emits it; hand-authored ` ```viz widget= ` payloads decode INTO it — the modal
canvas and an inline widget are the same host consuming one type. It is an anti-corruption
WIRE contract: field names match the Cortex oracle's JSON exactly (`cardId`, `layoutKind`,
`fn`, `structureType`), so its goldens will compare directly in the adapt step.

The codec asymmetry is the load-bearing decision, mapped from circe to serde: serialization
is FAITHFUL (every field emitted, `None` as `null` — pinned by a dedicated test), while
deserialization is TOLERANT — a bare authored `{title?, steps}` wraps into a one-case
`VizCases`, a plain-string `annotation` lands in `.body` (custom `Deserialize` over an
untagged wire enum), and omitted fields default. Two deliberate strict points survive the
tolerance: a node without an `id` and a cursor without a `target` are loud decode errors.
`NodeId` is a newtype (`serde(transparent)`) — a string on the wire, never interchangeable
with other strings. `kind`/`layoutKind` stay strings: an OPEN renderer vocabulary.

## The vocabulary, dispatch, colours, playback

- **`vocabulary`** — the ONE authored table (ADR-S027): `viz=<structure>[:<root>]` over 17
  structures; `parse` keeps dotted roots (`list:self.head`), an unknown token is `None`
  (honest error card, never a silent guess); `layout()` maps each structure to its geometry
  family (Cells/Grid/Tree/Chain/Graph); `token()` round-trips through `from_name` for all 17.
- **`render_family`** — the pure half of dispatch, shared so the modal and inline widgets
  agree: 12 families (6 geometric SVG + the step-33 bespoke HTML chrome), and the match is
  EXHAUSTIVE — adding a structure forces a family (open/closed principle, compiler-enforced).
- **`markers`** — the role-based cursor palette ported field-for-field: `i`/`head`/`left`
  deep blue, `current` indigo, `j`/`previous` mulberry, `next` moss, `tail`/`fast`/`right`
  bordeaux; aliases (`cur`→`current`, `lo`→`low`); seven fallback hues by first appearance.
  The WIRE carries these canonical hexes (parity-pinned); the client maps them to theme
  tokens later.
- **`playback`** — the one pure stepper (Cortex had three): manual steps PAUSE, the timer
  tick stops at the end, and pressing play at the end REWINDS first. Deliberately named
  `Playback`, not FSM — `CodeExecutor` already owns that word (qna Q36).

## Tests

+34, mirroring the oracle suites case-for-case: codecs 5 (the three VERBATIM authored
payloads — array with string annotation, list with null slots, BST with omitted fields — the
adapter wire round-trip, and the faithful-encoder pin), vocabulary 7, render family 4
(exhaustive dispatch incl. the exactly-3-Cells count), markers 6, playback 11. Suite: 265
Rust + 40 vitest. No browser verify — nothing consumes the spine yet; the geometry families
and the widget host arrive next and draw with it.

Next: the geometry families — `Geometry` constants, `Layout::union` + the layout-once
invariant, linear/grid/tree/chain, and the seeded deterministic force layout.
