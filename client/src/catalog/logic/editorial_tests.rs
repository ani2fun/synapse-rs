//! Native tests for the editorial document model. The fixtures mirror the REAL authored
//! content in synapse-content/dsa: the flat single-approach format, the multi-approach
//! format (`## Brute / ## Optimal …` with `###` subsections incl. `### Edge Case`), and the
//! degradation shapes (arbitrary headings, plain fences, empty files).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use super::*;

const FLAT: &str = "\
## Intuition

Iterate and test each candidate.

## Approach

1. Initialize a list.
2. Iterate from 1 to n.

## Solution

```python solution time=O(N^2) space=O(1)
# Iterate from 1 to n
print(1)
```

```java solution time=O(N^2) space=O(1)
class Main {}
```

## Complexity Analysis

**Time Complexity:** O(N^2). The overall complexity will be O(N^2), where N is the number of rows.

**Space Complexity:** O(1) – Using a couple of variables i.e., constant space.
";

const MULTI: &str = "\
## Brute

### Intuition

Check every value.

### Approach

1. Iterate from 1 to n.

### Edge Case

Watch out for 1.

### Solution

```python solution time=O(N) space=O(K)
# a comment, not a heading
print(1)
```

### Complexity Analysis

**Time Complexity:** O(N) – Iterating N times.

**Space Complexity:** O(K), where K is the number of divisors.

## Optimal

### Intuition

Pairs mirror around the square root.

### Approach

1. Iterate to sqrt(n).

### Solution

```python solution time=O(sqrt(N)+K*log(K)) space=O(sqrt(N))
print(2)
```

### Complexity Analysis

**Time Complexity:** O(sqrt(N)) + O(K*log(K)) – Gather then sort.

**Space Complexity:** O(sqrt(N)) – At most 2*sqrt(N) divisors.
";

