//! The reading-preferences panel (oracle: `ReaderPrefs`, post-33 `7ed3909`): a bottom-right FAB
//! opening a popover of segmented controls — size · leading · type family · width — dismissed
//! by Esc, the scrim, or the FAB itself. (The dark-mode row joins with the theme step.)

use leptos::ev;
use leptos::prelude::*;

use crate::catalog::logic::prefs::{FAMILIES, LEADINGS, Prefs, SIZES, WIDTHS};
use crate::catalog::state::PrefsStore;

#[component]
pub fn ReaderPrefsFab() -> impl IntoView {
    let open = RwSignal::new(false);
    let store = PrefsStore::from_context();

    let esc = window_event_listener(ev::keydown, move |event| {
        if event.key() == "Escape" {
            open.set(false);
        }
    });
    on_cleanup(move || esc.remove());

    view! {
        <div class="reader-prefs">
            <button
                class="reader-prefs-fab"
                title="Reading preferences"
                aria-label="Reading preferences"
                on:click=move |_| open.update(|o| *o = !*o)
            >
                "Aa"
            </button>
            {move || {
                open.get()
                    .then(|| view! { <div class="reader-prefs-scrim" on:click=move |_| open.set(false)></div> })
            }}
            {move || open.get().then(|| panel(store))}
        </div>
    }
}

fn panel(store: PrefsStore) -> impl IntoView {
    view! {
        <div class="reader-prefs-pop">
            <div class="reader-prefs-pop__eyebrow">"Reading preferences"</div>
            {segmented("Size", &SIZES, store, |p| p.size, |p, t| Prefs { size: t, ..p }, false)}
            {segmented("Leading", &LEADINGS, store, |p| p.leading, |p, t| Prefs { leading: t, ..p }, false)}
            {segmented("Type family", &FAMILIES, store, |p| p.family, |p, t| Prefs { family: t, ..p }, true)}
            {segmented("Width", &WIDTHS, store, |p| p.width, |p, t| Prefs { width: t, ..p }, false)}
            <button class="reader-prefs__reset" on:click=move |_| store.reset()>
                "Reset to defaults"
            </button>
        </div>
    }
}

/// A three-way segmented control; `preview` renders each option in the font its token names.
fn segmented(
    label: &'static str,
    options: &'static [(&'static str, &'static str); 3],
    store: PrefsStore,
    read: fn(&Prefs) -> &'static str,
    write: fn(Prefs, &'static str) -> Prefs,
    preview: bool,
) -> impl IntoView {
    let buttons: Vec<_> = options
        .iter()
        .map(|&(token, display)| {
            let class = if preview {
                format!("reader-prefs__opt reader-prefs__opt--{token}")
            } else {
                "reader-prefs__opt".to_owned()
            };
            view! {
                <button
                    class=class
                    class:reader-prefs__opt--active=move || read(&store.prefs.read()) == token
                    on:click=move |_| store.commit(write(store.prefs.get_untracked(), token))
                >
                    {display}
                </button>
            }
        })
        .collect();
    view! {
        <div class="reader-prefs__group">
            <div class="reader-prefs__label">{label}</div>
            <div class="reader-prefs__seg">{buttons}</div>
        </div>
    }
}
