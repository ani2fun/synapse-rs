//! The tests + verdict panels (oracle: `WorkbenchTests` + `verdictPanel`, step-15 scope): case
//! chips seeding the values grid, the Run-with-stdin seam, and the submit lifecycle from 202 to
//! the flattened outcome.

use std::collections::BTreeMap;

use leptos::prelude::*;
use synapse_shared::execution::{TestSpec, Verdict};
use synapse_shared::submission::SubmissionDto;

use crate::execution::logic;
use crate::execution::state::{SubmitState, SubmitStore};

/// Per-block test-panel state: the active case, the editable values grid, the sparse
/// per-case verdict map (only cases that have actually been Run carry a badge — oracle:
/// `WorkbenchState.verdicts`), and the case a launch was fired for (so the arriving result
/// is judged against THAT case, never against whichever chip is selected by then).
#[derive(Clone, Copy)]
pub struct TestsState {
    pub active_case: RwSignal<usize>,
    pub values: RwSignal<BTreeMap<String, String>>,
    pub verdicts: RwSignal<BTreeMap<usize, Verdict>>,
    pub ran_case: RwSignal<Option<usize>>,
}

impl TestsState {
    pub fn new(spec: &TestSpec) -> Self {
        Self {
            active_case: RwSignal::new(0),
            values: RwSignal::new(logic::seed_values(spec, 0)),
            verdicts: RwSignal::new(BTreeMap::new()),
            ran_case: RwSignal::new(None),
        }
    }
}

#[component]
pub fn TestsPanel(
    spec: StoredValue<TestSpec>,
    tests: TestsState,
    /// Fired on chip click AFTER the state re-seed — the block clears its stale run output
    /// (oracle: `switchCase` resets the FSM; earlier badges stay on the chips).
    on_switch: Callback<usize>,
) -> impl IntoView {
    let case_count = spec.read_value().cases.len();
    let chips: Vec<_> = (0..case_count)
        .map(|index| {
            view! {
                <button
                    class="wb__chip"
                    class:wb__chip--active=move || tests.active_case.get() == index
                    class:wb__chip--ok=move || {
                        tests.verdicts.read().get(&index) == Some(&Verdict::Accepted)
                    }
                    class:wb__chip--fail=move || {
                        matches!(
                            tests.verdicts.read().get(&index),
                            Some(Verdict::WrongAnswer | Verdict::Errored)
                        )
                    }
                    on:click=move |_| {
                        tests.active_case.set(index);
                        tests.values.set(logic::seed_values(&spec.read_value(), index));
                        on_switch.run(index);
                    }
                >
                    {format!("Case {}", index + 1)}
                    {move || match tests.verdicts.read().get(&index) {
                        Some(Verdict::Accepted) => {
                            Some(view! { <span class="wb__tick">"✓"</span> })
                        }
                        Some(Verdict::WrongAnswer | Verdict::Errored) => {
                            Some(view! { <span class="wb__tick">"✗"</span> })
                        }
                        _ => None,
                    }}
                </button>
            }
        })
        .collect();

    let fields: Vec<_> = spec
        .read_value()
        .args
        .iter()
        .map(|arg| {
            let id = arg.id.clone();
            let input_id = id.clone();
            let placeholder = arg.placeholder.clone().unwrap_or_default();
            view! {
                <label class="wb__field">
                    <span class="wb__field-label">{arg.label.clone()}</span>
                    <input
                        class="wb__input"
                        placeholder=placeholder
                        prop:value=move || tests.values.read().get(&id).cloned().unwrap_or_default()
                        on:input=move |ev| {
                            let value = event_target_value(&ev);
                            tests.values.update(|v| {
                                v.insert(input_id.clone(), value);
                            });
                        }
                    />
                </label>
            }
        })
        .collect();

    let expected = move || {
        logic::expected_for(&spec.read_value(), tests.active_case.get())
            .map(|e| view! { <div class="wb__expected"><span class="wb__field-label">"Expected"</span><pre>{e}</pre></div> })
    };

    view! {
        <div class="wb__tests">
            <div class="wb__chips">{chips}</div>
            <div class="wb__values">{fields}</div>
            {expected}
        </div>
    }
}

#[component]
pub fn VerdictPanel(submit: SubmitStore) -> impl IntoView {
    view! {
        {move || match submit.state.get() {
            SubmitState::Idle => ().into_any(),
            SubmitState::Judging(id) => view! {
                <div class="wb__verdict wb__verdict--judging">
                    "Judging against the hidden suite… " <span class="wb__verdict-id">{id}</span>
                </div>
            }
            .into_any(),
            SubmitState::Failed(message) => view! {
                <div class="wb__verdict wb__verdict--failed">"Submit failed: " {message}</div>
            }
            .into_any(),
            SubmitState::Done(dto) => done_panel(&dto).into_any(),
        }}
    }
}

fn done_panel(dto: &SubmissionDto) -> impl IntoView + use<> {
    let counts = format!("{} / {}", dto.passed.unwrap_or(0), dto.total.unwrap_or(0));
    match dto.verdict.as_deref() {
        Some("accepted") => view! {
            <div class="wb__verdict wb__verdict--accepted">"Accepted ✓ — " {counts} " cases"</div>
        }
        .into_any(),
        Some("rejected") => {
            let failure = dto.first_failure.clone().map(|f| {
                view! {
                    <div class="wb__failure">
                        <div class="wb__field-label">{format!("First failure — case {}", f.index + 1)}</div>
                        {f.expected.map(|e| view! { <pre class="wb__failure-line">"expected: " {e}</pre> })}
                        <pre class="wb__failure-line">"stdout:   " {f.stdout}</pre>
                        {(!f.stderr.is_empty())
                            .then(|| view! { <pre class="wb__failure-line">"stderr:   " {f.stderr}</pre> })}
                    </div>
                }
            });
            view! {
                <div class="wb__verdict wb__verdict--rejected">
                    "Wrong answer ✗ — " {counts} " cases passed"
                    {failure}
                </div>
            }
            .into_any()
        }
        _ => {
            let detail = dto.detail.clone().unwrap_or_default();
            view! {
                <div class="wb__verdict wb__verdict--failed">
                    "The judge failed mid-suite — " {counts} " passed. " {detail}
                </div>
            }
            .into_any()
        }
    }
}
