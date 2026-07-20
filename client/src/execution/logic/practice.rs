//! The embedded practice-problem decode (oracle: `PracticeBlocks.scala`,
//! docs/embedded-practice-problems.md; grown here with APPROACH TABS). Pure: URI-decoded
//! attribute strings in, a `PracticeSpec` out — the workbench half reuses the SAME
//! `parse_variants`/`TestSpec` decode the plain workbench placeholders use. A malformed
//! practice problem (no variants, blank statement) decodes to `None` and silently
//! disappears — it never crashes the reader.

use serde::Deserialize;
use synapse_shared::execution::TestSpec;

use super::blocks::{Variant, parse_variants};

/// One editorial approach: the tab label ("Brute Force" · "Optimal" · "Editorial") + its
/// markdown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Approach {
    pub label: String,
    pub md: String,
}

/// One authored practice problem: the statement, the starter workbench, the editorials.
#[derive(Debug, Clone, PartialEq)]
pub struct PracticeSpec {
    pub problem_md: String,
    pub variants: Vec<Variant>,
    pub spec: Option<TestSpec>,
    pub editorials: Vec<Approach>,
}

/// The wire shape of one `data-editorials` entry.
#[derive(Debug, Deserialize)]
struct EditorialWire {
    #[serde(default)]
    tag: String,
    #[serde(default)]
    md: String,
}

/// Decoded attribute payloads → the spec. `None` when the statement is blank or no variant
/// survives (`parse_variants` already drops blank-language entries).
#[must_use]
pub fn decode_practice(
    problem_md: &str,
    variants_json: &str,
    spec_json: Option<&str>,
    editorials_json: Option<&str>,
) -> Option<PracticeSpec> {
    let problem_md = problem_md.trim();
    if problem_md.is_empty() {
        return None;
    }
    let variants = parse_variants(variants_json).filter(|v| !v.is_empty())?;
    let spec = spec_json.and_then(|json| serde_json::from_str::<TestSpec>(json).ok());
    let wire: Vec<EditorialWire> = editorials_json
        .and_then(|json| serde_json::from_str(json).ok())
        .unwrap_or_default();
    Some(PracticeSpec {
        problem_md: problem_md.to_owned(),
        variants,
        spec,
        editorials: label_approaches(&wire),
    })
}

/// `approach-brute-force-1` → "Brute Force" (numbered only when the same kind repeats);
/// a bare/unrecognised tag → "Editorial". Order is authoring order.
fn label_approaches(wire: &[EditorialWire]) -> Vec<Approach> {
    let kinds: Vec<String> = wire.iter().map(|e| approach_kind(&e.tag)).collect();
    let mut counters: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    wire.iter()
        .zip(&kinds)
        .filter(|(e, _)| !e.md.trim().is_empty())
        .map(|(e, kind)| {
            let repeats = kinds.iter().filter(|k| *k == kind).count() > 1;
            let label = if repeats {
                let n = counters.entry(kind.as_str()).or_insert(0);
                *n += 1;
                format!("{kind} {n}")
            } else {
                kind.clone()
            };
            Approach {
                label,
                md: e.md.trim().to_owned(),
            }
        })
        .collect()
}

/// The human kind behind a tag: strip `approach-`, strip a trailing `-<n>`, title-case.
fn approach_kind(tag: &str) -> String {
    let Some(kind) = tag.strip_prefix("approach-") else {
        return "Editorial".to_owned();
    };
    let kind = kind
        .rsplit_once('-')
        .filter(|(_, n)| n.chars().all(|c| c.is_ascii_digit()))
        .map_or(kind, |(head, _)| head);
    let words: Vec<String> = kind
        .split('-')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            chars.next().map_or_else(String::new, |first| {
                first.to_uppercase().collect::<String>() + chars.as_str()
            })
        })
        .collect();
    if words.is_empty() {
        "Editorial".to_owned()
    } else {
        words.join(" ")
    }
}

