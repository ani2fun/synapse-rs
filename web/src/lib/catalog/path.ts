// The lesson path a page is about, read from its URL. Both the reader islands (workbench,
// widgets) and the editor's preview need the SAME directory-mirror segments the payload was
// fetched for, so the derivation lives here once rather than being copied — and privately —
// into each island.
//
// A lesson is reachable at two URLs: `/synapse/<path>` (reading it) and `/edit/<path>` (editing
// it). The preview on the edit page hydrates real workbenches, and they key their submissions on
// this path, so it must resolve on both.

const PREFIXES = ["/synapse/", "/edit/"] as const;

/** The `category…/book/chapter…/lesson` segments, or `[]` when the URL is neither a lesson nor an
 *  edit page. Empty segments (a trailing slash, a doubled slash) are dropped. */
export function lessonPathFromUrl(pathname: string = window.location.pathname): string[] {
  const prefix = PREFIXES.find((p) => pathname.startsWith(p));
  if (!prefix) return [];
  return pathname
    .slice(prefix.length)
    .split("/")
    .filter((segment) => segment !== "");
}
