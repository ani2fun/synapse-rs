-- Per-user completion progress. The sidebar ✓ ticks were localStorage-only — per browser,
-- unsynced, and blind to the signed-in identity (a hard reload kept them because localStorage
-- is not the HTTP cache). This is the smallest table that makes completion the ACCOUNT's, not
-- the device's: one row per (user, lesson) the reader has finished — a prose lesson read to the
-- end, or a problem with an accepted judged submission.
--
-- user_id is the opaque Keycloak `sub` (the same value submissions.user_id stores). Progress is
-- convenience state the account owns: resetting it (DELETE /api/progress) clears these rows and
-- NOTHING else — a reset never touches the submissions history.
create table problem_progress (
    user_id      text        not null,
    lesson_path  text        not null,
    completed_at timestamptz not null default now(),
    primary key (user_id, lesson_path)
);

-- The read is always "every lesson this user has finished" — see ProblemProgressStore::list_for.
create index problem_progress_user on problem_progress (user_id);