fn labels(sections: &[SectionDoc]) -> Vec<&str> {
    sections.iter().map(|s| s.label.as_str()).collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// PARSE — the two authored formats
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn the_flat_format_is_one_approach_with_canonical_sections() {
    let doc = parse_editorial(FLAT);
    assert!(!doc.multi);
    assert_eq!(doc.preamble, "");
    assert_eq!(doc.approaches.len(), 1);
    let approach = &doc.approaches[0];
    assert_eq!(approach.label, "");
    assert_eq!(
        labels(&approach.sections),
        ["Intuition", "Approach", "Solution", "Complexity Analysis"]
    );
    assert_eq!(
        approach.sections.iter().map(|s| s.kind).collect::<Vec<_>>(),
        [
            SectionKind::Intuition,
            SectionKind::Approach,
            SectionKind::Solution,
            SectionKind::Complexity
        ]
    );
    // Bodies carry no heading line, and the fence bodies stay intact.
    assert!(approach.sections[0].md.starts_with("Iterate and test"));
    assert!(approach.sections[2].md.contains("```python solution"));
    assert_eq!(approach.time.as_deref(), Some("O(N^2)"));
    assert_eq!(approach.space.as_deref(), Some("O(1)"));
}

#[test]
fn the_multi_format_is_detected_and_split_per_approach() {
    let doc = parse_editorial(MULTI);
    assert!(doc.multi);
    assert_eq!(doc.approaches.len(), 2);
    assert_eq!(doc.approaches[0].label, "Brute");
    assert_eq!(doc.approaches[1].label, "Optimal");
    assert_eq!(
        labels(&doc.approaches[0].sections),
        [
            "Intuition",
            "Approach",
            "Edge Case",
            "Solution",
            "Complexity Analysis"
        ]
    );
    assert_eq!(doc.approaches[0].sections[2].kind, SectionKind::Other);
    assert_eq!(doc.approaches[0].time.as_deref(), Some("O(N)"));
    assert_eq!(doc.approaches[0].space.as_deref(), Some("O(K)"));
    assert_eq!(doc.approaches[1].time.as_deref(), Some("O(sqrt(N)+K*log(K))"));
    assert_eq!(doc.approaches[1].space.as_deref(), Some("O(sqrt(N))"));
}

#[test]
fn prose_before_the_first_heading_is_the_preamble() {
    let doc = parse_editorial(&format!("A word up front.\n\n{FLAT}"));
    assert_eq!(doc.preamble, "A word up front.");
    // In the multi format, prose under an approach heading BEFORE its first `###`
    // becomes an unlabeled leading section of that approach.
    let doc = parse_editorial(&MULTI.replace(
        "## Brute\n\n### Intuition",
        "## Brute\n\nA framing line.\n\n### Intuition",
    ));
    assert_eq!(doc.approaches[0].sections[0].label, "");
    assert_eq!(doc.approaches[0].sections[0].md, "A framing line.");
    assert_eq!(doc.approaches[0].sections[1].label, "Intuition");
}

#[test]
fn two_sections_without_canonical_subheadings_stay_single() {
    // Arbitrary legacy headings: sectioned, but no stepper.
    let doc = parse_editorial("## Walkthrough\n\nProse.\n\n## Proof\n\nMore prose.");
    assert!(!doc.multi);
    assert_eq!(doc.approaches.len(), 1);
    assert_eq!(labels(&doc.approaches[0].sections), ["Walkthrough", "Proof"]);
    assert!(
        doc.approaches[0]
            .sections
            .iter()
            .all(|s| s.kind == SectionKind::Other)
    );
}

#[test]
fn headings_inside_code_fences_are_content() {
    let md = "## Solution\n\n```python solution time=O(N) space=O(1)\n## not a heading\n### also not\nprint(1)\n```\n";
    let doc = parse_editorial(md);
    assert_eq!(doc.approaches[0].sections.len(), 1);
    assert!(doc.approaches[0].sections[0].md.contains("## not a heading"));
}

#[test]
fn a_single_h1_document_falls_back_to_h1_sections() {
    let doc = parse_editorial("# Idea\n\nProse.\n\n# Code\n\nMore.");
    assert_eq!(labels(&doc.approaches[0].sections), ["Idea", "Code"]);
}

#[test]
fn a_headingless_or_empty_document_degrades() {
    let doc = parse_editorial("Just prose, nothing else.");
    assert!(!doc.multi);
    assert_eq!(doc.preamble, "Just prose, nothing else.");
    assert_eq!(doc.approaches.len(), 1);
    assert!(doc.approaches[0].sections.is_empty());

    let empty = parse_editorial("   \n  ");
    assert!(empty.approaches.is_empty());
    assert_eq!(empty.preamble, "");
}

// ─────────────────────────────────────────────────────────────────────────────
// SPOILER WRAPPER — the inline editorial arrives wearing `<details>`
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn the_outer_spoiler_wrapper_comes_off() {
    let wrapped = format!("<details>\n<summary>Editorial</summary>\n\n{FLAT}\n</details>");
    let doc = parse_editorial(&wrapped);
    assert_eq!(doc.approaches[0].sections.len(), 4);
    assert!(!doc.preamble.contains("<summary"));
    for section in &doc.approaches[0].sections {
        assert!(!section.md.contains("</details>"));
    }
}

#[test]
fn a_nested_details_inside_a_section_survives() {
    let wrapped = "<details>\n<summary>Editorial</summary>\n\n## Intuition\n\n<details>\n<summary>Hint</summary>\nA hint.\n</details>\n\n## Approach\n\nSteps.\n</details>";
    let doc = parse_editorial(wrapped);
    let intuition = &doc.approaches[0].sections[0];
    assert!(intuition.md.contains("<summary>Hint</summary>"));
    assert!(intuition.md.contains("<details>"));
    // The nested closer stays; only the OUTER closer (the last one) was removed.
    assert!(intuition.md.contains("</details>"));
    assert!(!doc.approaches[0].sections[1].md.contains("</details>"));
}

// ─────────────────────────────────────────────────────────────────────────────
// SOLUTION SYNTHESIS — fences without a Solution heading
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn fences_without_a_solution_heading_synthesize_one() {
    let md = "## Brute\n\n### Intuition\n\nIdea.\n\n### Approach\n\n1. Steps.\n\n```python solution time=O(N) space=O(1)\nprint(1)\n```\n\n### Complexity Analysis\n\n**Time Complexity:** O(N) – Linear.\n\n## Optimal\n\n### Intuition\n\nBetter idea.\n\n### Approach\n\n```python solution time=O(1) space=O(1)\nprint(2)\n```\n";
    let doc = parse_editorial(md);
    assert!(doc.multi);
    assert_eq!(
        labels(&doc.approaches[0].sections),
        ["Intuition", "Approach", "Solution", "Complexity Analysis"]
    );
    assert_eq!(doc.approaches[0].sections[1].md, "1. Steps.");
    assert!(doc.approaches[0].sections[2].md.starts_with("```python solution"));
    // The second approach's fence was the WHOLE Approach body — the section becomes
    // the Solution outright rather than leaving an empty husk behind.
    assert_eq!(labels(&doc.approaches[1].sections), ["Intuition", "Solution"]);
}

#[test]
fn an_explicit_solution_heading_suppresses_synthesis() {
    let doc = parse_editorial(FLAT);
    let solutions = doc.approaches[0]
        .sections
        .iter()
        .filter(|s| s.kind == SectionKind::Solution)
        .count();
    assert_eq!(solutions, 1);
}

#[test]
fn plain_fences_trigger_neither_synthesis_nor_claims() {
    let md = "## Solution steps\n\n```python\nprint(1)\n```\n";
    let doc = parse_editorial(md);
    assert_eq!(labels(&doc.approaches[0].sections), ["Solution steps"]);
    assert_eq!(doc.approaches[0].time, None);
    assert_eq!(doc.approaches[0].space, None);
}

// ─────────────────────────────────────────────────────────────────────────────
// KINDS AND CLAIMS
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn section_kinds_match_however_they_were_typed() {
    assert_eq!(section_kind("Intuition"), SectionKind::Intuition);
    assert_eq!(section_kind("  APPROACH "), SectionKind::Approach);
    assert_eq!(section_kind("Solution"), SectionKind::Solution);
    assert_eq!(section_kind("Solutions"), SectionKind::Solution);
    assert_eq!(section_kind("Code"), SectionKind::Solution);
    assert_eq!(section_kind("Complexity"), SectionKind::Complexity);
    assert_eq!(section_kind("Complexity  Analysis"), SectionKind::Complexity);
    assert_eq!(section_kind("Edge Case"), SectionKind::Other);
    assert_eq!(section_kind(""), SectionKind::Other);
}

#[test]
fn the_first_solution_meta_wins_and_spaced_values_survive() {
    let md = "prose\n\n```python solution time=O(log(min(N1, N2))) space=O(1)\nprint(1)\n```\n\n```java solution time=O(N) space=O(N)\nx\n```";
    assert_eq!(
        first_solution_meta(md).as_deref(),
        Some("python solution time=O(log(min(N1, N2))) space=O(1)")
    );
    let doc = parse_editorial(&format!("## Solution\n\n{md}"));
    assert_eq!(doc.approaches[0].time.as_deref(), Some("O(log(min(N1, N2)))"));
    assert_eq!(first_solution_meta("```python\nprint(1)\n```"), None);
}

// ─────────────────────────────────────────────────────────────────────────────
// COMPLEXITY PROSE — every separator the real content uses
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn complexity_prose_reads_the_dash_form() {
    let parsed = complexity_prose(
        "**Time Complexity:** O(N) – Iterating N times.\n\n**Space Complexity:** O(1) – Constant space.",
    )
    .unwrap();
    assert_eq!(
        parsed.time,
        Some(("O(N)".to_owned(), "Iterating N times.".to_owned()))
    );
    assert_eq!(
        parsed.space,
        Some(("O(1)".to_owned(), "Constant space.".to_owned()))
    );
}

#[test]
fn complexity_prose_reads_period_and_comma_forms() {
    // Both shapes verified in the real content.
    let parsed = complexity_prose(
        "**Time Complexity:** O(N^2). The overall complexity will be O(N^2).\n\n**Space Complexity:** O(K), where K is the number of divisors.",
    )
    .unwrap();
    assert_eq!(
        parsed.time,
        Some((
            "O(N^2)".to_owned(),
            "The overall complexity will be O(N^2).".to_owned()
        ))
    );
    assert_eq!(
        parsed.space,
        Some(("O(K)".to_owned(), "where K is the number of divisors.".to_owned()))
    );
}

#[test]
fn a_compound_value_runs_to_the_real_separator() {
    let parsed =
        complexity_prose("**Time Complexity:** O(sqrt(N)) + O(K*Log(K)) – Gather then sort.").unwrap();
    assert_eq!(
        parsed.time,
        Some((
            "O(sqrt(N)) + O(K*Log(K))".to_owned(),
            "Gather then sort.".to_owned()
        ))
    );
    // Separator characters INSIDE parentheses never split the value.
    let parsed = complexity_prose("**Time Complexity:** O(min(N1, N2)) – Bounded by the smaller.").unwrap();
    assert_eq!(
        parsed.time,
        Some(("O(min(N1, N2))".to_owned(), "Bounded by the smaller.".to_owned()))
    );
}

#[test]
fn a_wrapped_paragraph_joins_before_parsing() {
    let parsed = complexity_prose("**Time Complexity:** O(N)\n– Split across\nlines.").unwrap();
    assert_eq!(
        parsed.time,
        Some(("O(N)".to_owned(), "Split across lines.".to_owned()))
    );
}

#[test]
fn missing_axes_and_garbage_degrade() {
    let only_time = complexity_prose("**Time Complexity:** O(N) – Linear.").unwrap();
    assert!(only_time.space.is_none());
    assert!(complexity_prose("No markers here at all.").is_none());
    // A marker without any O-group is not card material.
    assert!(complexity_prose("**Time Complexity:** basically fast").is_none());
    // Markers inside fences are code, not claims.
    assert!(complexity_prose("```\n**Time Complexity:** O(N) – nope\n```").is_none());
}

// ─────────────────────────────────────────────────────────────────────────────
// DISPLAY HELPERS
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn pretty_o_matches_the_design_rendering() {
    assert_eq!(pretty_o("O(sqrt(N))"), "O(√N)");
    assert_eq!(pretty_o("O(sqrt(N)+K*log(K))"), "O(√N+K·log(K))");
    assert_eq!(pretty_o("O(N^2)"), "O(N²)");
    assert_eq!(pretty_o("O(N^1.5)"), "O(N^1.5)");
    assert_eq!(pretty_o("O(log N)"), "O(log N)");
    assert_eq!(pretty_o("O(N)"), "O(N)");
    // An unclosed sqrt( is passed through rather than mangled.
    assert_eq!(pretty_o("O(sqrt(N"), "O(sqrt(N");
}

#[test]
fn the_scroll_spy_picks_the_last_section_past_the_threshold() {
    assert_eq!(active_section(&[], 84.0), 0);
    assert_eq!(active_section(&[200.0, 400.0], 84.0), 0);
    assert_eq!(active_section(&[-300.0, 20.0, 400.0], 84.0), 1);
    assert_eq!(active_section(&[-300.0, -20.0, 60.0], 84.0), 2);
}

#[test]
fn approach_restore_reuses_the_section_matcher() {
    let labels = ["Brute".to_owned(), "Optimal".to_owned()];
    assert_eq!(super::super::pane::section_index(&labels, "optimal"), 1);
    assert_eq!(super::super::pane::section_index(&labels, "gone"), 0);
}
