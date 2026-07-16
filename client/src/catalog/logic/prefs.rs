//! Reading-preferences tokens (oracle: `ReadingPrefs` — the pure half). Four independent
//! choices, each a small allow-list; persisted as one `|`-joined string. Unknown tokens degrade
//! per-field to the default (a bad stored value must never break the reader).

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prefs {
    pub size: &'static str,
    pub leading: &'static str,
    pub family: &'static str,
    pub width: &'static str,
}

pub const DEFAULT_PREFS: Prefs = Prefs {
    size: "md",
    leading: "normal",
    family: "sans",
    width: "standard",
};

pub const SIZES: [(&str, &str); 3] = [("sm", "Small"), ("md", "Medium"), ("lg", "Large")];
pub const LEADINGS: [(&str, &str); 3] = [("tight", "Tight"), ("normal", "Comfortable"), ("loose", "Loose")];
pub const FAMILIES: [(&str, &str); 3] = [("serif", "Serif"), ("sans", "Sans"), ("mono", "Mono")];
pub const WIDTHS: [(&str, &str); 3] = [("narrow", "Narrow"), ("standard", "Standard"), ("wide", "Wide")];

fn canonical(options: &[(&'static str, &str); 3], token: &str, default: &'static str) -> &'static str {
    options
        .iter()
        .find(|(t, _)| *t == token)
        .map_or(default, |(t, _)| *t)
}

/// Parse a stored `size|leading|family|width` string; anything malformed degrades per field.
pub fn parse(stored: Option<&str>) -> Prefs {
    let Some(stored) = stored else {
        return DEFAULT_PREFS;
    };
    let parts: Vec<&str> = stored.split('|').collect();
    let [s, l, f, w] = parts.as_slice() else {
        return DEFAULT_PREFS;
    };
    Prefs {
        size: canonical(&SIZES, s, DEFAULT_PREFS.size),
        leading: canonical(&LEADINGS, l, DEFAULT_PREFS.leading),
        family: canonical(&FAMILIES, f, DEFAULT_PREFS.family),
        width: canonical(&WIDTHS, w, DEFAULT_PREFS.width),
    }
}

pub fn serialize(prefs: &Prefs) -> String {
    format!(
        "{}|{}|{}|{}",
        prefs.size, prefs.leading, prefs.family, prefs.width
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_degrades_per_field() {
        let stored = serialize(&Prefs {
            size: "lg",
            leading: "tight",
            family: "mono",
            width: "wide",
        });
        assert_eq!(stored, "lg|tight|mono|wide");
        let parsed = parse(Some(&stored));
        assert_eq!(parsed.size, "lg");
        assert_eq!(parsed.family, "mono");

        // One bad token degrades ONLY that field.
        let mixed = parse(Some("lg|banana|mono|wide"));
        assert_eq!(mixed.size, "lg");
        assert_eq!(mixed.leading, "normal");
        assert_eq!(mixed.family, "mono");
    }

    #[test]
    fn absent_or_malformed_storage_is_the_default() {
        assert_eq!(parse(None), DEFAULT_PREFS);
        assert_eq!(parse(Some("")), DEFAULT_PREFS);
        assert_eq!(parse(Some("only|three|parts")), DEFAULT_PREFS);
        assert_eq!(parse(Some("way|too|many|parts|here")), DEFAULT_PREFS);
    }
}
