-- The submit allowlist (oracle changeset 002-submission-allowlist, step 21): keyed by the
-- IdP USERNAME (human-grantable), stored lowercase — the verifier canonicalises once, so the
-- compare is apples-to-apples. submissions.user_id keeps storing the opaque sub.
create table submission_allowlist (
    username   text        primary key,
    note       text,
    granted_at timestamptz not null default now()
);

-- Dev realm seeds (throwaway realm-file users).
insert into submission_allowlist (username, note) values ('tester', 'dev realm seed');
insert into submission_allowlist (username, note) values ('test1', 'dev realm seed');
