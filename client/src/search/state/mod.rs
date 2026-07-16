//! Palette state (the state-layer half): open/query/selection live app-wide so the header
//! button and the global ⌘K listener drive the same singleton palette.

use leptos::prelude::*;

#[derive(Clone, Copy)]
pub struct SearchStore {
    pub is_open: RwSignal<bool>,
    pub query: RwSignal<String>,
    pub selected: RwSignal<usize>,
}

impl SearchStore {
    /// Created ONCE in `App` and provided as context.
    pub fn provide() {
        provide_context(Self {
            is_open: RwSignal::new(false),
            query: RwSignal::new(String::new()),
            selected: RwSignal::new(0),
        });
    }

    pub fn from_context() -> Self {
        expect_context::<Self>()
    }

    pub fn open(self) {
        self.query.set(String::new());
        self.selected.set(0);
        self.is_open.set(true);
    }

    pub fn close(self) {
        self.is_open.set(false);
    }

    pub fn toggle(self) {
        if self.is_open.get_untracked() {
            self.close();
        } else {
            self.open();
        }
    }
}
