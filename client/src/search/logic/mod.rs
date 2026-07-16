//! Pure search over the flattened library (oracle: `LibrarySearch`): every lesson, every book
//! (linked to its first lesson), every blog post — ranked prefix (100) > word-start (80) >
//! substring (60) > subsequence (30), with a +10 bonus for matching the LABEL over the
//! breadcrumb, kind as the tiebreak (lessons first), shorter labels before longer.

use synapse_shared::blog::BlogSummaryDto;
use synapse_shared::catalog::{BookDto, BookEntryDto, CatalogEntryDto, SynapseIndexDto};

use crate::router::page::Page;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Lesson,
    Book,
    Blog,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchEntry {
    pub label: String,
    pub sublabel: String,
    pub kind: Kind,
    pub page: Page,
}

/// Flatten the whole library into searchable entries.
pub fn entries(index: &SynapseIndexDto, blog: &[BlogSummaryDto]) -> Vec<SearchEntry> {
    let mut all = Vec::new();
    flatten_catalog(&index.entries, &[], &mut all);
    all.extend(blog.iter().map(|post| SearchEntry {
        label: post.title.clone(),
        sublabel: "Blog".to_owned(),
        kind: Kind::Blog,
        page: Page::BlogPost(post.slug.clone()),
    }));
    all
}

fn flatten_catalog(entries: &[CatalogEntryDto], crumb: &[String], out: &mut Vec<SearchEntry>) {
    for entry in entries {
        match entry {
            CatalogEntryDto::Category(category) => {
                let mut crumb = crumb.to_vec();
                crumb.push(category.title.clone());
                flatten_catalog(&category.entries, &crumb, out);
            }
            CatalogEntryDto::Book(book) => flatten_book(book, crumb, out),
        }
    }
}

fn flatten_book(book: &BookDto, crumb: &[String], out: &mut Vec<SearchEntry>) {
    // The book itself: one entry linked to its first lesson (depth-first).
    if let Some(first) = first_lesson_path(book) {
        let mut sub = crumb.to_vec();
        sub.push("Book".to_owned());
        out.push(SearchEntry {
            label: book.title.clone(),
            sublabel: sub.join(" › "),
            kind: Kind::Book,
            page: Page::Lesson(first),
        });
    }
    let mut crumb = crumb.to_vec();
    crumb.push(book.title.clone());
    let mut prefix: Vec<String> = book.category_path.clone();
    prefix.push(book.slug.clone());
    flatten_entries(&book.entries, &crumb, &prefix, out);
}

fn flatten_entries(
    entries: &[BookEntryDto],
    crumb: &[String],
    prefix: &[String],
    out: &mut Vec<SearchEntry>,
) {
    for entry in entries {
        match entry {
            BookEntryDto::Chapter(chapter) => {
                let mut crumb = crumb.to_vec();
                crumb.push(chapter.title.clone());
                let mut prefix = prefix.to_vec();
                prefix.push(chapter.slug.clone());
                flatten_entries(&chapter.entries, &crumb, &prefix, out);
            }
            BookEntryDto::Lesson(lesson) => {
                let mut path = prefix.to_vec();
                path.push(lesson.slug.clone());
                out.push(SearchEntry {
                    label: lesson.title.clone(),
                    sublabel: crumb.join(" › "),
                    kind: Kind::Lesson,
                    page: Page::Lesson(path),
                });
            }
        }
    }
}

fn first_lesson_path(book: &BookDto) -> Option<Vec<String>> {
    fn dive(entries: &[BookEntryDto], prefix: &[String]) -> Option<Vec<String>> {
        for entry in entries {
            match entry {
                BookEntryDto::Lesson(lesson) => {
                    let mut path = prefix.to_vec();
                    path.push(lesson.slug.clone());
                    return Some(path);
                }
                BookEntryDto::Chapter(chapter) => {
                    let mut prefix = prefix.to_vec();
                    prefix.push(chapter.slug.clone());
                    if let Some(path) = dive(&chapter.entries, &prefix) {
                        return Some(path);
                    }
                }
            }
        }
        None
    }
    let mut prefix: Vec<String> = book.category_path.clone();
    prefix.push(book.slug.clone());
    dive(&book.entries, &prefix)
}

pub const LIMIT: usize = 20;

/// Rank and cap. An empty query returns everything (capped); a no-match query returns nothing.
pub fn search(query: &str, all: &[SearchEntry]) -> Vec<SearchEntry> {
    let query = query.trim();
    if query.is_empty() {
        return all.iter().take(LIMIT).cloned().collect();
    }
    let mut ranked: Vec<(&SearchEntry, i32)> = all
        .iter()
        .filter_map(|entry| rank(query, entry).map(|score| (entry, score)))
        .collect();
    ranked.sort_by_key(|(entry, score)| (-score, kind_order(entry.kind), entry.label.len()));
    ranked.into_iter().take(LIMIT).map(|(e, _)| e.clone()).collect()
}

fn kind_order(kind: Kind) -> u8 {
    match kind {
        Kind::Lesson => 0,
        Kind::Book => 1,
        Kind::Blog => 2,
    }
}

/// The label carries a +10 bonus over the breadcrumb; the best of the two wins.
fn rank(query: &str, entry: &SearchEntry) -> Option<i32> {
    let on_label = score(query, &entry.label).map(|s| s + 10);
    let on_crumb = score(query, &entry.sublabel);
    on_label.into_iter().chain(on_crumb).max()
}

fn score(query: &str, text: &str) -> Option<i32> {
    let q = query.to_lowercase();
    let t = text.to_lowercase();
    if t.starts_with(&q) {
        Some(100)
    } else if t
        .split(|c: char| !c.is_ascii_alphanumeric())
        .any(|word| word.starts_with(&q))
    {
        Some(80)
    } else if t.contains(&q) {
        Some(60)
    } else if is_subsequence(&q, &t) {
        Some(30)
    } else {
        None
    }
}

fn is_subsequence(query: &str, text: &str) -> bool {
    let mut chars = query.chars();
    let mut want = chars.next();
    for c in text.chars() {
        if Some(c) == want {
            want = chars.next();
        }
    }
    want.is_none()
}

#[cfg(test)]
#[path = "logic_tests.rs"]
mod tests;
