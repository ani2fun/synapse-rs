//! The submission adapters — the Postgres store and the sidecar-or-fence suite resolver.

mod postgres;
mod problem_tests;

pub use postgres::PostgresSubmissionRepository;
pub use problem_tests::FsProblemTests;
