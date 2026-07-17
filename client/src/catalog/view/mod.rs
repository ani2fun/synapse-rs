//! The catalog views (the view layer): the library (browse) and the reader (lesson + sidebar).
//! Markdown crosses the island bridge; everything else is signals → DOM.

mod c4;
mod c4_docs;
pub mod chrome;
mod diagrams;
mod library;
mod prefs;
mod problem;
mod reader;
pub mod sidebar;
mod tour;

pub use library::LibraryPage;
pub use prefs::ReaderPrefsFab;
pub use reader::LessonPage;
