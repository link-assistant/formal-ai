use super::webassembly::{interop_module, rust_file};
use super::{CompiledSubstitutionFile, SubstitutionProgramIr};

pub(super) fn emit(
    ir: &SubstitutionProgramIr,
    stem: &str,
) -> (CompiledSubstitutionFile, Vec<CompiledSubstitutionFile>) {
    let wasm_source = rust_file(ir, stem);
    (
        CompiledSubstitutionFile {
            name: format!("{stem}.mjs"),
            contents: interop_module(&wasm_source.name),
        },
        vec![wasm_source],
    )
}
