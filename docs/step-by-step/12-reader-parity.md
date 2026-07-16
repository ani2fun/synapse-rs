# Step 12 — Reader parity: preferences, collapsible chapters, full-width problems

*(oracle: synapse steps 12–13 reader slices + the post-33 absorptions `7ed3909`
reading-preferences panel and `a95e3fb` full-width problem pages / collapsible categories —
`ReaderPrefs.scala` ported)*

## Reading preferences (the oracle's three-piece split)

- **Pure logic** (`catalog/logic/prefs.rs`, native-tested): four independent choices — size ·
  leading · family · width — each a small token allow-list, persisted as ONE `|`-joined string;
  a bad stored token degrades PER FIELD to its default (a corrupt localStorage entry must never
  break the reader).
- **State** (`PrefsStore`): provided in `App` so the stored prefs reflect onto `<html>` as
  `data-reader-*` attributes BEFORE the first paint — the choice survives navigation, applies to
  every reading surface, no flash. One `commit` writes signal + localStorage + attributes in the
  same breath.
- **View** (`ReaderPrefsFab`): the bottom-right `Aa` FAB → a popover of segmented controls
  (the family row previews itself in its own font), dismissed by Esc, scrim, or the FAB;
  reset-to-defaults. The dark-mode row joins with the theme step, as it did in the oracle.

The stylesheet half reads the attributes: three sizes (0.925/1/1.125 rem), three leadings,
serif/sans/mono stacks, and three column widths (38/46/60 rem) on `.lesson`.

## Collapsible sidebar chapters

The sidebar now renders the book's REAL interior tree instead of the flattened reading order:
chapters are `<details>` groups (nesting recursively), lessons are links with the fine-grained
`current` tracking from step 07. The chapter containing the current lesson renders open, so
navigation always lands unfolded — on the DSA book that's 33 collapsed chapters and exactly one
open.

## Full-width problem pages

`frontmatter.kind == "problem"` puts `lesson--problem` on the lesson wrapper and the column cap
comes off — the workbench needs the width. Found in verify: the width-pref selector carries
`html[attr]` specificity, so the override must outrank it explicitly (the CSS comment names the
trap).

## Verified

126 Rust + 40 vitest; clippy `-D warnings`; purity/caps/fmt/budget. In-browser: defaults reflect
on `<html>` at boot; picking Large/Serif/Wide updates attributes, localStorage
(`lg|normal|serif|wide`), computed styles (Georgia 18 px, 960 px column) live; prefs survive a
full reload; Esc/scrim dismiss; the flip-characters problem page spans full width with Monaco
still hydrating; the DSA sidebar collapses 33 chapters with the current one open; zero console
errors.
