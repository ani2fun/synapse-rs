//! FFI externs the viz surfaces need. The crate's wasm-bindgen glue imports `@editor/loader` /
//! `@tracer/loader`, resolved by the Astro app's Vite alias config — so the extern declarations
//! travel with the crate instead of reaching back into a host crate.

pub mod editor;
pub mod tracer;
