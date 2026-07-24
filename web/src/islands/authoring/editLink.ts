// The "Suggest an edit" affordance on a lesson page.
//
// The link is server-rendered VISIBLE but GATED — the problem workbench's Submit grammar: someone
// who cannot use it still sees that it exists and, on hover, how to ask for access. That default
// is also the safe one, because it is what a reader gets if this island never runs.
//
// This upgrades it in exactly two ways:
//   · the server says the caller may edit  → live link, ordinary tooltip;
//   · editing is switched off entirely     → removed, since there is nothing to ask for.
// Anything else leaves the gated default alone.
//
// One network call, cached across the page's lifetime. `canEdit` already folds in "editing is
// enabled", "signed in" and "on the content-editor list", so there is nothing to re-derive here.

import * as api from "../../lib/api/client";
import * as log from "../../lib/log";
import { AUTH_CHANGED } from "../workbench/contracts";

const LINK = "[data-edit-link]";
const TIP = "[data-edit-tip]";
const GATED = "lesson-edit-link--gated";
const ACTIVE_TIP = "Edit this page and open a change request";

/** Swallow clicks while gated — `aria-disabled` is a promise to assistive tech, not a behaviour,
 *  and an anchor with an href still navigates. Registered once, and it checks the live class so it
 *  stops mattering the moment the link is upgraded. */
function blockGatedClicks(link: HTMLAnchorElement): void {
  link.addEventListener("click", (event) => {
    if (link.classList.contains(GATED)) event.preventDefault();
  });
}

function activate(link: HTMLAnchorElement, tip: HTMLElement | null): void {
  link.classList.remove(GATED);
  link.removeAttribute("aria-disabled");
  tip?.setAttribute("data-tip", ACTIVE_TIP);
}

async function refresh(): Promise<void> {
  const link = document.querySelector<HTMLAnchorElement>(LINK);
  const tip = document.querySelector<HTMLElement>(TIP);
  if (!link) return;
  try {
    const config = await api.editConfig();
    if (!config.enabled) {
      // The deployment does not offer editing at all — an affordance nobody can ever earn is
      // worse than none, so it goes away rather than sitting there permanently gated.
      (tip ?? link).remove();
      return;
    }
    if (config.canEdit) {
      activate(link, tip);
      log.info(`edit: "Suggest an edit" is live (${config.mode})`);
    } else {
      log.debug("edit: not a content editor — the affordance stays gated");
    }
  } catch {
    // Routes absent (editing off) or the call failed — leave the gated default rather than
    // promising a link that would 404 or 403.
    log.debug("edit: config unavailable — the affordance stays gated");
  }
}

function init(): void {
  const link = document.querySelector<HTMLAnchorElement>(LINK);
  if (!link) return;
  blockGatedClicks(link);
  void refresh();
  // Auth resolves asynchronously and can flip after first paint (check-sso, a later sign-in), so
  // re-ask when it changes rather than reading a one-shot snapshot.
  window.addEventListener(AUTH_CHANGED, () => void refresh());
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", init);
} else {
  init();
}
