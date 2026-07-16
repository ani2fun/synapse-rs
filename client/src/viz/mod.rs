//! The widget spine (oracle: `WidgetHost` + `RendererRegistry` + the SVG render families,
//! ADR-S028): one host consumes `VizCases` — whether it came from an authored
//! ` ```viz widget= ` fence or (later) the live-trace adapter — dispatches through the pure
//! `RenderFamily` decision, and drives every animation with the one `Playback` stepper.
//! Layout is computed ONCE over the union of steps; the step signal only toggles drawing.

pub mod blocks;
pub mod host;
pub mod registry;
pub mod render;
pub mod transport;
