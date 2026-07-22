import * as log from "../lib/log";
// The library landing's post-hydration chrome. Vanilla TS, no framework — the SSR page is plain
// HTML and reading progress is a browser-only concern (`localStorage` has no SSR equivalent), so
// there is nothing here for a component framework to hydrate INTO; this script just mutates the
// DOM Astro already rendered.
//
// Three jobs:
//   (a) inject a progress bar + pct into each book card's footer (`.lib-card__progress` / `--all`),
//   (b) render the `.lib-continue` "pick up where you left off" card above the grid, and
//   (c) the "Start reading" CTA's smooth-scroll-with-header-offset (the grid's bounding-rect top
//       + `scrollY` − 80, the sticky header's height).
//
// ISLAND-PROPS PATTERN CHOSEN HERE: index.astro embeds the SAME `SynapseIndexDto` it already
// fetched for SSR as a `<script type="application/json" id="library-index-data">` blob
// (`<` escaped to `<` so a title containing "</script" can't break the tag out early).
// This island parses that blob ONCE and re-uses the PURE `tree.ts`/`progress.ts` helpers the
// SSR page already computed with — no second network round trip, and no duplicated
// flatten/lookup logic living twice in two languages of client code. Each card carries
// `data-book-slug` (see BookCard.astro) so this script can re-resolve a book without
// re-walking the whole catalog tree per card; the continue card resolves its book through
// `bookOf` the same way.
import * as storage from "../lib/storage";
import * as api from "../lib/api/client";
import { AUTH_CHANGED, isAuthed } from "./workbench/contracts";
import { completedCount, parse as parseDone } from "../lib/catalog/progress";
import { bookOf, findBook, readingOrder } from "../lib/catalog/tree";
import type { SynapseIndex } from "../lib/api/client";

function readIndex(): SynapseIndex | null {
  const el = document.getElementById("library-index-data");
  if (!el?.textContent) return null;
  try {
    return JSON.parse(el.textContent) as SynapseIndex;
  } catch {
    return null;
  }
}

/** (a) A card progress bar + pct — shown only once there is something to report, so an untouched
 *  library stays exactly as SSR rendered it. */
function injectProgressChips(index: SynapseIndex, done: Set<string>): void {
  if (done.size > 0) log.debug(`library: injecting progress chips (${done.size} finished lessons)`);
  for (const card of document.querySelectorAll<HTMLElement>("[data-book-slug]")) {
    const slug = card.dataset.bookSlug;
    if (!slug) continue;
    const book = findBook(index, slug);
    if (!book) continue;
    const total = readingOrder(book).length;
    const count = completedCount(book, done);

    const footer = card.querySelector(".lib-card__footer");
    const cta = footer?.querySelector(".lib-card__cta");
    if (!footer || !cta) continue;
    // Idempotent: a re-inject (after the signed-in server merge) replaces the device-only chip.
    footer.querySelector(".lib-card__progress")?.remove();
    if (count <= 0 || total <= 0) continue;

    const pct = Math.round((count / total) * 100);
    const chip = document.createElement("div");
    chip.className = count === total ? "lib-card__progress lib-card__progress--all" : "lib-card__progress";
    const bar = document.createElement("div");
    bar.className = "lib-card__progress-bar";
    const fill = document.createElement("div");
    fill.className = "lib-card__progress-fill";
    fill.style.setProperty("--pct", `${pct}%`);
    bar.append(fill);
    const label = document.createElement("span");
    label.className = "lib-card__progress-pct";
    label.textContent = count === total ? "Done" : `${pct}%`;
    chip.append(bar, label);
    footer.insertBefore(chip, cta);
  }
}

/** (b) "Pick up where you left off" — renders nothing until there IS a last lesson. The
 *  title/book come from the index rather than being stored alongside the path: a stored title
 *  would go stale the moment a lesson is renamed. */
function renderContinueCard(index: SynapseIndex): void {
  const mount = document.getElementById("lib-continue-mount");
  const last = storage.get(storage.READER_LAST_KEY);
  if (last != null) log.debug(`library: resume card → ${last}`);
  if (!mount) return;
  const path = storage.get(storage.READER_LAST_KEY);
  if (!path) return;

  const segments = path.split("/").filter((segment) => segment !== "");
  const book = bookOf(index, segments);
  if (!book) return;
  const entry = readingOrder(book).find((candidate) => candidate.path === path);
  if (!entry) return;

  const a = document.createElement("a");
  a.className = "lib-continue";
  a.href = `/synapse/${path}`;

  const label = document.createElement("span");
  label.className = "lib-continue__label";
  label.textContent = "Pick up where you left off";

  const title = document.createElement("span");
  title.className = "lib-continue__title";
  title.textContent = entry.lesson.title;

  const bookLine = document.createElement("span");
  bookLine.className = "lib-continue__book";
  bookLine.textContent = book.title;

  a.append(label, title, bookLine);
  mount.replaceChildren(a);
}

/** (c) Smooth-jump to the grid, offset for the sticky header. */
function wireStartReading(): void {
  document.getElementById("lib-start-reading")?.addEventListener("click", () => {
    const grid = document.getElementById("library-grid");
    if (!grid) return;
    const top = grid.getBoundingClientRect().top + window.scrollY - 80;
    window.scrollTo(0, top);
  });
}

/** Merge the signed-in reader's account progress into the chips (now or when auth adopts late).
 *  Anonymous → `listProgress` returns `[]`, a no-op; the device chips already showed. */
function syncChipsFromServer(index: SynapseIndex, done: Set<string>): void {
  if (!isAuthed()) return;
  void api
    .listProgress()
    .then((server) => {
      let added = false;
      for (const path of server) {
        if (!done.has(path)) {
          done.add(path);
          added = true;
        }
      }
      if (added) injectProgressChips(index, done);
    })
    .catch((error) => log.debug(`library progress sync skipped: ${error instanceof Error ? error.message : String(error)}`));
}

function init(): void {
  wireStartReading();
  const index = readIndex();
  if (!index) return;
  const done = parseDone(storage.get(storage.READER_PROGRESS_KEY));
  injectProgressChips(index, done);
  renderContinueCard(index);
  syncChipsFromServer(index, done);
  window.addEventListener(AUTH_CHANGED, () => syncChipsFromServer(index, done));
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", init);
} else {
  init();
}
