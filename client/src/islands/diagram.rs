//! The `@diagram` island (oracle: `MermaidView` via `@diagram/loader`). The extern binds the
//! tiny loader; the loader dynamic-imports mermaid, so the multi-hundred-KB chunk lands only
//! on lessons that actually contain a mermaid diagram.

use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "@diagram/loader")]
extern "C" {
    #[wasm_bindgen(js_name = renderMermaid)]
    fn render_mermaid_js(target: &web_sys::HtmlElement, src: &str) -> js_sys::Promise;
}

/// Render mermaid source into `target` as an inline SVG. A malformed diagram rejects —
/// callers show the loud error card (ADR-S026), never a blank figure.
pub async fn render_mermaid(target: &web_sys::HtmlElement, src: &str) -> Result<(), JsValue> {
    wasm_bindgen_futures::JsFuture::from(render_mermaid_js(target, src)).await?;
    Ok(())
}
