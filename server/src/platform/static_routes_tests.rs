//! Head rendering (step 50). These are the pure half — the substitution and escaping — so they
//! run natively and fast. The wiring (does a real lesson URL get a real title?) is
//! `server/tests/static_routes_it.rs`, which drives the whole router.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;

const INDEX: &str = "<!doctype html>\n<html lang=\"en\">\n  <head>\n    <meta charset=\"utf-8\" />\n    <title>Synapse</title>\n  </head>\n  <body></body>\n</html>\n";

#[test]
fn the_title_is_replaced_not_appended() {
    let out = render_head(INDEX, "Lists · Singly — Synapse", "About lists.", "/x");
    assert!(out.contains("<title>Lists · Singly — Synapse</title>"));
    assert!(
        !out.contains("<title>Synapse</title>"),
        "the ORIGINAL title must be gone — a second <title> is ignored by every consumer, so \
         appending would have looked like it worked"
    );
    assert_eq!(out.matches("<title>").count(), 1);
}

#[test]
fn the_tags_land_inside_the_head() {
    let out = render_head(INDEX, "T", "D", "https://example.test/x");
    let head_close = out.find("</head>").unwrap();
    for needle in [
        "name=\"description\" content=\"D\"",
        "rel=\"canonical\" href=\"https://example.test/x\"",
        "property=\"og:title\" content=\"T\"",
        "property=\"og:url\" content=\"https://example.test/x\"",
        "name=\"twitter:card\" content=\"summary\"",
    ] {
        let at = out.find(needle).unwrap_or_else(|| panic!("missing {needle}"));
        assert!(at < head_close, "{needle} must be inside <head>");
    }
    assert_eq!(
        out.matches("</head>").count(),
        1,
        "the close tag is not duplicated"
    );
}

#[test]
fn authored_content_cannot_break_out_of_an_attribute() {
    // A summary is authored prose. A stray quote is a typo; unescaped it is an injection.
    let nasty = "Quote \" and <script>alert('x')</script> & an ampersand";
    let out = render_head(INDEX, nasty, nasty, "/x");
    assert!(!out.contains("<script>"), "no raw tag survives");
    assert!(
        !out.contains("content=\"Quote \" and"),
        "the quote must not terminate the attribute early"
    );
    assert!(out.contains("&quot;"));
    assert!(out.contains("&lt;script&gt;"));
    assert!(out.contains("&amp;"));
}

#[test]
fn an_index_without_a_title_tag_is_left_alone_but_still_gets_its_tags() {
    let no_title = "<html><head><meta charset=\"utf-8\" /></head><body></body></html>";
    let out = render_head(no_title, "T", "D", "/x");
    assert!(
        !out.contains("<title>"),
        "nothing to replace, so nothing is invented"
    );
    assert!(
        out.contains("og:title"),
        "the injected tags do not depend on the title element"
    );
}

#[test]
fn a_malformed_title_element_degrades_instead_of_corrupting() {
    // An unclosed <title> would make a naive slice run to the end of the document.
    let broken = "<html><head><title>Synapse</head><body></body></html>";
    assert_eq!(
        replace_title(broken, "T"),
        broken,
        "no close tag means no substitution — never a truncated document"
    );
}

#[test]
fn the_book_leads_the_title() {
    let meta = PageMeta {
        title: "Singly linked lists".to_owned(),
        description: None,
        book_title: "Data structures".to_owned(),
    };
    assert_eq!(
        title_for(&meta),
        "Data structures · Singly linked lists — Synapse",
        "the left of the string is what survives truncation in a tab strip and a search result"
    );
}
