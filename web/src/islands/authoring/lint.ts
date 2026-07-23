// The lint strip under the editor, and the gate on the review dialog's Submit button.
//
// A contributor who cannot read a diff also cannot always tell WHY a page renders wrong. These
// checks catch the handful of mechanical mistakes that break a page for every reader — an
// unclosed fence, lost frontmatter — before a reviewer's time is spent on them. Two of them
// (`frontmatter-lost`, `title-lost`) mirror EXACTLY what the server's `validate` refuses, so the
// client never lets someone submit a change the API will 400; the rest are warnings that inform
// without blocking.
//
// Pure over the source text — no DOM, no render — so it is cheap to run on every keystroke and
// covered by plain vitest.

import { splitFrontmatter, titleOf } from "../../lib/markdown/frontmatter";

export type Severity = "error" | "warning";

export interface Finding {
  readonly severity: Severity;
  /** 1-indexed source line, or 0 for a whole-file finding (e.g. a lost fence). */
  readonly line: number;
  readonly message: string;
}

/** The languages the sandbox can run — a ```lang run fence naming anything else will not run. */
const RUNNABLE_LANGUAGES = new Set([
  "python",
  "java",
  "scala",
  "c",
  "cpp",
  "c++",
  "go",
  "rust",
  "kotlin",
  "typescript",
  "javascript",
  "sql",
]);

/** Every finding, most severe first. An empty list means nothing is blocking submission. */
export function lint(original: string, source: string): Finding[] {
  const findings: Finding[] = [
    ...frontmatterFindings(original, source),
    ...fenceFindings(source),
    ...headingFindings(source),
  ];
  // Errors first, then by line, so the strip reads top-down within each severity.
  return findings.sort((a, b) => rank(a.severity) - rank(b.severity) || a.line - b.line);
}

/** Whether any finding blocks submission. */
export function hasBlocker(findings: Finding[]): boolean {
  return findings.some((f) => f.severity === "error");
}

function rank(severity: Severity): number {
  return severity === "error" ? 0 : 1;
}

// ── the server-mirroring errors ───────────────────────────────────────────────

function frontmatterFindings(original: string, source: string): Finding[] {
  const had = splitFrontmatter(original).hasFence;
  const has = splitFrontmatter(source).hasFence;
  if (had && !has) {
    return [
      {
        severity: "error",
        line: 1,
        message:
          "The frontmatter block (the '---' section at the very top) is missing or unclosed. " +
          "It carries the page title and summary — restore it before submitting.",
      },
    ];
  }
  if (titleOf(source) === null) {
    return [
      {
        severity: "error",
        line: 1,
        message: "The page has no title. Add a 'title:' in the frontmatter or a '# ' heading.",
      },
    ];
  }
  return [];
}

// ── fence hygiene ─────────────────────────────────────────────────────────────

/** An odd number of ``` fences means one never closed — everything after it renders as code. */
function fenceFindings(source: string): Finding[] {
  const lines = source.split("\n");
  let open: number | null = null;
  const findings: Finding[] = [];
  lines.forEach((line, i) => {
    if (/^\s*```/.test(line)) {
      if (open === null) open = i + 1;
      else open = null;
    }
  });
  if (open !== null) {
    findings.push({
      severity: "error",
      line: open,
      message: "This code fence (```) is never closed — the rest of the page will render as code.",
    });
  }
  findings.push(...runFenceLanguageFindings(lines));
  return findings;
}

/** A ```lang run fence naming a language the sandbox does not know will render but never run. */
function runFenceLanguageFindings(lines: string[]): Finding[] {
  const findings: Finding[] = [];
  let inFence = false;
  lines.forEach((line, i) => {
    const opener = /^\s*```(\S+)?(.*)$/.exec(line);
    if (!opener) return;
    if (inFence) {
      inFence = false;
      return;
    }
    inFence = true;
    const lang = (opener[1] ?? "").toLowerCase();
    const meta = opener[2] ?? "";
    const isRun = /(?:^|\s)run(?:$|\s)/.test(meta);
    if (isRun && lang !== "" && !RUNNABLE_LANGUAGES.has(lang)) {
      findings.push({
        severity: "warning",
        line: i + 1,
        message: `'${lang}' is marked runnable but the sandbox cannot run it — the Run button will not appear.`,
      });
    }
  });
  return findings;
}

// ── heading structure ─────────────────────────────────────────────────────────

/** A heading that jumps a level (## straight to ####) reads oddly and breaks the on-this-page
 *  outline. A warning, not a block — some authored pages do it on purpose. Fenced code is
 *  skipped so a `#` comment inside a block is not mistaken for a heading. */
function headingFindings(source: string): Finding[] {
  const findings: Finding[] = [];
  let previous = 0;
  let inFence = false;
  source.split("\n").forEach((line, i) => {
    if (/^\s*```/.test(line)) {
      inFence = !inFence;
      return;
    }
    if (inFence) return;
    const heading = /^(#{1,6})\s/.exec(line);
    if (!heading) return;
    const level = heading[1].length;
    if (previous !== 0 && level > previous + 1) {
      findings.push({
        severity: "warning",
        line: i + 1,
        message: `This heading jumps from level ${previous} to ${level} — add the missing level in between.`,
      });
    }
    previous = level;
  });
  return findings;
}
