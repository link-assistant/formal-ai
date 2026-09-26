//! IR → concrete source, one backend per language (issue #1138, plan 02 L6,
//! L19).
//!
//! `compose` dispatches through [`lowering_for`] instead of refusing every
//! language but Python, and a gap is named rather than opaque: a
//! [`LoweringGap`] says which node has no lowering in which language.

pub mod python;
pub mod rust;

use crate::coding::fragment_catalog::FragmentCatalog;
use crate::coding::program_ir::ProgramIr;

/// Lower a typed IR to concrete source in one language.
pub trait LanguageLowering {
    /// Catalog slug, matching `ProgramLanguage::slug`.
    fn language(&self) -> &'static str;

    /// Render the whole program. `Err` names the first unsupported node so the
    /// gap is specific rather than "renderer unavailable".
    fn lower(&self, ir: &ProgramIr, catalog: &FragmentCatalog) -> Result<String, LoweringGap>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweringGap {
    pub language: String,
    pub node: String,
    pub detail: String,
}

/// Every registered lowering, in catalog order.
#[must_use]
pub fn lowerings() -> Vec<&'static dyn LanguageLowering> {
    static PYTHON: python::PythonLowering = python::PythonLowering;
    static RUST: rust::RustLowering = rust::RustLowering;
    vec![&PYTHON, &RUST]
}

#[must_use]
pub fn lowering_for(language: &str) -> Option<&'static dyn LanguageLowering> {
    lowerings()
        .into_iter()
        .find(|lowering| lowering.language() == language.trim().to_ascii_lowercase())
}
