// The diff drives the review dialog's "Changes" step — a contributor who cannot read a GitHub
// diff sees exactly what they are proposing here, so the counts and the row kinds must be right.
import { describe, expect, it } from "vitest";

import { diffLines } from "./diff";

describe("diffLines", () => {
  it("reports no change for identical text", () => {
    const diff = diffLines("a\nb\nc\n", "a\nb\nc\n");
    expect(diff.added).toBe(0);
    expect(diff.removed).toBe(0);
    expect(diff.rows.every((r) => r.kind === "unchanged")).toBe(true);
  });

  it("reports a pure insertion", () => {
    const diff = diffLines("a\nc\n", "a\nb\nc\n");
    expect(diff.added).toBe(1);
    expect(diff.removed).toBe(0);
    expect(diff.rows.find((r) => r.kind === "added")?.text).toBe("b");
  });

  it("reports a pure deletion", () => {
    const diff = diffLines("a\nb\nc\n", "a\nc\n");
    expect(diff.added).toBe(0);
    expect(diff.removed).toBe(1);
    expect(diff.rows.find((r) => r.kind === "removed")?.text).toBe("b");
  });

  it("reports a replaced line as a removal then an addition", () => {
    const diff = diffLines("a\nold\nc\n", "a\nnew\nc\n");
    expect(diff.added).toBe(1);
    expect(diff.removed).toBe(1);
    const kinds = diff.rows.map((r) => r.kind);
    expect(kinds).toEqual(["unchanged", "removed", "added", "unchanged"]);
  });

  it("carries line numbers on the side each row belongs to", () => {
    const diff = diffLines("keep\ndrop\n", "keep\nadd\n");
    const removed = diff.rows.find((r) => r.kind === "removed");
    const added = diff.rows.find((r) => r.kind === "added");
    expect(removed?.oldLine).toBe(2);
    expect(removed?.newLine).toBeUndefined();
    expect(added?.newLine).toBe(2);
    expect(added?.oldLine).toBeUndefined();
  });

  it("normalises CRLF so a line-ending change alone is not a diff", () => {
    const diff = diffLines("a\nb\n", "a\r\nb\r\n");
    expect(diff.added).toBe(0);
    expect(diff.removed).toBe(0);
  });
});
