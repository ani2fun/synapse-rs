//! The blog domain (oracle: `BlogPost` + `BlogFrontmatter`): one post per markdown file, a
//! lenient frontmatter fence, and graceful degradation — a malformed date or read-minutes value
//! becomes `None`, never an error. The fence parser is a deliberate TWIN of the catalog's, not
//! an import: bounded contexts own their vocabulary (the oracle duplicated it for the same
//! reason).

use std::collections::BTreeMap;

use chrono::NaiveDate;

/// One published post.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlogPost {
    pub slug: String,
    pub title: String,
    pub summary: Option<String>,
    pub published_at: Option<NaiveDate>,
    pub tags: Vec<String>,
    pub read_minutes: Option<i32>,
    pub eyebrow: Option<String>,
    pub body: String,
}

/// The listing card — every field of the post except the body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlogSummary {
    pub slug: String,
    pub title: String,
    pub summary: Option<String>,
    pub published_at: Option<NaiveDate>,
    pub tags: Vec<String>,
    pub read_minutes: Option<i32>,
    pub eyebrow: Option<String>,
}

impl BlogPost {
    /// Parse one raw markdown file. The slug is the fallback title; unparseable metadata
    /// degrades field-by-field.
    #[must_use]
    pub fn parse(slug: &str, raw: &str) -> Self {
        let (fields, body) = fields_and_body(raw);
        Self {
            slug: slug.to_owned(),
            title: fields.get("title").cloned().unwrap_or_else(|| slug.to_owned()),
            summary: fields.get("summary").cloned(),
            published_at: fields
                .get("publishedAt")
                .and_then(|d| NaiveDate::parse_from_str(d.trim(), "%Y-%m-%d").ok()),
            tags: fields.get("tags").map(|v| inline_list(v)).unwrap_or_default(),
            read_minutes: fields.get("readMinutes").and_then(|m| m.trim().parse().ok()),
            eyebrow: fields.get("eyebrow").cloned(),
            body,
        }
    }

    #[must_use]
    pub fn summary_view(&self) -> BlogSummary {
        BlogSummary {
            slug: self.slug.clone(),
            title: self.title.clone(),
            summary: self.summary.clone(),
            published_at: self.published_at,
            tags: self.tags.clone(),
            read_minutes: self.read_minutes,
            eyebrow: self.eyebrow.clone(),
        }
    }
}

/// A fence exists only when the FIRST line is `---` and a closing `---` follows; anything
/// malformed degrades to "no fence" (empty fields, the whole content as body).
fn fields_and_body(content: &str) -> (BTreeMap<String, String>, String) {
    let lines: Vec<&str> = content
        .split('\n')
        .map(|l| l.strip_suffix('\r').unwrap_or(l))
        .collect();
    if lines.first().map(|l| l.trim_end()) != Some("---") {
        return (BTreeMap::new(), content.to_owned());
    }
    let Some(end) = lines
        .iter()
        .skip(1)
        .position(|l| l.trim_end() == "---")
        .map(|i| i + 1)
    else {
        return (BTreeMap::new(), content.to_owned());
    };
    let mut fields = BTreeMap::new();
    for line in &lines[1..end] {
        let Some(idx) = line.find(':') else { continue };
        if idx == 0 {
            continue;
        }
        let key = line[..idx].trim().to_owned();
        let value = unquote(line[idx + 1..].trim()).to_owned();
        if !value.is_empty() {
            fields.insert(key, value);
        }
    }
    (fields, lines[end + 1..].join("\n"))
}

/// Inline flow-style lists only: `[a, b, "c d"]`.
fn inline_list(value: &str) -> Vec<String> {
    let inner = value
        .trim()
        .strip_prefix('[')
        .and_then(|v| v.strip_suffix(']'))
        .unwrap_or(value);
    inner
        .split(',')
        .map(|item| unquote(item.trim()).to_owned())
        .filter(|item| !item.is_empty())
        .collect()
}

fn unquote(s: &str) -> &str {
    for quote in ['"', '\''] {
        if s.len() >= 2 && s.starts_with(quote) && s.ends_with(quote) {
            return &s[1..s.len() - 1];
        }
    }
    s
}

#[cfg(test)]
#[path = "blog_tests.rs"]
mod tests;
