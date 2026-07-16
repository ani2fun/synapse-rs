//! One runnable code block (oracle: `Workbench`): toolbar (eyebrow · language tabs · Edit ·
//! Run · Submit), the Monaco island, the tests panel when the block carries an authored
//! suite, the output panel (judged against the active case), and the verdict panel.
//! Multi-variant (step 30): adjacent run fences are LANGUAGE TABS over ONE Monaco —
//! each variant keeps its own buffer/run state in its own `BlockStore`; switching swaps the
//! editor's value + tokenizer in place. Practice mode drops Submit (embedded practice is
//! Edit + Run only) and accepts the editorial's Copy-to-editor feed, which lands in the tab
//! MATCHING the solution's language.

use leptos::prelude::*;
use leptos::task::spawn_local;
use synapse_shared::execution::{RunResult, TestSpec, Verdict, judge, stdin_for};

use crate::execution::logic::{self, ExecutorState, RunState, Variant};
use crate::execution::state::{BlockStore, SubmitState, SubmitStore};
use crate::execution::view::workbench::{TestsPanel, TestsState, VerdictPanel};
use crate::identity::state::AuthStore;
use crate::islands::editor::{self, MountedEditor};

// Component props are moved by design (leptos owns them for the view's lifetime); the length
// is the component's cohesive wiring — props → stores → callbacks → view — and splitting it
// would hide the flow (the oracle keeps `Workbench.apply` as one unit too).
#[allow(clippy::needless_pass_by_value, clippy::too_many_lines)]
#[component]
pub fn RunnableBlock(
    variants: Vec<Variant>,
    spec: Option<TestSpec>,
    lesson_path: Vec<String>,
    // Passed through hydration: blocks mount OUT-OF-TREE (`mount_to` starts a fresh root
    // owner), so App's context is out of reach — the reader captures the store in-tree.
    auth: AuthStore,
    // The Coach's snapshot of what the learner sees: (source, language), kept current on
    // every edit; the pane reads it only at send time.
    code_sink: RwSignal<(String, String)>,
    // Same out-of-tree rule as `auth`: context is unreachable here, so the theme rides in.
    theme: crate::shell::theme::ThemeStore,
    viz_modal: crate::viz::modal::VizModalStore,
    /// Embedded practice (step 30): Run only — the Submit verb never renders.
    #[prop(optional)]
    practice: bool,
    /// The editorial's Copy-to-editor feed: `(tick, language, code)` — the tick makes
    /// re-copies of the same code fire; the language picks (and switches to) its tab.
    #[prop(optional)]
    load_code: Option<RwSignal<(u32, String, String)>>,
) -> impl IntoView {
    let stores: Vec<BlockStore> = variants.iter().map(|v| BlockStore::new(&v.source)).collect();
    let active = RwSignal::new(0_usize);
    code_sink.set((variants[0].source.clone(), variants[0].language.clone()));
    let submit = SubmitStore::new();
    let first = variants[0].clone();
    let variants = StoredValue::new(variants);
    let spec = spec.map(StoredValue::new);
    let tests = spec.map(|s| TestsState::new(&s.read_value()));
    let has_submit = spec.is_some() && !practice;
    let lesson_path = StoredValue::new(lesson_path);
    let mounted: StoredValue<Option<MountedEditor>, LocalStorage> = StoredValue::new_local(None);
    let editor_ref: NodeRef<leptos::html::Div> = NodeRef::new();

    let store_at = {
        let stores = stores.clone();
        move |i: usize| stores[i.min(stores.len() - 1)]
    };
    let active_store = {
        let store_at = store_at.clone();
        move || store_at(active.get_untracked())
    };
    let variant_at = move |i: usize| variants.read_value()[i].clone();

    // The Run seam: with a suite, stdin is the active case's values through the SHARED shape.
    let stdin = move || match (spec, tests) {
        (Some(spec), Some(tests)) => Some(stdin_for(&spec.read_value().args, &tests.values.get_untracked())),
        _ => None,
    };
    let run = {
        let active_store = active_store.clone();
        move || active_store().launch(variant_at(active.get_untracked()).language, stdin())
    };
    let do_submit = {
        let active_store = active_store.clone();
        move || {
            // The auth gate covers the ⇧⌘⏎ keymap path too — the button is merely disabled.
            if !has_submit || !auth.authed() {
                return;
            }
            submit.submit(
                lesson_path.read_value().clone(),
                variant_at(active.get_untracked()).language,
                active_store().state.get_untracked().code,
            );
        }
    };

    let run_click = run.clone();
    let submit_click = do_submit.clone();

    // Switching tabs swaps the ONE Monaco in place: buffer, tokenizer, read-only state —
    // each variant's edits live on in its own store.
    let switch_to = {
        let store_at = store_at.clone();
        move |i: usize| {
            if i == active.get_untracked() {
                return;
            }
            active.set(i);
            let store = store_at(i);
            let variant = variant_at(i);
            let code = store.state.get_untracked().code;
            mounted.with_value(|editor| {
                if let Some(editor) = editor {
                    editor.set_value(&code);
                    editor.set_language(&variant.language);
                    editor.set_read_only(!store.unlocked.get_untracked());
                }
            });
            code_sink.set((code, variant.language));
        }
    };

    // Visualise (step 28): a Python/Java variant with a viz= hint traces through the SAME
    // /api/run and plays in the modal. The stdin snapshot mirrors Run's.
    let visualisable = Memo::new(move |_| logic::can_visualise(&variants.read_value()[active.get()]));
    let open_visualise = {
        let active_store = active_store.clone();
        move |_| {
            let variant = variant_at(active.get_untracked());
            let Some(hint) = variant.viz else { return };
            let Some((structure, root)) = synapse_shared::viz::vocabulary::VizStructure::parse(&hint) else {
                return;
            };
            let key = crate::viz::session::Key {
                language: variant.language,
                source: active_store().state.get_untracked().code,
                structure,
                root,
                stdin: stdin().unwrap_or_default(),
            };
            viz_modal.open(crate::viz::session::obtain(key));
        }
    };

    // Mount monaco once the container exists; the handle + closures live in `mounted` and are
    // dropped (→ disposed) when the block unmounts.
    {
        let run = run.clone();
        let do_submit = do_submit.clone();
        let store_at = store_at.clone();
        Effect::new(move |_| {
            let Some(node) = editor_ref.get() else { return };
            if mounted.read_value().is_some() {
                return;
            }
            let value = store_at(0).state.get_untracked().code;
            let lang = variant_at(0).language;
            let run = run.clone();
            let do_submit = do_submit.clone();
            let store_at = store_at.clone();
            spawn_local(async move {
                let change_store = store_at.clone();
                let toggle_store = store_at.clone();
                let callbacks = editor::EditorCallbacks {
                    on_change: Box::new(move |code: String| {
                        change_store(active.get_untracked())
                            .state
                            .update(|s| *s = s.set_code(&code));
                        code_sink.update(|(source, _)| *source = code);
                    }),
                    on_run: Box::new(run),
                    on_toggle_edit: Box::new(move || {
                        // The auth gate (oracle: canEditSignal = authed && unlocked).
                        if !auth.authed() {
                            return;
                        }
                        let i = active.get_untracked();
                        let store = toggle_store(i);
                        store.toggle_edit(&variants.read_value()[i].source);
                        sync_editor(mounted, store);
                    }),
                    on_submit: has_submit.then(move || Box::new(do_submit) as Box<dyn FnMut()>),
                };
                let dark = theme.is_dark();
                match editor::mount(&node, &value, &lang, true, dark, callbacks).await {
                    Ok(handle) => mounted.set_value(Some(handle)),
                    Err(error) => leptos::logging::error!("monaco island failed: {error:?}"),
                }
            });
        });
    }
    // Monaco paints its own canvas — the toggle re-themes it (setTheme is global+idempotent).
    Effect::new(move |_| {
        let dark = theme.mode.get() == crate::shell::theme::Mode::Dark;
        mounted.with_value(|editor| {
            if let Some(editor) = editor {
                editor.set_theme(dark);
            }
        });
    });
    // The editorial's Copy-to-editor: land in the tab matching the solution's language
    // (fall back to the active tab), overwrite that buffer, and show it.
    if let Some(load) = load_code {
        let store_at = store_at.clone();
        let switch_to = switch_to.clone();
        Effect::new(move |prev: Option<u32>| {
            let (tick, lang, code) = load.get();
            if tick == 0 || prev == Some(tick) {
                return tick;
            }
            let target = variants
                .read_value()
                .iter()
                .position(|v| v.language.eq_ignore_ascii_case(&lang))
                .unwrap_or_else(|| active.get_untracked());
            switch_to(target);
            store_at(target).state.update(|s| *s = s.set_code(&code));
            mounted.with_value(|editor| {
                if let Some(editor) = editor {
                    editor.set_value(&code);
                }
            });
            code_sink.set((code, variant_at(target).language));
            tick
        });
    }
    on_cleanup(move || mounted.set_value(None));

    let running = {
        let store_at = store_at.clone();
        Memo::new(move |_| store_at(active.get()).state.read().run_state == RunState::Running)
    };
    let judging = Memo::new(move |_| matches!(submit.state.get(), SubmitState::Judging(_)));
    let unlocked = {
        let store_at = store_at.clone();
        Memo::new(move |_| store_at(active.get()).unlocked.get())
    };
    let height = format!("height: {}px;", editor::default_height_px(&first.source));
    let active_state: Signal<ExecutorState> = {
        let store_at = store_at.clone();
        Signal::derive(move || store_at(active.get()).state.get())
    };

    // One static pill for a single language; TABS when the fence group carried variants.
    let lang_count = variants.read_value().len();
    let lang_chrome = if lang_count > 1 {
        let switch_to = switch_to.clone();
        let tabs: Vec<_> = (0..lang_count)
            .map(|i| {
                let label = logic::display_lang(&variants.read_value()[i].language);
                let switch_to = switch_to.clone();
                view! {
                    <button
                        class="wb__lang-tab"
                        class:wb__lang-tab--active=move || active.get() == i
                        on:click=move |_| switch_to(i)
                    >
                        {label}
                    </button>
                }
            })
            .collect();
        view! { <span class="wb__lang-tabs">{tabs}</span> }.into_any()
    } else {
        let pill = logic::display_lang(&first.language);
        view! { <span class="wb__lang-pill">{pill}</span> }.into_any()
    };

    let toggle_store = store_at.clone();
    let toolbar = view! {
        <div class="runnable__bar">
            <span class="wb__eyebrow"><span class="wb__prompt">">_"</span>" CODE"</span>
            <span class="wb__actions">
                {lang_chrome}
                <span
                    class="wb__tip"
                    data-tip=move || {
                        if !auth.authed() {
                            "Sign in to edit this code"
                        } else if unlocked.get() {
                            "Editing — your changes stay on this page (⌘E toggles)"
                        } else {
                            "Edit this code — changes stay on this page (⌘E)"
                        }
                    }
                >
                    <button
                        class="wb__ghost"
                        class:wb__ghost--live=move || unlocked.get()
                        prop:disabled=move || !auth.authed()
                        on:click=move |_| {
                            let i = active.get_untracked();
                            let store = toggle_store(i);
                            store.toggle_edit(&variants.read_value()[i].source);
                            sync_editor(mounted, store);
                        }
                    >
                        {move || if unlocked.get() { "Editing" } else { "Edit" }}
                    </button>
                </span>
                {has_submit.then(|| view! {
                    // Anonymous readers see WHY it's off (the step-20 allowlist hint); the
                    // server re-enforces regardless — this is UX, not the gate.
                    <span
                        class="wb__tip"
                        data-tip=move || {
                            if auth.authed() {
                                "Submit against the hidden suite (⇧⌘⏎)"
                            } else {
                                "Sign in to submit. Submit runs your code against every hidden \
                                 test and saves your attempt. Saving needs an approved account \
                                 — ask the operator for access."
                            }
                        }
                    >
                        <button
                            class="wb__submit"
                            prop:disabled=move || !auth.authed() || judging.get()
                            on:click=move |_| submit_click()
                        >
                            {move || if judging.get() { "Judging…" } else { "Submit" }}
                        </button>
                    </span>
                })}
                {move || visualisable.get().then(|| view! {
                    <button
                        class="wb__ghost"
                        title="Trace this code and watch the structure animate"
                        on:click=open_visualise.clone()
                    >
                        "Visualise"
                    </button>
                })}
                <button
                    class="runnable__run"
                    title="Run (⌘⏎)"
                    prop:disabled=move || running.get()
                    on:click=move |_| run_click()
                >
                    {move || if running.get() { "Running…" } else { "▶ Run" }}
                </button>
            </span>
        </div>
    };

    view! {
        <div class="runnable not-prose">
            {toolbar}
            <div class="runnable__editor" node_ref=editor_ref style=height></div>
            {match (spec, tests) {
                (Some(spec), Some(tests)) => Some(view! { <TestsPanel spec=spec tests=tests /> }),
                _ => None,
            }}
            <Output state=active_state spec=spec tests=tests />
            <VerdictPanel submit=submit />
        </div>
    }
}

