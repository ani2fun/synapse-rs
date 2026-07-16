//! The client's app-map (oracle: `Page.scala`). URL ↔ `Page` parsing is pure (no DOM, no IO);
//! pages join the enum as their steps land (blog, account, admin arrive with theirs).

/// Every page the SPA can show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Page {
    Library,
    Lesson(Vec<String>),
    Blog,
    BlogPost(String),
    Account,
    Admin,
    NotFound(String),
}

impl Page {
    /// Parse already-split, already-decoded URL path segments.
    pub fn from_segments(segments: &[&str]) -> Self {
        let segments: Vec<&str> = segments.iter().copied().filter(|s| !s.is_empty()).collect();
        match segments.split_first() {
            None => Self::Library,
            Some((&"synapse", rest)) if !rest.is_empty() => {
                Self::Lesson(rest.iter().map(|s| (*s).to_owned()).collect())
            }
            Some((&"blog", [])) => Self::Blog,
            Some((&"blog", [slug])) => Self::BlogPost((*slug).to_owned()),
            Some((&"account", [])) => Self::Account,
            Some((&"admin", [])) => Self::Admin,
            _ => Self::NotFound(segments.join("/")),
        }
    }

    /// The canonical URL — directory-mirror for lessons (ADR-S010).
    pub fn url(&self) -> String {
        match self {
            Self::Library => "/".to_owned(),
            Self::Lesson(path) => format!("/synapse/{}", path.join("/")),
            Self::Blog => "/blog".to_owned(),
            Self::BlogPost(slug) => format!("/blog/{slug}"),
            Self::Account => "/account".to_owned(),
            Self::Admin => "/admin".to_owned(),
            Self::NotFound(raw) => format!("/{raw}"),
        }
    }

    /// A directory-mirror path string (`a/b/c`) → its segments, dropping empties.
    pub fn segments_of(path: &str) -> Vec<String> {
        path.split('/')
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_app_map() {
        assert_eq!(Page::from_segments(&[]), Page::Library);
        assert_eq!(Page::from_segments(&["", ""]), Page::Library);
        assert_eq!(
            Page::from_segments(&["synapse", "learn", "dsa", "intro"]),
            Page::Lesson(vec!["learn".into(), "dsa".into(), "intro".into()])
        );
        // A bare /synapse is not a lesson.
        assert_eq!(
            Page::from_segments(&["synapse"]),
            Page::NotFound("synapse".into())
        );
        assert_eq!(Page::from_segments(&["blog"]), Page::Blog);
        assert_eq!(Page::from_segments(&["account"]), Page::Account);
        assert_eq!(Page::from_segments(&["admin"]), Page::Admin);
        assert_eq!(
            Page::from_segments(&["blog", "hello"]),
            Page::BlogPost("hello".into())
        );
        // Blog posts are flat — deeper paths are not pages.
        assert_eq!(
            Page::from_segments(&["blog", "a", "b"]),
            Page::NotFound("blog/a/b".into())
        );
        assert_eq!(
            Page::from_segments(&["ghost", "town"]),
            Page::NotFound("ghost/town".into())
        );
    }

    #[test]
    fn urls_round_trip() {
        let lesson = Page::Lesson(vec!["learn".into(), "dsa".into(), "intro".into()]);
        assert_eq!(lesson.url(), "/synapse/learn/dsa/intro");
        assert_eq!(Page::Library.url(), "/");
        assert_eq!(Page::Blog.url(), "/blog");
        assert_eq!(Page::BlogPost("hello".into()).url(), "/blog/hello");
        assert_eq!(Page::segments_of("a//b/c/"), vec!["a", "b", "c"]);
    }
}
