/**
 * "My change requests" on the account page — the prose edits this contributor has proposed, so
 * they can follow their own suggestions to the pull request without leaving Synapse. Reads
 * `GET /api/edits`; a 403 (not a content editor) or a 404 (editing off) is not an error to show —
 * the section simply does not render, so an ordinary reader's account page is unchanged.
 */
import { useEffect, useState } from "preact/hooks";

import { myEdits } from "../../lib/api/client";
import type { EditRequest } from "../../lib/api/client";

type Load = { kind: "loading" } | { kind: "ready"; rows: EditRequest[] } | { kind: "absent" };

export function ChangeRequests() {
  const [load, setLoad] = useState<Load>({ kind: "loading" });

  useEffect(() => {
    let live = true;
    void (async () => {
      try {
        const rows = await myEdits();
        if (live) setLoad({ kind: "ready", rows });
      } catch {
        // Not a content editor, or editing is off — show nothing rather than an error.
        if (live) setLoad({ kind: "absent" });
      }
    })();
    return () => {
      live = false;
    };
  }, []);

  // Nothing to show until there is at least one request — a contributor with no edits yet does not
  // need an empty section.
  if (load.kind !== "ready" || load.rows.length === 0) return null;

  return (
    <section class="account-page__progress">
      <p class="account-page__section-head">My change requests</p>
      <p class="account-page__section-note">Prose edits you have proposed. Review and merging happen on the pull request.</p>
      <div class="edit-requests">
        {load.rows.map((row) => (
          <div class="edit-requests__row" key={row.id}>
            <span class="edit-requests__page">{row.lessonPath}</span>
            <span class={`edit-requests__pill edit-requests__pill--${row.state}`}>{row.state}</span>
            <span class="edit-requests__branch">{row.branch}</span>
            {row.prUrl ? (
              <a class="edit-requests__link" href={row.prUrl} target="_blank" rel="noopener noreferrer">
                View PR{row.prNumber ? ` #${row.prNumber}` : ""} →
              </a>
            ) : (
              <span class="edit-requests__link">recorded (dry run)</span>
            )}
          </div>
        ))}
      </div>
    </section>
  );
}
