//! The problem page's remembered panes — which tab, which editorial section, how wide the left
//! pane. Pure half: the vocabulary, the record format, and the two matchers.
//!
//! The section is remembered by LABEL, never by index. Editorial sections are whatever the
//! author wrote under `##`, so index 1 on one problem is "Solution" and on the next is
//! "Optimisation" — but "Solution" means "Solution" everywhere, which is exactly the carry-over
//! a reader is asking for when they set it.

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

    fn parse(token: &str) -> Self {
        match token {
            "editorial" => Self::Editorial,
            "coach" => Self::Coach,
            "submissions" => Self::Submissions,
            _ => Self::Description,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// THE RECORD — `tab|left_pct|section`, degrading per field
// ─────────────────────────────────────────────────────────────────────────────

/// The splitter's travel, matching the drag clamp in the view.
pub const MIN_LEFT_PCT: f64 = 28.0;
pub const MAX_LEFT_PCT: f64 = 64.0;
pub const DEFAULT_LEFT_PCT: f64 = 46.0;

#[derive(Debug, Clone, PartialEq)]
pub struct PanePrefs {
    pub tab: Tab,
    pub left_pct: f64,
    pub section: String,
}

impl Default for PanePrefs {
    fn default() -> Self {
        Self {
            tab: Tab::Description,
            left_pct: DEFAULT_LEFT_PCT,
            section: String::new(),
        }
    }
}

/// Parse a stored `tab|left_pct|section` record; anything malformed degrades per field.
///
/// `section` is LAST and absorbs the remainder deliberately: a `##` heading may legitimately
/// contain a pipe, and a heading like `Two pointers | O(n)` must not corrupt the record.
pub fn parse(stored: Option<&str>) -> PanePrefs {
    let Some(stored) = stored else {
        return PanePrefs::default();
    };
    let mut parts = stored.splitn(3, '|');
    let (Some(tab), Some(left), Some(section)) = (parts.next(), parts.next(), parts.next()) else {
        return PanePrefs::default();
    };
    PanePrefs {
        tab: Tab::parse(tab),
        left_pct: left
            .parse::<f64>()
            .map_or(DEFAULT_LEFT_PCT, |pct| pct.clamp(MIN_LEFT_PCT, MAX_LEFT_PCT)),
        section: section.to_owned(),
    }
}

/// Two decimals, matching the precision the view actually renders — a raw drag lands on
/// `55.67703952901598`, and there is no reason to keep sixteen digits of it.
pub fn serialize(prefs: &PanePrefs) -> String {
    format!("{}|{:.2}|{}", prefs.tab.slug(), prefs.left_pct, prefs.section)
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

    #[test]
    fn tab_slugs_round_trip_and_degrade() {
        for tab in TABS {
            assert_eq!(Tab::parse(tab.slug()), tab);
        }
        assert_eq!(Tab::parse("banana"), Tab::Description);
        assert_eq!(Tab::parse(""), Tab::Description);
    }

    #[test]
    fn record_round_trips() {
        let prefs = PanePrefs {
            tab: Tab::Editorial,
            left_pct: 52.5,
            section: "Complexity Analysis".to_owned(),
        };
        assert_eq!(serialize(&prefs), "editorial|52.50|Complexity Analysis");
        assert_eq!(parse(Some(&serialize(&prefs))), prefs);
        // A raw drag value is rounded to the precision the view renders, not kept whole.
        let dragged = PanePrefs {
            left_pct: 55.677_039_529_015_98,
            ..prefs
        };
        assert_eq!(serialize(&dragged), "editorial|55.68|Complexity Analysis");
    }

    #[test]
    fn one_bad_field_does_not_take_the_others_down() {
        let mixed = parse(Some("banana|52|Solution"));
        assert_eq!(mixed.tab, Tab::Description);
        assert!((mixed.left_pct - 52.0).abs() < f64::EPSILON);
        assert_eq!(mixed.section, "Solution");

        let bad_width = parse(Some("editorial|banana|Solution"));
        assert_eq!(bad_width.tab, Tab::Editorial);
        assert!((bad_width.left_pct - DEFAULT_LEFT_PCT).abs() < f64::EPSILON);
    }

    /// The reason `section` is the LAST field: a heading is free text and may contain a pipe.
    #[test]
    fn a_pipe_in_the_heading_survives_the_round_trip() {
        assert_eq!(
            parse(Some("editorial|52.4|Two pointers | O(n)")).section,
            "Two pointers | O(n)"
        );
        let prefs = PanePrefs {
            tab: Tab::Editorial,
            left_pct: 46.0,
            section: "a|b".to_owned(),
        };
        assert_eq!(parse(Some(&serialize(&prefs))).section, "a|b");
    }

    #[test]
    fn absent_or_short_records_are_the_default() {
        assert_eq!(parse(None), PanePrefs::default());
        assert_eq!(parse(Some("")), PanePrefs::default());
        assert_eq!(parse(Some("editorial")), PanePrefs::default());
        assert_eq!(parse(Some("editorial|52")), PanePrefs::default());
        assert_eq!(parse(Some("garbage")), PanePrefs::default());
    }

    #[test]
    fn left_pct_clamps_to_the_splitter_travel() {
        assert!((parse(Some("x|52.5|y")).left_pct - 52.5).abs() < f64::EPSILON);
        assert!((parse(Some("x|999|y")).left_pct - MAX_LEFT_PCT).abs() < f64::EPSILON);
        assert!((parse(Some("x|1|y")).left_pct - MIN_LEFT_PCT).abs() < f64::EPSILON);
        assert!((parse(Some("x|banana|y")).left_pct - DEFAULT_LEFT_PCT).abs() < f64::EPSILON);
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
