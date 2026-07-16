# Step 32 — The landing tour: the library page becomes a guided tour

*(oracle: the post-33 landing arc — `SynapseTour.scala` + the final `LibraryPage`, commits
b24ae9d → 0d2a1f1; the FINAL design, where the carousel replaced the static hero.)*

## The hero

The landing's hero is now just the pill — a glowing dot + "A guided tour — everything
Synapse can do" — with the carousel as the centerpiece and two CTAs beneath: **Start
reading** (a button that smooth-scrolls to `#library-grid`, offset for the sticky header)
and **Read the blog**. The oracle's intermediate "First Principles Library / Read. Run.
Understand." headline and stats strip were overwritten before landing — they are not
ported (the chapter presents final design; so did the oracle's).

## The carousel

`SynapseTour`: four slides, auto-advancing every 7 s (paused while hovered, wrap-around),
dot + arrow navigation, the `NN / 04 — eyebrow` footer label. The slide view re-renders
exactly once per index change — a fresh visual per visit, stable while shown. Slides:

1. **The Library** — four live book cards (System Design · DSA · Low Level Design ·
   Programming Languages) whose hrefs resolve REACTIVELY through
   `find_book`/`first_lesson_path` once the catalog index loads (fallback `/` until then).
2. **Runnable code** — the hand-tokenized Python mockup (Python · Java tabs, ▶ Run, the
   doubled-list program, its output). The prose still says "Python, Java, SQL, Go and
   more" — the oracle's intentional tabs/prose mismatch, preserved.
3. **Find your way** — the reader mockup: chapter rail (Replication active), the
   Leaders & Followers page, pager, minimap.
4. **See it work** — a REAL `WidgetHost`, the same component every lesson uses, fed a
   hand-authored `VizCases`: "Reverse in place · two pointers", three steps over
   `[a,e,i,o,u]` with left/right cursors and per-step changed cells.

## The book grid

`lib-grid` cards replace the old list: a mono meta line (direct-chapter count · recursive
lesson count · ~minutes), title, description, up to three tags, and the Read → CTA;
category entries become full-width `lib-group` bands with a nested grid. A book with no
lessons renders as a dimmed non-link card. The nav math is pure catalog logic —
`find_book` (DFS by globally-unique slug), `lesson_count`, `chapter_count` — pinned by
native tests beside the existing `first_lesson_path` suite.

## Verified live

The pill copy, four dots, both CTAs, and the grid (8 cards under 2 category bands, "The
books") all render; the auto-advance had already stepped the tour by the time the first
check ran (proof the timer runs), hover-pause held slide 04 steady; slide 1's four cards
resolved to real first-lesson URLs; slide 2 showed Python · Java tabs, 5 lines, and
`[2, 4, 6, 8, 10]`; slide 4 played the real widget — 5 cells, transport `1 / 3`, the
authored title and step-1 caption. Suite: 349 Rust (+2 nav pins) + 44 vitest; bundle
557/700 KiB gz.

Next: the mobile navigation drawer + the LikeC4 fullscreen chrome.
