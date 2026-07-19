# Step 51 — Reading progress

*(a returning reader was indistinguishable from a first-time one.)*

## What was missing

Nothing persisted about reading. No completion, no streaks, no spaced repetition, and the
"progress" bar in the reader chrome was scroll offset — recomputed from zero on every page, kept
nowhere. The six localStorage keys were all cosmetic: theme, four reader preferences, the sidebar
face, the problem-pane tab, the workbench language.

So the library looked identical on the tenth visit and the first. Nine books, no indication of
which one you had started, and no route back to where you stopped.

## Anonymous, not authenticated

The instinct is a `lesson_progress` table behind sign-in. That is the wrong first move here, and
the reason is in the numbers rather than the design: signing in currently buys a bigger run
budget, submission history and an admin flag — and production runs with
`SUBMISSION_ALLOWLIST_ENFORCED=true`, so saving an attempt needs a manually granted row. A
progress feature behind that gate would reach almost nobody.

The reader who is actually here is anonymous. So progress is device-local, works with no account,
and costs the server nothing. If step 49's readership data ever shows returning signed-in
readers, a server-side store becomes worth building — but that is a decision to make with
evidence, not ahead of it.

## Two keys, not one packed record

`prefs.rs` packs four fields into one `|`-joined string parsed by an exact-arity slice pattern:

```rust
let [s, l, f, w] = parts.as_slice() else { return DEFAULT_PREFS };
```

A fifth field makes every previously-stored four-part value fall through to the defaults —
silently resetting the size, leading, family and width of every existing reader. There is a test
pinning exactly that behaviour, which is how the trap is known rather than suspected.

So progress gets its **own keys**: `reader-progress` (the finished set) and `reader-last` (where
to resume). Two keys, one job each, and a key that fails to read costs only itself.

The finished set is newline-separated because it is a **list**, not a fixed record. There is no
arity to get wrong, a blank or stray line is skipped rather than poisoning the rest, and lesson
paths are `/`-joined slugs so they can contain neither a newline nor a `|`. It is a `BTreeSet`,
so the serialised form is stable and does not churn on every commit.

## The decision that could not be tested where it lived

Auto-completion fires when the reader reaches the end. The obvious implementation is three
characters of arithmetic inside `ChromeState::recompute`, and it has two traps that both make it
silently never fire:

- **A lesson shorter than the viewport** has `track <= 0`, so `scroll / track` pins at `0.0`
  forever. There is nothing to scroll — which is precisely the case where it has all been seen.
- **Problem pages have no window scroll at all** (the panes scroll internally), so they never
  reach the recompute. They do not auto-complete, deliberately.

I could not verify either in a browser: the preview environment reports `innerHeight: 0`, so the
window cannot scroll and `scrollTo` is a no-op. Resizing did not help.

Rather than ship the load-bearing decision unverified, it moved out of the view layer into
`logic::progress::is_at_end(scroll, track)` — pure, and covered natively by `cargo test`. The
threshold is `0.98` rather than `1.0` because the last pixel is unreachable on many devices
(rubber-banding, sub-pixel rounding), and a threshold nobody can cross is a feature nobody has.
A non-finite ratio reads as "not laid out yet", not as "finished".

That is the useful shape of the constraint: an environment I could not test in pushed the logic
somewhere I could.

## Privacy is not optional here

`identity/state`'s erase path iterated one key:

```rust
const LOCAL_KEYS: [&str; 1] = ["reader-prefs"];
```

Reading progress had to join it. A font size is a preference of the *device* — the oracle
deliberately excluded the theme for that reason — but what someone has read is theirs, and
"erase all my data" has to mean it. `erase_all_data` reloads after clearing, so the in-memory
signals rebuild from empty storage with no extra wiring.

`crate::storage` gained a `remove()` for this; it had only `get` and `set`.

## What this deliberately does not do

**No quiz memory.** 251 quiz blocks still persist nothing. It belongs with this work, but it
needs a stable per-quiz identity that does not exist today — the hydration loop has a positional
index it currently discards, and an authored `id` means touching the fence contract and its
vitest suite. It is also arguably a reversal of a documented design statement (`quiz/mod.rs`:
"ungraded prose furniture, not submissions"), which deserves its own step rather than a
paragraph in this one.

**No manual "mark as done" control.** Auto-completion covers prose lessons; problem pages get
nothing, and the honest answer for a problem is that it is finished when it is *solved*, which
means wiring this to submissions rather than to a button.

**No streaks, no daily goals, no spaced repetition.** Those are retention mechanics for a product
with an audience. This is the smallest thing that makes a second visit different from a first.

**Nothing on the server.** Revisit when step 49 says returning signed-in readers exist.

## Verified

Ten pure tests in `logic::progress`, including both auto-complete traps and the not-laid-out
case. In-browser at :5373 against real content, seeding the set directly because the scroll path
is not exercisable here:

```
sidebar          4 links, 2 marked done — correct titles, ✓ glyph, --status-ok colour
library chip     "2/4 read" on the one started book; no chip on untouched books
completed book   "4/4 read", --all variant, status-ok tint
resume card      → /synapse/synapse-features/reading-a-lesson/reading-a-lesson
                   "Reading a Synapse Lesson" · "Synapse Features"
after erase      0 chips, no resume card, both keys gone — clean pre-feature state
console          no errors
```

The resume card resolves its title from the index rather than storing it: a stored title goes
stale the moment a lesson is renamed, and the path is the only thing that has to stay true.

Also confirmed in passing, from step 50: `document.title` now tracks SPA navigation —
`Synapse Features · Reading a Synapse Lesson — Synapse` on a client-side route change.

433 rust (+10) + 74 vitest. Critical path 646/700 KiB gz (+9).

## The lesson

**When the environment cannot test the code, move the code.** The scroll threshold was three
characters in a view function, and the preview's zero-height viewport meant no browser check
could ever reach it. The reflex is to note it as unverified and move on. Extracting it to a pure
function took a few minutes and turned the one genuinely load-bearing decision in the feature —
plus both of its silent-failure traps — from "read carefully and hoped" into three tests that run
on every push.
