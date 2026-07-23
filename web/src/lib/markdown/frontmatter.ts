// The frontmatter splitter, in TypeScript — the client half of the server's
// `platform::frontmatter::fields_and_body`. The preview needs the lesson's `title`/`summary` out
// of the buffer being edited (the reader's header shows them above the body), and the lint needs
// to know whether a fence exists at all.
//
// This MUST agree with the Rust splitter, or a contributor sees a title in the preview that the
// server then rejects, or vice versa. Its tests mirror the Rust cases so the two cannot drift —
// see frontmatter.test.ts and server/src/catalog/domain/frontmatter_tests.rs.
//
// The leniency contract, verbatim from the Rust side: a fence exists ONLY when the first line is
// `---` and a closing `---` follows; anything malformed degrades to "no fence, the whole content
// is the body". Missing metadata never breaks a page.

export interface Frontmatter {
  /** `key: value` pairs from the fence, quotes stripped. Empty when there is no fence. */
  readonly fields: ReadonlyMap<string, string>;
  /** The content below the fence — or the whole content when there is no fence. */
  readonly body: string;
  /** Whether a well-formed fence was found. */
  readonly hasFence: boolean;
}

/** Split content into its fence fields and its body. */
export function splitFrontmatter(content: string): Frontmatter {
  const lines = content.split("\n").map((l) => (l.endsWith("\r") ? l.slice(0, -1) : l));
  if (lines[0]?.trimEnd() !== "---") {
    return { fields: new Map(), body: content, hasFence: false };
  }
  const end = lines.findIndex((line, i) => i >= 1 && line.trimEnd() === "---");
  if (end === -1) {
    return { fields: new Map(), body: content, hasFence: false };
  }

  const fields = new Map<string, string>();
  for (const line of lines.slice(1, end)) {
    const idx = line.indexOf(":");
    if (idx <= 0) continue; // no colon, or a colon in column 0 — neither is a field
    const key = line.slice(0, idx).trim();
    const value = stripMatchingQuotes(line.slice(idx + 1).trim());
    if (value !== "") fields.set(key, value);
  }
  return { fields, body: lines.slice(end + 1).join("\n"), hasFence: true };
}

/** The lesson's title: frontmatter `title:`, else the first `# ` heading — the SAME order the
 *  catalog resolves it in, so the preview's title matches what the page will show. `null` means
 *  the page would render with no title, which is exactly what the server refuses. */
export function titleOf(content: string): string | null {
  const { fields, body } = splitFrontmatter(content);
  const fromFence = fields.get("title");
  if (fromFence && fromFence.trim() !== "") return fromFence.trim();
  for (const line of body.split("\n")) {
    if (line.startsWith("# ")) {
      const heading = line.slice(2).trim();
      if (heading !== "") return heading;
    }
  }
  return null;
}

/** The lesson's one-line summary (the reader's lede), or `null`. */
export function summaryOf(content: string): string | null {
  const summary = splitFrontmatter(content).fields.get("summary");
  return summary && summary.trim() !== "" ? summary.trim() : null;
}

/** Strip a single pair of matching quotes, mirroring the Rust `strip_matching_quotes`. */
function stripMatchingQuotes(value: string): string {
  if (value.length >= 2) {
    const first = value[0];
    if ((first === '"' || first === "'") && value[value.length - 1] === first) {
      return value.slice(1, -1);
    }
  }
  return value;
}
