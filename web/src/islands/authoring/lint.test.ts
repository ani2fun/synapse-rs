// The lint's job is to catch, on the client, the mechanical mistakes that break a page for every
// reader — before a reviewer's time is spent on them. Two rules (lost fence, lost title) MUST
// agree with the server's `validate`, so those are asserted as blockers here.
import { describe, expect, it } from "vitest";

import { hasBlocker, lint } from "./lint";

const WITH_FENCE = "---\ntitle: Thinking in Tradeoffs\n---\n\nBody prose.\n";

describe("lint — server-mirroring blockers", () => {
  it("passes a normal edit", () => {
    const findings = lint(WITH_FENCE, WITH_FENCE.replace("Body prose.", "Sharper prose."));
    expect(hasBlocker(findings)).toBe(false);
  });

  it("blocks when a lesson that had frontmatter loses it", () => {
    const findings = lint(WITH_FENCE, "# Thinking in Tradeoffs\n\nBody prose.\n");
    expect(hasBlocker(findings)).toBe(true);
    expect(findings[0].severity).toBe("error");
  });

  it("blocks an unclosed frontmatter fence", () => {
    const findings = lint(WITH_FENCE, "---\ntitle: Still Here\n\nBody with no closing fence.\n");
    expect(hasBlocker(findings)).toBe(true);
  });

  it("blocks a proposal with no title left", () => {
    const findings = lint(WITH_FENCE, "---\nsummary: only a lede\n---\n\nBody with no heading.\n");
    expect(hasBlocker(findings)).toBe(true);
  });

  it("does not require a fence on a lesson that never had one", () => {
    const findings = lint("# Heading\n\nprose\n", "# Heading\n\nbetter prose\n");
    expect(hasBlocker(findings)).toBe(false);
  });
});

describe("lint — code fences", () => {
  it("flags an unclosed code fence as a blocker", () => {
    const source = "---\ntitle: T\n---\n\n```python\nprint(1)\n\nmore prose\n";
    const findings = lint(source, source);
    expect(hasBlocker(findings)).toBe(true);
    expect(findings.some((f) => /never closed/.test(f.message))).toBe(true);
  });

  it("accepts balanced code fences", () => {
    const source = "---\ntitle: T\n---\n\n```python\nprint(1)\n```\n\nprose\n";
    expect(hasBlocker(lint(source, source))).toBe(false);
  });

  it("warns (not blocks) a run fence in a language the sandbox cannot run", () => {
    const source = "---\ntitle: T\n---\n\n```haskell run\nmain = pure ()\n```\n";
    const findings = lint(source, source);
    expect(hasBlocker(findings)).toBe(false);
    expect(findings.some((f) => f.severity === "warning" && /haskell/.test(f.message))).toBe(true);
  });

  it("does not warn on a runnable language", () => {
    const source = "---\ntitle: T\n---\n\n```python run\nprint(1)\n```\n";
    const findings = lint(source, source);
    expect(findings.some((f) => /sandbox cannot run/.test(f.message))).toBe(false);
  });
});

describe("lint — heading structure", () => {
  it("warns on a heading that jumps a level", () => {
    const source = "---\ntitle: T\n---\n\n## Two\n\n#### Four\n";
    const findings = lint(source, source);
    expect(findings.some((f) => f.severity === "warning" && /jumps/.test(f.message))).toBe(true);
  });

  it("does not treat a # inside a code fence as a heading", () => {
    const source = "---\ntitle: T\n---\n\n## Two\n\n```python\n#### not a heading\n```\n\n### Three\n";
    const findings = lint(source, source);
    expect(findings.some((f) => /jumps/.test(f.message))).toBe(false);
  });
});
