//! Blog state (oracle: `BlogStore`): the listing is cached app-wide (the list page and the ⌘K
//! palette share it; a failure re-arms the fetch), posts are fetch-per-navigation.

use leptos::prelude::*;
use leptos::task::spawn_local;
use synapse_shared::blog::{BlogPostDto, BlogSummaryDto};

use crate::api::{self, AsyncResult};

/// The app-level blog store. `Copy` — signal handles, not data.
#[derive(Clone, Copy)]
pub struct BlogStore {
    list: RwSignal<AsyncResult<Vec<BlogSummaryDto>>>,
    list_started: StoredValue<bool>,
}

impl BlogStore {
    /// Created ONCE in `App` and provided as context (the `CatalogStore` pattern).
    pub fn provide() {
        provide_context(Self {
            list: RwSignal::new(AsyncResult::Loading),
            list_started: StoredValue::new(false),
        });
    }

    pub fn from_context() -> Self {
        expect_context::<Self>()
    }

    /// The shared listing signal — first caller fetches; a failure re-arms.
    pub fn list(self) -> RwSignal<AsyncResult<Vec<BlogSummaryDto>>> {
        if !self.list_started.get_value() {
            self.list_started.set_value(true);
            self.list.set(AsyncResult::Loading);
            spawn_local(async move {
                match api::blog_list().await {
                    Ok(posts) => self.list.set(AsyncResult::Loaded(posts)),
                    Err(message) => {
                        self.list_started.set_value(false);
                        self.list.set(AsyncResult::Failed(message));
                    }
                }
            });
        }
        self.list
    }
}

/// One post fetch, spawned per navigation.
pub fn load_post(slug: String) -> RwSignal<AsyncResult<BlogPostDto>> {
    let state = RwSignal::new(AsyncResult::Loading);
    spawn_local(async move {
        match api::blog_post(&slug).await {
            Ok(post) => state.set(AsyncResult::Loaded(post)),
            Err(message) => state.set(AsyncResult::Failed(message)),
        }
    });
    state
}
