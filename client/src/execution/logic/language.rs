//! The client's fence-alias vocabulary — one table, mirroring
//! `server/execution/domain/Language::aliases`. The server stays the authority; an alias added
//! there joins this table in the same step.
//!
//! Two jobs, and the second is why the table exists at all: telling a runnable fence from a
//! plaintext one, and folding every spelling of a language onto ONE canonical token so a stored
//! preference of `python` still matches a block whose fence says `py` or `python3`.

use super::Variant;

/// `(canonical token, aliases)` — the canonical token is always the first alias, so the stored
/// value is the same string the server's `Language::resolve` would land on.
const LANGUAGES: [(&str, &[&str]); 11] = [
    ("python", &["python", "py", "python3"]),
    ("java", &["java"]),
    ("scala", &["scala"]),
    ("c", &["c"]),
    ("cpp", &["cpp", "c++", "cxx"]),
    ("go", &["go", "golang"]),
    ("rust", &["rust", "rs"]),
    ("kotlin", &["kotlin", "kt"]),
    ("typescript", &["typescript", "ts"]),
    ("javascript", &["javascript", "js", "node"]),
    ("sql", &["sql", "sqlite"]),
];

/// Fold a fence alias onto its canonical token: trimmed, case-insensitive; blank or unknown →
/// `None` (which is also the "this fence is not runnable" answer).
pub fn canonical_lang(alias: &str) -> Option<&'static str> {
    let needle = alias.trim().to_lowercase();
    if needle.is_empty() {
        return None;
    }
    LANGUAGES
        .iter()
        .find(|(_, aliases)| aliases.contains(&needle.as_str()))
        .map(|(canonical, _)| *canonical)
}

/// Which variant a block should open on, given the reader's stored language preference.
///
/// Falls back to 0 whenever the preference cannot be honoured — absent, blank, a language this
/// build doesn't know, or simply not among THIS block's variants. Built from `position`, so the
/// result is structurally in-bounds for any inputs; callers index `variants` with it directly.
pub fn preferred_index(variants: &[Variant], preferred: Option<&str>) -> usize {
    let Some(wanted) = preferred.and_then(canonical_lang) else {
        return 0;
    };
    variants
        .iter()
        .position(|v| canonical_lang(&v.language) == Some(wanted))
        .unwrap_or(0)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn variant(language: &str) -> Variant {
        Variant {
            language: language.to_owned(),
            source: String::new(),
            viz: None,
        }
    }

    #[test]
    fn every_spelling_of_a_language_folds_onto_one_token() {
        for alias in ["python", "py", "python3", "PYTHON", "  Python  ", "Py"] {
            assert_eq!(canonical_lang(alias), Some("python"), "{alias}");
        }
        assert_eq!(canonical_lang("c++"), Some("cpp"));
        assert_eq!(canonical_lang("NODE"), Some("javascript"));
    }

    /// The alias the client's old flat table was missing — the server has run it since step 09,
    /// so assert the fix rather than trusting the table was copied correctly.
    #[test]
    fn sqlite_resolves_to_sql() {
        assert_eq!(canonical_lang("sqlite"), Some("sql"));
        assert_eq!(canonical_lang("sql"), Some("sql"));
    }

    #[test]
    fn unknown_and_blank_aliases_are_none() {
        assert_eq!(canonical_lang("cobol"), None);
        assert_eq!(canonical_lang(""), None);
        assert_eq!(canonical_lang("   "), None);
        assert_eq!(canonical_lang("plaintext"), None);
    }

    /// Ported from the server's `aliases_are_globally_unique_and_round_trip` — an alias claimed
    /// by two languages would make the preference resolve differently depending on table order.
    #[test]
    fn aliases_are_globally_unique_and_round_trip() {
        let mut seen: Vec<&str> = Vec::new();
        for (canonical, aliases) in LANGUAGES {
            assert_eq!(
                aliases.first(),
                Some(&canonical),
                "canonical must lead its aliases"
            );
            for alias in aliases {
                assert!(!seen.contains(alias), "duplicate alias: {alias}");
                seen.push(alias);
                assert_eq!(canonical_lang(alias), Some(canonical));
            }
        }
    }

    #[test]
    fn preferred_index_finds_the_wanted_variant() {
        let variants = [variant("python"), variant("java")];
        assert_eq!(preferred_index(&variants, Some("java")), 1);
        assert_eq!(preferred_index(&variants, Some("python")), 0);
    }

    #[test]
    fn preferred_index_matches_across_aliases() {
        let variants = [variant("java"), variant("py")];
        assert_eq!(preferred_index(&variants, Some("python")), 1);
        let variants = [variant("java"), variant("python3")];
        assert_eq!(preferred_index(&variants, Some("python")), 1);
    }

    #[test]
    fn preferred_index_falls_back_to_the_first_variant() {
        let variants = [variant("python"), variant("java")];
        assert_eq!(preferred_index(&variants, None), 0);
        assert_eq!(preferred_index(&variants, Some("")), 0);
        assert_eq!(preferred_index(&variants, Some("cobol")), 0);
        // The honest case this whole fallback exists for: a page that simply hasn't got it.
        assert_eq!(preferred_index(&variants, Some("rust")), 0);
    }

    /// `runnable.rs`'s `variant_at` indexes without clamping — this is the invariant that makes
    /// that safe, so assert it over the whole cross-product rather than a happy path.
    #[test]
    fn preferred_index_is_always_in_bounds() {
        let pool = ["python", "java", "rs", "sqlite", "not-a-language", ""];
        for len in 1..=pool.len() {
            let variants: Vec<Variant> = pool[..len].iter().map(|l| variant(l)).collect();
            for preference in pool {
                let i = preferred_index(&variants, Some(preference));
                assert!(i < variants.len(), "{preference} over {len} variants → {i}");
            }
            assert!(preferred_index(&variants, None) < variants.len());
        }
    }
}
