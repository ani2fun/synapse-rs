//! Placeholder hydration (oracle: `RunnableBlocks.discover` + `MarkdownView.mountBlocks`):
//! after the lesson HTML lands via `inner_html`, find every `div.workbench`, decode its
//! `data-variants`, and mount a live `RunnableBlock` into it. The returned boxed unmount
//! handles keep the mounts alive — dropping them (lesson navigation / unmount) tears the
//! blocks down, which disposes their monaco editors.

use std::any::Any;

use leptos::prelude::*;

use crate::execution::logic;
use crate::execution::view::RunnableBlock;
use crate::hydration::{self, IslandStores};

pub fn hydrate_workbenches(
    root: &web_sys::HtmlElement,
    lesson_path: &[String],
    code_sink: RwSignal<(String, String)>,
    stores: IslandStores,
) -> Vec<Box<dyn Any>> {
    hydration::mount_each(root, "div.workbench", |element| {
        let variants = hydration::decoded_attr(&element, "data-variants")
            .and_then(|json| logic::parse_variants(&json))
            .filter(|variants| !variants.is_empty())?;
        // The authored suite rides in data-spec (absent on plain lesson blocks).
        let spec =
            hydration::decoded_attr(&element, "data-spec").and_then(|json| serde_json::from_str(&json).ok());
        let path = lesson_path.to_vec();
        Some(hydration::mount(element, move || {
            view! { <RunnableBlock variants=variants spec=spec lesson_path=path code_sink=code_sink stores=stores /> }
        }))
    })
}
