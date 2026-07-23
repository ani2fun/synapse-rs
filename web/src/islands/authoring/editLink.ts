// The "Suggest an edit" affordance on a lesson page. The link is server-rendered `hidden`; this
// reveals it ONLY for a caller the server says may edit. So an ordinary reader never sees it, a
// deployment with editing off never shows it, and the lesson page pays nothing but one small
// config fetch for the whole feature.
//
// One network call, cached across the page's lifetime. `canEdit` already folds in "signed in" and
// "on the content-editor list" and "editing is enabled", so there is nothing to re-derive here.

import * as api from "../../lib/api/client";
import * as log from "../../lib/log";
import { AUTH_CHANGED } from "../workbench/contracts";

const LINK_SELECTOR = "[data-edit-link]";

async function refresh(): Promise<void> {
  const link = document.querySelector<HTMLElement>(LINK_SELECTOR);
  if (!link) return;
  try {
    const config = await api.editConfig();
    const show = config.enabled && config.canEdit;
    link.hidden = !show;
    if (show) log.info(`edit: "Suggest an edit" available (${config.mode})`);
  } catch {
    // Editing off (the routes 404) or the config call failed — leave the link hidden. A reader
    // must never see a broken affordance.
    link.hidden = true;
  }
}

function init(): void {
  if (!document.querySelector(LINK_SELECTOR)) return;
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
