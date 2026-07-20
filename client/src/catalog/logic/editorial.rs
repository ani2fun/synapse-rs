//! The editorial document model (pure half of the redesigned Editorial tab). One markdown
//! string in — the sidecar or the inline `<details>` tail, they share a shape — and a typed
//! `EditorialDoc` out: approaches for the stepper, sections for the jump bar, complexity
//! claims for the cards.
//!
//! Two authored formats exist. Type 1 is flat (`## Intuition` / `## Approach` /
//! `## Solution` / `## Complexity Analysis`); type 2 nests the same set as `###`
//! subsections under one `##` heading PER APPROACH (`## Brute Force`, `## Optimal 1`, …).
//! Both are parsed by the same splitter; `multi` is
//! detected, never declared. Anything else — arbitrary headings, plain fences, no headings
//! at all — degrades to a plain sectioned document and the view shows no stepper or cards.

use super::pane::normalize_label;
use crate::execution::logic::solution_complexities;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionKind {
    Intuition,
    Approach,
    Solution,
    Complexity,
    Other,
}

/// One rendered section: the author's heading (empty for an approach's leading prose) and
/// its body WITHOUT the heading line — the view renders its own numbered header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionDoc {
    pub label: String,
    pub kind: SectionKind,
    pub md: String,
}

/// One approach: its heading label ("" for the single format), the complexity claims from
/// its first `solution` fence meta, and its sections in authoring order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApproachDoc {
    pub label: String,
    pub time: Option<String>,
    pub space: Option<String>,
    pub sections: Vec<SectionDoc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorialDoc {
    /// Prose before the first heading — always visible, above the jump bar's sections.
    pub preamble: String,
    /// One entry for the single format, two or more when the stepper applies.
    pub approaches: Vec<ApproachDoc>,
    /// Whether `approaches` came from top-level `##` approach headings.
    pub multi: bool,
}

// ─────────────────────────────────────────────────────────────────────────────
// PARSE
// ─────────────────────────────────────────────────────────────────────────────

#[must_use]
pub fn parse_editorial(md: &str) -> EditorialDoc {
    let md = strip_spoiler_wrapper(md);
    if md.trim().is_empty() {
        return EditorialDoc {
            preamble: String::new(),
            approaches: Vec::new(),
            multi: false,
        };
    }
    let (preamble, top) = {
        let (preamble, sections) = split_at_headings(&md, "## ");
        if sections.is_empty() {
            split_at_headings(&md, "# ")
        } else {
            (preamble, sections)
        }
    };
    if top.is_empty() {
        // No headings at all: one bare approach, everything stays in the preamble.
        let (time, space) = complexity_claims(first_solution_meta(&md));
        return EditorialDoc {
            preamble: md.trim().to_owned(),
            approaches: vec![ApproachDoc {
                label: String::new(),
                time,
                space,
                sections: Vec::new(),
            }],
            multi: false,
        };
    }

    let multi = top.len() >= 2 && top.iter().any(|(_, body)| has_canonical_subheading(body));
    let approaches = if multi {
        top.into_iter()
            .map(|(label, body)| {
                let (leading, subs) = split_at_headings(&body, "### ");
                let mut sections = Vec::new();
                if !leading.is_empty() {
                    sections.push(SectionDoc {
                        label: String::new(),
                        kind: SectionKind::Other,
                        md: leading,
                    });
                }
                sections.extend(subs.into_iter().map(section));
                build_approach(label, sections)
            })
            .collect()
    } else {
        vec![build_approach(
            String::new(),
            top.into_iter().map(section).collect(),
        )]
    };
    EditorialDoc {
        preamble,
        approaches,
        multi,
    }
}

