//! The guided-tour carousel (oracle: `SynapseTour.scala`, the landing's centerpiece): four
//! slides — the library, runnable code, the reader, and a REAL Visualise widget playing a
//! hand-authored two-pointer reverse. Auto-advances every 7 s, pauses on hover, wraps; the
//! slide rebuilds exactly once per index change (a fresh widget per visit, stable while
//! shown).

use crate::viz::engine::graph::{Annotation, NodeId, VizCases, VizCursor, VizGraph, VizNode, VizStep};
use crate::viz::engine::vocabulary::VizStructure;
use leptos::prelude::*;
use synapse_shared::catalog::SynapseIndexDto;

use crate::catalog::logic;
use crate::viz::host::WidgetHost;

const SLIDE_MS: u32 = 7_000;
const N: usize = 4;

#[component]
pub fn SynapseTour(index: Signal<Option<SynapseIndexDto>>) -> impl IntoView {
    let idx = RwSignal::new(0_usize);
    let paused = StoredValue::new(false);
    let timer: StoredValue<Option<gloo_timers::callback::Interval>, LocalStorage> =
        StoredValue::new_local(None);
    timer.set_value(Some(gloo_timers::callback::Interval::new(SLIDE_MS, move || {
        if !paused.get_value() {
            idx.update(|i| *i = (*i + 1) % N);
        }
    })));
    on_cleanup(move || timer.set_value(None));
    let go = move |i: i64| {
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_possible_wrap
        )]
        idx.set(i.rem_euclid(N as i64) as usize);
    };

    let dots: Vec<_> = (0..N)
        .map(|i| {
            view! {
                <button
                    class="syn-tour__dot"
                    class:syn-tour__dot--active=move || idx.get() == i
                    aria-label=format!("Go to slide {}", i + 1)
                    on:click=move |_| idx.set(i)
                ></button>
            }
        })
        .collect();

    view! {
        <div
            class="syn-tour"
            on:mouseenter=move |_| paused.set_value(true)
            on:mouseleave=move |_| paused.set_value(false)
        >
            <div class="syn-tour__stage">{move || slide_view(idx.get(), index)}</div>
            <div class="syn-tour__foot">
                <div class="syn-tour__label">
                    {move || {
                        let i = idx.get();
                        format!("{:02} / {N:02} — {}", i + 1, EYEBROWS[i])
                    }}
                </div>
                <div class="syn-tour__dots">{dots}</div>
                <div class="syn-tour__nav">
                    <button
                        class="syn-tour__arrow"
                        aria-label="Previous"
                        on:click=move |_| {
                            #[allow(clippy::cast_possible_wrap)]
                            go(idx.get_untracked() as i64 - 1);
                        }
                    >
                        <span class="syn-tour__arrow-ic syn-tour__arrow-ic--flip">{chevron()}</span>
                    </button>
                    <button
                        class="syn-tour__arrow syn-tour__arrow--primary"
                        aria-label="Next"
                        on:click=move |_| {
                            #[allow(clippy::cast_possible_wrap)]
                            go(idx.get_untracked() as i64 + 1);
                        }
                    >
                        <span class="syn-tour__arrow-ic">{chevron()}</span>
                    </button>
                </div>
            </div>
        </div>
    }
}

const EYEBROWS: [&str; N] = ["The Library", "Runnable code", "Find your way", "See it work"];

#[allow(clippy::too_many_lines)] // four authored slides, one cohesive block
fn slide_view(i: usize, index: Signal<Option<SynapseIndexDto>>) -> AnyView {
    let (title, desc, bullets): (&str, &str, [&str; 3]) = match i {
        0 => (
            "Open a book, start reading.",
            "The home library lists every book Synapse has — each one set like a publication \
             you'd actually want to read, with rendered diagrams, interactive widgets and \
             runnable code.",
            [
                "Re-read any time — no paywall, no ads",
                "Pick up exactly where you left off",
                "Yours to keep — 100% of the writing",
            ],
        ),
        1 => (
            "Run it right in the page.",
            "Every code block is live. Press Run and it executes in the browser — Python, \
             Java, SQL, Go and more. Sign in to make the example your own.",
            [
                "No setup, no install — it just runs",
                "Edit the source once you sign in",
                "A full Monaco editor, same as your IDE",
            ],
        ),
        2 => (
            "Never lose the thread.",
            "A book is a lot of pages. The reader keeps you oriented — a chapter sidebar, a \
             live table of contents, breadcrumbs and a minimap, all one glance away.",
            [
                "Sidebar, TOC, breadcrumbs & minimap",
                "Resume reading lands you back in place",
                "Type size, leading and width, your way",
            ],
        ),
        _ => (
            "Watch the idea move.",
            "Diagrams and step-through visualizations turn an abstract mechanism into \
             something you can see — arrays, trees, graphs and system designs, animated as \
             they run.",
            [
                "Step through an algorithm line by line",
                "Mermaid, D2 & interactive C4 diagrams",
                "Practice problems with a hidden test suite",
            ],
        ),
    };
    let visual = match i {
        0 => visual_library(index),
        1 => visual_code(),
        2 => visual_reader(),
        _ => visual_viz(),
    };
    let bullet_views: Vec<_> = bullets
        .iter()
        .map(|b| {
            view! {
                <li class="syn-tour__bullet">
                    <span class="syn-tour__tick" aria-hidden="true"></span>
                    {*b}
                </li>
            }
        })
        .collect();
    view! {
        <div class="syn-tour__slide">
            <div class="syn-tour__left">
                <div class="syn-tour__eyebrow">{format!("{:02} — {}", i + 1, EYEBROWS[i])}</div>
                <h2 class="syn-tour__title">{title}</h2>
                <p class="syn-tour__desc">{desc}</p>
                <ul class="syn-tour__bullets">{bullet_views}</ul>
            </div>
            <div class="syn-tour__right" aria-hidden="true">{visual}</div>
        </div>
    }
    .into_any()
}

