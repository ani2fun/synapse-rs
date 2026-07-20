//! Reactive catalog state (oracle: `CatalogStore.scala`, the state layer). The store lives in
//! Leptos CONTEXT, created under `App`'s owner — a module-level cache would tie the signal to
//! whichever page touched it first and go inert when that page unmounts (found the hard way in
//! this step's browser verify). The index is fetched once and shared by the library page and
//! every lesson's sidebar; the cache drops on failure so a transient miss doesn't pin a broken
//! index for the whole session. Lessons are fetch-per-navigation (the server caches the build).

use leptos::prelude::*;
use leptos::task::spawn_local;
use synapse_shared::catalog::{LessonPayloadDto, SynapseIndexDto};

use crate::api::{self, AsyncResult};
use crate::catalog::logic::progress;

/// The app-level catalog store. `Copy` — signal handles, not data.
#[derive(Clone, Copy)]
pub struct CatalogStore {
    index: RwSignal<AsyncResult<SynapseIndexDto>>,
    index_started: StoredValue<bool>,
}

impl CatalogStore {
    /// Created ONCE in `App` and provided as context.
    pub fn provide() {
        provide_context(Self {
            index: RwSignal::new(AsyncResult::Loading),
            index_started: StoredValue::new(false),
        });
    }

    pub fn from_context() -> Self {
        expect_context::<Self>()
    }

    /// The shared index signal — the first caller triggers the fetch; a failure re-arms it so
    /// the next navigation re-fetches.
    pub fn index(self) -> RwSignal<AsyncResult<SynapseIndexDto>> {
        if !self.index_started.get_value() {
            self.index_started.set_value(true);
            self.index.set(AsyncResult::Loading);
            crate::log::info("loading catalog index");
            spawn_local(async move {
                match api::index().await {
                    Ok(idx) => {
                        crate::log::debug(&format!(
                            "catalog index loaded: {} top-level entries",
                            idx.entries.len()
                        ));
                        self.index.set(AsyncResult::Loaded(idx));
                    }
                    Err(message) => {
                        crate::log::error(&format!("index: {message}"));
                        self.index_started.set_value(false);
                        self.index.set(AsyncResult::Failed(message));
                    }
                }
            });
        }
        self.index
    }
}

/// One lesson fetch, spawned per navigation.
pub fn load_lesson(path: Vec<String>) -> RwSignal<AsyncResult<LessonPayloadDto>> {
    let state = RwSignal::new(AsyncResult::Loading);
    crate::log::info(&format!("loading lesson: {}", path.join("/")));
    spawn_local(async move {
        match api::lesson(&path).await {
            Ok(payload) => {
                crate::log::debug(&format!("lesson loaded: {}", payload.frontmatter.title));
                state.set(AsyncResult::Loaded(payload));
            }
            Err(message) => {
                crate::log::error(&format!("lesson: {message}"));
                state.set(AsyncResult::Failed(message));
            }
        }
    });
    state
}

// ─────────────────────────────────────────────────────────────────────────────
// READING PREFERENCES (oracle: `ReadingPrefs` — the persisted half)
// ─────────────────────────────────────────────────────────────────────────────
// Loaded from localStorage and reflected onto `<html>` as `data-reader-*`
// attributes the stylesheet reads — set once BEFORE first paint (provide() runs
// in App's body), surviving navigation with no flash and no per-page rewiring.

use crate::catalog::logic::prefs::{self, Prefs};

const PREFS_KEY: &str = "reader-prefs";

#[derive(Clone, Copy)]
pub struct PrefsStore {
    pub prefs: RwSignal<Prefs>,
}

impl PrefsStore {
    /// Created ONCE in `App`: load → reflect onto `<html>` → provide as context.
    pub fn provide() {
        let loaded = prefs::parse(crate::storage::get(PREFS_KEY).as_deref());
        apply_to_html(&loaded);
        provide_context(Self {
            prefs: RwSignal::new(loaded),
        });
    }

    pub fn from_context() -> Self {
        expect_context::<Self>()
    }

    /// Commit one change: signal + localStorage + the `<html>` attributes, in one breath.
    pub fn commit(self, next: Prefs) {
        apply_to_html(&next);
        crate::storage::set(PREFS_KEY, &prefs::serialize(&next));
        self.prefs.set(next);
    }

    pub fn reset(self) {
        self.commit(prefs::DEFAULT_PREFS);
    }
}

fn apply_to_html(p: &Prefs) {
    let Some(root) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
    else {
        return;
    };
    let _ = root.set_attribute("data-reader-size", p.size);
    let _ = root.set_attribute("data-reader-leading", p.leading);
    let _ = root.set_attribute("data-reader-family", p.family);
    let _ = root.set_attribute("data-reader-width", p.width);
}