/// The canonical kind behind a heading, via the same normalisation the remembered-section
/// matcher uses. `starts_with` on complexity covers both "Complexity" and
/// "Complexity Analysis".
#[must_use]
pub fn section_kind(label: &str) -> SectionKind {
    let norm = normalize_label(label);
    match norm.as_str() {
        "intuition" => SectionKind::Intuition,
        "approach" => SectionKind::Approach,
        "solution" | "solutions" | "code" => SectionKind::Solution,
        _ if norm.starts_with("complexity") => SectionKind::Complexity,
        _ => SectionKind::Other,
    }
}

fn section((label, md): (String, String)) -> SectionDoc {
    let kind = section_kind(&label);
    SectionDoc { label, kind, md }
}

fn build_approach(label: String, mut sections: Vec<SectionDoc>) -> ApproachDoc {
    synthesize_solution(&mut sections);
    let meta = sections.iter().find_map(|s| first_solution_meta(&s.md));
    let (time, space) = complexity_claims(meta);
    ApproachDoc {
        label,
        time,
        space,
        sections,
    }
}

/// Fence-aware split at line-start heading markers. The heading line is consumed into the
/// label; bodies and the preamble come back trimmed.
fn split_at_headings(md: &str, marker: &str) -> (String, Vec<(String, String)>) {
    let mut in_fence = false;
    let mut preamble: Vec<&str> = Vec::new();
    let mut sections: Vec<(String, Vec<&str>)> = Vec::new();
    for line in md.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
        } else if !in_fence && line.starts_with(marker) {
            sections.push((line[marker.len()..].trim().to_owned(), Vec::new()));
            continue;
        }
        match sections.last_mut() {
            Some((_, body)) => body.push(line),
            None => preamble.push(line),
        }
    }
    (
        preamble.join("\n").trim().to_owned(),
        sections
            .into_iter()
            .map(|(label, body)| (label, body.join("\n").trim().to_owned()))
            .collect(),
    )
}

/// Whether a top-level section body carries a canonical `###` subsection — the multi-approach
/// discriminator (an approach heading is free text; its CONTENTS are the recognisable part).
fn has_canonical_subheading(body: &str) -> bool {
    let (_, subs) = split_at_headings(body, "### ");
    subs.iter()
        .any(|(label, _)| section_kind(label) != SectionKind::Other)
}

/// The inline editorial arrives still wearing its spoiler wrapper (`problem_content_split`
/// cuts AT the `<details` line). Per-section fragment rendering would smear that unbalanced
/// pair across fragments, so the OUTER wrapper — and only it — comes off here: the opening
/// line, its `<summary>` line, and the last fence-outside `</details>`. Nested details deeper
/// in the document are content and survive.
fn strip_spoiler_wrapper(md: &str) -> String {
    let trimmed = md.trim();
    if !trimmed.starts_with("<details") {
        return md.to_owned();
    }
    let mut lines: Vec<&str> = trimmed.lines().collect();
    lines.remove(0);
    if let Some(first) = lines.iter().position(|line| !line.trim().is_empty()) {
        let head = lines[first].trim();
        if head.starts_with("<summary") && head.ends_with("</summary>") {
            lines.remove(first);
        }
    }
    let mut in_fence = false;
    let mut last_close = None;
    for (i, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
        } else if !in_fence && line.trim() == "</details>" {
            last_close = Some(i);
        }
    }
    if let Some(at) = last_close {
        lines.remove(at);
    }
    lines.join("\n").trim().to_owned()
}

// ─────────────────────────────────────────────────────────────────────────────
// SOLUTION FENCES — the synthesized section and the complexity claims
// ─────────────────────────────────────────────────────────────────────────────

