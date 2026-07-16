# Step 25 — The design system + dark mode: the token layer, at last

*(oracle: the ADR-S015/S018 token system + `Theme.scala` + the header chrome — the visual
parity the RS client had been deferring behind simplified placeholder CSS. Also a hard
prerequisite: the upcoming viz renderers map wire hexes to `--viz-role-*` theme tokens.)*

## The tokens (`styles/tokens.css`)

The oracle's palette VERBATIM: HSL channel triplets (`45 44% 96%`, no `hsl()` wrapper),
consumed as `hsl(var(--token))` — `:root` is light, **`.dark` on `<html>`** is dark (the
shadcn convention, NOT `data-theme`), so one rule works in both themes. Surfaces
(`--background`/`--card`/`--popover`), the teal `--primary` pair, muted/accent/destructive,
the run-status pair (`--status-ok/err` + their bg tints — the dark theme deliberately
re-declares only the TEXT halves), borders/input/ring, `--radius: 0.875rem` with derived
md/sm, the `--syn-*` syntax palette, and the theme-aware **`--viz-role-*`** pointer palette
(light + brightened dark) the renderers will consume. Plus the two load-bearing base rules:
the universal `border-color: hsl(var(--border))` reset (Tailwind v4 defaults borders to
`currentColor`) and the body bg/fg/font.

## Fonts, the bootstrap, the store

`index.html` gains the oracle's Google Fonts link (Noto Sans + Literata + JetBrains Mono)
and the PRE-PAINT bootstrap verbatim: stored `"theme"` wins, else the OS preference, toggled
onto `<html>` before first paint — no wrong-theme flash (the step-19 CSP already allows the
inline script by design). `ThemeStore` mirrors the live class into a signal (seeded FROM the
class, so the first toggle-icon paint is right), and `set()` does class → storage → signal
in one breath.

## The chrome and the sweep

The header becomes the oracle's polished bar: fixed, translucent
(`hsl(var(--background)/0.85)` + blur), the brand chip (the 3-node graph SVG on a primary
tile — recolors through the tokens) + mono wordmark, the centred ⌘K search affordance, then
Blog · the account chip · the **theme toggle** (lucide sun/moon on the shadcn outline icon
button). Every component stylesheet swept onto the tokens — shell/library/reader (out of
index.html into `shell.css`), runnable, blog, cmdk, account, coach, markdown prose — with
the two deliberate exceptions kept exactly as the oracle ruled: **rendered code fences stay
a fixed dark slab in BOTH themes** (the shiki `--shiki-*` block, untokenized) and **authored
diagram cards stay fixed-light** (mermaid/d2 render light; a white card keeps their labels
legible on a dark page).

## Monaco

Monaco paints its own canvas, not the CSS cascade: the island's `synapse-light`/`dark`
themes are picked at mount from the live theme (threaded as a PROP through hydration — the
step-17 out-of-tree lesson applies to the theme store too), and an effect re-themes every
mounted editor on toggle (`monaco.editor.setTheme` is global + idempotent).

## Verified live

Booted with no stored theme on a dark-OS machine → painted dark immediately (no flash).
Toggle → light: the warm cream library with white cards, teal chrome, moon icon;
`localStorage.theme = "light"` persisted. Problem page → toggle → dark: prose light-on-dark,
inline-code chips muted, the code slab unchanged (fixed dark), **Monaco re-themed live to
`#181b21`**, the runnable card on `--card`. Suite: 312 Rust + 40 vitest; 450/700 KiB gz.

Next: the adapt pipeline + the 16 cortex-goldens — the heart of RS-P7.
