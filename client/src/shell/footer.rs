//! The landing-page footer (step 46) — the quiet line at the end of the page, and the build
//! the reader is running.
//!
//! Landing page only, deliberately. Lesson pages own their own vertical rhythm (the reader
//! chrome, the pager cards) and problem pages are a fixed-height two-pane layout with no page
//! scroll at all — a footer there would either be unreachable or would fight the panes.

use leptos::prelude::*;

/// Baked at compile time by `build.rs`: the release build-arg (`github.sha`), else the local
/// git HEAD, else `"dev"`.
const VERSION: &str = env!("SYNAPSE_VERSION");

const REPO: &str = "https://github.com/ani2fun/synapse";

/// Seven characters is the git convention and what every tool echoes back — enough to
/// identify a commit, short enough to read at 12px. `"dev"` passes through untouched.
fn short(version: &str) -> &str {
    if version.len() > 7 && version.chars().all(|c| c.is_ascii_hexdigit()) {
        &version[..7]
    } else {
        version
    }
}

#[component]
pub fn SiteFooter() -> impl IntoView {
    let sha = short(VERSION);
    // Only a real commit gets a link — "dev" would 404, and a dead link in a footer is worse
    // than plain text.
    let is_commit = VERSION.len() >= 7 && VERSION.chars().all(|c| c.is_ascii_hexdigit());
    view! {
        <footer class="site-footer">
            <div class="site-footer__inner">
                <div class="site-footer__left">
                    <p class="site-footer__line">
                        "© 2026 Aniket Kakde · Synapse — read it, run it, understand it."
                    </p>
                    <p class="site-footer__line site-footer__muted">
                        "Built with Rust, Leptos and WebAssembly. "
                        <a class="site-footer__link" href=REPO rel="noopener" target="_blank">
                            "Source on GitHub"
                        </a>
                    </p>
                </div>
                <div class="site-footer__right">
                    {if is_commit {
                        view! {
                            <a
                                class="site-footer__version"
                                href=format!("{REPO}/commit/{VERSION}")
                                rel="noopener"
                                target="_blank"
                                title=format!("Deployed build {VERSION} — open this commit on GitHub")
                            >
                                <span class="site-footer__version-label">"version"</span>
                                <span class="site-footer__sha">{sha}</span>
                            </a>
                        }
                        .into_any()
                    } else {
                        view! {
                            <span class="site-footer__version" title="Local build — not a released commit">
                                <span class="site-footer__version-label">"version"</span>
                                <span class="site-footer__sha">{sha}</span>
                            </span>
                        }
                        .into_any()
                    }}
                </div>
            </div>
        </footer>
    }
}
