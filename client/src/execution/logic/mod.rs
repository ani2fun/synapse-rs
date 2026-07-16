//! Pure execution logic (the logic layer — no leptos, no web-sys; purity-gated,
//! native-tested).

mod blocks;
mod executor;

pub use blocks::{Variant, display_lang, expected_for, parse_variants, seed_values};
pub use executor::{EditMode, ExecutorState, RunHandle, RunState, changed_line_count, is_dirty};
