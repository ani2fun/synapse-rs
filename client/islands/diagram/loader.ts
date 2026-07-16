// ──────────────────────────────────────────────────────────────────
// DIAGRAM LOADER
// tiny dynamic-import gateway so mermaid lands in its own chunk
// ──────────────────────────────────────────────────────────────────
// Same trick as @editor/loader and @markdown/loader: the wasm side imports
// THIS module (tiny, eager), and the dynamic import() below makes Vite split
// mermaid.ts + mermaid into an on-demand chunk, fetched once when the first
// mermaid diagram mounts and cached after.
//
// Oracle deviation, on purpose (same as @markdown): the oracle exports
// loadRenderMermaid() → Promise<fn>; a flat async call is the friendlier
// wasm-bindgen FFI shape, so the load-then-call is folded in here.

import type { renderMermaidInto } from "./mermaid";

type RenderMermaidFn = typeof renderMermaidInto;

let cached: Promise<RenderMermaidFn> | null = null;

/** Render `src` into `target` as an inline SVG; mermaid loads lazily on first call. */
export async function renderMermaid(target: HTMLElement, src: string): Promise<void> {
  if (!cached) cached = import("./mermaid").then((m) => m.renderMermaidInto);
  const render = await cached;
  await render(target, src);
}
