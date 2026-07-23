// A dependency-free line diff for the review dialog's "Changes" step. A contributor who cannot
// read a GitHub diff still needs to SEE exactly what they are proposing before they submit it, so
// this renders the same added/removed/unchanged lines the pull request will show.
//
// Classic LCS over lines (Myers would be faster, but a lesson is a few hundred lines and this
// runs once when the dialog opens). The output is a flat row list the view maps to styled lines,
// plus the +added / -removed counts the header shows.

export type DiffKind = "added" | "removed" | "unchanged";

export interface DiffRow {
  readonly kind: DiffKind;
  readonly text: string;
  /** 1-indexed line number in the OLD file (absent for added lines). */
  readonly oldLine?: number;
  /** 1-indexed line number in the NEW file (absent for removed lines). */
  readonly newLine?: number;
}

export interface Diff {
  readonly rows: DiffRow[];
  readonly added: number;
  readonly removed: number;
}

export function diffLines(oldText: string, newText: string): Diff {
  const a = splitLines(oldText);
  const b = splitLines(newText);
  const rows = walk(a, b, lcs(a, b));
  return {
    rows,
    added: rows.filter((r) => r.kind === "added").length,
    removed: rows.filter((r) => r.kind === "removed").length,
  };
}

/** Split into lines, dropping a single trailing newline so "a\n" is one line, not two. */
function splitLines(text: string): string[] {
  const normalised = text.replace(/\r\n/g, "\n").replace(/\r/g, "\n");
  const lines = normalised.split("\n");
  if (lines.length > 0 && lines[lines.length - 1] === "") lines.pop();
  return lines;
}

/** The LCS length table — `table[i][j]` is the LCS of `a[i:]` and `b[j:]`. */
function lcs(a: string[], b: string[]): number[][] {
  const table: number[][] = Array.from({ length: a.length + 1 }, () => new Array<number>(b.length + 1).fill(0));
  for (let i = a.length - 1; i >= 0; i--) {
    for (let j = b.length - 1; j >= 0; j--) {
      table[i][j] = a[i] === b[j] ? table[i + 1][j + 1] + 1 : Math.max(table[i + 1][j], table[i][j + 1]);
    }
  }
  return table;
}

/** Walk the table into a row list, preferring removals before additions at a divergence so a
 *  changed line reads as old-then-new. */
function walk(a: string[], b: string[], table: number[][]): DiffRow[] {
  const rows: DiffRow[] = [];
  let i = 0;
  let j = 0;
  let oldLine = 1;
  let newLine = 1;
  while (i < a.length && j < b.length) {
    if (a[i] === b[j]) {
      rows.push({ kind: "unchanged", text: a[i], oldLine: oldLine++, newLine: newLine++ });
      i++;
      j++;
    } else if (table[i + 1][j] >= table[i][j + 1]) {
      rows.push({ kind: "removed", text: a[i], oldLine: oldLine++ });
      i++;
    } else {
      rows.push({ kind: "added", text: b[j], newLine: newLine++ });
      j++;
    }
  }
  while (i < a.length) rows.push({ kind: "removed", text: a[i++], oldLine: oldLine++ });
  while (j < b.length) rows.push({ kind: "added", text: b[j++], newLine: newLine++ });
  return rows;
}
