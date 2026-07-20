//! The problem page's redesigned Editorial pane (the Claude Design import, step 57): an
//! approach STEPPER for multi-approach editorials (numbered circles over a connector rail,
//! per-approach complexities), a sticky JUMP bar with a scroll-spy over one continuously
//! scrolling document of numbered sections, the solution gated behind a "Reveal the
//! solution" card (collapses again on approach switch — supersedes step 37's
//! always-revealed rule, by design), and the Complexity section rendered as Time/Space
//! cards when its prose parses.
//!
//! The pure half lives in `logic::editorial` — this file only spends the parsed
//! `EditorialDoc`: fragments render through the markdown island per section (memoized by
//! content hash, so approach re-visits are cheap) and hydrate the same island set as every
//! other pane, with `mount_gated_solutions` standing in for the practice widget's
//! always-revealed `mount_solutions`.

use std::any::Any;

use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;

use crate::catalog::logic::editorial::{self, ApproachDoc, EditorialDoc, SectionDoc, SectionKind};
use crate::catalog::logic::pane;
use crate::catalog::state;
use crate::execution::view::SolutionViewer;
use crate::hydration::{self, IslandStores};
use crate::islands::markdown;
use crate::shell::theme::ThemeStore;

/// The spy threshold and the sections' `scroll-margin-top` are a pair: a section counts as
/// active once its top passes 84px below the container top, and a jump lands it at 70px.
const SPY_THRESHOLD_PX: f64 = 84.0;

pub fn editorial_pane(md: &str, load_code: RwSignal<(u32, String, String)>, stores: IslandStores) -> AnyView {
    let doc = editorial::parse_editorial(md);
    if doc.approaches.is_empty() {
        return view! { <p class="psub__note">"No editorial yet for this problem."</p> }.into_any();
    }
    let approach_labels: Vec<String> = doc.approaches.iter().map(|a| a.label.clone()).collect();
    let active_approach = RwSignal::new(pane::section_index(
        &approach_labels,
        &state::editorial_approach(),
    ));
    // The remembered SECTION restore applies to the first rendered approach only — switching
    // approaches afterwards always starts at the top.
    let restore_section = StoredValue::new(true);
    let multi = doc.multi;
    let doc = StoredValue::new(doc);

    view! {
        <div class="pwb-epane">
            {multi.then(|| approach_stepper(doc, active_approach))}
            {(!multi).then(|| single_approach_bar(doc))}
            {move || {
                let index = active_approach.get();
                let (approach, preamble) = doc.with_value(|d| {
                    let at = index.min(d.approaches.len() - 1);
                    (d.approaches[at].clone(), d.preamble.clone())
                });
                approach_body(approach, preamble, restore_section, load_code, stores)
            }}
        </div>
    }
    .into_any()
}

// ─────────────────────────────────────────────────────────────────────────────
// THE SUB-HEAD STRIP — stepper (multi) / complexity bar (single)
// ─────────────────────────────────────────────────────────────────────────────

/// The approach stepper: N equal-width columns over one connector rail. Circle states are
/// positional — active is filled, everything BEFORE it reads as travelled ("done"), the
/// rest as ahead — mirroring the brute → optimal journey the strip's hint names.
fn approach_stepper(doc: StoredValue<EditorialDoc>, active: RwSignal<usize>) -> impl IntoView {
    let count = doc.with_value(|d| d.approaches.len());
    // The rail spans circle-centre to circle-centre: half a column in from each edge.
    #[allow(clippy::cast_precision_loss)]
    let rail_inset = format!(
        "left: {inset:.2}%; right: {inset:.2}%;",
        inset = 50.0 / count as f64
    );
    let buttons: Vec<_> = (0..count)
        .map(|i| {
            let (label, time, space) = doc.with_value(|d| {
                let a = &d.approaches[i];
                (a.label.clone(), a.time.clone(), a.space.clone())
            });
            let remembered = label.clone();
            view! {
                <button
                    class="pwb-estep__btn"
                    on:click=move |_| {
                        if active.get_untracked() != i {
                            active.set(i);
                            state::set_editorial_approach(&remembered);
                        }
                    }
                >
                    <span
                        class="pwb-estep__num"
                        class:pwb-estep__num--active=move || active.get() == i
                        class:pwb-estep__num--done={move || active.get() > i}
                    >
                        {(i + 1).to_string()}
                    </span>
                    <span
                        class="pwb-estep__name"
                        class:pwb-estep__name--active=move || active.get() == i
                    >
                        {label}
                    </span>
                    <span class="pwb-estep__metrics">
                        {time.map(|t| view! {
                            <span
                                class="pwb-estep__time"
                                class:pwb-estep__time--active=move || active.get() == i
                            >
                                {editorial::pretty_o(&t)}
                            </span>
                        })}
                        {space.map(|s| view! {
                            <span class="pwb-estep__space">{format!("space {}", editorial::pretty_o(&s))}</span>
                        })}
                    </span>
                </button>
            }
        })
        .collect();
    view! {
        <div class="pwb-estep">
            <div class="pwb-estep__head">
                <span class="pwb-estep__label">"Approaches"</span>
                <span class="pwb-estep__hint">"brute → optimal"</span>
            </div>
            <div class="pwb-estep__row">
                <div class="pwb-estep__rail" style=rail_inset aria-hidden="true"></div>
                {buttons}
            </div>
        </div>
    }
}

