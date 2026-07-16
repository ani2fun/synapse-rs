//! The reader (oracle: the lesson page of steps 07–08/12): sidebar (the owning book's reading
//! order from the SHARED cached index) + the lesson body across the markdown island + prev/next.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use synapse_shared::catalog::LessonPayloadDto;

use crate::api::AsyncResult;
use crate::catalog::{logic, state};
use crate::islands;
use crate::router::page::Page;

#[component]
pub fn LessonPage() -> impl IntoView {
    let params = use_params_map();
    // The catch-all param is reactive: navigating lesson → lesson re-renders this memo, and
    // everything below keys off it (fetch-per-navigation, oracle semantics).
    let path = Memo::new(move |_| Page::segments_of(&params.read().get("path").unwrap_or_default()));

    view! {
        <div class="reader">
            <aside class="reader-sidebar">
                <Sidebar path=path />
            </aside>
            <article class="reader-main">
                {move || {
                    let segments = path.get();
                    let lesson = state::load_lesson(segments.clone());
                    view! { <LessonBody lesson=lesson segments=segments /> }
                }}
            </article>
        </div>
        // OUTSIDE the grid on purpose (oracle step-38 prod bug): the drawer's in-flow
        // wrapper would otherwise become a phantom third grid item at desktop width.
        <ReaderNavDrawer path=path />
    }
}

/// The mobile navigation drawer (oracle: `ReaderNavDrawer`, step 38): a bottom-LEFT FAB
/// (<1024px only — the desktop sidebar hides there) opens an off-canvas drawer reusing the
/// SAME `Sidebar`, always expanded. Three closes: scrim, Escape, any nav-link tap.
#[component]
fn ReaderNavDrawer(path: Memo<Vec<String>>) -> impl IntoView {
    let open = RwSignal::new(false);
    let esc = window_event_listener(leptos::ev::keydown, move |event| {
        if event.key() == "Escape" && open.get_untracked() {
            open.set(false);
        }
    });
    on_cleanup(move || esc.remove());
    view! {
        <div>
            <button
                class="reader-nav-fab"
                aria-label="Contents"
                aria-expanded=move || open.get().to_string()
                on:click=move |_| open.set(true)
            >
                <svg class="reader-nav-fab__ic" viewBox="0 0 24 24" fill="none" stroke="currentColor"
                     stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                    <rect width="18" height="18" x="3" y="3" rx="2"></rect>
                    <path d="M9 3v18 M14 9l3 3-3 3"></path>
                </svg>
            </button>
            {move || open.get().then(|| view! {
                <div>
                    <div class="reader-nav-scrim" on:click=move |_| open.set(false)></div>
                    <aside
                        class="reader-nav-drawer"
                        // Close-on-navigate: any click landing on (or inside) an <a> closes;
                        // the link itself still routes.
                        on:click=move |event| {
                            use wasm_bindgen::JsCast;
                            let closes = event
                                .target()
                                .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                                .and_then(|el| el.closest("a").ok().flatten())
                                .is_some();
                            if closes {
                                open.set(false);
                            }
                        }
                    >
                        <div class="reader-nav-drawer__head">
                            <span class="reader-nav-drawer__title">"Contents"</span>
                            <button
                                class="reader-nav-drawer__close"
                                aria-label="Close"
                                on:click=move |_| open.set(false)
                            >
                                "✕"
                            </button>
                        </div>
                        <Sidebar path=path />
                    </aside>
                </div>
            })}
        </div>
    }
}

#[component]
fn Sidebar(path: Memo<Vec<String>>) -> impl IntoView {
    let index = state::CatalogStore::from_context().index();
    // The owning book changes only across books — memoized so lesson→lesson navigation inside
    // one book re-renders nothing but the per-item `current` classes below.
    let book = Memo::new(move |_| match index.get() {
        AsyncResult::Loaded(idx) => logic::book_of(&idx, &path.get()).cloned(),
        AsyncResult::Loading | AsyncResult::Failed(_) => None,
    });
    view! {
        {move || {
            book.get()
                .map(|book| {
                    let prefix = logic::book_prefix(&book);
                    let items = sidebar_entries(&book.entries, &prefix, path);
                    view! {
                        <nav>
                            <h2 class="sidebar-book">{book.title.clone()}</h2>
                            <ul class="sidebar-lessons">{items}</ul>
                        </nav>
                    }
                })
        }}
    }
}

/// One book-interior level: lessons link; chapters are COLLAPSIBLE groups (post-33 `a95e3fb`),
/// open when they contain the current lesson so navigation always lands unfolded.
fn sidebar_entries(
    entries: &[synapse_shared::catalog::BookEntryDto],
    prefix: &[String],
    path: Memo<Vec<String>>,
) -> Vec<AnyView> {
    use synapse_shared::catalog::BookEntryDto;
    entries
        .iter()
        .map(|entry| match entry {
            BookEntryDto::Lesson(lesson) => {
                let mut segments = prefix.to_vec();
                segments.push(lesson.slug.clone());
                let full = segments.join("/");
                let href = format!("/synapse/{full}");
                // Fine-grained: each item tracks the path itself.
                let is_current = Memo::new(move |_| path.get().join("/") == full);
                view! {
                    <li class:current=move || is_current.get()>
                        <a href=href>{lesson.title.clone()}</a>
                    </li>
                }
                .into_any()
            }
            BookEntryDto::Chapter(chapter) => {
                let mut segments = prefix.to_vec();
                segments.push(chapter.slug.clone());
                let contains_current = path.get_untracked().join("/").starts_with(&segments.join("/"));
                let children = sidebar_entries(&chapter.entries, &segments, path);
                view! {
                    <li class="sidebar-chapter">
                        <details open=contains_current>
                            <summary class="sidebar-chapter__title">{chapter.title.clone()}</summary>
                            <ul class="sidebar-lessons">{children}</ul>
                        </details>
                    </li>
                }
                .into_any()
            }
        })
        .collect()
}