// ─────────────────────────────────────────────────────────────────────────────
// PROBLEM PANES — read at mount, written on click
// ─────────────────────────────────────────────────────────────────────────────
// Free functions, not a context store, deliberately: `ProblemWorkbench` is rebuilt from scratch
// on every navigation, so reading once at creation is all the carry-over needs. Nothing here is
// reactive, so nothing here needs a signal — the `SidebarMode` precedent, not the `PrefsStore`
// one.

use crate::catalog::logic::pane::{self, PanePrefs, Tab};

const PANE_KEY: &str = "problem-pane";

pub fn pane_prefs() -> PanePrefs {
    pane::parse(crate::storage::get(PANE_KEY).as_deref())
}

fn commit_pane(next: &PanePrefs) {
    crate::storage::set(PANE_KEY, &pane::serialize(next));
}

pub fn set_pane_tab(tab: Tab) {
    commit_pane(&PanePrefs { tab, ..pane_prefs() });
}

pub fn set_pane_section(section: &str) {
    commit_pane(&PanePrefs {
        section: section.to_owned(),
        ..pane_prefs()
    });
}

pub fn set_pane_left_pct(left_pct: f64) {
    commit_pane(&PanePrefs {
        left_pct,
        ..pane_prefs()
    });
}

// The remembered editorial APPROACH gets its own key rather than a fourth `PanePrefs` field:
// an approach label is free text like `section`, and the pipe-delimited record can only let
// ONE field absorb the remainder — a second free-text field would corrupt on pipes (and a
// format change would reset everyone's stored panes, the step-46 prefs lesson).
const APPROACH_KEY: &str = "problem-approach";

pub fn editorial_approach() -> String {
    crate::storage::get(APPROACH_KEY).unwrap_or_default()
}

pub fn set_editorial_approach(label: &str) {
    crate::storage::set(APPROACH_KEY, label);
}

// ── Reading progress (step 51) ────────────────────────────────────────────────
// Anonymous and device-local, deliberately. Signing in currently buys a bigger run budget,
// submission history and an admin flag — and production enforces the submit allowlist — so a
// progress feature behind sign-in would reach almost nobody. This one works for the reader who
// is actually here.

const PROGRESS_KEY: &str = "reader-progress";
const LAST_KEY: &str = "reader-last";

/// Two signals, two keys: what has been finished, and where to resume. Kept apart so a key that
/// fails to read costs only itself — see the note in `logic::progress`.
#[derive(Clone, Copy)]
pub struct ProgressStore {
    done: RwSignal<std::collections::BTreeSet<String>>,
    last: RwSignal<Option<String>>,
}

impl ProgressStore {
    /// Created ONCE in `App`, like every other store here.
    pub fn provide() {
        provide_context(Self {
            done: RwSignal::new(progress::parse(crate::storage::get(PROGRESS_KEY).as_deref())),
            last: RwSignal::new(crate::storage::get(LAST_KEY).filter(|s| !s.is_empty())),
        });
    }

    pub fn from_context() -> Self {
        expect_context::<Self>()
    }

    pub fn done(self) -> RwSignal<std::collections::BTreeSet<String>> {
        self.done
    }

    pub fn last(self) -> RwSignal<Option<String>> {
        self.last
    }

    /// Reactive — reading this inside a view subscribes it to the set.
    pub fn is_done(self, path: &str) -> bool {
        self.done.read().contains(path)
    }

    /// Mark finished or unfinished. Idempotent: re-marking an already-finished lesson writes
    /// nothing, so scrolling to the bottom twice does not churn storage.
    pub fn set_done(self, path: &str, done: bool) {
        let changed = self.done.try_update(|set| {
            if done {
                set.insert(path.to_owned())
            } else {
                set.remove(path)
            }
        });
        if changed == Some(true) {
            crate::storage::set(PROGRESS_KEY, &progress::serialize(&self.done.read()));
        }
    }

    /// Record where the reader is, for "continue where you left off". Separate from `set_done`
    /// because opening a lesson is not finishing it.
    pub fn visit(self, path: &str) {
        if self.last.get_untracked().as_deref() == Some(path) {
            return;
        }
        crate::storage::set(LAST_KEY, path);
        self.last.set(Some(path.to_owned()));
    }

    /// "Erase all my data" — reading progress is personal in a way a font size is not.
    pub fn clear(self) {
        crate::storage::remove(PROGRESS_KEY);
        crate::storage::remove(LAST_KEY);
        self.done.set(std::collections::BTreeSet::new());
        self.last.set(None);
    }
}
