//! The problem page's tab vocabulary, its splitter width, and the label matcher the editorial
//! shares.
//!
//! Step 47 also carried the ACTIVE TAB and the active editorial section across problem pages.
//! Step 65 removed both: opening a new problem on someone else's last choice — the Editorial tab,
//! scrolled to Solution — is a spoiler you never asked for, and the reader who wants that can
//! click twice. A new problem starts on Description, at the top. The splitter width stays, because
//! dragging a pane is a layout act rather than a place in the material.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Description,
    Editorial,
    Coach,
    Submissions,
}

/// Tab order, which is also render order. Each tab's slug keys the `problem-tab--<slug>`
/// modifier that hands it its own `--tab-hue`; the practice widget reuses `.problem-tab`
/// WITHOUT a modifier, so it keeps the default teal.
pub const TABS: [Tab; 4] = [Tab::Description, Tab::Editorial, Tab::Coach, Tab::Submissions];

impl Tab {
    pub fn slug(self) -> &'static str {
        match self {
            Self::Description => "description",
            Self::Editorial => "editorial",
            Self::Coach => "coach",
            Self::Submissions => "submissions",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Description => "Description",
            Self::Editorial => "Editorial",
            Self::Coach => "Coach",
            Self::Submissions => "Submissions",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// THE SPLITTER WIDTH — the only thing the problem page still remembers
// ─────────────────────────────────────────────────────────────────────────────

/// The splitter's travel, matching the drag clamp in the view.
pub const MIN_LEFT_PCT: f64 = 28.0;
pub const MAX_LEFT_PCT: f64 = 64.0;
pub const DEFAULT_LEFT_PCT: f64 = 46.0;

/// Parse a stored splitter width. Anything unreadable — including a step-47 `tab|pct|section`
/// record, which is why an existing reader's width resets exactly once — degrades to the default.
pub fn parse_left_pct(stored: Option<&str>) -> f64 {
    stored
        .and_then(|s| s.parse::<f64>().ok())
        .map_or(DEFAULT_LEFT_PCT, |pct| pct.clamp(MIN_LEFT_PCT, MAX_LEFT_PCT))
}

/// Two decimals, matching the precision the view actually renders — a raw drag lands on
/// `55.67703952901598`, and there is no reason to keep sixteen digits of it.
pub fn serialize_left_pct(left_pct: f64) -> String {
    format!("{left_pct:.2}")
}

// ─────────────────────────────────────────────────────────────────────────────
// SECTION MATCHING — by normalised label, never by index
// ─────────────────────────────────────────────────────────────────────────────

/// Lowercase, trimmed, inner whitespace collapsed — so `"Complexity  Analysis"` from one
/// problem's heading matches `"Complexity Analysis"` from another's.
pub fn normalize_label(label: &str) -> String {
    label
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Which section to reveal, given this editorial's labels and the remembered one. No match —
/// including a blank preference or an editorial that has no sections — reveals the first.
pub fn section_index(labels: &[String], preferred: &str) -> usize {
    let wanted = normalize_label(preferred);
    if wanted.is_empty() {
        return 0;
    }
    labels
        .iter()
        .position(|label| normalize_label(label) == wanted)
        .unwrap_or(0)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < f64::EPSILON
    }

    #[test]
    fn the_width_round_trips_at_the_precision_the_view_renders() {
        assert_eq!(serialize_left_pct(52.5), "52.50");
        assert!(close(parse_left_pct(Some(&serialize_left_pct(52.5))), 52.5));
        // A raw drag lands on sixteen digits; only two of them are kept.
        assert_eq!(serialize_left_pct(55.677_039_529_015_98), "55.68");
    }

    #[test]
    fn the_width_clamps_to_the_splitter_travel() {
        assert!(close(parse_left_pct(Some("999")), MAX_LEFT_PCT));
        assert!(close(parse_left_pct(Some("1")), MIN_LEFT_PCT));
    }

    /// Includes a step-47 `tab|pct|section` record: unreadable now, and deliberately so — the
    /// width resets once rather than the format growing a legacy branch forever.
    #[test]
    fn anything_unreadable_is_the_default_width() {
        for stored in [None, Some(""), Some("banana"), Some("editorial|52.50|Solution")] {
            assert!(close(parse_left_pct(stored), DEFAULT_LEFT_PCT), "{stored:?}");
        }
    }

    #[test]
    fn labels_match_however_they_were_typed() {
        assert_eq!(
            normalize_label("Complexity Analysis"),
            normalize_label("  complexity   analysis ")
        );
        let labels = [
            "Approach".to_owned(),
            "Solution".to_owned(),
            "Complexity Analysis".to_owned(),
        ];
        assert_eq!(section_index(&labels, "Complexity Analysis"), 2);
        assert_eq!(section_index(&labels, "  SOLUTION  "), 1);
    }

    #[test]
    fn an_unmatched_section_falls_back_to_the_first() {
        let labels = ["Approach".to_owned(), "Solution".to_owned()];
        assert_eq!(section_index(&labels, "Proof of correctness"), 0);
        assert_eq!(section_index(&labels, ""), 0);
        assert_eq!(section_index(&[], "Solution"), 0);
    }

    #[test]
    fn duplicate_labels_resolve_to_the_first() {
        let labels = ["Solution".to_owned(), "Solution".to_owned()];
        assert_eq!(section_index(&labels, "Solution"), 0);
    }
}