/// A solution fence's meta carries `time=O(…) space=O(…)` claims, extracted here. A value
/// may contain spaces (`time=O(log N)`, `time=O(min(N1, N2))`) — following tokens are pulled
/// in until its parentheses balance, so whitespace inside the O-group never truncates it.
#[must_use]
pub fn solution_complexities(meta: &str) -> Vec<(String, String)> {
    fn depth_delta(s: &str) -> i32 {
        s.chars().fold(0, |depth, c| match c {
            '(' => depth + 1,
            ')' => depth - 1,
            _ => depth,
        })
    }
    let mut out = Vec::new();
    let mut tokens = meta.split_whitespace();
    while let Some(token) = tokens.next() {
        let Some((name, value)) = token.split_once('=') else {
            continue;
        };
        if name != "time" && name != "space" {
            continue;
        }
        let mut value = value.to_owned();
        let mut depth = depth_delta(&value);
        while depth > 0 {
            let Some(next) = tokens.next() else { break };
            value.push(' ');
            value.push_str(next);
            depth += depth_delta(next);
        }
        out.push((name.to_owned(), value));
    }
    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    const VARIANTS: &str = r#"[{"lang":"python","source":"print(1)"}]"#;

    #[test]
    fn decodes_a_full_practice_problem() {
        let spec = decode_practice(
            "State the problem.",
            VARIANTS,
            Some(r#"{"args":[{"id":"n","label":"n","type":"int"}],"cases":[{"args":{"n":"3"}}]}"#),
            Some(r#"[{"tag":"","md":"The editorial."}]"#),
        )
        .unwrap();
        assert_eq!(spec.variants.len(), 1);
        assert!(spec.spec.is_some());
        assert_eq!(spec.editorials.len(), 1);
        assert_eq!(spec.editorials[0].label, "Editorial");
        assert_eq!(spec.editorials[0].md, "The editorial.");
    }

    #[test]
    fn a_blank_statement_or_empty_variants_reads_as_no_widget() {
        assert!(decode_practice("  ", VARIANTS, None, None).is_none());
        assert!(decode_practice("Statement", "[]", None, None).is_none());
        assert!(decode_practice("Statement", "not json", None, None).is_none());
    }

    #[test]
    fn a_malformed_spec_degrades_to_no_tests_and_blank_editorials_drop() {
        let spec = decode_practice(
            "Statement",
            VARIANTS,
            Some("not json"),
            Some(r#"[{"tag":"approach-optimal-1","md":"   "}]"#),
        )
        .unwrap();
        assert!(spec.spec.is_none());
        assert!(spec.editorials.is_empty());
    }

    #[test]
    fn approach_tags_become_titled_tabs_in_authoring_order() {
        let spec = decode_practice(
            "Statement",
            VARIANTS,
            None,
            Some(
                r#"[{"tag":"approach-brute-force-1","md":"Try all."},
                    {"tag":"approach-optimal-1","md":"Two pointers."}]"#,
            ),
        )
        .unwrap();
        let labels: Vec<&str> = spec.editorials.iter().map(|a| a.label.as_str()).collect();
        assert_eq!(labels, ["Brute Force", "Optimal"]);
    }

    #[test]
    fn a_repeated_kind_numbers_its_tabs() {
        let spec = decode_practice(
            "Statement",
            VARIANTS,
            None,
            Some(
                r#"[{"tag":"approach-brute-force-1","md":"A."},
                    {"tag":"approach-brute-force-2","md":"B."},
                    {"tag":"approach-optimal-1","md":"C."}]"#,
            ),
        )
        .unwrap();
        let labels: Vec<&str> = spec.editorials.iter().map(|a| a.label.as_str()).collect();
        assert_eq!(labels, ["Brute Force 1", "Brute Force 2", "Optimal"]);
    }

    #[test]
    fn extracts_time_and_space_claims_from_a_solution_meta() {
        assert_eq!(
            solution_complexities("solution time=O(n) space=O(1)"),
            vec![
                ("time".to_owned(), "O(n)".to_owned()),
                ("space".to_owned(), "O(1)".to_owned())
            ]
        );
        assert!(solution_complexities("solution").is_empty());
    }

    /// Real authored metas put spaces INSIDE the O-group — the value runs until its
    /// parentheses balance, never to the first space.
    #[test]
    fn a_spaced_complexity_value_survives_whole() {
        assert_eq!(
            solution_complexities("solution time=O(log N) space=O(1)"),
            vec![
                ("time".to_owned(), "O(log N)".to_owned()),
                ("space".to_owned(), "O(1)".to_owned())
            ]
        );
        assert_eq!(
            solution_complexities("solution time=O(log(min(N1, N2))) space=O(min(N1, N2))"),
            vec![
                ("time".to_owned(), "O(log(min(N1, N2)))".to_owned()),
                ("space".to_owned(), "O(min(N1, N2))".to_owned())
            ]
        );
        // An unbalanced value swallows the rest rather than panicking.
        assert_eq!(
            solution_complexities("solution time=O(log space=O(1)"),
            vec![("time".to_owned(), "O(log space=O(1)".to_owned())]
        );
    }
}
