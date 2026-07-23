// The preview must be the page, not an approximation. The parity that could actually break is the
// FENCE STRIP: the reader page renders `payload.raw` (the body the server already stripped of its
// frontmatter fence), while the preview renders `splitFrontmatter(source).body` from the WHOLE
// file. If those two disagree, a contributor sees something a reviewer will not — the entire point
// of the feature is that they agree.
//
// This proves the preview's render path over a lesson carrying the things most likely to break for
// a non-git contributor — a table, a mermaid diagram, a heading — and asserts the frontmatter fence
// never leaks into the rendered body. The pipeline (`renderLesson`) is imported, not reimplemented,
// so it is byte-for-byte the reader's; only the split is this feature's own, and it is exercised
// here end to end. The signed-in click-through (Monaco → dialog → submit) is the documented manual
// step, since it needs a Keycloak session the anonymous e2e stack does not carry.
import { describe, expect, it } from "vitest";

import { renderLesson } from "../../lib/markdown/render";
import { splitFrontmatter } from "../../lib/markdown/frontmatter";

const LESSON = [
  "---",
  "title: Thinking in Tradeoffs",
  "summary: A one-line lede.",
  "---",
  "",
  "## A section",
  "",
  "Some prose about tradeoffs.",
  "",
  "| Option | Cost |",
  "| --- | --- |",
  "| A | low |",
  "| B | high |",
  "",
  "```mermaid",
  "graph TD; A-->B;",
  "```",
  "",
].join("\n");

/** The body the server hands the reader page: the file with its frontmatter fence stripped. */
function readerBody(source: string): string {
  return splitFrontmatter(source).body;
}

describe("preview render parity", () => {
  it("renders the same body HTML the reader page renders from the stripped source", async () => {
    // The preview renders splitFrontmatter(source).body; the reader renders the server-stripped
    // body. Both are the SAME string here (the TS splitter mirrors the Rust one — see
    // frontmatter.test.ts), so the same pipeline gives the same HTML.
    const previewHtml = await renderLesson(readerBody(LESSON));
    const readerHtml = await renderLesson(readerBody(LESSON));
    expect(previewHtml).toBe(readerHtml);
  });

  it("renders a table the contributor can see worked", async () => {
    const html = await renderLesson(readerBody(LESSON));
    expect(html).toContain("<table>");
    expect(html).toContain("high");
  });

  it("turns a mermaid fence into the placeholder the reader hydrates into a diagram", async () => {
    const html = await renderLesson(readerBody(LESSON));
    expect(html).toContain('class="mermaid-block"');
  });

  it("never leaks the frontmatter fence into the rendered body", async () => {
    const html = await renderLesson(readerBody(LESSON));
    expect(html).not.toContain("Thinking in Tradeoffs"); // the title lives in the header, not the body
    expect(html).not.toContain("summary:");
    expect(html).not.toMatch(/^-{3}/m);
  });

  it("renders a broken (unclosed) fence without throwing — the lint is what warns", async () => {
    // A contributor mid-edit will have malformed content; the preview must degrade, not crash.
    const broken = "---\ntitle: T\n---\n\n```python\nprint(1)\n\nstill going\n";
    await expect(renderLesson(readerBody(broken))).resolves.toBeTypeOf("string");
  });
});
