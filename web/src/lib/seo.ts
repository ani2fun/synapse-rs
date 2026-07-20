// The document head's pure half (oracle: client/src/seo.rs). Its DOM-touching half
// (`set_title`/`set_description`) does not port: that existed to patch a stale tab after a
// client-side SPA navigation, and Astro has no such moment — `output: "server"` re-renders the
// whole document, head included, on every navigation (layouts/Base.astro's props). Only the
// FORMAT survives, because a per-page title still has to match `platform::static_routes`
// wherever the server computes one.

/** The site name, and the fallback title for any page without one of its own. */
export const SITE_NAME = "Synapse";

/**
 * `Book · Lesson — Synapse`, matching `platform::static_routes::title_for` exactly. The book
 * leads because the left of the string is what survives truncation in a tab strip.
 */
export function titleForLesson(bookTitle: string | null, lessonTitle: string): string {
  return bookTitle ? `${bookTitle} · ${lessonTitle} — ${SITE_NAME}` : `${lessonTitle} — ${SITE_NAME}`;
}
