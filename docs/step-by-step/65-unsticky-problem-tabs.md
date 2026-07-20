# Step 65 — The tabs stop remembering

*(step 47 taught the problem page to carry your last tab across pages. Eighteen steps later, out
it comes.)*

## What it did, and why that was wrong

Step 47 read a screenshot annotation — "if I have selected Editorial and Solution pill, it should
remain the same for next problem page as well, so that user don't have to click every time" — and
built exactly that. The active tab and the active editorial section persisted in `localStorage` and
were restored on every problem page.

It worked. The complaint that killed it is not that it failed:

> Sometime users does not want to keep it selected the same tab across the pages.

Which is the part the original design never weighed. **Carry-over is the wrong default for a
control whose whole job is deciding how much help you want.** The Editorial tab holds the answer.
Restoring it means a problem you have not read yet opens on its solution — a spoiler nobody asked
for on this page, delivered because of a choice made on a different one. The reader who genuinely
wants the answer is one click away; the reader who does not has already had it.

That asymmetry is the whole argument. The cost of the feature falls on the person who did not want
it, and the cost of not having it is a single click.

So: **every problem opens on Description, at the top.**

## What stayed, and the line between them

Three other things on this page still carry across, and the sorting rule is whether the setting
describes *the room* or *your place in the material*:

| Kept | Why |
|---|---|
| The splitter width | How you arranged the room. Resetting a pane drag every page would be its own annoyance, and it reveals nothing. |
| The approach stepper (step 57) | Which approach you were reading is a position in the material — but it is only ever visible *after* you have chosen the Editorial tab yourself, so it cannot spoil a problem you have not opened. |
| The workbench language (step 47's other half) | Nothing to do with how much of the answer you see. |

The user was asked about the approach stepper specifically, since it is the closest sibling and
their rationale arguably covered it. They kept it.

## The record format shrank

`problem-pane` held `tab|left_pct|section`. It now holds a bare width. A stored step-47 record no
longer parses, so an existing reader's splitter resets exactly once.

That was a choice, not an oversight. The alternative — a branch that reads the middle field of a
legacy pipe record — would live in the codebase forever to save one drag. A test pins the legacy
string as an explicitly-unreadable input so the behaviour is stated rather than implied.

## Two things that nearly broke on the way out

Removing a feature you wrote yourself is more dangerous than removing someone else's, because you
recognise the code and stop reading.

**`normalize_label` and `section_index` are not mine to delete.** Both arrived in step 47 for the
section restore. Both have since been adopted: `logic::editorial` uses `normalize_label` for its
heading parser, and the approach memory — the thing this step deliberately *keeps* — resolves its
remembered label through `section_index`. Deleting the section restore's helpers would have taken
the approach stepper down with it.

**`restore_section` was doing two jobs.** Its name says one. It restored the remembered section
*and* distinguished the first render from an approach switch — and that second job is what decides
whether the reused scroll container gets reset. Leptos keeps the container's DOM node across an
approach re-render, so without the reset a switch lands you mid-document (step 57 found that live).
Only the first job went. The flag survives, renamed `first_body`, which is what it was actually
about.

A one-line rename is the tell: when a boolean needs renaming after you remove one of its uses, it
was overloaded, and the removal is the moment that becomes visible.

## Verified

470 rust + 83 vitest; conventions, fmt and clippy clean.

Live, replaying the reported scenario exactly: Editorial selected, the Solution jump pill clicked —
`problem-pane` stayed `null`, nothing written — then Next landed on **Description**. Across two
further navigations every page opened on Description while the splitter held 58.21% and the
language held Java, which is the split this step is for.

## The shape of this step

Two commits: the removal (`e4d3758`) and this chapter. The removal was already on public `main`
when the step was numbered, and squashing after the fact would rewrite published history — the same
call step 40 records. The tag marks the tip; `step-65` compiles and its tests pass.

## The lesson

**A feature request is evidence about one reader at one moment, not a specification.** The step-47
screenshot asked for carry-over and carry-over is what it got, with no one asking what it costs the
next reader — or the same reader in a different mood. The question the original design skipped is
the one that eventually removed it: *who pays when this is wrong, and how hard is it to undo by
hand?* Here the answer was "the person who wanted a fresh problem" and "one click" — a ratio that
should have been decided before the code, not eighteen steps after it.
