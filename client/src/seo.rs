//! The document head, client side (step 50).
//!
//! The server renders the correct `<title>` and meta tags for the URL a visitor LANDS on
//! (`platform::static_routes`). That covers crawlers and shared links, which never run JS —
//! but it goes stale the moment the SPA navigates, because a client-side route change does not
//! re-fetch the document. Without this, opening lesson A and clicking through to lesson B
//! leaves the tab, the history entry and the bookmark all saying A.
//!
//! The title FORMAT is duplicated from the server on purpose. The alternative is shipping the
//! rendered title down the wire on every lesson payload, which is a wire field and a contract
//! to keep in step, to save one `format!`. The pin is
//! `title_for_lesson`'s unit test against the server's documented shape.

/// The site name, and the fallback title for any page without one of its own.
pub const SITE_NAME: &str = "Synapse";

/// `Book · Lesson — Synapse`, matching `platform::static_routes::title_for` exactly. The book
/// leads because the left of the string is what survives truncation in a tab strip.
pub fn title_for_lesson(book_title: Option<&str>, lesson_title: &str) -> String {
    match book_title {
        Some(book) => format!("{book} · {lesson_title} — {SITE_NAME}"),
        None => format!("{lesson_title} — {SITE_NAME}"),
    }
}

/// Set `document.title`. Silently does nothing off-DOM — a title is never worth a panic.
pub fn set_title(title: &str) {
    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
        document.set_title(title);
    }
}

/// Point `<meta name="description">` at `description`, creating the tag if the server did not
/// (dev serves Vite's index, which has no injected head).
pub fn set_description(description: &str) {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let existing = document
        .query_selector("meta[name=\"description\"]")
        .ok()
        .flatten();
    let element = if let Some(element) = existing {
        element
    } else {
        let Ok(created) = document.create_element("meta") else {
            return;
        };
        let _ = created.set_attribute("name", "description");
        if let Some(head) = document.head() {
            let _ = head.append_child(&created);
        }
        created
    };
    let _ = element.set_attribute("content", description);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_book_leads_the_lesson_title() {
        assert_eq!(
            title_for_lesson(Some("DSA"), "Singly linked lists"),
            "DSA · Singly linked lists — Synapse",
            "must match platform::static_routes::title_for, or the tab changes on navigation"
        );
    }

    #[test]
    fn an_unknown_book_still_produces_a_usable_title() {
        assert_eq!(
            title_for_lesson(None, "Intro"),
            "Intro — Synapse",
            "the index may not have loaded yet; a lesson title alone still beats the placeholder"
        );
    }
}
