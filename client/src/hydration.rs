//! The hydration seam (step 58): how out-of-tree islands mount, and the store caravan
//! they carry.
//!
//! The markdown pipeline plants placeholder `div`s (`div.workbench`, `div.quiz-block`, …)
//! whose payloads ride URI-encoded `data-*` attributes; once the rendered HTML lands, each
//! feature's hydrator finds its placeholders and mounts live components into them. Every
//! hydrator used to hand-roll the same mechanics — and thread the same four stores through
//! every level. Both halves of that seam live here now: [`IslandStores`] (what a mounted
//! island carries) and the mount kit (how a placeholder becomes a component).
//!
//! This is view-side infrastructure — the client's `platform/`-equivalent. It touches
//! `leptos`/`web-sys`, so it can never live under a `logic/` path (purity gate), and it is
//! not FFI, so it does not belong in `islands/`.

use std::any::Any;

use leptos::prelude::*;
use wasm_bindgen::JsCast;

// ─────────────────────────────────────────────────────────────────────────────
// THE STORE CARAVAN
// ─────────────────────────────────────────────────────────────────────────────

/// The out-of-tree store caravan, bundled.
///
/// Hydrated islands mount via `leptos::mount::mount_to`, which starts a FRESH root owner —
/// App's context is unreachable from inside them (the step-17 lesson). The fix is
/// capture-then-carry: a surface still under App's owner calls [`IslandStores::capture`]
/// and threads the bundle into every mount as one `Copy` prop. This struct is the canonical
/// home of that rule; prop sites just point here.
///
/// Only App-level context stores belong in the bundle. Per-render signals (`code_sink`,
/// `load_code`, `c4_selected`, …) are minted by the page that owns them and stay separate
/// props — bundling them would break the "context snapshot" meaning of this struct.
///
/// Deliberate width trade: carriers take the whole bundle even when they read only part of
/// it (`RunnableBlock` never touches `codebench`) — a second, narrower bundle type would be
/// more total interface than it saves on a `Copy` struct. Leaf components that need ONE
/// store keep their narrow prop instead.
#[derive(Clone, Copy)]
pub struct IslandStores {
    pub auth: crate::identity::state::AuthStore,
    pub theme: crate::shell::theme::ThemeStore,
    pub viz_modal: crate::viz::modal::VizModalStore,
    pub codebench: crate::execution::view::CodebenchStore,
}

impl IslandStores {
    /// Capture the four context stores. IN-TREE ONLY: call this inside a component still
    /// under App's owner (the reader, the problem page) — never from inside a mounted
    /// island, where the contexts are gone and `expect_context` panics.
    #[must_use]
    pub fn capture() -> Self {
        Self {
            auth: crate::identity::state::AuthStore::from_context(),
            theme: crate::shell::theme::ThemeStore::from_context(),
            viz_modal: crate::viz::modal::VizModalStore::from_context(),
            codebench: crate::execution::view::CodebenchStore::from_context(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// THE MOUNT KIT
// ─────────────────────────────────────────────────────────────────────────────

/// Every element under `root` matching `selector`, already narrowed to `HtmlElement`.
/// Non-element nodes (there are none in practice) are skipped silently.
#[must_use]
pub fn elements(root: &web_sys::HtmlElement, selector: &str) -> Vec<web_sys::HtmlElement> {
    let mut found = Vec::new();
    let Ok(nodes) = root.query_selector_all(selector) else {
        return found;
    };
    for index in 0..nodes.length() {
        let Some(node) = nodes.get(index) else { continue };
        if let Ok(element) = node.dyn_into::<web_sys::HtmlElement>() {
            found.push(element);
        }
    }
    found
}

/// A `data-*` payload: read + URI-decode. The pipeline plants every payload
/// `encodeURIComponent`-ed, so the decode is part of reading, not a caller concern.
#[must_use]
pub fn decoded_attr(element: &web_sys::HtmlElement, name: &str) -> Option<String> {
    let encoded = element.get_attribute(name)?;
    js_sys::decode_uri_component(&encoded).ok().map(String::from)
}

/// Mount one island into `element` and box its unmount handle. The boxed handle is the
/// island's lifetime: hold it to keep the mount alive, drop it to tear the island down
/// (which disposes its reactive owner — and any Monaco editor with it).
#[must_use]
pub fn mount<F, N>(element: web_sys::HtmlElement, view_fn: F) -> Box<dyn Any>
where
    F: FnOnce() -> N + 'static,
    N: IntoView + 'static,
{
    Box::new(leptos::mount::mount_to(element, view_fn))
}

/// The whole discovery loop: for each `selector` match under `root`, `mount_one` decodes
/// the placeholder and mounts — returning `None` skips it (the decode-or-skip rule: a
/// malformed placeholder is left inert, never a panic). The returned handles keep the
/// mounts alive; see [`mount`].
#[must_use]
pub fn mount_each(
    root: &web_sys::HtmlElement,
    selector: &str,
    mount_one: impl FnMut(web_sys::HtmlElement) -> Option<Box<dyn Any>>,
) -> Vec<Box<dyn Any>> {
    elements(root, selector)
        .into_iter()
        .filter_map(mount_one)
        .collect()
}
