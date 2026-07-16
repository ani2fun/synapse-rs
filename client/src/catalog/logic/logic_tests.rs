//! Native tests for the pure catalog logic.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;
use synapse_shared::catalog::{CategoryDto, ChapterDto};

fn lesson(slug: &str) -> BookEntryDto {
    BookEntryDto::Lesson(LessonDto {
        slug: slug.to_owned(),
        title: slug.to_owned(),
        order: None,
        essential: true,
    })
}

fn book() -> BookDto {
    BookDto {
        slug: "dsa".to_owned(),
        title: "DSA".to_owned(),
        description: String::new(),
        tags: vec![],
        estimated_reading_minutes: None,
        order: None,
        category_path: vec!["learn".to_owned()],
        entries: vec![
            lesson("intro"),
            BookEntryDto::Chapter(ChapterDto {
                slug: "lists".to_owned(),
                title: "Lists".to_owned(),
                order: None,
                entries: vec![lesson("singly")],
            }),
        ],
    }
}

fn index() -> SynapseIndexDto {
    SynapseIndexDto {
        entries: vec![CatalogEntryDto::Category(CategoryDto {
            slug: "learn".to_owned(),
            title: "Learn".to_owned(),
            description: None,
            icon: None,
            order: None,
            entries: vec![CatalogEntryDto::Book(book())],
        })],
    }
}

#[test]
fn reading_order_is_preorder_with_full_paths() {
    let paths: Vec<String> = reading_order(&book()).into_iter().map(|(p, _)| p).collect();
    assert_eq!(paths, vec!["learn/dsa/intro", "learn/dsa/lists/singly"]);
}

#[test]
fn first_lesson_path_is_the_cover_target() {
    assert_eq!(first_lesson_path(&book()).unwrap(), "learn/dsa/intro");
}

#[test]
fn book_of_descends_categories_to_the_owning_book() {
    let idx = index();
    let path: Vec<String> = ["learn", "dsa", "lists", "singly"]
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
    assert_eq!(book_of(&idx, &path).unwrap().slug, "dsa");
    let missing: Vec<String> = ["learn", "nope", "x"].iter().map(|s| (*s).to_owned()).collect();
    assert!(book_of(&idx, &missing).is_none());
}

#[test]
fn find_book_descends_categories_by_globally_unique_slug() {
    let idx = index();
    assert_eq!(find_book(&idx, "dsa").unwrap().title, "DSA");
    assert!(find_book(&idx, "nope").is_none());
}

#[test]
fn card_counts_lessons_recursively_and_chapters_directly() {
    let b = book();
    assert_eq!(lesson_count(&b), 2);
    assert_eq!(chapter_count(&b), 1);
}

fn hop(tag: &str, classes: &str, id: Option<&str>) -> C4PathHop {
    (tag.to_owned(), classes.to_owned(), id.map(str::to_owned))
}

#[test]
fn a_node_body_click_resolves_to_its_dotted_fqn() {
    let path = vec![
        hop("DIV", "likec4-element", None),
        hop(
            "DIV",
            "react-flow__node react-flow__node-element",
            Some("btPersonal.btSmallWeb"),
        ),
        hop("DIV", "react-flow__pane", None),
    ];
    assert_eq!(resolve_c4_node(&path).as_deref(), Some("btPersonal.btSmallWeb"));
}

#[test]
fn a_button_before_the_node_is_likec4s_own_control() {
    let path = vec![
        hop("BUTTON", "mantine-ActionIcon-root", None),
        hop("DIV", "react-flow__node", Some("sfClient")),
    ];
    assert_eq!(resolve_c4_node(&path), None);
}

#[test]
fn edges_and_token_substrings_never_resolve() {
    let edge = vec![hop("G", "react-flow__edge", Some("hash-1a2b"))];
    assert_eq!(resolve_c4_node(&edge), None);
    let substring = vec![hop("DIV", "react-flow__node-toolbar", Some("x"))];
    assert_eq!(resolve_c4_node(&substring), None);
    let empty_id = vec![hop("DIV", "react-flow__node", Some(""))];
    assert_eq!(resolve_c4_node(&empty_id), None);
}
