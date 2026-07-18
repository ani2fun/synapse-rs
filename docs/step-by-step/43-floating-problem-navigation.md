# Step 43 — Floating problem-page navigation

*(the other half of step 42: the right controls, in the wrong corner — and the reader chrome
that should never have been on a problem page at all.)*

## The corner was wrong

Step 42 put Contents, Prev and Next in the crumb row's top-right, because that is where the
empty space was. Empty space is not the same as reachable space. You work a problem page
top-to-bottom and inside the panes; the top-right is the furthest point from both the reader's
eye and the mouse, and it is the one corner nothing else on the page uses — which is exactly
why it looked available.

They are now floating pills in the **bottom** corners: Contents above Prev on the left, Next on
the right, 56px apart on the same rhythm the prose reader's right-edge FAB stack already uses.
The crumb row goes back to being breadcrumbs.

## Which required clearing the corners

Those corners were not actually free. The prose reader's chrome was rendering on problem pages
— and had been since step 37, independently of anything step 42 did. `ReaderPage` mounts its
FABs as siblings of the lesson body, so they never learned that the body underneath them had
been replaced by a two-pane workbench.

A problem page has no window scroll (the panes scroll internally) and no sidebar column, so:

- **FocusFab** was visible and inert. Focus mode hides reader chrome a problem page does not
  have. This is the one the report named.
- **ScrollTop** and **MiniMap** were dead. Both key off a window scroll that never happens, so
  one sat permanently `--hidden` and the other permanently empty.
- **the sidebar-restore button** was a live bug rather than clutter. `SidebarMode` persists, so
  hiding the sidebar on a prose lesson and then opening a problem left a floating "Show the
  sidebar" control for a sidebar with no column to come back to.
- **the mobile drawer FAB** is now redundant: the Contents pill opens the same drawer at every
  width, and both wanted the same bottom-left corner.

All five are gated on `kind != problem` now. `TocFab` had guarded itself since step 37, and that
is precisely what made the omission easy to miss — the guard existed, in the same file, and had
simply never been extended to its four neighbours. A single correct instance reads as a
convention being followed.

## Small things

On phones the step labels drop to their arrows, so three pills do not cover the panes they exist
to help you leave. Contents keeps its word: a lone icon there reads as a menu of unknown
contents, which is the one thing a Contents button must not be.

The pills stay solid teal. They are the page's primary way out, and an outlined control reads as
disabled chrome sitting beside a live Run button.

## Verified live

At 1280px: all seven reader controls absent or `display: none` on a problem page; the three pills
at exactly `left:20/bottom:76`, `left:20/bottom:20` and `right:20/bottom:20`; Contents opened the
drawer with all 33 DSA entries and closed it; Next moved to Pattern 3 with the workbench
remounted and the pills re-pointed at pattern-02/pattern-04. Prose lessons keep every control and
gained no `.pwb-fab`. At 375px the arrows collapse and clear each other.

379 rust + 74 vitest. Critical path 636/700 KiB gz.

## The lesson

**One guarded instance is not a convention.** `TocFab` checked `is_problem` and its four
neighbours did not, and the review that added the check saw a correct file rather than an
incomplete pattern. When a component learns about a new page kind, the question is not "does this
one handle it" but "who else is mounted in the same breath" — the siblings are where the gap
lives.
