//! The Visualise modal (oracle: `VisualiseModal` + `SourcePane` + `FramesPanel`, ADR-S031):
//! a near-fullscreen player over the SAME `WidgetHost` the inline widgets use — the case
//! strip, the read-only source pane with current/next line highlights, the frames panel, and
//! the program output. Esc closes; Space toggles play; ←/→ step. (Diff-mode stops, the
//! timeline drawer, and deep links join with the wrap step.)

use leptos::prelude::*;
use leptos::task::spawn_local;
use synapse_shared::viz::graph::{VizCases, VizGraph};
use synapse_shared::viz::playback::State;

use crate::islands::editor::{self, EditorCallbacks, MountedEditor};
use crate::viz::host::WidgetHost;
use crate::viz::session::{self, Session, TraceState};

/// What the modal shows (set by the workbench's Visualise button).
#[derive(Clone)]
pub struct ModalSession {
    pub session: Session,
}

#[derive(Clone, Copy)]
pub struct VizModalStore {
    pub current: RwSignal<Option<ModalSession>>,
}

impl VizModalStore {
    pub fn provide() {
        provide_context(Self {
            current: RwSignal::new(None),
        });
    }

    pub fn from_context() -> Self {
        expect_context::<Self>()
    }

    pub fn open(self, session: Session) {
        self.current.set(Some(ModalSession { session }));
    }

    pub fn close(self) {
        self.current.set(None);
    }
}

/// Mounted once in the shell (like the search palette).
#[component]
pub fn VisualiseModal() -> impl IntoView {
    let store = VizModalStore::from_context();
    let esc = window_event_listener(leptos::ev::keydown, move |event| {
        if event.key() == "Escape" && store.current.get_untracked().is_some() {
            store.close();
        }
    });
    on_cleanup(move || esc.remove());
    view! {
        {move || {
            store.current.get().map(|modal| {
                view! {
                    <div class="viz-modal">
                        <div class="viz-modal__scrim" on:click=move |_| store.close()></div>
                        <div class="viz-modal__frame">
                            <ModalBar modal=modal.clone() store=store />
                            <div class="viz-modal__body">
                                <ModalBody modal=modal.clone() />
                            </div>
                        </div>
                    </div>
                }
            })
        }}
    }
}

#[component]
fn ModalBar(modal: ModalSession, store: VizModalStore) -> impl IntoView {
    let retrace = modal.session.clone();
    let title = modal.session.key.structure.token();
    view! {
        <div class="viz-modal__bar">
            <span class="viz-modal__eyebrow"><span class="viz-modal__prompt">"◆"</span>" VISUALISE"</span>
            <span class="viz-modal__title">{title}</span>
            <span class="viz-modal__bar-spacer"></span>
            <button class="viz-modal__retrace" on:click=move |_| session::force(&retrace)>
                "↻ Re-trace"
            </button>
            <button class="viz-modal__close" aria-label="Close" on:click=move |_| store.close()>
                "✕"
            </button>
        </div>
    }
}

#[component]
fn ModalBody(modal: ModalSession) -> impl IntoView {
    let state = modal.session.state;
    view! {
        {move || match state.get() {
            TraceState::Tracing => view! {
                <div class="viz-modal__status">
                    <span class="viz-modal__spinner"></span>
                    "Tracing your code…"
                </div>
            }
            .into_any(),
            TraceState::Failed(message) => view! {
                <div class="viz-modal__failed">
                    <p class="viz-modal__failed-title">"Couldn't visualise this run"</p>
                    <pre class="viz-modal__failed-msg">{message}</pre>
                </div>
            }
            .into_any(),
            TraceState::Ready(cases, program_out) => {
                ready(&modal, &cases, &program_out).into_any()
            }
        }}
    }
}

#[allow(clippy::too_many_lines)] // the modal's ready layout is one cohesive block
fn ready(modal: &ModalSession, cases: &VizCases, program_out: &str) -> impl IntoView + use<> {
    let case_idx = RwSignal::new(0usize);
    let cases = cases.clone();
    let case_count = cases.cases.len();
    let key = modal.session.key.clone();
    // One playback per case; switching cases resets it (a fresh graph = a fresh animation).
    let step_state = RwSignal::new(State::initial(
        i64::try_from(cases.cases.first().map_or(1, |g| g.steps.len())).unwrap_or(1),
    ));
    Effect::new(move |prev: Option<usize>| {
        let idx = case_idx.get();
        if prev.is_some_and(|p| p != idx) {
            // count is corrected below when the graph renders
            step_state.set(State::initial(1));
        }
        idx
    });

    // Space / arrows drive the shared stepper.
    let keys = window_event_listener(leptos::ev::keydown, move |event| match event.key().as_str() {
        " " => {
            event.prevent_default();
            step_state.update(|s| *s = s.toggle_play());
        }
        "ArrowRight" => step_state.update(|s| *s = s.next()),
        "ArrowLeft" => step_state.update(|s| *s = s.previous()),
        _ => {}
    });
    on_cleanup(move || keys.remove());

    let strip_cases = cases.clone();
    let host_cases = cases.clone();
    let pane_cases = cases.clone();
    let program_out = program_out.to_owned();
    view! {
        <div class="viz-modal__ready">
            {(case_count > 1).then(|| view! {
                <div class="viz-modal__casestrip">
                    {(0..strip_cases.cases.len())
                        .map(|i| {
                            let label = format!("Case {}", i + 1);
                            view! {
                                <button
                                    class="viz-modal__case"
                                    class:viz-modal__case--active=move || case_idx.get() == i
                                    on:click=move |_| {
                                        case_idx.set(i);
                                    }
                                >
                                    {label}
                                </button>
                            }
                        })
                        .collect::<Vec<_>>()}
                </div>
            })}
            <div class="viz-modal__layout">
                <div class="viz-modal__canvas-col">
                    {move || {
                        let idx = case_idx.get().min(host_cases.cases.len() - 1);
                        let graph = host_cases.cases[idx].clone();
                        step_state.update(|s| {
                            s.count = graph.steps.len().max(1);
                            s.index = s.index.min(s.count - 1);
                        });
                        let one = VizCases { cases: vec![graph] };
                        view! {
                            <WidgetHost
                                name="trace".to_owned()
                                structure=Some(key.structure)
                                cases=Some(one)
                                external=step_state
                                legend=true
                            />
                        }
                    }}
                </div>
                <div class="viz-modal__side-col">
                    <SourcePane
                        source=key.source.clone()
                        language=key.language.clone()
                        cases=pane_cases
                        case_idx=case_idx
                        step_state=step_state
                    />
                    <FramesPanel cases=cases.clone() case_idx=case_idx step_state=step_state />
                </div>
            </div>
            {(!program_out.is_empty()).then(|| view! {
                <details class="viz-modal__output">
                    <summary>"Program output"</summary>
                    <pre>{program_out.clone()}</pre>
                </details>
            })}
        </div>
    }
}

