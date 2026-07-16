-- The submissions store (oracle: Liquibase changeset 001-submissions, verbatim shape).
-- The state ADT flattens here and only here: the check constraint keeps the flattened row
-- honest — completed ⟺ outcome and completed_at present.
create table submissions (
    id           uuid primary key,
    lesson_path  text        not null,
    language     text        not null,
    source       text        not null,
    user_id      text,
    created_at   timestamptz not null,
    status       text        not null check (status in ('pending', 'judging', 'completed')),
    outcome      jsonb,
    completed_at timestamptz,
    constraint completed_shape check
        ((status = 'completed') = (outcome is not null and completed_at is not null))
);

create index submissions_lesson_recency on submissions (lesson_path, created_at desc);