// ─────────────────────────────────────────────────────────────────────────────
// SLIDE VISUALS
// ─────────────────────────────────────────────────────────────────────────────

/// Four live book cards: hrefs resolve reactively once the index loads (fallback "/").
fn visual_library(index: Signal<Option<SynapseIndexDto>>) -> AnyView {
    let card = move |slug: &'static str,
                     thumb: &'static str,
                     mark: AnyView,
                     name: &'static str,
                     sub: &'static str| {
        let href = move || {
            index
                .get()
                .as_ref()
                .and_then(|idx| logic::find_book(idx, slug))
                .and_then(logic::first_lesson_path)
                .map_or_else(|| "/".to_owned(), |path| format!("/synapse/{path}"))
        };
        view! {
            <a class="tour-lib__card" href=href>
                <div class=format!("tour-lib__thumb {thumb}")>{mark}</div>
                <div class="tour-lib__name">{name}</div>
                <div class="tour-lib__sub">{sub}</div>
            </a>
        }
    };
    view! {
        <div class="tour-lib">
            {card("system-design-from-first-principles", "tour-lib__thumb--a", mark_graph(), "System Design", "Distributed · Architecture")}
            {card("dsa", "tour-lib__thumb--b", mark_tree(), "Data Structures & Algorithms", "Python · Java")}
            {card("low-level-design", "tour-lib__thumb--c", mark_layers(), "Low Level Design", "OOP · SOLID · UML")}
            {card("python", "tour-lib__thumb--d", mark_code(), "Programming Languages", "Python · Java · SQL")}
        </div>
    }
    .into_any()
}

/// The runnable-block mockup: Python · Java tabs, five hand-tokenized lines, the output.
fn visual_code() -> AnyView {
    let line = |n: usize, src: AnyView| {
        view! {
            <div class="tour-code__line">
                <span class="tour-code__ln">{n}</span>
                <span class="tour-code__src">{src}</span>
            </div>
        }
    };
    view! {
        <div class="tour-code">
            <div class="tour-code__bar">
                <div class="tour-code__langs">
                    <span class="tour-code__lang tour-code__lang--active">"Python"</span>
                    <span class="tour-code__lang">"Java"</span>
                </div>
                <span class="tour-code__run">"▶ Run"</span>
            </div>
            <div class="tour-code__body">
                {line(1, view! { "nums = [" <span class="tk-num">"1"</span> ", " <span class="tk-num">"2"</span> ", " <span class="tk-num">"3"</span> ", " <span class="tk-num">"4"</span> ", " <span class="tk-num">"5"</span> "]" }.into_any())}
                {line(2, view! { "doubled = []" }.into_any())}
                {line(3, view! { <span class="tk-kw">"for"</span> " n " <span class="tk-kw">"in"</span> " nums:" }.into_any())}
                {line(4, view! { "    doubled." <span class="tk-fn">"append"</span> "(n " <span class="tk-op">"*"</span> " " <span class="tk-num">"2"</span> ")" }.into_any())}
                {line(5, view! { <span class="tk-fn">"print"</span> "(doubled)" }.into_any())}
            </div>
            <div class="tour-code__out">
                <div class="tour-code__out-label">"Output"</div>
                <div class="tour-code__out-val">"[2, 4, 6, 8, 10]"</div>
            </div>
        </div>
    }
    .into_any()
}

/// The reader mockup: chapter rail · page · minimap.
fn visual_reader() -> AnyView {
    let chapter = |n: usize, name: &'static str, active: bool| {
        let class = if active {
            "tour-reader__chapter tour-reader__chapter--active"
        } else {
            "tour-reader__chapter"
        };
        view! { <div class=class><span>{n}</span>" "{name}</div> }
    };
    view! {
        <div class="tour-reader">
            <div class="tour-reader__nav">
                <div class="tour-reader__eyebrow">"Chapters"</div>
                {chapter(1, "Foundations", false)}
                {chapter(2, "Storage", false)}
                {chapter(3, "Replication", true)}
                {chapter(4, "Partitioning", false)}
                {chapter(5, "Consistency", false)}
            </div>
            <div class="tour-reader__page">
                <div class="tour-reader__crumb">"Systems / Replication"</div>
                <div class="tour-reader__title">"Leaders & Followers"</div>
                <p class="tour-reader__text">
                    "A replica designated the leader accepts all writes. Followers receive the \
                     same changes as a log and apply them in order…"
                </p>
                <div class="tour-reader__bar" style="width: 100%"></div>
                <div class="tour-reader__bar" style="width: 72%"></div>
                <div class="tour-reader__pager">
                    <span class="tour-reader__prev">"← Storage"</span>
                    <span class="tour-reader__next">"Partitioning →"</span>
                </div>
            </div>
            <div class="tour-reader__mini">
                <div class="tour-reader__mini-bar tour-reader__mini-bar--on"></div>
                <div class="tour-reader__mini-bar"></div>
                <div class="tour-reader__mini-bar"></div>
                <div class="tour-reader__mini-bar"></div>
            </div>
        </div>
    }
    .into_any()
}

