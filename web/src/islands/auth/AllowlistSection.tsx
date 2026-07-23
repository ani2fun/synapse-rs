/**
 * One allowlist section: the grants table, the grant form, revoke per row, and a status banner.
 * `/admin` renders TWO of these — the submit allowlist and the content-editor allowlist — which
 * are identical in shape (username · optional note · granted-at, with grant-upsert and revoke) and
 * differ only in meaning and endpoint. So the grammar lives here once and each section is one call
 * with its own verbs and copy; each keeps its OWN load/status state, so a failure in one never
 * blanks the other. Classes are the shared `admin__*` / `account-page__*` (web/styles/account.css).
 */
import { useEffect, useState } from "preact/hooks";

import { ApiFailure } from "../../lib/api/client";
import type { AllowlistEntry, GrantRequest } from "../../lib/api/client";

type ActionStatus =
  | { kind: "idle" }
  | { kind: "busy"; message: string }
  | { kind: "ok"; message: string }
  | { kind: "error"; message: string };

type Entries =
  | { kind: "loading" }
  | { kind: "loaded"; rows: AllowlistEntry[] }
  | { kind: "failed"; message: string };

export interface AllowlistSectionProps {
  readonly title: string;
  readonly blurb: string;
  readonly load: () => Promise<AllowlistEntry[]>;
  readonly grant: (request: GrantRequest) => Promise<AllowlistEntry>;
  readonly revoke: (username: string) => Promise<void>;
}

function failureMessage(error: unknown): string {
  return error instanceof ApiFailure ? error.message : error instanceof Error ? error.message : String(error);
}

export function AllowlistSection(props: AllowlistSectionProps) {
  const [status, setStatus] = useState<ActionStatus>({ kind: "idle" });
  const [entries, setEntries] = useState<Entries>({ kind: "loading" });
  const [username, setUsername] = useState("");
  const [note, setNote] = useState("");

  const reload = () => {
    void (async () => {
      try {
        const rows = await props.load();
        setEntries({ kind: "loaded", rows });
      } catch (error) {
        setEntries({ kind: "failed", message: failureMessage(error) });
      }
    })();
  };

  useEffect(reload, []);

  const grant = () => {
    if (username.trim() === "") {
      setStatus({ kind: "error", message: "A grant needs a username" });
      return;
    }
    const request = { username, note: note.trim() === "" ? null : note };
    setStatus({ kind: "busy", message: "Granting…" });
    void (async () => {
      try {
        const entry = await props.grant(request);
        setStatus({ kind: "ok", message: `Granted '${entry.username}'.` });
        setUsername("");
        setNote("");
        reload();
      } catch (error) {
        setStatus({ kind: "error", message: failureMessage(error) });
      }
    })();
  };

  const revoke = (name: string) => {
    setStatus({ kind: "busy", message: `Revoking '${name}'…` });
    void (async () => {
      try {
        await props.revoke(name);
        setStatus({ kind: "ok", message: `Revoked '${name}'.` });
        reload();
      } catch (error) {
        setStatus({ kind: "error", message: failureMessage(error) });
      }
    })();
  };

  return (
    <section class="admin__section">
      <h2 class="admin__section-title">{props.title}</h2>
      <p class="account-page__meta">{props.blurb}</p>
      <StatusBanner status={status} />
      <form
        class="admin__grant"
        onSubmit={(event) => {
          event.preventDefault();
          grant();
        }}
      >
        <input
          class="admin__input"
          placeholder="username"
          value={username}
          onInput={(event) => setUsername((event.target as HTMLInputElement).value)}
        />
        <input
          class="admin__input admin__input--note"
          placeholder="note (optional)"
          value={note}
          onInput={(event) => setNote((event.target as HTMLInputElement).value)}
        />
        <button class="admin__grant-btn" type="submit">
          Grant
        </button>
      </form>
      <EntriesTable entries={entries} revoke={revoke} />
    </section>
  );
}

function EntriesTable({ entries, revoke }: { entries: Entries; revoke: (name: string) => void }) {
  if (entries.kind === "loading") return <p class="account-page__loading">Loading grants…</p>;
  if (entries.kind === "failed")
    return <p class="account-page__status account-page__status--error">{entries.message}</p>;
  if (entries.rows.length === 0) return <p class="account-page__meta">No grants yet.</p>;
  return (
    <table class="admin__table">
      <thead>
        <tr>
          <th>Username</th>
          <th>Note</th>
          <th>Granted</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        {entries.rows.map((entry) => (
          <tr key={entry.username}>
            <td class="admin__cell-user">{entry.username}</td>
            <td>{entry.note ?? ""}</td>
            <td>{entry.grantedAt.split("T")[0] ?? ""}</td>
            <td>
              <button class="admin__revoke" onClick={() => revoke(entry.username)}>
                Revoke
              </button>
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

function StatusBanner({ status }: { status: ActionStatus }) {
  if (status.kind === "idle") return null;
  const cls =
    status.kind === "busy"
      ? "account-page__status account-page__status--busy"
      : status.kind === "ok"
        ? "account-page__status account-page__status--ok"
        : "account-page__status account-page__status--error";
  const icon = status.kind === "busy" ? "…" : status.kind === "ok" ? "✓" : "✗";
  return (
    <p class={cls}>
      <span class="account-page__status-icon">{icon}</span> {status.message}
    </p>
  );
}
