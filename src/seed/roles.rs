//! Semantic-role identifiers for the meaning lexicon (issue #386).
//!
//! Recognition code never matches a hardcoded natural-language word; it asks
//! the [lexicon](super::lexicon) "which surface forms evidence role *X*?" and
//! names only the language-independent role. Those role identifiers live here,
//! in one registry, so the loader in [`super::meanings`] stays focused on
//! parsing and querying and keeps clear of the seed file-size guard.
//!
//! Each constant is the exact `role` string a meaning declares in
//! `data/seed/meanings*.lino`. A role only needs a constant when *code* queries
//! it; roles that exist purely to group data (for example
//! `web_navigation_concept`) stay in the seed without a mirror here.

mod intent;
mod language;
mod program;
mod reasoning;
mod tooling;

pub use intent::*;
pub use language::*;
pub use program::*;
pub use reasoning::*;
pub use tooling::*;