/// The verified type-2 shape has NO `### Solution` heading — the fences sit at the tail of
/// `### Approach`. When no Solution section exists but a `solution` fence does, the owning
/// section splits at the first fence line and the tail becomes a synthetic Solution section
/// right after it. An explicit Solution heading suppresses this entirely.
fn synthesize_solution(sections: &mut Vec<SectionDoc>) {
    if sections.iter().any(|s| s.kind == SectionKind::Solution) {
        return;
    }
    for i in 0..sections.len() {
        let Some(fence_line) = solution_fence_start(&sections[i].md) else {
            continue;
        };
        let lines: Vec<&str> = sections[i].md.lines().collect();
        let head = lines[..fence_line].join("\n").trim().to_owned();
        let tail = lines[fence_line..].join("\n").trim().to_owned();
        let solution = SectionDoc {
            label: "Solution".to_owned(),
            kind: SectionKind::Solution,
            md: tail,
        };
        if head.is_empty() {
            sections[i] = solution;
        } else {
            sections[i].md = head;
            sections.insert(i + 1, solution);
        }
        return;
    }
}

/// Line index of the first fence OPENING whose info string carries the whitespace-delimited
/// `solution` token (the same predicate render.ts groups on). Plain fences don't count.
fn solution_fence_start(md: &str) -> Option<usize> {
    let mut in_fence = false;
    for (i, line) in md.lines().enumerate() {
        let t = line.trim_start();
        if t.starts_with("```") {
            if !in_fence {
                let info = t.trim_start_matches('`').trim();
                if info.split_whitespace().any(|word| word == "solution") {
                    return Some(i);
                }
            }
            in_fence = !in_fence;
        }
    }
    None
}

/// The full info string of the first `solution` fence, e.g.
/// `python solution time=O(N) space=O(K)` — `solution_complexities` reads the claims out.
#[must_use]
pub fn first_solution_meta(md: &str) -> Option<String> {
    let at = solution_fence_start(md)?;
    md.lines()
        .nth(at)
        .map(|line| line.trim_start().trim_start_matches('`').trim().to_owned())
}

fn complexity_claims(meta: Option<String>) -> (Option<String>, Option<String>) {
    let Some(meta) = meta else {
        return (None, None);
    };
    let pairs = solution_complexities(&meta);
    let find = |key: &str| {
        pairs
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.clone())
    };
    (find("time"), find("space"))
}

// ─────────────────────────────────────────────────────────────────────────────
// COMPLEXITY PROSE — `**Time Complexity:** O(…) – explanation` → the two cards
// ─────────────────────────────────────────────────────────────────────────────

/// One card's content: the O-value and its explanation prose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComplexityProse {
    pub time: Option<(String, String)>,
    pub space: Option<(String, String)>,
}

/// Parse a Complexity section's authored `**Time Complexity:** … / **Space Complexity:** …`
/// paragraphs. Either axis may miss; when BOTH do the section isn't card-shaped and the
/// caller renders the prose as-is.
#[must_use]
pub fn complexity_prose(md: &str) -> Option<ComplexityProse> {
    let mut time = None;
    let mut space = None;
    let mut in_fence = false;
    let lines: Vec<&str> = md.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let t = lines[i].trim();
        if t.starts_with("```") {
            in_fence = !in_fence;
            i += 1;
            continue;
        }
        let marker = (!in_fence).then(|| strip_marker(t)).flatten();
        let Some((is_time, rest)) = marker else {
            i += 1;
            continue;
        };
        // The paragraph: the marker line's remainder plus following lines until a blank
        // line, a fence, or the next marker.
        let mut paragraph = vec![rest.trim()];
        i += 1;
        while i < lines.len() {
            let next = lines[i].trim();
            if next.is_empty() || next.starts_with("```") || strip_marker(next).is_some() {
                break;
            }
            paragraph.push(next);
            i += 1;
        }
        let parsed = split_value_prose(&paragraph.join(" "));
        if is_time {
            time = time.or(parsed);
        } else {
            space = space.or(parsed);
        }
    }
    (time.is_some() || space.is_some()).then_some(ComplexityProse { time, space })
}

fn strip_marker(line: &str) -> Option<(bool, &str)> {
    let lower = line.to_lowercase();
    for (needle, is_time) in [("**time complexity:**", true), ("**space complexity:**", false)] {
        if lower.starts_with(needle) {
            return Some((is_time, &line[needle.len()..]));
        }
    }
    None
}

