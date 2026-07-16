//! The theme store (oracle: `Theme.scala`, ADR-S018). The SOURCE OF TRUTH is the `.dark`
//! class on `<html>` — the pre-paint bootstrap in `index.html` sets it before first render
//! (stored `"theme"` value, else the OS preference), and this store mirrors it into a signal
//! seeded from the live class, so the first paint of the toggle icon is already right.

use leptos::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Light,
    Dark,
}

#[derive(Clone, Copy)]
pub struct ThemeStore {
    pub mode: RwSignal<Mode>,
}

pub fn html_is_dark() -> bool {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
        .is_some_and(|root| root.class_list().contains("dark"))
}

impl ThemeStore {
    /// Created ONCE in `App` — seeded from what the bootstrap already painted.
    pub fn provide() {
        let mode = if html_is_dark() { Mode::Dark } else { Mode::Light };
        provide_context(Self {
            mode: RwSignal::new(mode),
        });
    }

    pub fn from_context() -> Self {
        expect_context::<Self>()
    }

    pub fn is_dark(self) -> bool {
        self.mode.get_untracked() == Mode::Dark
    }

    /// Class → storage → signal, in one breath (storage failures — private mode — are fine;
    /// the class still flips, the choice just doesn't persist).
    pub fn set(self, next: Mode) {
        let dark = next == Mode::Dark;
        if let Some(root) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.document_element())
        {
            let _ = root.class_list().toggle_with_force("dark", dark);
        }
        if let Some(Ok(Some(storage))) = web_sys::window().map(|w| w.local_storage()) {
            let _ = storage.set_item("theme", if dark { "dark" } else { "light" });
        }
        self.mode.set(next);
    }

    pub fn toggle(self) {
        let next = match self.mode.get_untracked() {
            Mode::Dark => Mode::Light,
            Mode::Light => Mode::Dark,
        };
        self.set(next);
    }
}
