//! Pure catalog navigation over the WIRE DTOs (the logic layer: no leptos, no web-sys — the
//! purity gate greps it; plain `cargo test` covers it natively).

pub mod prefs;

use synapse_shared::catalog::{BookDto, BookEntryDto, CatalogEntryDto, LessonDto, SynapseIndexDto};

/// A book's URL prefix segments: `categoryPath + slug`.
pub fn book_prefix(book: &BookDto) -> Vec<String> {
    let mut segments = book.category_path.clone();
    segments.push(book.slug.clone());
    segments
}

/// Every lesson of a book with its FULL directory-mirror path, pre-order — the sidebar's and
/// the library card's source of truth.
pub fn reading_order(book: &BookDto) -> Vec<(String, LessonDto)> {
    fn collect(entries: &[BookEntryDto], prefix: &[String], out: &mut Vec<(String, LessonDto)>) {
        for entry in entries {
            match entry {
                BookEntryDto::Lesson(lesson) => {
                    let mut segments = prefix.to_vec();
                    segments.push(lesson.slug.clone());
                    out.push((segments.join("/"), lesson.clone()));
                }
                BookEntryDto::Chapter(chapter) => {
                    let mut segments = prefix.to_vec();
                    segments.push(chapter.slug.clone());
                    collect(&chapter.entries, &segments, out);
                }
            }
        }
    }
    let mut out = Vec::new();
    collect(&book.entries, &book_prefix(book), &mut out);
    out
}

/// Where a book's cover card points: its first lesson in reading order.
pub fn first_lesson_path(book: &BookDto) -> Option<String> {
    reading_order(book).into_iter().next().map(|(path, _)| path)
}

/// The book a lesson path belongs to: the entry whose `categoryPath + slug` prefixes the path.
pub fn book_of<'a>(index: &'a SynapseIndexDto, lesson_path: &[String]) -> Option<&'a BookDto> {
    fn find<'a>(entries: &'a [CatalogEntryDto], path: &[String]) -> Option<&'a BookDto> {
        let (first, rest) = path.split_first()?;
        entries.iter().find_map(|entry| match entry {
            CatalogEntryDto::Book(book) if book.slug == *first => Some(book),
            CatalogEntryDto::Category(category) if category.slug == *first => find(&category.entries, rest),
            _ => None,
        })
    }
    find(&index.entries, lesson_path)
}

/// The book with a globally-unique slug, DFS through categories (oracle: `CatalogNav.findBook`).
pub fn find_book<'a>(index: &'a SynapseIndexDto, slug: &str) -> Option<&'a BookDto> {
    fn dfs<'a>(entries: &'a [CatalogEntryDto], slug: &str) -> Option<&'a BookDto> {
        entries.iter().find_map(|entry| match entry {
            CatalogEntryDto::Book(book) if book.slug == slug => Some(book),
            CatalogEntryDto::Book(_) => None,
            CatalogEntryDto::Category(category) => dfs(&category.entries, slug),
        })
    }
    dfs(&index.entries, slug)
}

/// Recursive lesson-leaf count — the card's "N lessons" line.
pub fn lesson_count(book: &BookDto) -> usize {
    reading_order(book).len()
}

/// DIRECT chapter children only (the oracle counts top-level chapters on the card).
pub fn chapter_count(book: &BookDto) -> usize {
    book.entries
        .iter()
        .filter(|entry| matches!(entry, BookEntryDto::Chapter(_)))
        .count()
}

/// One hop of a click's composed path, target-first: `(tag_name, class_attr, data_id)`.
pub type C4PathHop = (String, String, Option<String>);

/// Resolve a click inside the LikeC4 viewer to an element FQN (oracle: `C4NodeResolver`).
/// Walking target-first: a `<button>` BEFORE the node is one of LikeC4's own controls
/// (relationships/details) — let the viewer keep it. A node must carry the EXACT
/// `react-flow__node` class token (edges carry random-hash ids but not the token) and a
/// non-empty `data-id` — the dotted element FQN.
pub fn resolve_c4_node(path: &[C4PathHop]) -> Option<String> {
    for (tag, classes, data_id) in path {
        if tag.eq_ignore_ascii_case("button") {
            return None;
        }
        let is_node = classes.split_whitespace().any(|c| c == "react-flow__node");
        if is_node {
            return data_id.clone().filter(|id| !id.is_empty());
        }
    }
    None
}

#[cfg(test)]
#[path = "logic_tests.rs"]
mod tests;