/// The single-approach bar: ★ + the claims from the solution fence meta. An editorial
/// without claims renders no strip at all — legacy content degrades to just the sections.
fn single_approach_bar(doc: StoredValue<EditorialDoc>) -> Option<impl IntoView> {
    let (time, space) = doc.with_value(|d| (d.approaches[0].time.clone(), d.approaches[0].space.clone()));
    if time.is_none() && space.is_none() {
        return None;
    }
    Some(view! {
        <div class="pwb-ebar">
            <span class="pwb-ebar__id">
                <span class="pwb-ebar__star">"★"</span>
                <span class="pwb-ebar__name">"Solution"</span>
            </span>
            <span class="pwb-ebar__pills">
                {time.map(|t| view! {
                    <span class="pwb-ebar__pill pwb-ebar__pill--time">
                        {format!("time {}", editorial::pretty_o(&t))}
                    </span>
                })}
                {space.map(|s| view! {
                    <span class="pwb-ebar__pill pwb-ebar__pill--space">
                        {format!("space {}", editorial::pretty_o(&s))}
                    </span>
                })}
            </span>
        </div>
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// THE SCROLLING BODY — jump bar, spy, numbered sections
// ─────────────────────────────────────────────────────────────────────────────

fn approach_body(
    approach: ApproachDoc,
    preamble: String,
    restore_section: StoredValue<bool>,
    load_code: RwSignal<(u32, String, String)>,
    stores: IslandStores,
) -> AnyView {
    let scroll_ref: NodeRef<leptos::html::Div> = NodeRef::new();
    let active_section = RwSignal::new(0_usize);
    // Only LABELED sections join the jump bar and spy; an approach's unlabeled leading
    // prose renders headerless above them.
    let labels: Vec<String> = approach
        .sections
        .iter()
        .filter(|s| !s.label.is_empty())
        .map(|s| s.label.clone())
        .collect();

    // The FIRST body restores the remembered section; every later body is an approach
    // switch and starts at the top EXPLICITLY — Leptos reuses the scroll container's DOM
    // node across the re-render, so its old scrollTop would otherwise survive the switch
    // (verified live: switching mid-document landed mid-document).
    if restore_section.get_value() {
        restore_section.set_value(false);
        let start = pane::section_index(&labels, &state::pane_prefs().section);
        if start != 0 {
            active_section.set(start);
            Effect::new(move |ran: Option<bool>| {
                if ran == Some(true) {
                    return true;
                }
                let Some(container) = scroll_ref.get() else {
                    return false;
                };
                scroll_section_into_view(&container, start, false);
                true
            });
        }
    } else {
        Effect::new(move |ran: Option<bool>| {
            if ran == Some(true) {
                return true;
            }
            let Some(container) = scroll_ref.get() else {
                return false;
            };
            scroll_body_to_top(&container);
            true
        });
    }

    // The spy: rAF-throttled, reading each labeled section's top relative to the container.
    let spy_pending = StoredValue::new(false);
    let on_scroll = move |_| {
        if spy_pending.get_value() {
            return;
        }
        spy_pending.set_value(true);
        request_animation_frame(move || {
            spy_pending.set_value(false);
            let Some(container) = scroll_ref.get_untracked() else {
                return;
            };
            let container_top = container.get_bounding_client_rect().top();
            let Ok(nodes) = container.query_selector_all("[data-esec]") else {
                return;
            };
            let tops: Vec<f64> = (0..nodes.length())
                .filter_map(|i| nodes.get(i)?.dyn_into::<web_sys::Element>().ok())
                .map(|el| el.get_bounding_client_rect().top() - container_top)
                .collect();
            if tops.is_empty() {
                return;
            }
            let next = editorial::active_section(&tops, SPY_THRESHOLD_PX);
            if active_section.get_untracked() != next {
                active_section.set(next);
            }
        });
    };

    let solution_tag = approach.label.clone();
    let claims = (approach.time.clone(), approach.space.clone());
    let mut number = 0_usize;
    let sections: Vec<_> = approach
        .sections
        .into_iter()
        .map(|section| {
            if section.label.is_empty() {
                markdown_fragment(section.md, load_code, stores).into_any()
            } else {
                let index = number;
                number += 1;
                let tag = (section.kind == SectionKind::Solution && !solution_tag.is_empty())
                    .then(|| solution_tag.clone());
                section_block(index, section, tag, claims.clone(), load_code, stores).into_any()
            }
        })
        .collect();

    view! {
        <div
            class="pwb__pane-scroll synapse-prose pwb-escroll"
            node_ref=scroll_ref
            on:scroll=on_scroll
        >
            {(labels.len() > 1).then(|| jump_bar(labels.clone(), active_section, scroll_ref))}
            {(!preamble.is_empty()).then(|| markdown_fragment(preamble, load_code, stores))}
            {sections}
        </div>
    }
    .into_any()
}

fn jump_bar(
    labels: Vec<String>,
    active_section: RwSignal<usize>,
    scroll_ref: NodeRef<leptos::html::Div>,
) -> impl IntoView {
    let pills: Vec<_> = labels
        .into_iter()
        .enumerate()
        .map(|(i, label)| {
            // Remembered by LABEL — the same carry-over rule the old section pills had.
            let remembered = label.clone();
            view! {
                <button
                    class="pwb-ejump__pill"
                    class:pwb-ejump__pill--active=move || active_section.get() == i
                    on:click=move |_| {
                        if let Some(container) = scroll_ref.get_untracked() {
                            scroll_section_into_view(&container, i, true);
                        }
                        active_section.set(i);
                        state::set_pane_section(&remembered);
                    }
                >
                    {label}
                </button>
            }
        })
        .collect();
    view! {
        <div class="pwb-ejump">
            <span class="pwb-ejump__label">"Jump"</span>
            {pills}
        </div>
    }
}

/// An approach switch starts at the top: the pane when it scrolls, else the page (below
/// the 1024px breakpoint the pane grows and the PAGE carries the content — there the
/// window comes back up to the pane, whose stepper the reader just clicked).
fn scroll_body_to_top(container: &web_sys::HtmlElement) {
    if container.scroll_height() > container.client_height() + 1 {
        container.set_scroll_top(0);
    } else if let Some(window) = web_sys::window() {
        let top = window.scroll_y().unwrap_or(0.0) + container.get_bounding_client_rect().top();
        window.scroll_to_with_x_and_y(0.0, (top - 80.0).max(0.0));
    }
}

/// Scrolls the PANE when the pane is the scroller — `scrollIntoView` was tried first and
/// walks every scrollable ancestor, so each jump also crept the page itself down 60-odd
/// pixels (verified live). Below the 1024px breakpoint the pane stops scrolling (the PAGE
/// carries the content), so the same math targets the window instead. The 70px offset
/// lands the section header just clear of the sticky jump bar, paired with the 84px spy
/// threshold.
fn scroll_section_into_view(container: &web_sys::HtmlElement, index: usize, smooth: bool) {
    const JUMP_OFFSET_PX: f64 = 70.0;
    let Ok(Some(section)) = container.query_selector(&format!("[data-esec='{index}']")) else {
        return;
    };
    if container.scroll_height() > container.client_height() + 1 {
        let options = web_sys::ScrollToOptions::new();
        options.set_behavior(if smooth {
            web_sys::ScrollBehavior::Smooth
        } else {
            web_sys::ScrollBehavior::Auto
        });
        let delta = section.get_bounding_client_rect().top()
            - container.get_bounding_client_rect().top()
            - JUMP_OFFSET_PX;
        options.set_top(f64::from(container.scroll_top()) + delta);
        container.scroll_to_with_scroll_to_options(&options);
    } else if let Some(window) = web_sys::window() {
        // Plain x/y like chrome.rs's scroll_to_heading — the page's own
        // `scroll-behavior: smooth` supplies the easing.
        let top =
            window.scroll_y().unwrap_or(0.0) + section.get_bounding_client_rect().top() - JUMP_OFFSET_PX;
        window.scroll_to_with_x_and_y(0.0, top.max(0.0));
    }
}

/// One numbered section. The Complexity section renders as Time/Space CARDS when its prose
/// parses (missing axes fall back to the fence-meta claims); anything else — and a
/// Complexity section that doesn't parse — renders its markdown as-is.
fn section_block(
    index: usize,
    section: SectionDoc,
    tag: Option<String>,
    claims: (Option<String>, Option<String>),
    load_code: RwSignal<(u32, String, String)>,
    stores: IslandStores,
) -> impl IntoView {
    let body = if section.kind == SectionKind::Complexity {
        match editorial::complexity_prose(&section.md) {
            Some(parsed) => {
                let time = parsed.time.or_else(|| claims.0.map(|v| (v, String::new())));
                let space = parsed.space.or_else(|| claims.1.map(|v| (v, String::new())));
                complexity_cards(time, space).into_any()
            }
            None => markdown_fragment(section.md, load_code, stores).into_any(),
        }
    } else {
        markdown_fragment(section.md, load_code, stores).into_any()
    };
    view! {
        <section class="pwb-esection" data-esec=index.to_string()>
            <div class="pwb-esection__head">
                <span class="pwb-esection__no">{format!("{:02}", index + 1)}</span>
                <h3 class="pwb-esection__title">{section.label}</h3>
                {tag.map(|t| view! { <span class="pwb-esection__tag">{t}</span> })}
            </div>
            {body}
        </section>
    }
}

fn complexity_cards(time: Option<(String, String)>, space: Option<(String, String)>) -> impl IntoView {
    view! {
        <div class="pwb-ecx">
            {time.map(|(value, prose)| view! {
                <div class="pwb-ecx__card">
                    <span class="pwb-ecx__kind">{icon_clock()}" Time"</span>
                    <span class="pwb-ecx__value pwb-ecx__value--time">{editorial::pretty_o(&value)}</span>
                    {(!prose.is_empty()).then(|| view! { <p class="pwb-ecx__prose">{prose}</p> })}
                </div>
            })}
            {space.map(|(value, prose)| view! {
                <div class="pwb-ecx__card">
                    <span class="pwb-ecx__kind">{icon_grid()}" Space"</span>
                    <span class="pwb-ecx__value pwb-ecx__value--space">{editorial::pretty_o(&value)}</span>
                    {(!prose.is_empty()).then(|| view! { <p class="pwb-ecx__prose">{prose}</p> })}
                </div>
            })}
        </div>
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// FRAGMENTS — rendered markdown + the gated solution mount
// ─────────────────────────────────────────────────────────────────────────────

/// One rendered markdown fragment: the same island set as the description pane, except
/// solutions mount GATED. Authored `<details>` fully inside a fragment still force-open —
/// this tab is the answer; only the CODE asks to be revealed.
fn markdown_fragment(
    md: String,
    load_code: RwSignal<(u32, String, String)>,
    stores: IslandStores,
) -> impl IntoView {
    let node_ref: NodeRef<leptos::html::Div> = NodeRef::new();
    let mounts: StoredValue<Vec<Box<dyn Any>>, LocalStorage> = StoredValue::new_local(Vec::new());
    Effect::new(move |ran: Option<bool>| {
        if ran == Some(true) {
            return true;
        }
        let Some(node) = node_ref.get() else { return false };
        let md = md.clone();
        spawn_local(async move {
            match markdown::render(&md).await {
                Ok(html) => {
                    node.set_inner_html(&html);
                    if let Ok(all) = node.query_selector_all("details") {
                        for i in 0..all.length() {
                            if let Some(details) =
                                all.get(i).and_then(|n| n.dyn_into::<web_sys::Element>().ok())
                            {
                                let _ = details.set_attribute("open", "");
                            }
                        }
                    }
                    // Same-breath mounts (the step-11 rule): inner_html content and its
                    // islands must land together or Leptos render effects race the DOM.
                    mounts.update_value(|m| {
                        m.extend(mount_gated_solutions(&node, load_code, stores.theme));
                        m.extend(crate::execution::view::hydrate_fence_groups(
                            &node,
                            stores.codebench,
                        ));
                        m.extend(crate::catalog::view::diagrams::hydrate_diagrams(&node));
                        m.extend(crate::viz::blocks::mount_widgets(&node));
                    });
                }
                Err(_) => node.set_text_content(Some(&md)),
            }
        });
        true
    });
    on_cleanup(move || mounts.set_value(Vec::new()));
    view! { <div class="pwb-efrag" node_ref=node_ref></div> }
}

/// `mount_solutions`' twin for the editorial pane: the same `.solution-block` discovery,
/// but each block mounts a `GatedSolution` — collapsed behind the reveal card by default.
fn mount_gated_solutions(
    root: &web_sys::HtmlElement,
    load_code: RwSignal<(u32, String, String)>,
    theme: ThemeStore,
) -> Vec<Box<dyn Any>> {
    hydration::mount_each(root, "div.solution-block", |element| {
        let variants = hydration::decoded_attr(&element, "data-variants")
            .and_then(|json| crate::execution::logic::parse_variants(&json))
            .filter(|v| !v.is_empty())?;
        Some(hydration::mount(element, move || {
            view! { <GatedSolution variants=variants load_code=load_code theme=theme /> }
        }))
    })
}

/// The reveal gate. The viewer mounts on reveal (visible, so Monaco measures right away)
/// and unmounts on Hide; an approach switch re-creates the fragment, so it collapses again.
#[allow(clippy::needless_pass_by_value)]
#[component]
fn GatedSolution(
    variants: Vec<crate::execution::logic::Variant>,
    load_code: RwSignal<(u32, String, String)>,
    theme: ThemeStore,
) -> impl IntoView {
    let revealed = RwSignal::new(false);
    let variants = StoredValue::new(variants);
    view! {
        {move || if revealed.get() {
            view! {
                <div class="pwb-ereveal-open">
                    <div class="pwb-ereveal-open__bar">
                        <button class="wb__ghost" on:click=move |_| revealed.set(false)>
                            {icon_chevron_up()}
                            " Hide"
                        </button>
                    </div>
                    <SolutionViewer
                        variants=variants.read_value().clone()
                        load_code=load_code
                        theme=theme
                    />
                    <p class="pwb-ereveal__note">
                        {icon_info()}
                        " Reference only — edit and run it in the panel on the right."
                    </p>
                </div>
            }
            .into_any()
        } else {
            view! {
                <button class="pwb-ereveal" on:click=move |_| revealed.set(true)>
                    <span class="pwb-ereveal__icon">{icon_code()}</span>
                    <span class="pwb-ereveal__copy">
                        <span class="pwb-ereveal__title">"Reveal the solution"</span>
                        <span class="pwb-ereveal__sub">
                            "Give the intuition a shot before you peek at the code."
                        </span>
                    </span>
                </button>
            }
            .into_any()
        }}
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// GLYPHS (Lucide, stroke=currentColor like the tab icons)
// ─────────────────────────────────────────────────────────────────────────────

fn icon_code() -> AnyView {
    view! {
        <svg viewBox="0 0 24 24" width="19" height="19" fill="none" stroke="currentColor"
             stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="m9 18 6-6-6-6"></path>
            <path d="M4 6v12" opacity="0.4"></path>
        </svg>
    }
    .into_any()
}

fn icon_chevron_up() -> AnyView {
    view! {
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor"
             stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="m18 15-6-6-6 6"></path>
        </svg>
    }
    .into_any()
}

fn icon_info() -> AnyView {
    view! {
        <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor"
             stroke-width="2" stroke-linecap="round" aria-hidden="true">
            <circle cx="12" cy="12" r="10"></circle>
            <path d="M12 16v-4M12 8h.01"></path>
        </svg>
    }
    .into_any()
}

fn icon_clock() -> AnyView {
    view! {
        <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor"
             stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <circle cx="12" cy="12" r="9"></circle>
            <path d="M12 7v5l3 2"></path>
        </svg>
    }
    .into_any()
}

fn icon_grid() -> AnyView {
    view! {
        <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor"
             stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <rect x="4" y="4" width="16" height="16" rx="2"></rect>
            <path d="M4 9h16M9 20V9"></path>
        </svg>
    }
    .into_any()
}
