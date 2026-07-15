//! The execution adapters (oracle: `execution/infrastructure/`) — the go-judge wire protocol,
//! per-language recipes, the Java entrypoint normaliser, and the HTTP runner.

pub mod java_rewriter;
pub mod recipe;
mod runner;
pub mod wire;

pub use runner::GoJudgeRunner;