/// Locking/unlocking must reach monaco too: read-only flips in place, and a revert rewrites
/// the buffer.
fn sync_editor(mounted: StoredValue<Option<MountedEditor>, LocalStorage>, store: BlockStore) {
    mounted.with_value(|editor| {
        if let Some(editor) = editor {
            let state = store.state.get_untracked();
            editor.set_read_only(!store.unlocked.get_untracked());
            if editor.get_value() != state.code {
                editor.set_value(&state.code);
            }
        }
    });
}

#[component]
fn Output(
    state: Signal<ExecutorState>,
    spec: Option<StoredValue<TestSpec>>,
    tests: Option<TestsState>,
) -> impl IntoView {
    view! {
        {move || {
            let state = state.get();
            if let Some(error) = &state.error {
                return error_panel(error).into_any();
            }
            if let Some(result) = &state.result {
                let expected = match (spec, tests) {
                    (Some(spec), Some(tests)) => {
                        logic::expected_for(&spec.read_value(), tests.active_case.get())
                    }
                    _ => None,
                };
                return result_panel(result, expected.as_deref()).into_any();
            }
            if state.run_state == RunState::Running {
                return view! { <div class="runnable__out runnable__out--running">"Running…"</div> }
                    .into_any();
            }
            ().into_any()
        }}
    }
}

