//! The library landing (oracle: `LibraryPage.scala`, final post-33 design): the hero pill +
//! the guided-tour carousel + CTAs, then the book grid — category bands and cards linking
//! to each book's first lesson. The hero renders before the catalog loads; the grid waits.

use leptos::prelude::*;
use synapse_shared::catalog::{BookDto, CatalogEntryDto, SynapseIndexDto};

use crate::api::AsyncResult;
use crate::catalog::logic;
use crate::catalog::state;
use crate::catalog::view::tour::SynapseTour;

#[component]
pub fn LibraryPage() -> impl IntoView {
    let index = state::CatalogStore::from_context().index();
    let index_opt: Signal<Option<SynapseIndexDto>> = Signal::derive(move || match index.get() {
        AsyncResult::Loaded(idx) => Some(idx),
        _ => None,
    });
    view! {
        <div class="library">
            <section class="lib-hero">
                <div class="lib-hero__pill">
                    <span class="lib-hero__dot" aria-hidden="true"></span>
                    "A guided tour — everything Synapse can do"
                </div>
                <SynapseTour index=index_opt />
                <div class="lib-hero__ctas">
                    <button class="lib-hero__cta lib-hero__cta--primary" on:click=|_| scroll_to_grid()>
                        {book_icon()}
                        "Start reading"
                    </button>
                    <a class="lib-hero__cta lib-hero__cta--ghost" href="/blog">"Read the blog"</a>
                </div>
            </section>
            <ContinueCard index=index_opt />
            {move || match index.get() {
                AsyncResult::Loading => view! { <p class="muted">"Loading the library…"</p> }.into_any(),
                AsyncResult::Failed(message) => {
                    view! { <p class="error">"The library failed to load: " {message}</p> }.into_any()
                }
                AsyncResult::Loaded(idx) => book_grid(&idx).into_any(),
            }}
            <crate::shell::footer::SiteFooter />
        </div>
    }
}

/// Smooth-jump to the grid, offset for the sticky header.
fn scroll_to_grid() {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let Some(grid) = document.get_element_by_id("library-grid") else {
        return;
    };
    if let Some(window) = web_sys::window() {
        let top = grid.get_bounding_client_rect().top() + window.scroll_y().unwrap_or(0.0) - 80.0;
        window.scroll_to_with_x_and_y(0.0, top);
    }
}

fn book_grid(index: &SynapseIndexDto) -> impl IntoView + use<> {
    view! {
        <section id="library-grid" class="lib-section library__body">
            <h2 class="lib-section__title">"The books"</h2>
            <div class="lib-grid">{entries_view(&index.entries)}</div>
        </section>
    }
}

fn entries_view(entries: &[CatalogEntryDto]) -> Vec<AnyView> {
    entries
        .iter()
        .map(|entry| match entry {
            CatalogEntryDto::Category(category) => {
                let books: Vec<_> = category
                    .entries
                    .iter()
                    .filter_map(|e| match e {
                        CatalogEntryDto::Book(book) => Some(book_card(book)),
                        CatalogEntryDto::Category(_) => None,
                    })
                    .collect();
                view! {
                    <div class="lib-group">
                        <div class="lib-group__title">{category.title.clone()}</div>
                        <div class="lib-grid lib-grid--nested">{books}</div>
                    </div>
                }
                .into_any()
            }
            CatalogEntryDto::Book(book) => book_card(book),
        })
        .collect()
}

/// "N/M read" — shown only once there is something to report, so an untouched library stays
/// uncluttered and the chip always means "you have started this".
fn progress_chip(book: &BookDto) -> AnyView {
    let progress = crate::catalog::state::ProgressStore::from_context();
    let book = book.clone();
    view! {
        {move || {
            let done = progress.done().read();
            let count = logic::progress::completed_count(&book, &done);
            let total = logic::lesson_count(&book);
            (count > 0 && total > 0)
                .then(|| {
                    view! {
                        <span
                            class="lib-card__progress"
                            class:lib-card__progress--all=count == total
                        >
                            {format!("{count}/{total} read")}
                        </span>
                    }
                })
        }}
    }
    .into_any()
}

/// "Pick up where you left off" — the one thing that makes a second visit different from a
/// first. Renders nothing until there IS a last lesson, so a new reader never sees an empty
/// shelf, and nothing when the index has not loaded, because the title would be a raw slug.
#[component]
fn ContinueCard(index: Signal<Option<SynapseIndexDto>>) -> impl IntoView {
    let progress = state::ProgressStore::from_context();
    move || {
        let path = progress.last().get()?;
        let index = index.get()?;
        let segments: Vec<String> = path.split('/').map(str::to_owned).collect();
        // The title comes from the index rather than being stored alongside the path: a stored
        // title would go stale the moment a lesson is renamed, and the path is the only thing
        // that has to stay true.
        let book = logic::book_of(&index, &segments)?;
        let title = logic::reading_order(book)
            .into_iter()
            .find(|(p, _)| *p == path)
            .map(|(_, lesson)| lesson.title)?;
        Some(view! {
            <a class="lib-continue" href=format!("/synapse/{path}")>
                <span class="lib-continue__label">"Pick up where you left off"</span>
                <span class="lib-continue__title">{title}</span>
                <span class="lib-continue__book">{book.title.clone()}</span>
            </a>
        })
    }
}

fn book_card(book: &BookDto) -> AnyView {
    let chapters = logic::chapter_count(book);
    let lessons = logic::lesson_count(book);
    let mut meta: Vec<String> = Vec::new();
    if chapters > 0 {
        meta.push(format!("{chapters} {}", plural(chapters, "chapter")));
    }
    meta.push(format!("{lessons} {}", plural(lessons, "lesson")));
    if let Some(minutes) = book.estimated_reading_minutes {
        meta.push(format!("~{minutes} min"));
    }
    let meta = meta.join(" · ");
    let tags: Vec<_> = book
        .tags
        .iter()
        .take(3)
        .map(|t| view! { <span class="lib-card__tag">{t.clone()}</span> })
        .collect();
    let body = view! {
        <div class="lib-card__meta">{book_icon()}<span>{meta}</span></div>
        <div class="lib-card__title">{book.title.clone()}</div>
        {(!book.description.is_empty())
            .then(|| view! { <p class="lib-card__desc">{book.description.clone()}</p> })}
        <div class="lib-card__footer">
            {tags}
            {progress_chip(book)}
            <span class="lib-card__cta">"Read" {arrow_icon()}</span>
        </div>
    };
    match logic::first_lesson_path(book) {
        Some(path) => view! {
            <a class="lib-card" href=format!("/synapse/{path}")>{body}</a>
        }
        .into_any(),
        None => view! { <div class="lib-card lib-card--empty">{body}</div> }.into_any(),
    }
}

fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        word.to_owned()
    } else {
        format!("{word}s")
    }
}

fn book_icon() -> impl IntoView {
    view! {
        <svg class="lib-card__meta-ic" viewBox="0 0 24 24" width="14" height="14" fill="none"
             stroke="currentColor" stroke-width="2" stroke-linecap="round"
             stroke-linejoin="round" aria-hidden="true">
            <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"></path>
            <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"></path>
        </svg>
    }
}

fn arrow_icon() -> impl IntoView {
    view! {
        <svg class="lib-card__cta-ic" viewBox="0 0 24 24" width="14" height="14" fill="none"
             stroke="currentColor" stroke-width="2" stroke-linecap="round"
             stroke-linejoin="round" aria-hidden="true">
            <path d="M5 12h14 M13 6l6 6-6 6"></path>
        </svg>
    }
}
