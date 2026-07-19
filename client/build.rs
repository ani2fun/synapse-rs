//! Bakes the shipped version into the client bundle (step 46).
//!
//! The footer shows which build the reader is actually running, which is the thing worth
//! knowing when someone reports a bug. That has to be a COMPILE-TIME constant in the wasm: a
//! value fetched from the server would describe the server, and a browser holding a cached
//! bundle across a deploy would then report a version it is not running.
//!
//! Resolution order:
//!   1. `SYNAPSE_VERSION` — set by the Dockerfile from the release build-arg (`github.sha`).
//!      This is the only path that matters in production, because `.dockerignore` excludes
//!      `.git`, so the fallback below cannot reach the image.
//!   2. `git rev-parse HEAD` — the dev loop, where the env var is absent but git is right there.
//!   3. `"dev"` — a tarball with neither. Never silently blank.
//!
//! `rerun-if-env-changed` is the load-bearing line. Cargo does not track `env!()` reads on its
//! own, so without it a rebuild after a new commit would happily reuse the previously baked
//! string and the footer would confidently display the wrong SHA.

use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=SYNAPSE_VERSION");
    // A new commit must re-bake the constant, so the build depends on HEAD moving.
    println!("cargo:rerun-if-changed=../.git/HEAD");

    let version = std::env::var("SYNAPSE_VERSION")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or_else(git_head)
        .unwrap_or_else(|| "dev".to_owned());

    println!("cargo:rustc-env=SYNAPSE_VERSION={}", version.trim());
}

fn git_head() -> Option<String> {
    let out = Command::new("git").args(["rev-parse", "HEAD"]).output().ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8(out.stdout).ok().map(|s| s.trim().to_owned())
}
