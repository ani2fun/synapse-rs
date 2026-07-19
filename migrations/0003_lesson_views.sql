-- Readership (step 49). The project could serve 442 lessons and answer nothing about which
-- of them anyone opened; every prioritisation decision was a guess. This is the smallest
-- table that ends that.
--
-- Content POPULARITY, not user tracking: there is deliberately no user id, no session, no
-- IP and no referrer here. `authed` is one bit distinguishing a signed-in reader from an
-- anonymous one, which is as far as attribution goes — the only questions this table can
-- answer are "which lessons get opened" and "which never do".
create table lesson_view (
    id          bigserial   primary key,
    lesson_path text        not null,
    viewed_at   timestamptz not null default now(),
    authed      boolean     not null
);

-- The read is always "top paths, recent first" — see LessonViewStore::top.
create index lesson_view_path_recency on lesson_view (lesson_path, viewed_at desc);