/// The REAL widget: the same `WidgetHost` every lesson uses, fed a hand-authored trace.
fn visual_viz() -> AnyView {
    view! {
        <div class="tour-viz">
            <WidgetHost
                name="array".to_owned()
                structure=Some(VizStructure::Array)
                cases=Some(reverse_trace())
            />
        </div>
    }
    .into_any()
}

/// Two-pointer in-place reverse of `[a, e, i, o, u]` — three authored steps.
fn reverse_trace() -> VizCases {
    let cell = |slot: i32, label: &str| VizNode {
        id: NodeId::new(slot.to_string()),
        label: label.to_owned(),
        kind: "cell".to_owned(),
        slot: Some(slot),
        ..VizNode::default()
    };
    let step = |vals: [&str; 5], l: i32, r: i32, changed: &[i32], note: &str| VizStep {
        nodes: vals
            .iter()
            .enumerate()
            .map(|(i, v)| {
                #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
                cell(i as i32, v)
            })
            .collect(),
        cursor: vec![
            VizCursor {
                name: "left".to_owned(),
                target: NodeId::new(l.to_string()),
                color: String::new(),
            },
            VizCursor {
                name: "right".to_owned(),
                target: NodeId::new(r.to_string()),
                color: String::new(),
            },
        ],
        changed: changed.iter().map(|i| NodeId::new(i.to_string())).collect(),
        annotation: Annotation {
            body: note.to_owned(),
            ..Annotation::default()
        },
        ..VizStep::default()
    };
    VizCases {
        cases: vec![VizGraph {
            title: "Reverse in place · two pointers".to_owned(),
            steps: vec![
                step(
                    ["a", "e", "i", "o", "u"],
                    0,
                    4,
                    &[],
                    "left = 0, right = 4 — swap arr[left] and arr[right], then step inward.",
                ),
                step(
                    ["u", "e", "i", "o", "a"],
                    1,
                    3,
                    &[0, 4],
                    "Swapped a and u. left → 1, right → 3.",
                ),
                step(
                    ["u", "o", "i", "e", "a"],
                    2,
                    2,
                    &[1, 3],
                    "Swapped e and o. left meets right — the array is reversed.",
                ),
            ],
            ..VizGraph::default()
        }],
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// MARKS (decorative 48-viewBox strokes, oracle `Icons.tour*`)
// ─────────────────────────────────────────────────────────────────────────────

fn mark(body: AnyView) -> AnyView {
    view! {
        <svg class="tour-lib__mark" viewBox="0 0 48 48" fill="none" stroke="currentColor"
             stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            {body}
        </svg>
    }
    .into_any()
}

fn mark_graph() -> AnyView {
    mark(
        view! {
            <circle cx="24" cy="12" r="5"></circle>
            <circle cx="12" cy="34" r="5"></circle>
            <circle cx="36" cy="34" r="5"></circle>
            <path d="M21 16.5 14.5 29.5 M27 16.5 33.5 29.5 M17 34h14"></path>
        }
        .into_any(),
    )
}

fn mark_tree() -> AnyView {
    mark(
        view! {
            <circle cx="24" cy="10" r="4.5"></circle>
            <circle cx="13" cy="26" r="4.5"></circle>
            <circle cx="35" cy="26" r="4.5"></circle>
            <circle cx="8" cy="40" r="3.5"></circle>
            <circle cx="18" cy="40" r="3.5"></circle>
            <path d="M21 13.5 15.5 22.5 M27 13.5 32.5 22.5 M11.5 30 9 36.5 M15 30 17 36.5"></path>
        }
        .into_any(),
    )
}

fn mark_layers() -> AnyView {
    mark(
        view! {
            <path d="M24 8 42 17 24 26 6 17z"></path>
            <path d="M6 26 24 35 42 26 M6 34 24 43 42 34"></path>
        }
        .into_any(),
    )
}

fn mark_code() -> AnyView {
    mark(
        view! {
            <path d="M17 14 8 24 17 34 M31 14 40 24 31 34 M27 10 21 38"></path>
        }
        .into_any(),
    )
}

fn chevron() -> AnyView {
    view! {
        <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor"
             stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="m9 18 6-6-6-6"></path>
        </svg>
    }
    .into_any()
}
