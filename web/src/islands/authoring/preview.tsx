// The live preview — the quality gate this whole feature turns on. It renders the edited buffer
// through the EXACT pipeline the reader page uses, into the EXACT DOM the reader hydrates, so
// "did my table render?" and "did my mermaid diagram parse?" are questions the contributor
// answers themselves, before a reviewer's time is spent on them.
//
// Three things make the preview the page rather than an approximation of it, and all three reuse
// what the reader already runs:
//   1. the same markdown pipeline — `renderLesson` from lib/markdown/render, lazily imported;
//   2. the same DOM + stylesheets — `.lesson > .lesson-header + .lesson-body.synapse-prose`, and
//      the edit page imports the reader's stylesheet set;
//   3. the same hydrators, scoped to THIS container — every hydrator takes a ParentNode root,
//      which is how islands/problem scopes its own pass.

import { render, h } from "preact";

import * as log from "../../lib/log";
import { splitFrontmatter, titleOf, summaryOf } from "../../lib/markdown/frontmatter";
import { VIZ_RESCAN } from "../workbench/contracts";

/** The header the reader shows above the body — eyebrow is omitted here (the preview has no book
 *  context), title and lede come from the EDITED frontmatter so a `title:` change shows at once. */
function renderHeader(host: HTMLElement, source: string): void {
  const title = titleOf(source) ?? "Untitled";
  const summary = summaryOf(source);
  host.replaceChildren();
  const h1 = document.createElement("h1");
  h1.className = "reader-prose__title";
  h1.textContent = title;
  host.append(h1);
  if (summary) {
    const lede = document.createElement("p");
    lede.className = "reader-prose__lede";
    lede.textContent = summary;
    host.append(lede);
  }
}

let vizRequested = false;

/**
 * Render `source` into the preview panes and hydrate the result. `header` and `body` are the two
 * hosts the edit page lays out; both are filled here so the preview matches the reader's header +
 * body structure exactly.
 */
export async function renderPreview(header: HTMLElement, body: HTMLElement, source: string): Promise<void> {
  renderHeader(header, source);

  // Only the body below the frontmatter is prose — the reader strips the fence before rendering,
  // and so must the preview, or the raw `--- title: … ---` block shows up as text.
  const { body: markdown } = splitFrontmatter(source);
  const { renderLesson } = await import("../../lib/markdown/render");
  const html = await renderLesson(markdown);
  body.innerHTML = html;

  await hydrate(body);
  log.debug("preview: rendered and hydrated");
}

let codebenchMounted = false;

/** Run the reader's own hydrators, scoped to the preview body — never the document-wide pass, so
 *  no page-level singletons fire. Every hydrator is imported by its own MODULE, not through
 *  `islands/widgets`, whose import would trigger that module's whole-document auto-hydration. */
async function hydrate(body: HTMLElement): Promise<void> {
  const [{ hydrateQuizzes }, { hydrateDiagrams }, { hydrateC4Embeds }, { hydrateWorkbenches }, { hydratePractices }] =
    await Promise.all([
      import("../widgets/Quiz"),
      import("../widgets/Diagrams"),
      import("../widgets/C4Embed"),
      import("../workbench"),
      import("../practice"),
    ]);

  hydrateQuizzes(body);
  hydrateDiagrams(body);
  hydrateC4Embeds(body, () => {
    /* the preview has no C4 docs panel — selecting a component is a no-op here */
  });
  hydrateWorkbenches(body);
  hydratePractices(body);
  // The "Try in Editor" modal is a body-mounted singleton — a fence-group button in the preview
  // opens it exactly as on the real page. Mounted from its component directly (not through
  // `islands/widgets`, whose import runs the whole-document pass), and only once.
  await mountCodebench();

  // ```viz fences plant `.viz-widget` placeholders that only the lazy wasm bundle mounts. Import
  // the loader the first time one appears, and on every later render nudge it to re-scan — the
  // marker-idempotent seam the crate already exposes, so a re-render never double-mounts.
  if (body.querySelector(".viz-widget")) {
    if (!vizRequested) {
      vizRequested = true;
      void import("../viz");
    }
    window.dispatchEvent(new Event(VIZ_RESCAN));
  }
}

async function mountCodebench(): Promise<void> {
  if (codebenchMounted) return;
  codebenchMounted = true;
  const { CodebenchModal } = await import("../widgets/Codebench");
  const host = document.createElement("div");
  document.body.appendChild(host);
  render(h(CodebenchModal, {}), host);
}
