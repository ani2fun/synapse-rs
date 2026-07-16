//! The ⌘K library search (oracle: `LibrarySearch` + `SearchPalette`, step 19) — entirely
//! client-side over the already-cached catalog index + blog listing. `logic/` is pure
//! (native-tested); `view/` is the palette.

pub mod logic;
pub mod state;
pub mod view;
