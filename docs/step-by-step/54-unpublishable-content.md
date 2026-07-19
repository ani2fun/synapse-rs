# Step 54 — The books that were never meant to ship

*(two of them were being served the whole time, and only git was stopping them reaching production.)*

## What I got wrong first

The product assessment listed this as a documentation task: 310,000 words adapted from a
commercial course, gitignored, *"inert today — record the decision so it cannot drift."* An
hour's work, no code.

That description was wrong, and checking rather than trusting it is what turned an hour of
paperwork into a real fix.

`local-only/` sits **inside the content root the server walks**. The walker excludes names
beginning `_` or `.`, plus two reserved aux dirs (`examples`, `c4`). `local-only` is none of
those. So it was indexed, exactly like any other content:

```
$ curl /api/synapse/index
  ...
  local-only/system-design-swiftly   ← 66 lessons, "adapted from Hello Interview's
  local-only/sql                        'System Design in a Hurry'"
```

Nine books, not seven. Served at real URLs on every developer machine. And two things shipped
this week made that worse without anyone noticing: step 50's `/sitemap.xml` enumerates every
lesson via `all_books()`, and step 49 records views of them into `lesson_view`.

## Why the protection was in the wrong place

The separation rested entirely on a `.gitignore` rule in the **content** repository. Three
things wrong with that, and they compound:

- **Wrong repo.** The rule is in `synapse-content`; the code deciding what to serve is here.
  Neither knows about the other.
- **Wrong layer.** It governs what is *committed*, not what is *served*.
- **Wrong moment.** It applies at push time. The only thing between those lessons and production
  was that nobody had yet run the wrong git command.

And the wrong command is an ordinary one. `git add -f`, or a blanket `git add -A` — which this
project has already been bitten by; the step-42 note records a stray gitlink reaching public
`main` exactly that way — or any restructure moving a book out of that folder. None of them look
like a mistake while you are doing it. The failure only becomes visible once a crawler has the
URLs, at which point the sitemap has been advertising them.

## The fix is one line, in machinery that already existed

```rust
const RESERVED_AUX_DIRS: [&str; 3] = ["examples", "c4", "local-only"];
```

The check runs order-prefix-stripped, so `01-local-only` is excluded too — adding an ordering
prefix must not quietly republish 66 lessons. That is the second test.

The `_`-prefix rule is an independent second route to the same outcome: renaming to
`_local-only` would also work. Both are kept. One is a name anyone might change; the other is a
decision recorded in code with tests attached.

Reasoning recorded in [ADR-RS002](../adr/rs002-derivative-content.md) — which also gives
`docs/adr/` its second entry, and states the thing that matters most: **if this material is ever
wanted publicly, the answer is to rewrite it from primary sources, not to relax the rule.**
Adapting someone else's course was the wrong shape to begin with; the fix is different content,
not different plumbing.

## What this deliberately does not do

**It does not touch the content repository.** The `.gitignore` rule stays as defence in depth,
and the directory keeps its name. This is a decision about what this server publishes, and it
belongs in the server.

**It does not add a general exclusion mechanism.** `RESERVED_AUX_DIRS` now mixes two kinds of
thing — aux dirs a book carries, and content that must not ship. That is a small semantic smudge,
accepted knowingly: a second mechanism would be more machinery than one directory justifies, and
the constant's comment says which entry is which.

**It does not try to detect derivative content.** There is no heuristic here and there should not
be. A human decided these two books do not ship; the code enforces that decision, it does not
make it.

## Verified

Against the real content tree, before and after:

```
before   9 books   local-only/system-design-swiftly and local-only/sql both in /api/synapse/index
after    7 books   0 local-only URLs in /sitemap.xml (263 total)
                   GET /api/synapse/local-only/system-design-swiftly/... → 404
```

Two unit tests pin it: a well-formed book inside `local-only/` yields no catalog entry *and* no
reachable lesson path, and the `01-`prefixed variant is excluded as well.

The conventions gate then caught something worth keeping: `walker_tests.rs` hit 532/500. Rather
than trim the reasoning out of the new tests, the file split along a real seam —
`walker_exclusion_tests.rs` now holds every test about what the walker *refuses* to index, which
is a coherent theme rather than an arbitrary halving. Two three-line fixture helpers are
duplicated across the two files; a shared `mod common` between siblings would be more structure
than they earn.

435 rust (+2) + 74 vitest + 7 e2e.

## The lesson

**A rule that lives in a different repository from the code it governs is a convention, not a
constraint.** The gitignore was doing real work and doing it correctly — right up until the
moment someone typed a normal git command. What made it fragile was not the rule but its
distance from the thing it protected: a different repo, a different layer, a different point in
time. Moving it into the walker made it survive all three, and cost one line and two tests.

It is also the third time in three steps that the failure shape was *a protection that could not
actually fire* — a guard that panicked instead of returning, a gate whose exit code was
swallowed, and now a rule enforced a repository away. Worth watching for.
