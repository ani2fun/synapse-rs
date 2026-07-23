// The TS frontmatter splitter must agree with the server's `platform::frontmatter::fields_and_body`
// and the catalog's title resolution — the preview shows a title the server then accepts, or
// rejects, so a divergence would be a real bug. These cases mirror
// server/src/catalog/domain/frontmatter_tests.rs.
import { describe, expect, it } from "vitest";

import { splitFrontmatter, summaryOf, titleOf } from "./frontmatter";

describe("splitFrontmatter", () => {
  it("splits a well-formed fence from the body", () => {
    const fm = splitFrontmatter("---\ntitle: Hello\nsummary: A lede\n---\n\nBody prose.\n");
    expect(fm.hasFence).toBe(true);
    expect(fm.fields.get("title")).toBe("Hello");
    expect(fm.fields.get("summary")).toBe("A lede");
    expect(fm.body).toBe("\nBody prose.\n");
  });

  it("treats content with no opening --- as all body", () => {
    const fm = splitFrontmatter("# Just a heading\n\nprose");
    expect(fm.hasFence).toBe(false);
    expect(fm.body).toBe("# Just a heading\n\nprose");
    expect(fm.fields.size).toBe(0);
  });

  it("treats an unclosed fence as no fence (the whole thing is body)", () => {
    const fm = splitFrontmatter("---\ntitle: Hello\n\nbody with no closing fence");
    expect(fm.hasFence).toBe(false);
    expect(fm.body).toContain("title: Hello");
  });

  it("strips matching quotes but leaves mismatched ones", () => {
    expect(splitFrontmatter(`---\ntitle: "Quoted"\n---\n`).fields.get("title")).toBe("Quoted");
    expect(splitFrontmatter(`---\ntitle: 'Quoted'\n---\n`).fields.get("title")).toBe("Quoted");
    expect(splitFrontmatter(`---\ntitle: "Half'\n---\n`).fields.get("title")).toBe(`"Half'`);
  });

  it("ignores blank values and colon-in-column-zero lines", () => {
    const fm = splitFrontmatter("---\ntitle:\n: weird\nkind: problem\n---\n");
    expect(fm.fields.has("title")).toBe(false);
    expect(fm.fields.get("kind")).toBe("problem");
  });

  it("handles CRLF line endings", () => {
    const fm = splitFrontmatter("---\r\ntitle: Hello\r\n---\r\nbody\r\n");
    expect(fm.hasFence).toBe(true);
    expect(fm.fields.get("title")).toBe("Hello");
  });
});

describe("titleOf", () => {
  it("prefers the frontmatter title over an h1", () => {
    expect(titleOf("---\ntitle: From Fence\n---\n# From Heading\nbody")).toBe("From Fence");
  });

  it("falls back to the first h1 when the fence has no title", () => {
    expect(titleOf("---\nkind: problem\n---\n# From Heading\nbody")).toBe("From Heading");
    expect(titleOf("# From Heading\nbody")).toBe("From Heading");
  });

  it("is null when there is no title anywhere — the state the server refuses", () => {
    expect(titleOf("---\nsummary: only a lede\n---\n\njust prose, no heading")).toBeNull();
    expect(titleOf("just prose")).toBeNull();
  });
});

describe("summaryOf", () => {
  it("returns the summary field, or null when blank/absent", () => {
    expect(summaryOf("---\nsummary: A lede\n---\n")).toBe("A lede");
    expect(summaryOf("---\ntitle: T\n---\n")).toBeNull();
  });
});
