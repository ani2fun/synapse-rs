//! The reader sidebar (oracle: `ReaderSidebar` + `SidebarMode`/`SidebarState`): the book's
//! reading order with collapsible chapters, the title/description head, the filter box, the
//! Learn-browse toggle, and the three modes — Expanded · Compact (numbered rail with the
//! progress ring) · Hidden (kept in DOM, CSS-faded). Mode persists in localStorage.

use leptos::prelude::*;
use synapse_shared::catalog::{BookDto, BookEntryDto, CatalogEntryDto, SynapseIndexDto};

use crate::api::AsyncResult;
use crate::catalog::logic;
use crate::catalog::state;

const MODE_KEY: &str = "reader-sidebar";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SidebarMode {
    Expanded,
    Compact,
    Hidden,
}

impl SidebarMode {
    pub fn token(self) -> &'static str {
        match self {
            Self::Expanded => "expanded",
            Self::Compact => "compact",
            Self::Hidden => "hidden",
        }
    }

    fn parse(token: &str) -> Self {
        match token {
            "compact" => Self::Compact,
            "hidden" | "collapsed" => Self::Hidden,
            _ => Self::Expanded,
        }
    }

    pub fn load() -> Self {
        web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|s| s.get_item(MODE_KEY).ok().flatten())
            .map_or(Self::Expanded, |t| Self::parse(&t))
    }

    pub fn persist(self) {
        if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            let _ = storage.set_item(MODE_KEY, self.token());
        }
    }
}

#[allow(clippy::too_many_lines)] // the sidebar's three faces are one cohesive component
#[component]
pub fn ReaderSidebar(
    path: Memo<Vec<String>>,
    mode: RwSignal<SidebarMode>,
    progress: RwSignal<f64>,
    /// The drawer reuses the sidebar always-expanded and without the collapse controls.
    #[prop(optional)]
    in_drawer: bool,
) -> impl IntoView {
    let index = state::CatalogStore::from_context().index();
    let book = Memo::new(move |_| match index.get() {
        AsyncResult::Loaded(idx) => logic::book_of(&idx, &path.get()).cloned(),
        AsyncResult::Loading | AsyncResult::Failed(_) => None,
    });
    let browsing = RwSignal::new(false);
    let query = RwSignal::new(String::new());

    view! {
        {move || {
            let current_mode = if in_drawer { SidebarMode::Expanded } else { mode.get() };
            match current_mode {
                SidebarMode::Compact => book.get().map(|b| compact_rail(&b, path, mode, progress).into_any()),
                _ => book.get().map(|b| {
                    let idx = match index.get() {
                        AsyncResult::Loaded(i) => Some(i),
                        _ => None,
                    };
                    expanded(&b, idx.as_ref(), path, mode, browsing, query, in_drawer).into_any()
                }),
            }
        }}
    }
}

fn top_row(
    back_label: &'static str,
    on_back: impl Fn() + 'static,
    mode: RwSignal<SidebarMode>,
    in_drawer: bool,
) -> impl IntoView {
    view! {
        <div class="reader-sidebar__toprow">
            <button class="reader-sidebar__back" on:click=move |_| on_back()>{back_label}</button>
            {(!in_drawer).then(|| view! {
                <div class="reader-sidebar__controls">
                    <button
                        class="reader-sidebar__hide"
                        title="Collapse to a rail"
                        on:click=move |_| {
                            mode.set(SidebarMode::Compact);
                            SidebarMode::Compact.persist();
                        }
                    >
                        <svg class="reader-sidebar__hide-ic" viewBox="0 0 24 24" fill="none" stroke="currentColor"
                             stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                            <path d="m11 17-5-5 5-5 M18 17l-5-5 5-5"></path>
                        </svg>
                    </button>
                    <button
                        class="reader-sidebar__hide"
                        title="Hide the sidebar"
                        on:click=move |_| {
                            mode.set(SidebarMode::Hidden);
                            SidebarMode::Hidden.persist();
                        }
                    >
                        <svg class="reader-sidebar__hide-ic" viewBox="0 0 24 24" fill="none" stroke="currentColor"
                             stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                            <rect width="18" height="18" x="3" y="3" rx="2"></rect>
                            <path d="M9 3v18 M16 15l-3-3 3-3"></path>
                        </svg>
                    </button>
                </div>
            })}
        </div>
    }
}

#[allow(clippy::too_many_lines)]
fn expanded(
    book: &BookDto,
    index: Option<&SynapseIndexDto>,
    path: Memo<Vec<String>>,
    mode: RwSignal<SidebarMode>,
    browsing: RwSignal<bool>,
    query: RwSignal<String>,
    in_drawer: bool,
) -> impl IntoView + use<> {
    let book = book.clone();
    let index = index.cloned();
    view! {
        <div class="reader-sidebar__inner">
            {move || {
                if browsing.get() {
                    let entries = index.as_ref().map(|i| i.entries.clone()).unwrap_or_default();
                    browse_children(&entries, &book, browsing, mode, in_drawer).into_any()
                } else {
                    book_children(&book, path, mode, browsing, query, in_drawer).into_any()
                }
            }}
        </div>
    }
}

