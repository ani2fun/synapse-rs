//! The reader's remembered workbench language.
//!
//! Read at mount, written only on an explicit dropdown click — never in `switch_to`, which
//! copy-to-editor also drives, and never at mount. That asymmetry is what stops a page from
//! clobbering the preference: a Python-only block renders no dropdown at all, so a
//! Java-preferring reader can cross any number of single-language pages untouched.

use crate::execution::logic::{Variant, canonical_lang, preferred_index};

const LANG_KEY: &str = "wb-language";

/// Which variant this block should open on. Falls back to 0 whenever the preference can't be
/// honoured, and is always in bounds — callers index `variants` with it directly.
pub fn index_for(variants: &[Variant]) -> usize {
    preferred_index(variants, crate::storage::get(LANG_KEY).as_deref())
}

/// Remember a language. Stores the CANONICAL token, so `py` and `python3` both come back as
/// `python` and match whatever a later page happens to spell its fence.
pub fn store(alias: &str) {
    if let Some(canonical) = canonical_lang(alias) {
        crate::storage::set(LANG_KEY, canonical);
    }
}
