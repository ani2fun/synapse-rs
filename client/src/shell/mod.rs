//! The app shell — chrome + the router feeding the pure app-map (oracle: `AppRouter.scala`).
//! The step-02 probe page is gone; its mechanics (signals, the island bridge, shared DTOs)
//! live on inside the features.

use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use crate::blog::view::{BlogListPage, BlogPostPage};
use crate::catalog::view::{LessonPage, LibraryPage};
use crate::search::view::{SearchButton, SearchPalette};

/// The root component `lib.rs` mounts.
#[component]
pub fn App() -> impl IntoView {
    // App-level stores live under the root owner — they outlive every page (state layer rule).
    // PrefsStore also reflects the stored reading prefs onto <html> BEFORE the first paint.
    crate::catalog::state::CatalogStore::provide();
    crate::catalog::state::PrefsStore::provide();
    crate::identity::state::AuthStore::provide();
    crate::blog::state::BlogStore::provide();
    crate::search::state::SearchStore::provide();
    view! {
        <Router>
            <header class="shell-header">
                <a class="shell-brand" href="/">"synapse-rs"</a>
                <span class="shell-tag">"the Rust rebuild"</span>
                <SearchButton />
                <span class="shell-spacer"></span>
                <a class="header__link" href="/blog">"Blog"</a>
                <crate::identity::view::AccountChip />
            </header>
            <main class="shell-main">
                <Routes fallback=|| view! { <p class="muted">"Not found."</p> }>
                    <Route path=path!("/") view=LibraryPage />
                    <Route path=path!("/synapse/*path") view=LessonPage />
                    <Route path=path!("/blog") view=BlogListPage />
                    <Route path=path!("/blog/:slug") view=BlogPostPage />
                    <Route path=path!("/account") view=crate::identity::view::AccountPage />
                </Routes>
            </main>
            <SearchPalette />
        </Router>
    }
}
