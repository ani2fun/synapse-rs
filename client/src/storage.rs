//! The one `localStorage` accessor. Every preference in the client persists through here:
//! reader prefs, the sidebar face, the theme, the workbench language, the problem panes.
//!
//! Both calls swallow failure by design — Safari's private mode and a cookies-disabled profile
//! both make `localStorage` throw rather than return `None`, and a preference that cannot be
//! saved must never take the page down with it.

/// Read a key; absent, unreadable, or storage-denied all read as `None`.
pub(crate) fn get(key: &str) -> Option<String> {
    web_sys::window()?.local_storage().ok()??.get_item(key).ok()?
}

/// Drop a key; a denied removal is silently a no-op. Added in step 51 for the account page's
/// "erase all my data", which must be able to take reading progress with it.
pub(crate) fn remove(key: &str) {
    if let Some(Ok(Some(storage))) = web_sys::window().map(|w| w.local_storage()) {
        let _ = storage.remove_item(key);
    }
}

/// Write a key; a denied write is silently a no-op.
pub(crate) fn set(key: &str, value: &str) {
    if let Some(Ok(Some(storage))) = web_sys::window().map(|w| w.local_storage()) {
        let _ = storage.set_item(key, value);
    }
}
