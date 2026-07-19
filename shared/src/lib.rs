//! The shared kernel (RS001): the wire contract — the DTOs and pure models BOTH sides need.
//! Compiles native (server + fast tests) and to `wasm32` (client, from step 02). Contexts grow
//! their own modules here as their DTOs land.
//!
//! The viz engine lived here from RS-P7 until step 45, mirroring Synapse's `shared/`
//! crossproject. It moved to `client/src/viz/engine/`: the server referenced it zero times
//! while it made up 86% of this crate (4,037 of 4,670 lines), so "shared" had come to describe
//! where the folder sat rather than anything true about the code. The usual reason to keep a
//! wasm-targeted engine here — native testability for the cortex goldens — did not apply,
//! because the client crate is an `rlib` that already compiles and tests natively.
//!
//! What remains is a genuine kernel: ~630 lines whose only dependency is `serde`.

pub mod api;
pub mod blog;
pub mod catalog;
pub mod execution;
pub mod identity;
pub mod submission;
pub mod tutor;
