//! The **quiz card** (oracle: `quiz/QuizCard` + `QuizBlocks`, step 16 — a thin flat feature,
//! ADR-S014's flat rule): one check-your-understanding question rendered from a ` ```quiz `
//! fence. Select an option, **Check** — the right answer tints green wherever it is, a wrong
//! pick tints red, and the verdict line says which; **Try again** resets. All state is two
//! signals; nothing leaves the component (quizzes are ungraded prose furniture, not
//! submissions).

use std::any::Any;

use leptos::prelude::*;
use serde::Deserialize;

use crate::hydration;

// ─────────────────────────────────────────────────────────────────────────────
// MODEL + DISCOVERY
// The pipeline plants `div.quiz-block[data-quiz]` (URI-encoded JSON); discovery
// decodes purely and skips malformed cards rather than crashing the page.
// ─────────────────────────────────────────────────────────────────────────────

/// One authored quiz card: `{prompt, options, answer, input?}` — `answer` is one of `options`
/// (render.ts shape-checks at parse time; the decode here just mirrors it).
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Quiz {
    pub prompt: String,
    pub options: Vec<String>,
    pub answer: String,
    #[serde(default)]
    pub input: Option<String>,
}

pub fn hydrate_quizzes(root: &web_sys::HtmlElement) -> Vec<Box<dyn Any>> {
    hydration::mount_each(root, "div.quiz-block", |element| {
        let quiz = hydration::decoded_attr(&element, "data-quiz")
            .and_then(|json| serde_json::from_str::<Quiz>(&json).ok())?;
        Some(hydration::mount(element, move || {
            view! { <QuizCard quiz=quiz /> }
        }))
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// THE CARD
// ─────────────────────────────────────────────────────────────────────────────

#[component]
fn QuizCard(quiz: Quiz) -> impl IntoView {
    let selected: RwSignal<Option<usize>> = RwSignal::new(None);
    let checked = RwSignal::new(false);
    let quiz = StoredValue::new(quiz);
    let options = quiz.read_value().options.clone();
    view! {
        <div class="quiz not-prose">
            <div class="quiz__head">
                <span class="wb__eyebrow">
                    <span class="wb__prompt">"?"</span>
                    "Quiz"
                </span>
                <p class="quiz__prompt">{quiz.read_value().prompt.clone()}</p>
            </div>
            {quiz.read_value().input.clone().map(|input| view! {
                <pre class="quiz__input"><code>{input}</code></pre>
            })}
            <div class="quiz__options">
                {options.into_iter().enumerate().map(|(i, option)| {
                    let is_answer = option == quiz.read_value().answer;
                    view! {
                        <button
                            type="button"
                            class="quiz__option"
                            class:quiz__option--selected=move || {
                                selected.get() == Some(i) && !checked.get()
                            }
                            class:quiz__option--right=move || checked.get() && is_answer
                            class:quiz__option--wrong=move || {
                                checked.get() && selected.get() == Some(i) && !is_answer
                            }
                            disabled=move || checked.get()
                            on:click=move |_| selected.set(Some(i))
                        >
                            {option}
                        </button>
                    }
                }).collect_view()}
            </div>
            <div class="quiz__foot">
                {move || {
                    if checked.get() {
                        let q = quiz.read_value();
                        let correct = selected
                            .get()
                            .and_then(|i| q.options.get(i))
                            .is_some_and(|picked| *picked == q.answer);
                        let answer = q.answer.clone();
                        view! {
                            <div class="quiz__verdict">
                                {if correct {
                                    view! { <span class="quiz__verdict-ok">"Correct ✓"</span> }
                                        .into_any()
                                } else {
                                    view! {
                                        <span class="quiz__verdict-no">
                                            {format!("Not quite — the answer is “{answer}”")}
                                        </span>
                                    }
                                        .into_any()
                                }}
                                <button
                                    type="button"
                                    class="quiz__again"
                                    on:click=move |_| {
                                        checked.set(false);
                                        selected.set(None);
                                    }
                                >
                                    "Try again"
                                </button>
                            </div>
                        }
                            .into_any()
                    } else {
                        view! {
                            <button
                                type="button"
                                class="quiz__check"
                                disabled=move || selected.get().is_none()
                                on:click=move |_| checked.set(true)
                            >
                                "Check"
                            </button>
                        }
                            .into_any()
                    }
                }}
            </div>
        </div>
    }
}
