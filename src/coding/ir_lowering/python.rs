//! The Python lowering backend (issue #1138, plan 02 L6).
//!
//! `python_render::render_function` moves behind [`LanguageLowering`]; the
//! per-language surface forms stay data, read from the fragment catalog.

use crate::coding::fragment_catalog::FragmentCatalog;
use crate::coding::ir_lowering::{LanguageLowering, LoweringGap};
use crate::coding::program_ir::ProgramIr;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PythonLowering;

impl LanguageLowering for PythonLowering {
    fn language(&self) -> &'static str {
        "python"
    }

    fn lower(&self, _ir: &ProgramIr, _catalog: &FragmentCatalog) -> Result<String, LoweringGap> {
        todo!("plan 02 leaf L6")
    }
}