/// Read-only Monaco over the RAW pre-wrap source (line numbers align with the captions),
/// with current/next line highlights following the stepper.
#[component]
fn SourcePane(
    source: String,
    language: String,
    cases: VizCases,
    case_idx: RwSignal<usize>,
    step_state: RwSignal<State>,
) -> impl IntoView {
    let node_ref: NodeRef<leptos::html::Div> = NodeRef::new();
    let mounted: StoredValue<Option<MountedEditor>, LocalStorage> = StoredValue::new_local(None);
    let source_for_mount = source.clone();
    Effect::new(move |_| {
        let Some(node) = node_ref.get() else { return };
        if mounted.read_value().is_some() {
            return;
        }
        let value = source_for_mount.clone();
        let lang = language.clone();
        spawn_local(async move {
            let callbacks = EditorCallbacks {
                on_change: Box::new(|_| {}),
                on_run: Box::new(|| {}),
                on_toggle_edit: Box::new(|| {}),
                on_submit: None,
            };
            let dark = crate::shell::theme::html_is_dark();
            match editor::mount(&node, &value, &lang, true, dark, callbacks).await {
                Ok(handle) => mounted.set_value(Some(handle)),
                Err(error) => leptos::logging::error!("source pane monaco failed: {error:?}"),
            }
        });
    });
    // Highlights re-fire on step AND on late Monaco arrival (mounted is plain storage; the
    // step signal ticks often enough that the first highlight lands right after mount).
    Effect::new(move |_| {
        let idx = case_idx.get();
        let state = step_state.get();
        let Some(graph) = cases.cases.get(idx) else { return };
        let Some(step) = graph.steps.get(state.index) else {
            return;
        };
        let current = u32::try_from(step.line.max(0)).unwrap_or(0);
        let next = graph
            .steps
            .get(state.index + 1)
            .and_then(|s| u32::try_from(s.line.max(0)).ok());
        mounted.with_value(|editor| {
            if let Some(editor) = editor {
                editor.set_line_highlights(current, next);
            }
        });
    });
    on_cleanup(move || mounted.set_value(None));
    view! { <div class="viz-modal__source" node_ref=node_ref></div> }
}

/// The call-stack panel: per-frame fn + locals, the active frame carrying the line chips.
#[component]
fn FramesPanel(cases: VizCases, case_idx: RwSignal<usize>, step_state: RwSignal<State>) -> impl IntoView {
    view! {
        <div class="viz-frames">
            {move || {
                let idx = case_idx.get();
                let state = step_state.get();
                let step = cases
                    .cases
                    .get(idx)
                    .and_then(|g: &VizGraph| g.steps.get(state.index));
                let Some(step) = step else {
                    return ().into_any();
                };
                let current = step.line;
                step.frames
                    .iter()
                    .map(|frame| {
                        let class = if frame.is_active { "viz-frame viz-frame--active" } else { "viz-frame" };
                        let chips = frame.is_active.then(|| view! {
                            <span class="viz-frame__lines">
                                <span class="viz-frame__line">{format!("L{current}")}</span>
                            </span>
                        });
                        let locals: Vec<_> = frame
                            .locals
                            .iter()
                            .map(|l| {
                                let lclass = if l.changed {
                                    "viz-frame__local viz-frame__local--changed"
                                } else {
                                    "viz-frame__local"
                                };
                                view! {
                                    <div class=lclass>
                                        <span class="viz-frame__local-name">{l.name.clone()}</span>
                                        <span class="viz-frame__local-type">{l.type_name.clone()}</span>
                                        <span class="viz-frame__local-value">{l.value.clone()}</span>
                                    </div>
                                }
                            })
                            .collect();
                        view! {
                            <div class=class>
                                <div class="viz-frame__fn">{frame.fn_name.clone()}{chips}</div>
                                <div class="viz-frame__locals">{locals}</div>
                            </div>
                        }
                    })
                    .collect::<Vec<_>>()
                    .into_any()
            }}
        </div>
    }
}
