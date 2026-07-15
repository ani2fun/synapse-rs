# Step 05 — The catalog infrastructure: the filesystem adapter and the content version

*(oracle: synapse step 05 + the ADR-S033 content-version forward note —
`FileSystemContentRepositoryLive`, `ContentCommitSha`; their specs + `CatalogAutoReloadSpec`
ported as temp-dir integration tests)*

## The filesystem adapter (`filesystem.rs`)

`FileSystemContentRepository { content_root, auto_reload }` implements the port:

- **`load_tree`** — recursive walk: every non-hidden dir loads, `book.json`/`category.json`
  decoded **leniently at every level** (unreadable/malformed → `None`, ADR-0001; serde stays out
  of the domain's way — the markers are pre-decoded here), `.md` regular files become
  `ContentFile`s, children sorted for determinism (the walker re-sorts by its own rules).
- **`read_lesson`** — `safe_under_root`: canonicalize BOTH the root and the resolved target
  (macOS `/tmp` is a symlink to `/private/tmp` — realpathing one side only would reject
  everything), require the target under the root AND a regular file. Defense-in-depth beneath
  the service's slug check; pinned by a traversal IT that plants a real secret outside the root.
- All filesystem work runs under `spawn_blocking` — the no-blocking-in-async rule, kept
  mechanically. A panicked blocking task resumes its unwind rather than masking the original.

## The content version (ADR-S010's two modes)

- **Dev (`auto_reload = true`)**: the watermark `"<newest mtime ms>:<file count>"` over regular
  files with hidden subtrees pruned — `.git` churn must not rebuild the index. Edits advance the
  mtime half; adds/deletes advance the count. FS hiccups degrade to `"0:0"`, never an error.
- **Prod (`auto_reload = false`)**: `read_commit_sha` (`commit_sha.rs`) — the checkout's HEAD
  SHA via three tiny reads, NO `git` binary, re-read per call: `.git` as a dir (plain clone) or
  a `gitdir:` pointer (git-sync sidecar / worktree) → `HEAD` → loose ref, else `packed-refs`
  scan; validated `[0-9a-f]{40,64}` (SHA-1 or SHA-256); anything unreadable → `"static"`. This
  is how prod re-indexes on git-sync advances with no redeploy.

## Tests (`server/tests/catalog_fs_it.rs`)

9 ITs against real temp dirs: marker decoding at depth + hidden pruning; the full
service-over-adapter round trip (index → lesson through `lesson_files`); traversal + missing →
`NotFound`; watermark advances on edit (deterministic — `File::set_modified`, no sleeps) and on
add, but NOT on hidden churn; prod mode reports the SHA; all four SHA resolutions (loose,
packed, detached, `gitdir:` pointer) + garbage → `"static"`.

## Verified

59 tests green (50 unit + 9 FS IT); clippy `-D warnings`; purity + caps + fmt green.
