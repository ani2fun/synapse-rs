# RS004 — Content edits are proposed in-app and land as pull requests

**Status:** accepted · 2026-07-23

## Context

Lesson prose lives in `ani2fun/synapse-content`, a separate git repository a git-sync sidecar
feeds into production; publishing is a `git push` there, never an image rebuild (RS001). The only
way to fix a typo or sharpen a paragraph was to clone that repository and open a pull request by
hand — which excludes every reader who does not already know git, i.e. most of them. A learning
platform whose readers spot the errors but cannot fix them wastes its best proofreaders.

The obvious shortcuts are both wrong. Letting the app write directly to the served content tree
would make the running process a second source of truth, racing the git-sync sidecar and
bypassing the review that every published word currently passes. Teaching contributors git is the
problem, restated.

## Decision

**A signed-in, allow-listed reader edits a lesson's markdown inside Synapse, previews it, and
submits; the server opens (or reuses) a pull request against the content repository on their
behalf.** The content repository stays the single source of truth, and every change still passes a
human review before it ships. This is a new `authoring` bounded context, hexagonal like the rest
(RS001), mounted only where a forge is configured — `CONTENT_FORGE=off` leaves the whole surface a
structural 404.

### Shape

| Concern | Choice |
|---|---|
| Where edits go | A pull request per (contributor, page). Review and merge stay on the forge — the owner approves every word |
| Forge access | GitHub REST over the existing `reqwest`, behind a `ContentForge` port. NO `git` binary, NO working copy — every op is a stateless HTTP call whose failure leaves nothing to clean up |
| Credentials | One fine-grained PAT, `contents:write` + `pull_requests:write` on the content repo alone, a sealed secret in `ani2fun/infra`. Never logged, never returned |
| Who may edit | A **separate** `content_editor_allowlist`, not the submit allowlist — see below |
| Branch | `edit/<username>/<lesson-path>`, `-2`/`-3` when an earlier proposal for the page already merged or closed |
| Reuse | A second edit while the pull request is still open is another COMMIT on the same branch, not a second request. The forge is asked for the live state before anything is reused — the stored state is only a cache |
| Editor surface | A dedicated `/edit/<path>` page, so the lesson page gains ZERO eager JS (only a small "Suggest an edit" link a tiny island un-hides for an allow-listed caller) |
| Drift | The editor is handed a fingerprint of the source; a submit against a moved file is a 409, and the forge's own blob-sha check is the second guard |
| Credential-free mode | `CONTENT_FORGE=dry-run` runs the WHOLE flow — gate, drift guard, validation, branch derivation, stored history — and skips only the forge call. Dev, CI and e2e run against it |

### Why the preview is unskippable

The preview is not a convenience — it is the quality gate. A contributor who cannot read a diff
must still be able to see that their table renders and their fence closed, and the cheapest place
to catch that is before a reviewer looks. So the rendered preview is step one of the submit
dialog, and a blocking lint error (lost frontmatter, no title — the same two things the server's
`validate` refuses) disables Submit there. The preview reuses the reader's exact pipeline, DOM and
hydrators, so what the contributor sees is what the page will be.

### Why a separate allowlist

The submit allowlist grants shared compute and storage for saving code attempts. This grant lets
someone open pull requests against a public repository under the deployment's own token — a much
larger blast radius. Keeping them separate means revoking one is never silently revoking the
other, and it makes the trust decision explicit at grant time rather than inherited.

## Alternatives considered

- **A GitHub App instead of a PAT.** Auto-rotating installation tokens and a bot identity are the
  better end state, but the App is more setup (create, install, two secrets) for the same
  contributor-facing behaviour. The `ContentForge` port is shaped so a `GitHubAppForge` slots in
  behind it with no change above the infrastructure layer — this is a deferral, not a dead end.
- **A working-copy clone the server commits to.** Rejected: the production image is debian-slim
  plus one binary and the Node sidecar, its filesystem is not a place to keep a clone, and a pod
  that restarts mid-push leaves one in an unknown state. Stateless REST has no such failure mode.
- **Direct writes to the served tree.** Rejected in the Context above — it destroys the
  single-source-of-truth and the review gate.
- **A WYSIWYG editor.** A much larger build that would fight the authored fence conventions
  (`run`, `solution`, `viz=`, `testcases`) the pipeline depends on. The preview pane is what makes
  raw markdown safe for a non-git contributor instead.

## Consequences

- Contributors need no git knowledge; maintainers keep full control at the merge.
- Merge/close is reflected back lazily — on the contributor's next submit for that page, or their
  next `/account` load — not in real time. A forge webhook is the obvious follow-up.
- v1 edits an existing lesson `.md` only: no sidecars (`.editorial.md`, `.tests.json`), no
  `book.json`, no new files, no media uploads, and no problem-page editor surface.
- Drafts are per-device (localStorage) until submitted; a cleared browser loses an unsubmitted
  draft. A `content_edit_draft` table is the follow-up if that ever bites.
