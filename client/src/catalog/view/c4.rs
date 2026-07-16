//! The LikeC4 lesson embed chrome (oracle: `C4Blocks` + `DiagramZoom.openIframe`,
//! commit d8b969a): every authored `<iframe src="/c4/…">` is wrapped so an Enlarge button
//! (top-LEFT — LikeC4 owns top-right) floats over it; Enlarge opens the near-fullscreen
//! iframe zoom with parity chrome — − / + buttons driving SYNTHETIC ctrl+wheel pinches at
//! the viewer's `.react-flow__pane`, a live % read from the viewport transform, and the
//! gesture hint. While a `.likec4-overlay[open]` dialog is up inside the iframe, OUR chrome
//! steps aside (its ✕ · Share · Export render exactly where ours sits). Everything relies
//! on the `/c4` proxy keeping the iframe same-origin; every access is try/caught by shape
//! (`Option` all the way down) so a cross-origin iframe simply gets no chrome.

use std::any::Any;

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;

/// Hide LikeC4's merged-workspace nav panel (its view picker lists EVERY diagram across
/// all books — `/c4` is one merged build); UX scoping only.
const SCOPE_CSS: &str = r#"[class~="layerStyle_likec4.panel"] { display: none !important; }"#;

// ─────────────────────────────────────────────────────────────────────────────
// DISCOVERY
// ─────────────────────────────────────────────────────────────────────────────

pub fn hydrate_c4_embeds(
    root: &web_sys::HtmlElement,
    selected: RwSignal<Option<String>>,
) -> Vec<Box<dyn Any>> {
    let mut handles: Vec<Box<dyn Any>> = Vec::new();
    let Ok(nodes) = root.query_selector_all("iframe[src^='/c4/']") else {
        return handles;
    };
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return handles;
    };
    for index in 0..nodes.length() {
        let Some(node) = nodes.get(index) else { continue };
        let Ok(frame) = node.dyn_into::<web_sys::HtmlIFrameElement>() else {
            continue;
        };
        let Some(parent) = frame.parent_node() else {
            continue;
        };
        let Some(src) = frame.get_attribute("src") else {
            continue;
        };
        // Wrap: <div.c4-embed> around the iframe (re-parenting reloads it — accepted; the
        // load listener below re-fires all wiring), plus a host div for the button mount.
        let Ok(wrap) = document.create_element("div") else {
            continue;
        };
        wrap.set_class_name("c4-embed");
        let _ = parent.insert_before(&wrap, Some(&frame));
        let _ = wrap.append_child(&frame);
        let Ok(host) = document.create_element("div") else {
            continue;
        };
        let _ = wrap.append_child(&host);
        let Ok(wrap) = wrap.dyn_into::<web_sys::HtmlElement>() else {
            continue;
        };
        let Ok(host) = host.dyn_into::<web_sys::HtmlElement>() else {
            continue;
        };
        let handle = leptos::mount::mount_to(host, move || {
            view! { <C4Embed frame=frame wrap=wrap src=src selected=selected /> }
        });
        handles.push(Box::new(handle));
    }
    handles
}

// ─────────────────────────────────────────────────────────────────────────────
// THE INLINE EMBED: Enlarge + overlay guard + scope style
// ─────────────────────────────────────────────────────────────────────────────

#[allow(clippy::needless_pass_by_value)]
#[component]
fn C4Embed(
    frame: web_sys::HtmlIFrameElement,
    wrap: web_sys::HtmlElement,
    src: String,
    selected: RwSignal<Option<String>>,
) -> impl IntoView {
    let open = RwSignal::new(false);

    // The overlay guard: watch `.likec4-overlay[open]` inside the SAME-ORIGIN iframe with a
    // MutationObserver (childList+subtree catch the dialog's first insertion; the `open`
    // attribute filter catches show/close — the <dialog> lingers once used). Re-wired on
    // every iframe load; the observer lives on the iframe's document, so navigation GCs it.
    {
        let guard_frame = frame.clone();
        let guard_wrap = wrap.clone();
        let wire = move || {
            let Some(doc) = guard_frame.content_document() else {
                return;
            };
            let Some(root) = doc.document_element() else {
                return;
            };
            let sync_wrap = guard_wrap.clone();
            let sync_doc = doc.clone();
            let sync = Closure::<dyn FnMut(js_sys::Array, web_sys::MutationObserver)>::new(
                move |_records: js_sys::Array, _obs: web_sys::MutationObserver| {
                    let overlay = sync_doc
                        .query_selector(".likec4-overlay[open]")
                        .ok()
                        .flatten()
                        .is_some();
                    let _ = sync_wrap
                        .class_list()
                        .toggle_with_force("c4-embed--overlay", overlay);
                },
            );
            if let Ok(observer) = web_sys::MutationObserver::new(sync.as_ref().unchecked_ref()) {
                let init = web_sys::MutationObserverInit::new();
                init.set_child_list(true);
                init.set_subtree(true);
                init.set_attributes(true);
                let filter = js_sys::Array::new();
                filter.push(&"open".into());
                init.set_attribute_filter(&filter);
                let _ = observer.observe_with_options(&root, &init);
            }
            sync.forget(); // owned by the iframe document's lifetime
            inject_scope_style(&doc);
            attach_node_bridge(&doc, selected);
        };
        let onload = Closure::<dyn FnMut()>::new({
            let wire = wire.clone();
            move || wire()
        });
        frame.set_onload(Some(onload.as_ref().unchecked_ref()));
        onload.forget();
        wire();
    }

    let overlay_src = src;
    view! {
        <button
            class="c4-embed__zoom"
            aria-label="Enlarge diagram"
            on:click=move |_| open.set(true)
        >
            "⤢ Enlarge"
        </button>
        {move || open.get().then(|| view! { <C4Zoom src=overlay_src.clone() open=open selected=selected /> })}
    }
}

