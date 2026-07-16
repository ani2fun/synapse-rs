//! The widget host (oracle: `WidgetHost`) — the ONE `VizCases` consumer: title, the scaled
//! canvas, the transport bar (only when there's more than one step), the reactive caption,
//! and the honest failure cards (never a blank box).

use leptos::prelude::*;
use synapse_shared::viz::graph::VizCases;
use synapse_shared::viz::playback::State;
use synapse_shared::viz::vocabulary::VizStructure;

use crate::viz::registry;
use crate::viz::transport::TransportBar;

// Component props are moved by design (leptos owns them for the view's lifetime).
#[allow(clippy::needless_pass_by_value)]
#[component]
pub fn WidgetHost(name: String, structure: Option<VizStructure>, cases: Option<VizCases>) -> impl IntoView {
    match (structure, cases) {
        (Some(structure), Some(cases)) => {
            let Some(graph) = cases.cases.iter().find(|g| !g.steps.is_empty()).cloned() else {
                return unavailable(structure.token());
            };
            let state = RwSignal::new(State::initial(i64::try_from(graph.steps.len()).unwrap_or(1)));
            let step_index = Signal::derive(move || state.get().index);
            let Some(canvas) = registry::render(structure, &cases, step_index) else {
                return unavailable(structure.token());
            };
            let title = (!graph.title.is_empty()).then(|| graph.title.clone());
            let multi_step = graph.steps.len() > 1;
            let caption_graph = graph.clone();
            view! {
                <div class="viz-widget-host not-prose">
                    {title.map(|t| view! { <div class="viz-widget-host__title">{t}</div> })}
                    <div class="viz-widget-host__canvas">
                        <div class="viz-widget-host__scale">{canvas}</div>
                    </div>
                    {multi_step.then(|| view! { <TransportBar state=state /> })}
                    <div class="viz-widget-host__caption">
                        {move || {
                            let i = state.get().index.min(caption_graph.steps.len() - 1);
                            caption_graph.steps[i].annotation.body.clone()
                        }}
                    </div>
                </div>
            }
            .into_any()
        }
        (None, _) => unavailable(&name),
        (Some(_), None) => bad_payload(&name),
    }
}

fn unavailable(name: &str) -> AnyView {
    let copy = format!("The “{name}” widget isn’t available yet.");
    view! {
        <div class="viz-widget-host viz-widget-host--unavailable not-prose"><span>{copy}</span></div>
    }
    .into_any()
}

fn bad_payload(name: &str) -> AnyView {
    let copy = format!("Couldn’t read the “{name}” widget payload.");
    view! {
        <div class="viz-widget-host viz-widget-host--error not-prose"><span>{copy}</span></div>
    }
    .into_any()
}
