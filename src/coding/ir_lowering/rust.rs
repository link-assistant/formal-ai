//! The Rust lowering backend — the second language, which proves the IR is not
//! Python-shaped (issue #1138, plan 02 L19).

use crate::coding::fragment_catalog::FragmentCatalog;
use crate::coding::ir_lowering::{LanguageLowering, LoweringGap};
use crate::coding::program_ir::ProgramIr;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RustLowering;

impl LanguageLowering for RustLowering {
    fn language(&self) -> &'static str {
        "rust"
    }

    fn lower(&self, _ir: &ProgramIr, _catalog: &FragmentCatalog) -> Result<String, LoweringGap> {
        todo!("plan 02 leaf L19")
    }
}
