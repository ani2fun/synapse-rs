//! The submission adapters — the Postgres store, the allowlist, and the sidecar-or-fence
//! suite resolver.

mod allowlist;
mod postgres;
mod problem_tests;

pub use allowlist::PostgresSubmissionAllowlist;
pub use postgres::PostgresSubmissionRepository;
pub use problem_tests::FsProblemTests;