fn book_children(
    book: &BookDto,
    path: Memo<Vec<String>>,
    mode: RwSignal<SidebarMode>,
    browsing: RwSignal<bool>,
    query: RwSignal<String>,
    in_drawer: bool,
) -> impl IntoView + use<> {
    let prefix = logic::book_prefix(book);
    let entries = book.entries.clone();
    let title = book.title.clone();
    let description = book.description.clone();
    view! {
        {top_row("← Learn", move || browsing.set(true), mode, in_drawer)}
        <div class="reader-sidebar__title">{title}</div>
        {(!description.is_empty()).then(|| view! { <div class="reader-sidebar__desc">{description}</div> })}
        <div class="reader-sidebar__search">
            <svg class="reader-sidebar__search-ic" viewBox="0 0 24 24" fill="none" stroke="currentColor"
                 stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <circle cx="11" cy="11" r="8"></circle>
                <path d="m21 21-4.3-4.3"></path>
            </svg>
            <input
                class="reader-sidebar__search-input"
                placeholder="Filter this book…"
                aria-label="Filter this book's lessons"
                prop:value=move || query.get()
                on:input=move |event| query.set(event_target_value(&event))
            />
            {move || (!query.get().is_empty()).then(|| view! {
                <button
                    class="reader-sidebar__search-clear"
                    aria-label="Clear filter"
                    on:click=move |_| query.set(String::new())
                >
                    "×"
                </button>
            })}
        </div>
        <ul class="reader-sidebar__tree">
            {move || {
                let q = query.get();
                let pruned = logic::prune_entries(&entries, &q);
                if pruned.is_empty() {
                    vec![view! { <li class="reader-sidebar__empty">"No lessons match."</li> }.into_any()]
                } else {
                    tree_nodes(&pruned, &prefix, path, !q.trim().is_empty())
                }
            }}
        </ul>
    }
}

fn tree_nodes(
    entries: &[BookEntryDto],
    prefix: &[String],
    path: Memo<Vec<String>>,
    searching: bool,
) -> Vec<AnyView> {
    entries
        .iter()
        .map(|entry| match entry {
            BookEntryDto::Lesson(lesson) => {
                let mut segments = prefix.to_vec();
                segments.push(lesson.slug.clone());
                let full = segments.join("/");
                let href = format!("/synapse/{full}");
                let is_current = Memo::new(move |_| path.get().join("/") == full);
                view! {
                    <li>
                        <a
                            class="reader-sidebar__link"
                            class:reader-sidebar__link--active=move || is_current.get()
                            href=href
                        >
                            {lesson.title.clone()}
                        </a>
                    </li>
                }
                .into_any()
            }
            BookEntryDto::Chapter(chapter) => {
                let mut segments = prefix.to_vec();
                segments.push(chapter.slug.clone());
                let contains = path.get_untracked().join("/").starts_with(&segments.join("/"));
                let children = tree_nodes(&chapter.entries, &segments, path, searching);
                view! {
                    <li>
                        <details class="reader-sidebar__section" open=searching || contains>
                            <summary class="reader-sidebar__summary">
                                <svg class="reader-sidebar__chevron" viewBox="0 0 24 24" fill="none"
                                     stroke="currentColor" stroke-width="2" stroke-linecap="round"
                                     stroke-linejoin="round" aria-hidden="true">
                                    <path d="m9 18 6-6-6-6"></path>
                                </svg>
                                <span class="reader-sidebar__name">{chapter.title.clone()}</span>
                            </summary>
                            <ul class="reader-sidebar__children">{children}</ul>
                        </details>
                    </li>
                }
                .into_any()
            }
        })
        .collect()
}

