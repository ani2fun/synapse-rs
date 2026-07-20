//! The code-block tab group (step 41). The pipeline wraps every display-language fence in a
//! `div.fence-group` — one `.fence-group__bar` host followed by one shiki `<figure>` per pane —
//! and this module mounts the chrome: language TABS when adjacent fences offered the same idea
//! in another language, a lone `▶` pill otherwise, with copy + "Try in Editor" on the far right
//! where they never cover the code.
//!
//! The panes are REAL rendered figures, not a payload we re-highlight: switching tabs only
//! toggles a class on them, so there is no second highlighting path to keep honest.

use std::any::Any;

use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::execution::logic::display_lang;
use crate::execution::view::codebench::{CodebenchRequest, CodebenchStore, runnable_fence};
use crate::execution::view::icons::{icon_check, icon_chevron_right, icon_copy, icon_play};
use crate::hydration;

// ─────────────────────────────────────────────────────────────────────────────
// DISCOVERY
// ─────────────────────────────────────────────────────────────────────────────

/// One pane: the fence's language alias, its source, and the figure the bar drives.
#[derive(Clone)]
struct Pane {
    language: String,
    code: String,
    figure: web_sys::Element,
}

/// Mount a header bar into every `.fence-group` the pipeline planted. Blocks inside interactive
/// hosts (workbench, solutions, quiz, diagrams) never become fence groups — the pipeline already
/// claimed those fences — so no exclusion dance is needed here.
pub fn hydrate_fence_groups(root: &web_sys::HtmlElement, store: CodebenchStore) -> Vec<Box<dyn Any>> {
    hydration::mount_each(root, "div.fence-group", |group| {
        // The mount target is the group's CHILD bar, not the placeholder itself.
        let bar = group
            .query_selector("div.fence-group__bar")
            .ok()
            .flatten()
            .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())?;
        let panes = collect_panes(&group);
        if panes.is_empty() {
            return None;
        }
        Some(hydration::mount(bar, move || {
            view! { <FenceBar panes=panes.clone() store=store /> }
        }))
    })
}

/// Read the panes straight off the rendered figures — `data-language` for the alias, the `<pre>`
/// text for the source (the same seam the old floating pill used).
fn collect_panes(group: &web_sys::HtmlElement) -> Vec<Pane> {
    let mut panes = Vec::new();
    let Ok(figures) = group.query_selector_all("figure[data-rehype-pretty-code-figure]") else {
        return panes;
    };
    for index in 0..figures.length() {
        let Some(node) = figures.get(index) else { continue };
        let Ok(figure) = node.dyn_into::<web_sys::Element>() else {
            continue;
        };
        let Ok(Some(pre)) = figure.query_selector("pre[data-language]") else {
            continue;
        };
        let Some(language) = pre.get_attribute("data-language") else {
            continue;
        };
        let code = pre.text_content().unwrap_or_default();
        if code.trim().is_empty() {
            continue;
        }
        panes.push(Pane {
            language,
            code,
            figure,
        });
    }
    panes
}

// ─────────────────────────────────────────────────────────────────────────────
// THE BAR
// ─────────────────────────────────────────────────────────────────────────────

/// Tabs (or a pill) on the left, actions on the right. The active pane is the ONE piece of
/// state; the figures are plain DOM, so an effect drives their visibility class directly.
#[component]
fn FenceBar(panes: Vec<Pane>, store: CodebenchStore) -> impl IntoView {
    let active = RwSignal::new(0_usize);
    let panes = StoredValue::new_local(panes);
    let count = panes.with_value(Vec::len);

    // Mount-once panes, `.hidden` for the rest — the practice widget's pattern. Runs on mount too,
    // so pane 0 wins before the first paint.
    Effect::new(move |_| {
        let current = active.get();
        panes.with_value(|panes| {
            for (index, pane) in panes.iter().enumerate() {
                let _ = pane
                    .figure
                    .class_list()
                    .toggle_with_force("fence-group__pane--hidden", index != current);
            }
        });
    });

    let tabs = (0..count)
        .map(|index| {
            let label = panes.with_value(|p| display_lang(&p[index].language));
            // The glyph + text alone leave the button unnamed to a screen reader, and nothing
            // announces WHICH language is showing — so name it and carry the pressed state.
            let aria = format!("Show the {label} version");
            view! {
                <button
                    class="fence-group__tab"
                    class:fence-group__tab--active=move || active.get() == index
                    aria-label=aria
                    aria-pressed=move || if active.get() == index { "true" } else { "false" }
                    on:click=move |_| active.set(index)
                >
                    {icon_chevron_right("fence-group__tab-ic")}
                    <span>{label}</span>
                </button>
            }
        })
        .collect_view();

    let head = move || {
        // One pane needs no tab bar — the design's lone-block variant is a static ▶ pill.
        let label = panes.with_value(|p| display_lang(&p[0].language));
        view! {
            <span class="fence-group__pill">
                {icon_play("fence-group__pill-ic")}
                <span>{label}</span>
            </span>
        }
    };

    let source_of = move || panes.with_value(|p| p[active.get_untracked()].code.clone());
    let language_of = move || panes.with_value(|p| p[active.get_untracked()].language.clone());
    // "Try in Editor" follows the SELECTED tab, and only appears when the sandbox speaks that
    // language — prose fences (bash, json, yaml) still get the bar and the copy button.
    let runnable = Memo::new(move |_| panes.with_value(|p| runnable_fence(&p[active.get()].language)));

    view! {
        <div class="fence-group__lead">
            {if count > 1 { tabs.into_any() } else { head().into_any() }}
        </div>
        <div class="fence-group__actions">
            {copy_button(source_of)}
            <Show when=move || runnable.get()>
                <button
                    class="fence-group__try"
                    title="Open this code in the editor — run it, feed it input, make it yours"
                    on:click=move |_| {
                        store.open(CodebenchRequest {
                            code: source_of(),
                            language: language_of(),
                        });
                    }
                >
                    {icon_play("fence-group__try-ic")}
                    <span>"Try in Editor"</span>
                </button>
            </Show>
        </div>
    }
}

/// The quiet copy button (the workbench's `copy_button`, minus the Monaco buffer — a fence's
/// source is fixed, so the closure just hands it over). Swaps to a check for 1.4 s.
fn copy_button(source: impl Fn() -> String + Copy + 'static) -> impl IntoView {
    let copied = RwSignal::new(false);
    view! {
        <button
            class="fence-group__copy"
            class:fence-group__copy--done=move || copied.get()
            aria-label="Copy code"
            title="Copy code"
            on:click=move |_| {
                if let Some(window) = web_sys::window() {
                    let _ = window.navigator().clipboard().write_text(&source());
                }
                copied.set(true);
                gloo_timers::callback::Timeout::new(1_400, move || copied.set(false)).forget();
            }
        >
            {move || if copied.get() {
                icon_check("fence-group__copy-ic").into_any()
            } else {
                icon_copy("fence-group__copy-ic").into_any()
            }}
        </button>
    }
}
