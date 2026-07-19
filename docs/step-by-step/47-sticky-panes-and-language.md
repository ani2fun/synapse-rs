# Step 47 — Sticky panes and a sticky language

*(a problem page that forgets everything you told it the moment you open the next one — and the
two preferences that fix it.)*

## The report

Two annotations on one screenshot, and they turn out to be the same defect:

> if I have selected Editorial and Solution pill, it should remain the same for next problem page
> as well. SO that user don't have to click every time

> Once selected it should remain same language selection for next problem pages or lesson pages

Work a chapter of problems and you re-click Editorial → Solution on every one of them, and a Java
learner is handed Python every single time. Neither is a rendering bug; both are the same missing
idea. `ProblemWorkbench` is rebuilt from scratch on every navigation — `reader.rs`'s
`{move || … <LessonBody/>}` disposes the previous body when the path memo changes — and every
selection lived in an `RwSignal` created in the component body. Nothing was persisted, so nothing
survived. The page was not forgetting; it had never been asked to remember.

## Two preferences, no store

The obvious move is a context store on the `PrefsStore` pattern. It is the wrong one here, for two
independent reasons.

Both language pickers are mounted **out of tree** via `mount_to` — `SolutionViewer` from
`mount_solutions`, `RunnableBlock` from `hydrate_workbenches` — and a fresh root owner cannot see
App context. That is the trap this codebase has hit before and now threads theme, auth, codebench
and viz-modal as props to avoid. Threading a fifth would have meant ~15 edited lines across five
files.

And nothing here is reactive. The read happens once when the component is created; the write
happens once when you click. A store buys reactivity and a single source of truth; this needs
neither. So both preferences follow the **`SidebarMode`** precedent instead — `parse` / `load` /
`persist` free functions over `localStorage`, no context, no props, no `App` wiring:

| Concern | Pure module | Accessor | Key |
|---|---|---|---|
| Language | `execution/logic/language.rs` | `execution/state/lang_pref.rs` | `wb-language` |
| Problem panes | `catalog/logic/pane.rs` | free fns in `catalog/state/` | `problem-pane` |

`storage.rs` is new and holds the one `get`/`set` pair. There were already five hand-rolled copies
of that four-line snippet in the client; this step declined to add a sixth and moved
`catalog/state` onto the shared one. Both calls swallow failure by design — Safari's private mode
makes `localStorage` *throw*, and a preference that cannot be saved must not take the page down.

## The section is remembered by label, never by index

The editorial's second pill row is built at runtime from whatever `##` headings the author wrote.
Index 1 is "Solution" on one problem and "Optimisation" on the next, so storing the index would
restore a different section every time and look like a bug. Storing the **normalised label**
(lowercased, trimmed, inner whitespace collapsed) is what actually carries: "Complexity Analysis"
means the same thing everywhere, and a problem that hasn't got it falls back to the first section.

That decision sets the record format. `problem-pane` is `tab|left_pct|section`, parsed with
`splitn(3, '|')` so the **free-text field is last** — a heading like `Two pointers | O(n)` is
perfectly legal markdown and would corrupt any record that put it in the middle. There is a test
for exactly that string.

## One canonical language table

`py`, `python3` and `Python` are the same language, and different pages spell it differently. A
preference stored as a raw fence alias would silently fail to match half the time, so
`canonical_lang` folds every alias onto one token and that token is what gets stored.

Which made the client's existing `RUNNABLE_FENCES` array redundant — a flat list of 21 aliases
whose only job was answering "is this runnable?", a question the new table answers better.
Deleting it collapsed three alias tables to two and fixed a real drift by construction: the flat
list had been missing `sqlite` since step 40, so a ` ```sqlite ` fence got no *Try in Editor*
button even though the server would happily run it.

## Where the initial index goes in

`RunnableBlock` assumed variant 0 in four places, and only one of them was the obvious one. Two
lines resolve it once:

```rust
let start = lang_pref::index_for(&variants);
let active = RwSignal::new(start);
code_sink.set((variants[start].source.clone(), variants[start].language.clone()));
let first = variants[start].clone();
```

Redefining `first` is the whole trick — it also carries the preference into the shiki placeholder
and into `default_height_px`, which had been sizing the editor from variant 0's source. The
editor's initial height now derives from the variant actually shown, which is what it always
should have been.

`preferred_index` is built from `Iterator::position`, so it is structurally in-bounds for any
inputs. That is load-bearing rather than incidental: `variant_at` indexes without clamping, unlike
its neighbour `store_at`, and a test asserts the invariant over the whole cross-product.

## The write is deliberately asymmetric

Read at mount; write **only** in the two dropdown click handlers. Never at mount, never in
`switch_to`, never on copy-to-editor — `switch_to` is also what "Copy to editor" calls, and
routing the write through it would let loading a solution silently rewrite your language.

What makes this robust is that the existing markup already enforces it: a single-variant block
renders a plain `<span>` pill instead of a dropdown, so on a Python-only page **there is no
writable control in the DOM**. A Java-preferring reader can cross any number of single-language
pages and the preference cannot be clobbered. Verified: `wb-language` still read `java` after a
Python-only lesson had rendered and fallen back to Python.

## Three guards that the naive version gets wrong

- **`subs_seen` must be seeded.** The Submissions pane is lazily gated on it, and it was only ever
  set by the click handler. Restoring that tab without seeding renders an empty pane.
- **Editorial can't be restored onto a problem that hasn't got one** — it would strand you on
  "No editorial yet for this problem." The restore falls back to Description.
- **The section restore is guarded on `start != 0`.** `RwSignal::set` notifies unconditionally, so
  setting 0 would re-run the reveal effect and fire a redundant `RELAYOUT_EVENT` on *every*
  editorial. The guard keeps the no-preference path byte-identical to before.

The section restore also has to sit at one specific point: inside the `spawn_local`, after
`sectionize_editorial` has created the `.pwb-esec` wrappers, and *before* `mount_solutions` runs.
Effects are queued, so by the time the reveal effect fires, the solution viewers have registered
their relayout listeners and the target section is already visible — its Monaco measures correctly
on the first try instead of mounting into a `display: none` box and waiting to be rescued.

The splitter persists on **release**, not on move: the `pointerup` listener is on the window and
fires for every click on the page, so it is gated on `dragging` rather than writing storage at
pointer rate.

## A bug this did not cause

Switching language *within* a page updates the toolbar pill but leaves Monaco's buffer on the
previous variant — `switch_to` finds `mounted` empty and skips the `set_value`. This looks like it
belongs to this step and does not: it reproduces identically on the commit before it, verified by
stashing the branch, rebuilding, and repeating the click. Navigation-restored language is
unaffected, because a fresh page mounts Monaco from `first`, which now honours the preference.
Left for its own step, where it can be diagnosed rather than patched around.

## Verified

Gates: conventions (`runnable.rs` lands at 784/800 — say plainly that it is now effectively frozen,
and the next change there splits the toolbar out first), fmt, clippy, 403 Rust tests (+17), 74
vitest.

In the browser, on real content: Java chosen on *If Else If* → **Switch Case** opens on Java with
the Java starter; Editorial + "Complexity Analysis" → the next problem opens on both, matched by
label at a different index; Submissions survives with a populated pane; the splitter drag persists
and a stored `999` clamps to 64%; corrupt values in both keys degrade to Description / 46% /
Python with a clean console; 45 fence groups still hydrate with *Try in Editor* on the Python one
and correctly absent on Bash and Toml.

Not verified live, and honestly so: the no-editorial fallback (every problem in the current
content has one) and the `sqlite` alias (no content uses that fence). Both are covered by unit
test only.