/// The Compact rail: one numbered tile per top-level chapter; the ACTIVE tile carries the
/// page-progress conic ring via `--progress` (0–100).
fn compact_rail(
    book: &BookDto,
    path: Memo<Vec<String>>,
    mode: RwSignal<SidebarMode>,
    progress: RwSignal<f64>,
) -> impl IntoView + use<> {
    let prefix = logic::book_prefix(book);
    let tiles: Vec<_> = book
        .entries
        .iter()
        .filter_map(|entry| match entry {
            BookEntryDto::Chapter(chapter) => Some(chapter.clone()),
            BookEntryDto::Lesson(_) => None,
        })
        .enumerate()
        .map(|(i, chapter)| {
            let mut segments = prefix.clone();
            segments.push(chapter.slug.clone());
            let chapter_prefix = segments.join("/");
            let label = format!("{:02}", i + 1);
            let tooltip = format!("{label} · {}", chapter.title);
            let first = chapter.entries.iter().find_map(|e| match e {
                BookEntryDto::Lesson(l) => Some(format!("{chapter_prefix}/{}", l.slug)),
                BookEntryDto::Chapter(c) => c.entries.iter().find_map(|e2| match e2 {
                    BookEntryDto::Lesson(l2) => Some(format!("{chapter_prefix}/{}/{}", c.slug, l2.slug)),
                    BookEntryDto::Chapter(_) => None,
                }),
            });
            let active = Memo::new(move |_| path.get().join("/").starts_with(&chapter_prefix));
            let style = move || {
                if active.get() {
                    format!("--progress: {:.1}", progress.get() * 100.0)
                } else {
                    String::new()
                }
            };
            match first {
                Some(target) => view! {
                    <a
                        class="reader-rail__tile"
                        class:reader-rail__tile--active=move || active.get()
                        style=style
                        href=format!("/synapse/{target}")
                        title=tooltip
                    >
                        <span class="reader-rail__num">{label}</span>
                    </a>
                }
                .into_any(),
                None => view! {
                    <span class="reader-rail__tile" title=tooltip>
                        <span class="reader-rail__num">{label}</span>
                    </span>
                }
                .into_any(),
            }
        })
        .collect();
    view! {
        <div class="reader-rail">
            <button
                class="reader-rail__expand"
                aria-label="Expand the sidebar"
                on:click=move |_| {
                    mode.set(SidebarMode::Expanded);
                    SidebarMode::Expanded.persist();
                }
            >
                <svg class="reader-rail__expand-ic" viewBox="0 0 24 24" fill="none" stroke="currentColor"
                     stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                    <rect width="18" height="18" x="3" y="3" rx="2"></rect>
                    <path d="M9 3v18 M14 9l3 3-3 3"></path>
                </svg>
            </button>
            <div class="reader-rail__tiles">{tiles}</div>
        </div>
    }
}

/// The Learn browse: categories (only the current book's category starts open) + book links.
fn browse_children(
    entries: &[CatalogEntryDto],
    current_book: &BookDto,
    browsing: RwSignal<bool>,
    mode: RwSignal<SidebarMode>,
    in_drawer: bool,
) -> impl IntoView + use<> {
    let current_slug = current_book.slug.clone();
    let nodes = browse_nodes(entries, &current_slug, browsing);
    view! {
        {top_row("← Back to lessons", move || browsing.set(false), mode, in_drawer)}
        <div class="reader-sidebar__learn-head">
            <a class="reader-sidebar__dash" href="/">
                <svg class="reader-sidebar__dash-ic" viewBox="0 0 24 24" fill="none" stroke="currentColor"
                     stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                    <path d="m3 9 9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path>
                    <path d="M9 22V12h6v10"></path>
                </svg>
                <span>"Your Dashboard"</span>
            </a>
            <div class="reader-sidebar__divider"></div>
            <div class="reader-sidebar__eyebrow">"Learn"</div>
        </div>
        <ul class="reader-sidebar__browse">{nodes}</ul>
    }
}

fn browse_nodes(entries: &[CatalogEntryDto], current_slug: &str, browsing: RwSignal<bool>) -> Vec<AnyView> {
    entries
        .iter()
        .map(|entry| match entry {
            CatalogEntryDto::Book(book) => {
                let active = book.slug == current_slug;
                match logic::first_lesson_path(book) {
                    Some(target) => {
                        let class = if active {
                            "reader-sidebar__book reader-sidebar__book--active"
                        } else {
                            "reader-sidebar__book"
                        };
                        view! {
                            <li>
                                <a
                                    class=class
                                    href=format!("/synapse/{target}")
                                    on:click=move |_| browsing.set(false)
                                >
                                    {book.title.clone()}
                                </a>
                            </li>
                        }
                        .into_any()
                    }
                    None => view! {
                        <li><span class="reader-sidebar__book reader-sidebar__book--empty">{book.title.clone()}</span></li>
                    }
                    .into_any(),
                }
            }
            CatalogEntryDto::Category(category) => {
                let contains = category_has_book(category, current_slug);
                let children = browse_nodes(&category.entries, current_slug, browsing);
                view! {
                    <li>
                        <details class="reader-sidebar__section" open=contains>
                            <summary class="reader-sidebar__summary">
                                <svg class="reader-sidebar__chevron" viewBox="0 0 24 24" fill="none"
                                     stroke="currentColor" stroke-width="2" stroke-linecap="round"
                                     stroke-linejoin="round" aria-hidden="true">
                                    <path d="m9 18 6-6-6-6"></path>
                                </svg>
                                <span class="reader-sidebar__name">{category.title.clone()}</span>
                            </summary>
                            <ul class="reader-sidebar__children">{children}</ul>
                        </details>
                    </li>
                }
                .into_any()
            }
        })
        .collect()
}

fn category_has_book(category: &synapse_shared::catalog::CategoryDto, slug: &str) -> bool {
    category.entries.iter().any(|entry| match entry {
        CatalogEntryDto::Book(book) => book.slug == slug,
        CatalogEntryDto::Category(inner) => category_has_book(inner, slug),
    })
}
