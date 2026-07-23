//! Where an editable lesson's source comes from (`FsLessonSource`) — resolved THROUGH the catalog
//! walker's lesson-file map, never by joining the URL path onto the content root. Real folders
//! carry `NN-` order prefixes (`/foundations/` is the directory `01-foundations`), so naive
//! joining cannot work.
//!
//! Routing every edit through the catalog's own resolver has a second effect worth naming: only
//! files the catalog already SERVES are reachable. `local-only/`, `_`-prefixed files and the
//! reserved aux dirs are excluded by the walker, so they are structurally uneditable here — the
//! protection is the same one that keeps them unservable, not a second list to keep in sync.
//!
//! The file comes back WHOLE, frontmatter fence included. The reader's payload carries the body
//! with the fence stripped; saving that back would delete the frontmatter.

use crate::authoring::application::{AuthoringError, LessonFile, LessonSource};
use crate::catalog::application::{ContentError, ContentRepository};
use crate::catalog::domain::{resolver, walker};

pub struct FsLessonSource<R> {
    repo: R,
}

impl<R> FsLessonSource<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

impl<R: ContentRepository> LessonSource for FsLessonSource<R> {
    async fn content_version(&self) -> String {
        self.repo.content_version().await
    }

    async fn file_for(&self, lesson_path: &[String]) -> Result<Option<LessonFile>, AuthoringError> {
        if lesson_path.is_empty() || !lesson_path.iter().all(|s| walker::slug_like(s)) {
            return Ok(None);
        }
        let tree = self.repo.load_tree().await.map_err(|e| unreadable(&e))?;
        let walk = walker::walk(&tree)
            .map_err(|error| AuthoringError::ContentUnreadable(format!("catalog index invalid: {error}")))?;
        let Some((book, in_book_path, _)) = resolver::resolve_lesson(&walk.catalog, lesson_path) else {
            return Ok(None);
        };
        let Some(file_path) = walk
            .lesson_files
            .get(&book.slug)
            .and_then(|files| files.get(&in_book_path))
        else {
            return Ok(None);
        };
        // Re-read rather than reuse the tree's copy: the walk may be a moment old, and the
        // fingerprint the editor is handed must describe the bytes on disk right now.
        match self.repo.read_lesson(file_path).await {
            Ok(source) => Ok(Some(LessonFile {
                file_path: file_path.clone(),
                source,
            })),
            Err(ContentError::NotFound(_)) => Ok(None),
            Err(error) => Err(unreadable(&error)),
        }
    }
}

fn unreadable(error: &ContentError) -> AuthoringError {
    AuthoringError::ContentUnreadable(error.to_string())
}

#[cfg(test)]
#[path = "lesson_source_tests.rs"]
mod tests;