/// The click-to-docs bridge (oracle: `attachNodeBridge`): a CAPTURE-phase click listener on
/// the same-origin iframe document. The composed path (target-first) feeds the pure
/// `resolve_c4_node`; on a hit the click is swallowed and the docs panel opens. Elements in
/// the path are from the IFRAME's realm — only realm-safe method calls, no checked casts.
fn attach_node_bridge(doc: &web_sys::Document, selected: RwSignal<Option<String>>) {
    use wasm_bindgen::JsCast;
    // Cross-realm rule: nothing in the composed path passes a parent-realm instanceof, so
    // every read goes through Reflect (window/document hops have no tagName and drop out).
    fn attr(target: &wasm_bindgen::JsValue, name: &str) -> Option<String> {
        let get = js_sys::Reflect::get(target, &"getAttribute".into()).ok()?;
        let get: js_sys::Function = get.unchecked_into();
        get.call1(target, &name.into()).ok()?.as_string()
    }
    let handler = Closure::<dyn FnMut(web_sys::Event)>::new(move |event: web_sys::Event| {
        let hops: Vec<crate::catalog::logic::C4PathHop> = event
            .composed_path()
            .iter()
            .filter_map(|target| {
                let tag = js_sys::Reflect::get(&target, &"tagName".into())
                    .ok()
                    .and_then(|v| v.as_string())?;
                let classes = attr(&target, "class").unwrap_or_default();
                let data_id = attr(&target, "data-id");
                Some((tag, classes, data_id))
            })
            .collect();
        if let Some(id) = crate::catalog::logic::resolve_c4_node(&hops) {
            event.stop_propagation();
            event.prevent_default();
            selected.set(Some(id));
        }
    });
    let options = web_sys::AddEventListenerOptions::new();
    options.set_capture(true);
    let _ = doc.add_event_listener_with_callback_and_add_event_listener_options(
        "click",
        handler.as_ref().unchecked_ref(),
        &options,
    );
    handler.forget(); // owned by the iframe document's lifetime
}

