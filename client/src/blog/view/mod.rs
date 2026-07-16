//! The blog views (oracle: `BlogListPage` + `BlogPostPage`): the card list and the single
//! column post with its Older/Newer pager. The post body crosses the markdown island exactly
//! like a lesson, minus workbench hydration; the rendered leading `<h1>` is stripped (the
//! header already shows the title).

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use synapse_shared::blog::{BlogPostDto, BlogSummaryDto};

use crate::api::AsyncResult;
use crate::blog::state;
use crate::islands;

#[component]
pub fn BlogListPage() -> impl IntoView {
    let list = state::BlogStore::from_context().list();
    view! {
        <div class="blog">
            <header class="blog-hero">
                <p class="blog-hero__eyebrow">"Writing"</p>
                <h1>"The blog"</h1>
            </header>
            {move || match list.get() {
                AsyncResult::Loading => view! { <p class="muted">"Loading…"</p> }.into_any(),
                AsyncResult::Failed(message) => {
                    view! { <p class="error">"The blog failed to load: " {message}</p> }.into_any()
                }
                AsyncResult::Loaded(posts) if posts.is_empty() => {
                    view! { <p class="muted">"No posts yet — check back soon."</p> }.into_any()
                }
                AsyncResult::Loaded(posts) => view! {
                    <div class="blog-list">
                        {posts.iter().map(card).collect::<Vec<_>>()}
                    </div>
                }
                .into_any(),
            }}
        </div>
    }
}

fn card(post: &BlogSummaryDto) -> impl IntoView + use<> {
    let href = format!("/blog/{}", post.slug);
    let eyebrow = post.eyebrow.clone();
    let summary = post.summary.clone();
    let date = post.published_at.clone();
    let read = post.read_minutes.map(|m| format!("{m} min read"));
    let tags = post.tags.clone();
    view! {
        <a class="blog-card" href=href>
            {eyebrow.map(|e| view! { <p class="blog-card__eyebrow">{e}</p> })}
            <h2 class="blog-card__title">{post.title.clone()}</h2>
            {summary.map(|s| view! { <p class="blog-card__summary">{s}</p> })}
            <div class="blog-card__meta">
                {(!date.is_empty()).then(|| view! { <span class="blog-card__date">{date.clone()}</span> })}
                {read.map(|r| view! { <span class="blog-card__read">{r}</span> })}
                {(!tags.is_empty()).then(|| view! {
                    <span class="blog-card__tags">
                        {tags.iter().map(|t| view! { <span class="blog-card__tag">{t.clone()}</span> }).collect::<Vec<_>>()}
                    </span>
                })}
            </div>
        </a>
    }
}

#[component]
pub fn BlogPostPage() -> impl IntoView {
    let params = use_params_map();
    let slug = Memo::new(move |_| params.read().get("slug").unwrap_or_default());
    view! {
        <div class="blog blog--post">
            {move || {
                let post = state::load_post(slug.get());
                view! { <PostBody post=post /> }
            }}
        </div>
    }
}

#[component]
fn PostBody(post: RwSignal<AsyncResult<BlogPostDto>>) -> impl IntoView {
    view! {
        {move || match post.get() {
            AsyncResult::Loading => view! { <p class="muted">"Loading…"</p> }.into_any(),
            AsyncResult::Failed(message) => {
                view! { <p class="error">"The post failed to load: " {message}</p> }.into_any()
            }
            AsyncResult::Loaded(post) => loaded_post(&post).into_any(),
        }}
    }
}

fn loaded_post(post: &BlogPostDto) -> impl IntoView + use<> {
    let body_ref: NodeRef<leptos::html::Div> = NodeRef::new();
    let raw = post.body.clone();
    spawn_local(async move {
        let Ok(rendered) = islands::markdown::render(&raw).await else {
            return;
        };
        let Some(body) = body_ref.get_untracked() else {
            return;
        };
        body.set_inner_html(&rendered);
        // The header already carries the title — drop the duplicated leading heading.
        if let Ok(Some(h1)) = body.query_selector("h1")
            && h1.previous_element_sibling().is_none()
        {
            h1.remove();
        }
    });

    let date = post.published_at.clone();
    let read = post.read_minutes.map(|m| format!("{m} min read"));
    view! {
        <article class="blog-post">
            <header class="blog-post__head">
                {post.eyebrow.clone().map(|e| view! { <p class="blog-post__eyebrow">{e}</p> })}
                <h1 class="blog-post__title">{post.title.clone()}</h1>
                <p class="blog-post__meta">
                    {(!date.is_empty()).then(|| view! { <span>{date.clone()}</span> })}
                    {read.map(|r| view! { <span>{r}</span> })}
                </p>
            </header>
            <div class="blog-post__body synapse-prose" node_ref=body_ref>
                <p>"rendering…"</p>
            </div>
            <nav class="blog-pager">
                {pager_card(post.prev.as_ref(), "← Older")}
                {pager_card(post.next.as_ref(), "Newer →")}
            </nav>
        </article>
    }
}

fn pager_card(slug: Option<&String>, label: &'static str) -> impl IntoView + use<> {
    slug.map(|slug| {
        let href = format!("/blog/{slug}");
        let title = humanize(slug);
        view! {
            <a class="blog-pager__card" href=href>
                <span class="blog-pager__label">{label}</span>
                <span class="blog-pager__title">{title}</span>
            </a>
        }
    })
}

/// `rebuilding-cortex` → `Rebuilding cortex` — good enough for a pager card.
fn humanize(slug: &str) -> String {
    let words = slug.replace(['-', '_'], " ");
    let mut chars = words.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default()
}
