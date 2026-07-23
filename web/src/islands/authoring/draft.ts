// The per-page draft. A contributor's edit lives in this browser's localStorage — never on the
// server — until they submit it, so a half-finished change costs the server nothing and survives
// a reload or an accidental navigation away.
//
// The draft is keyed by username AND lesson path, so one browser holds a separate draft for each
// page a person is editing, and a draft can never surface under a different account. It carries
// the fingerprint of the source it was started from, so a draft is only offered back when it
// still applies — if the page moved on disk meanwhile, the stale draft is discarded rather than
// reapplied over content it no longer matches.

import * as storage from "../../lib/storage";
import * as log from "../../lib/log";

export interface Draft {
  readonly source: string;
  /** epoch ms — shown to the contributor ("unsaved draft from 3 minutes ago"). */
  readonly savedAt: number;
  /** The fingerprint of the source this draft was started from. */
  readonly baseFingerprint: string;
}

function key(username: string, lessonPath: string): string {
  return `${storage.CONTENT_DRAFT_PREFIX}${username}:${lessonPath}`;
}

/** Persist a draft (debounced by the caller). A denied write is a silent no-op — the accessor
 *  already swallows it — so editing never breaks in a storage-denied profile. */
export function saveDraft(username: string, lessonPath: string, source: string, baseFingerprint: string): void {
  const draft: Draft = { source, savedAt: Date.now(), baseFingerprint };
  storage.set(key(username, lessonPath), JSON.stringify(draft));
}

/** The saved draft for this page, if one exists and still parses. */
export function loadDraft(username: string, lessonPath: string): Draft | null {
  const raw = storage.get(key(username, lessonPath));
  if (raw === null) return null;
  try {
    const draft = JSON.parse(raw) as Draft;
    if (typeof draft.source === "string" && typeof draft.baseFingerprint === "string") return draft;
  } catch {
    // A corrupt draft is not worth surfacing — drop it.
  }
  clearDraft(username, lessonPath);
  return null;
}

/** Drop the draft — on a successful submit, or when it no longer applies. */
export function clearDraft(username: string, lessonPath: string): void {
  storage.remove(key(username, lessonPath));
  log.debug(`draft: cleared for ${lessonPath}`);
}

/** A human "3 minutes ago" for the restore banner. */
export function savedAgo(savedAt: number): string {
  const seconds = Math.max(0, Math.round((Date.now() - savedAt) / 1000));
  if (seconds < 60) return "just now";
  const minutes = Math.round(seconds / 60);
  if (minutes < 60) return `${minutes} minute${minutes === 1 ? "" : "s"} ago`;
  const hours = Math.round(minutes / 60);
  if (hours < 24) return `${hours} hour${hours === 1 ? "" : "s"} ago`;
  const days = Math.round(hours / 24);
  return `${days} day${days === 1 ? "" : "s"} ago`;
}