fn inject_scope_style(doc: &web_sys::Document) {
    if doc.get_element_by_id("__syn-c4-inject").is_some() {
        return;
    }
    let Ok(style) = doc.create_element("style") else {
        return;
    };
    style.set_id("__syn-c4-inject");
    style.set_text_content(Some(SCOPE_CSS));
    let target = doc
        .head()
        .map(web_sys::Element::from)
        .or_else(|| doc.document_element());
    if let Some(target) = target {
        let _ = target.append_child(&style);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// THE FULLSCREEN IFRAME ZOOM
// A NEW iframe with the same src fills the modal (LikeC4 owns its own pan/zoom);
// one 300 ms poll reads the live scale % AND the overlay state.
// ─────────────────────────────────────────────────────────────────────────────

#[component]
fn C4Zoom(src: String, open: RwSignal<bool>, selected: RwSignal<Option<String>>) -> impl IntoView {
    let scale_pct: RwSignal<Option<i32>> = RwSignal::new(None);
    let overlay = RwSignal::new(false);
    let frame_ref: NodeRef<leptos::html::Iframe> = NodeRef::new();

    let esc = window_event_listener(leptos::ev::keydown, move |event| {
        if event.key() == "Escape" && open.get_untracked() {
            open.set(false);
        }
    });
    on_cleanup(move || esc.remove());

    // The scope style rides into the modal iframe too (idempotent; re-applied by the poll
    // until the document exists — the fresh iframe boots asynchronously).
    // The one poll: live scale % (the viewport's `scale(N)` transform) + overlay state.
    // A MutationObserver inside a live React canvas would fire every pan frame; one timer
    // parsing one transform is cheaper and dies with the modal.
    let poll: StoredValue<Option<gloo_timers::callback::Interval>, LocalStorage> =
        StoredValue::new_local(None);
    poll.set_value(Some(gloo_timers::callback::Interval::new(300, move || {
        let doc = frame_ref.get_untracked().and_then(|f| f.content_document());
        let Some(doc) = doc else { return };
        inject_scope_style(&doc);
        // Realm note: iframe elements fail parent-realm `instanceof` (dyn_into), so read
        // the inline style ATTRIBUTE instead of casting to HtmlElement.
        let pct = doc
            .query_selector(".react-flow__viewport")
            .ok()
            .flatten()
            .and_then(|vp| vp.get_attribute("style"))
            .and_then(|style| parse_scale(&style));
        scale_pct.set(pct);
        overlay.set(
            doc.query_selector(".likec4-overlay[open]")
                .ok()
                .flatten()
                .is_some(),
        );
    })));
    on_cleanup(move || poll.set_value(None));

    // ± steps ≈ ±25%: a synthetic ctrl+wheel pinch built in the IFRAME's own realm,
    // dispatched at the react-flow pane's centre (deltaY −16 in · +16 out).
    let zoom_step = move |zoom_in: bool| {
        let Some(frame) = frame_ref.get_untracked() else {
            return;
        };
        let Some(doc) = frame.content_document() else {
            return;
        };
        let Some(pane) = doc.query_selector(".react-flow__pane").ok().flatten() else {
            return;
        };
        let Some(win) = frame.content_window() else {
            return;
        };
        let rect = pane.get_bounding_client_rect();
        let Ok(ctor) = js_sys::Reflect::get(&win, &"WheelEvent".into()) else {
            return;
        };
        // The constructor comes from the IFRAME's realm — parent-realm instanceof (dyn_into)
        // is always false across realms, so the casts here are unchecked by design.
        let ctor: js_sys::Function = ctor.unchecked_into();
        let init = js_sys::Object::new();
        let delta: f64 = if zoom_in { -16.0 } else { 16.0 };
        let _ = js_sys::Reflect::set(&init, &"deltaY".into(), &delta.into());
        let _ = js_sys::Reflect::set(
            &init,
            &"clientX".into(),
            &(rect.left() + rect.width() / 2.0).into(),
        );
        let _ = js_sys::Reflect::set(
            &init,
            &"clientY".into(),
            &(rect.top() + rect.height() / 2.0).into(),
        );
        let _ = js_sys::Reflect::set(&init, &"bubbles".into(), &true.into());
        let _ = js_sys::Reflect::set(&init, &"cancelable".into(), &true.into());
        let _ = js_sys::Reflect::set(&init, &"ctrlKey".into(), &true.into());
        let args = js_sys::Array::of2(&"wheel".into(), &init);
        if let Ok(event) = js_sys::Reflect::construct(&ctor, &args) {
            let event: web_sys::Event = event.unchecked_into();
            let _ = pane.dispatch_event(&event);
        }
    };

    view! {
        <div class="diagram-zoom-scrim" on:click=move |_| open.set(false)>
            <div
                class="diagram-zoom diagram-zoom--fill"
                class:diagram-zoom--c4-overlay=move || overlay.get()
                on:click=|event| event.stop_propagation()
            >
                <button class="diagram-zoom__close" aria-label="Close" on:click=move |_| open.set(false)>
                    "✕ Close"
                </button>
                <div class="diagram-zoom__live">
                    <iframe
                        class="diagram-zoom__iframe"
                        src=src
                        title="LikeC4 diagram"
                        node_ref=frame_ref
                        on:load=move |_| {
                            if let Some(doc) = frame_ref.get_untracked().and_then(|f| f.content_document()) {
                                attach_node_bridge(&doc, selected);
                            }
                        }
                    ></iframe>
                    <div class="diagram-zoom__controls">
                        <button class="diagram-zoom__ctl" aria-label="Zoom out" on:click=move |_| zoom_step(false)>"−"</button>
                        <span class="diagram-zoom__level">
                            {move || scale_pct.get().map_or_else(|| "—".to_owned(), |p| format!("{p}%"))}
                        </span>
                        <button class="diagram-zoom__ctl" aria-label="Zoom in" on:click=move |_| zoom_step(true)>"+"</button>
                        <span class="diagram-zoom__hint">"or pinch / Ctrl+scroll to zoom · scroll or drag to pan"</span>
                    </div>
                </div>
            </div>
        </div>
    }
}

/// `"translate(12px, 3px) scale(1.25)"` → `Some(125)`.
fn parse_scale(transform: &str) -> Option<i32> {
    let start = transform.find("scale(")? + "scale(".len();
    let rest = &transform[start..];
    let end = rest.find(')')?;
    #[allow(clippy::cast_possible_truncation)]
    rest[..end]
        .parse::<f64>()
        .ok()
        .map(|s| (s * 100.0).round() as i32)
}