fn error_panel(error: &str) -> impl IntoView + use<> {
    view! {
        <div class="runnable__out runnable__out--error">
            <div class="runnable__status"><span class="runnable__badge runnable__badge--fail">"Error"</span></div>
            <pre class="runnable__stream">{error.to_owned()}</pre>
        </div>
    }
}

/// With an expected output the stdout is JUDGED (the wb-legend tint); without one it renders
/// plain — exactly the oracle's split.
fn result_panel(result: &RunResult, expected: Option<&str>) -> impl IntoView + use<> {
    let verdict = expected.map(|e| judge(result, Some(e)));
    let (badge_label, badge_ok) = match verdict {
        Some(Verdict::Accepted) => ("Accepted ✓".to_owned(), true),
        Some(Verdict::WrongAnswer) => ("Wrong answer ✗".to_owned(), false),
        _ => (result.status.label().to_owned(), result.status.is_success()),
    };
    let badge_class = if badge_ok {
        "runnable__badge runnable__badge--ok"
    } else {
        "runnable__badge runnable__badge--fail"
    };
    let stdout_class = match verdict {
        Some(Verdict::Accepted) => "runnable__stdout wb-legend--ok",
        Some(Verdict::WrongAnswer) => "runnable__stdout wb-legend--err",
        _ => "runnable__stdout",
    };
    let time = result.time_seconds.map(|s| format!("{s:.3} s"));
    let memory = result.memory_kb.map(|kb| format!("{} MB", kb / 1024));
    let stdout = result.stdout.clone();
    view! {
        <div class="runnable__out">
            <div class="runnable__status">
                <span class=badge_class>{badge_label}</span>
                {time.map(|t| view! { <span class="runnable__meta">{t}</span> })}
                {memory.map(|m| view! { <span class="runnable__meta">{m}</span> })}
            </div>
            {stream_block("compile output", &result.compile_output)}
            {stream_block("stderr", &result.stderr)}
            {if stdout.is_empty() {
                view! { <p class="runnable__empty">"(no output)"</p> }.into_any()
            } else {
                view! { <pre class=stdout_class>{stdout}</pre> }.into_any()
            }}
        </div>
    }
}

fn stream_block(label: &'static str, content: &str) -> Option<impl IntoView + use<>> {
    if content.is_empty() {
        return None;
    }
    let content = content.to_owned();
    Some(view! {
        <details class="runnable__details" open>
            <summary class="runnable__details-label">{label}</summary>
            <pre class="runnable__stream">{content}</pre>
        </details>
    })
}