/// Split an authored complexity paragraph into (O-value, explanation). Authors separate the
/// two with a dash (`O(1) – Using …`), a period (`O(1). As no extra …`) or a comma
/// (`O(1), The operations …`) — all verified in the real content. The value starts at the
/// first `O(` and runs to the first separator sitting OUTSIDE parentheses after at least one
/// closed group, so `O(min(N1, N2))` and `O(sqrt(N)) + O(K*log(K)) – …` both survive whole.
/// A paragraph without an `O(` group is not card material.
fn split_value_prose(text: &str) -> Option<(String, String)> {
    const SEPARATORS: [&str; 5] = [" – ", " — ", " - ", ". ", ", "];
    let text = text.trim();
    let start = text.find("O(")?;
    let tail = &text[start..];
    let mut depth = 0i32;
    let mut closed = false;
    let mut split = None;
    for (at, c) in tail.char_indices() {
        if depth == 0 && closed {
            let rest = &tail[at..];
            if let Some(len) = SEPARATORS
                .iter()
                .find_map(|sep| rest.starts_with(sep).then_some(sep.len()))
            {
                split = Some((at, len));
                break;
            }
        }
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    closed = true;
                }
            }
            _ => {}
        }
    }
    let (value, prose) = match split {
        Some((at, len)) => (&tail[..at], &tail[at + len..]),
        None => (tail, ""),
    };
    let value = value.trim().trim_end_matches(['.', ',']).trim_end();
    (!value.is_empty()).then(|| (value.to_owned(), prose.trim().to_owned()))
}

// ─────────────────────────────────────────────────────────────────────────────
// DISPLAY HELPERS
// ─────────────────────────────────────────────────────────────────────────────

/// Display prettifier for complexity claims: authors write `O(sqrt(N)+K*log(K))` and
/// `O(N^2)`, the design shows `O(√N+K·log(K))` and `O(N²)`. Purely cosmetic — never fed
/// back into parsing.
#[must_use]
pub fn pretty_o(o: &str) -> String {
    let mut out = String::with_capacity(o.len());
    let mut rest = o;
    while let Some(at) = rest.find("sqrt(") {
        out.push_str(&rest[..at]);
        let inner = &rest[at + "sqrt(".len()..];
        let mut depth = 1usize;
        let mut close = None;
        for (offset, c) in inner.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        close = Some(offset);
                        break;
                    }
                }
                _ => {}
            }
        }
        if let Some(offset) = close {
            out.push('√');
            out.push_str(&inner[..offset]);
            rest = &inner[offset + 1..];
        } else {
            out.push_str(&rest[at..]);
            rest = "";
        }
    }
    out.push_str(rest);
    superscript_powers(&out.replace('*', "·"))
}

/// `^` followed by a WHOLE integer becomes a superscript (`N^2` → `N²`); a fractional power
/// like `N^1.5` has no clean superscript and stays as authored.
fn superscript_powers(s: &str) -> String {
    const DIGITS: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '^' {
            out.push(c);
            continue;
        }
        let mut power = String::new();
        while let Some(d) = chars.peek().filter(|d| d.is_ascii_digit()) {
            power.push(*d);
            chars.next();
        }
        if power.is_empty() || chars.peek() == Some(&'.') {
            out.push('^');
            out.push_str(&power);
        } else {
            for d in power.chars() {
                let index = usize::try_from(u32::from(d) - u32::from('0')).unwrap_or(0);
                out.push(DIGITS[index]);
            }
        }
    }
    out
}

/// The scroll-spy: the ACTIVE section is the last one whose top has passed the threshold
/// (tops are relative to the scroll container's top; sections above have negative tops).
#[must_use]
pub fn active_section(section_tops: &[f64], threshold: f64) -> usize {
    let mut active = 0;
    for (i, top) in section_tops.iter().enumerate() {
        if *top <= threshold {
            active = i;
        }
    }
    active
}

#[cfg(test)]
#[path = "editorial_tests.rs"]
mod tests;
