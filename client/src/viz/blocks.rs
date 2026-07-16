//! Widget placeholder discovery (oracle: `WidgetBlocks`): the markdown pipeline plants
//! `div.viz-widget[data-widget][data-payload]`; this finds them and decodes. Structure
//! resolution and payload decode are INDEPENDENT, so the host can tell "unknown structure"
//! from "unreadable payload" — honest cards, never a blank box.

use synapse_shared::viz::graph::VizCases;
use synapse_shared::viz::vocabulary::VizStructure;
use wasm_bindgen::JsCast;

pub struct WidgetSpec {
    pub name: String,
    pub structure: Option<VizStructure>,
    pub cases: Option<VizCases>,
}

#[must_use]
pub fn discover(root: &web_sys::HtmlElement) -> Vec<(web_sys::HtmlElement, WidgetSpec)> {
    let mut out = Vec::new();
    let Ok(nodes) = root.query_selector_all("div.viz-widget") else {
        return out;
    };
    for index in 0..nodes.length() {
        let Some(node) = nodes.get(index) else { continue };
        let Ok(element) = node.dyn_into::<web_sys::HtmlElement>() else {
            continue;
        };
        let name = element.get_attribute("data-widget").unwrap_or_default();
        let payload = element.get_attribute("data-payload").unwrap_or_default();
        out.push((element, decode(&name, &payload)));
    }
    out
}

#[must_use]
pub fn decode(name: &str, encoded_payload: &str) -> WidgetSpec {
    let structure = VizStructure::from_name(name);
    let cases = js_sys::decode_uri_component(encoded_payload)
        .ok()
        .map(String::from)
        .and_then(|json| serde_json::from_str::<VizCases>(&json).ok());
    WidgetSpec {
        name: name.trim().to_owned(),
        structure,
        cases,
    }
}
