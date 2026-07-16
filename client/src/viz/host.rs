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
pub fn WidgetHost(
    name: String,
    structure: Option<VizStructure>,
    cases: Option<VizCases>,
    /// The modal drives the SAME stepper its keyboard reads (external playback).
    #[prop(optional)]
    external: Option<RwSignal<State>>,
    /// The modal shows the data-driven legend; inline widgets never do.
    #[prop(optional)]
    legend: bool,
) -> impl IntoView {
    match (structure, cases) {
        (Some(structure), Some(cases)) => {
            let Some(graph) = cases.cases.iter().find(|g| !g.steps.is_empty()).cloned() else {
                return unavailable(structure.token());
            };
            let state = external.unwrap_or_else(|| {
                RwSignal::new(State::initial(i64::try_from(graph.steps.len()).unwrap_or(1)))
            });
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
                    {legend.then(|| legend_view(&graph))}
                </div>
            }
            .into_any()
        }
        (None, _) => unavailable(&name),
        (Some(_), None) => bad_payload(&name),
    }
}

/// The data-driven legend (oracle: `DomKit.legend`): rows appear only when the trace uses
/// the cue.
fn legend_view(graph: &synapse_shared::viz::graph::VizGraph) -> AnyView {
    let any_cursor = graph.steps.iter().any(|s| !s.cursor.is_empty());
    let any_new = graph.steps.iter().any(|s| !s.highlight.is_empty());
    let any_changed = graph.steps.iter().any(|s| !s.changed.is_empty());
    let any_removed = graph.steps.iter().any(|s| !s.removed.is_empty());
    let item = |swatch_class: &'static str, text: &'static str| {
        view! {
            <span class="viz-legend__item">
                <span class=format!("viz-legend__swatch {swatch_class}")></span>
                <span class="viz-legend__text">{text}</span>
            </span>
        }
    };
    view! {
        <div class="viz-legend">
            {any_cursor.then(|| item("viz-legend__swatch--cursor", "▾ a variable points here"))}
            {any_new.then(|| item("viz-legend__swatch--new", "new this step"))}
            {any_changed.then(|| item("viz-legend__swatch--changed", "value changed"))}
            {any_removed.then(|| item("viz-legend__swatch--removed", "removed"))}
        </div>
    }
    .into_any()
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
