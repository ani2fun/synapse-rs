//! Lesson payloads — the typed frontmatter and the assembled lesson the service hands to the
//! HTTP layer.

use synapse_shared::execution::TestSpec;

use crate::catalog::domain::catalog::{Book, Lesson};

/// Typed frontmatter; `title` always resolves (fence → first H1 → humanized filename).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LessonFrontmatter {
    pub title: String,
    pub summary: Option<String>,
    pub essential: Option<bool>,
    pub kind: Option<String>,
    pub difficulty: Option<String>,
    pub topics: Option<Vec<String>>,
}

/// A parsed lesson source: frontmatter + the body with the fence stripped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parsed {
    pub frontmatter: LessonFrontmatter,
    pub body: String,
}

/// The assembled lesson. `prev_path`/`next_path` are IN-BOOK slug-paths here — the HTTP layer
/// prepends `categoryPath + bookSlug` to make the wire's full paths. `editorial` joins for
/// `kind: problem` lessons with a `.editorial.md` sidecar. `sample_tests` carries the
/// browser-visible SAMPLE cases from the `.tests.json` sidecar (hidden judge cases excluded);
/// the workbench reads them from here now that the description markdown holds no `testcases` fence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LessonContent {
    pub book: Book,
    pub lesson: Lesson,
    pub frontmatter: LessonFrontmatter,
    pub raw: String,
    pub prev_path: Option<String>,
    pub next_path: Option<String>,
    pub editorial: Option<String>,
    pub sample_tests: Option<TestSpec>,
}
