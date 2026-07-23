-- In-app prose editing. A reader who spots a typo edits the lesson markdown in Synapse and the
-- server opens a pull request against the content repository on their behalf — the content repo
-- stays the source of truth, and every word still passes through review before it ships.
--
-- Two tables, two jobs.

-- WHO MAY PROPOSE. Deliberately NOT submission_allowlist: that grant says "may spend shared
-- compute and storage saving attempts", this one says "may open pull requests against a public
-- repository under this deployment's token". Different blast radius, different list — revoking
-- one must never silently revoke the other. Keyed by the lowercase IdP username, exactly like
-- its sibling, because the verifier canonicalises once and the compare is apples-to-apples.
create table content_editor_allowlist (
    username   text        primary key,
    note       text,
    granted_at timestamptz not null default now()
);

-- Dev realm seeds (throwaway realm-file users).
insert into content_editor_allowlist (username, note) values ('tester', 'dev realm seed');

-- WHAT WAS PROPOSED. One row per (contributor, page, attempt) — the branch the server commits to
-- and the pull request it opened.
--
-- The reuse rule lives on this table: a second edit to the same page by the same person while the
-- pull request is still `open` becomes another COMMIT on the same branch, not a second request.
-- Once that pull request is merged or closed the row stops being reusable, and the next edit
-- allocates `attempt + 1` — which is what puts the `-2`, `-3` suffix on the branch name. `branch`
-- is unique because it is the thing the forge keys on; two rows claiming one ref would mean two
-- pull requests silently sharing commits.
--
-- pr_number/pr_url stay nullable: a dry-run deployment (no GitHub token) records the branch it
-- WOULD have pushed and opens nothing, so the whole flow is exercisable without credentials.
create table content_edit_request (
    id          uuid        primary key,
    username    text        not null,
    lesson_path text        not null,
    file_path   text        not null,
    branch      text        not null unique,
    attempt     int         not null,
    pr_number   bigint,
    pr_url      text,
    state       text        not null,
    commits     int         not null default 1,
    created_at  timestamptz not null default now(),
    updated_at  timestamptz not null default now()
);

-- Both reads are owner-scoped: "is there an open request for this person on this page" (the reuse
-- probe, on every submit) and "every request of mine" (the account page).
create index content_edit_request_owner_page on content_edit_request (username, lesson_path);
