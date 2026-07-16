//! Family → renderer dispatch (oracle: `RendererRegistry`). The DECISION is the shared pure
//! `RenderFamily::of`; this maps a family to a concrete Leptos renderer. The bespoke HTML
//! families (Queue/Buckets/LinkedList/Grid/Forest/HeapDual — the step-33 gallery) return
//! `None` at this stage → the host's honest "isn't available yet" card.

use leptos::prelude::*;
use synapse_shared::viz::geometry::{self, graph_layout, tree as tree_layout};
use synapse_shared::viz::graph::VizCases;
use synapse_shared::viz::render_family::RenderFamily;
use synapse_shared::viz::vocabulary::VizStructure;

use crate::viz::render::{cells, chain, graph_canvas, tree};

/// `None` when no case has steps OR the family has no renderer yet.
#[must_use]
pub fn render(structure: VizStructure, cases: &VizCases, step_index: Signal<usize>) -> Option<AnyView> {
    let graph = cases.cases.iter().find(|g| !g.steps.is_empty())?;
    match RenderFamily::of(structure) {
        RenderFamily::Cells => Some(cells::array(graph, step_index)),
        RenderFamily::Stack => Some(cells::stack(graph, step_index)),
        RenderFamily::Tree => Some(tree::tree(graph, step_index)),
        RenderFamily::Chain => Some(chain::chain(graph, step_index)),
        RenderFamily::Force => {
            let (un, ue) = geometry::union(graph);
            Some(graph_canvas::canvas(
                graph,
                graph_layout::graph(&un, &ue),
                step_index,
            ))
        }
        RenderFamily::Trie => {
            let (un, ue) = geometry::union(graph);
            Some(graph_canvas::canvas(
                graph,
                tree_layout::tree(&un, &ue),
                step_index,
            ))
        }
        // The bespoke HTML gallery joins in its own step.
        RenderFamily::Grid
        | RenderFamily::Buckets
        | RenderFamily::Queue
        | RenderFamily::LinkedList
        | RenderFamily::Forest
        | RenderFamily::HeapDual => None,
    }
}