#[component]
fn LessonBody(lesson: RwSignal<AsyncResult<LessonPayloadDto>>, segments: Vec<String>) -> impl IntoView {
    let segments = StoredValue::new(segments);
    view! {
        {move || match lesson.get() {
            AsyncResult::Loading => view! { <p class="muted">"Loading…"</p> }.into_any(),
            AsyncResult::Failed(message) => {
                view! { <p class="error">"Lesson failed to load: " {message}</p> }.into_any()
            }
            AsyncResult::Loaded(payload) => loaded_lesson(&payload, &segments.read_value()).into_any(),
        }}
    }
}

fn loaded_lesson(payload: &LessonPayloadDto, segments: &[String]) -> impl IntoView + use<> {
    // Captured IN-TREE: hydrated blocks mount out-of-tree and cannot reach App's context.
    let auth = crate::identity::state::AuthStore::from_context();
    let theme = crate::shell::theme::ThemeStore::from_context();
    let viz_modal = crate::viz::modal::VizModalStore::from_context();
    // The Coach's editor snapshot — filled by the hydrated workbench on mount + every edit.
    let code_ctx = RwSignal::new((String::new(), String::new()));
    // The body crosses the island bridge asynchronously; once the HTML lands, the interactive
    // placeholders hydrate (runnable blocks today; solutions/quizzes/diagrams with their
    // steps). The boxed unmount handles keep the mounts alive; clearing them (navigation /
    // unmount) tears the blocks down.
    let html = RwSignal::new(String::from("<p>rendering…</p>"));
    let body_ref: NodeRef<leptos::html::Div> = NodeRef::new();
    let mounts: StoredValue<Vec<Box<dyn std::any::Any>>, LocalStorage> = StoredValue::new_local(Vec::new());
    let raw = payload.raw.clone();
    let owned_segments = segments.to_vec();
    let problem_path_source = segments.join("/");
    spawn_local(async move {
        match islands::markdown::render(&raw).await {
            Ok(rendered) => {
                // The oracle's pattern (`el.innerHTML = html` then mount blocks): write the DOM
                // directly and hydrate in the same breath — no render-effect race. The signal
                // stays for the placeholder/error states only.
                let Some(body) = body_ref.get_untracked() else {
                    return;
                };
                body.set_inner_html(&rendered);
                let mut handles = crate::execution::view::hydrate_workbenches(
                    &body,
                    &owned_segments,
                    auth,
                    code_ctx,
                    theme,
                    viz_modal,
                );
                handles.extend(crate::catalog::view::diagrams::hydrate_diagrams(&body));
                handles.extend(crate::catalog::view::c4::hydrate_c4_embeds(&body));
                handles.extend(crate::execution::view::hydrate_practices(
                    &body,
                    &owned_segments,
                    auth,
                    code_ctx,
                    theme,
                    viz_modal,
                ));
                // The viz widgets (step 27): every planted `div.viz-widget` mounts a host.
                for (element, spec) in crate::viz::blocks::discover(&body) {
                    let handle = leptos::mount::mount_to(element, move || {
                        view! {
                            <crate::viz::host::WidgetHost
                                name=spec.name
                                structure=spec.structure
                                cases=spec.cases
                            />
                        }
                    });
                    handles.push(Box::new(handle));
                }
                mounts.set_value(handles);
            }
            Err(error) => html.set(format!("<p>markdown island failed: {error:?}</p>")),
        }
    });
    on_cleanup(move || mounts.set_value(Vec::new()));

    let nav_link = |target: &Option<String>, label: &'static str, class: &'static str| {
        target.clone().map(|path| {
            view! { <a class=class href=format!("/synapse/{path}")>{label}</a> }
        })
    };
    // Problem pages go full width (post-33 `a95e3fb`) — the workbench needs the column.
    let is_problem = payload.frontmatter.kind.as_deref() == Some("problem");
    let problem_path = is_problem.then_some(problem_path_source);
    view! {
        <div class="lesson" class:lesson--problem=is_problem>
            <header class="lesson-header">
                <p class="lesson-book muted">{payload.book.title.clone()}</p>
                <h1>{payload.frontmatter.title.clone()}</h1>
                {payload.frontmatter.summary.clone().map(|s| view! { <p class="lesson-summary">{s}</p> })}
            </header>
            <div class="lesson-body synapse-prose" node_ref=body_ref inner_html=move || html.get()></div>
            {is_problem.then(|| view! {
                <crate::tutoring::CoachPane problem=problem_path.clone() code_ctx=code_ctx />
            })}
            <nav class="lesson-nav">
                {nav_link(&payload.prev, "← Previous", "nav-prev")}
                {nav_link(&payload.next, "Next →", "nav-next")}
            </nav>
            <super::ReaderPrefsFab />
        </div>
    }
}
