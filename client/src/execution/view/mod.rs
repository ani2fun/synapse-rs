//! The runnable-block views (oracle: `Workbench` + `WorkbenchOutput`, step-11 scope).

mod hydrate;
mod practice;
mod runnable;
mod workbench;

pub use hydrate::hydrate_workbenches;
pub use practice::hydrate_practices;
pub use runnable::RunnableBlock;
