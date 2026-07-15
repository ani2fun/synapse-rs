//! The catalog's outbound adapters (oracle: `catalog/infrastructure/`) — the filesystem
//! repository over `SYNAPSE_ROOT` and the git-SHA content version (ADR-S010/S033).

mod commit_sha;
mod filesystem;

pub use commit_sha::read_commit_sha;
pub use filesystem::FileSystemContentRepository;
