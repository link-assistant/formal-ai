use formal_ai::memory_program::{
    compile_memory_program, MemoryProgramCompileError, MemoryProgramLimits,
};

#[test]
fn catalog_has_closed_primitives_and_fifteen_families() {
    let catalog = include_str!("../../data/seed/memory-programs.lino");
    assert_eq!(
        catalog
            .lines()
            .filter(|line| line.starts_with("  primitive "))
            .count(),
        8
    );
    assert!(
        catalog
            .lines()
            .filter(|line| line.starts_with("  family "))
            .count()
            >= 15
    );
}

#[test]
fn a_memory_shaped_unknown_request_is_an_explicit_gap() {
    let error = compile_memory_program(
        "transpose every fact matrix",
        MemoryProgramLimits::default(),
    )
    .expect_err("no seeded family provides matrix transposition");
    assert!(matches!(
        error,
        MemoryProgramCompileError::ProgramGap { .. }
    ));
}

#[test]
fn generic_fact_checks_are_not_misclassified_as_memory_program_gaps() {
    for request in [
        "fact-check this dialogue",
        "проверь факты в диалоге",
        "इस संवाद के तथ्यों की जाँच करें",
        "核查此对话中的事实",
        "verifica los hechos de este diálogo",
    ] {
        assert_eq!(
            compile_memory_program(request, MemoryProgramLimits::default()),
            Err(MemoryProgramCompileError::NotMemoryProgram),
            "{request} must remain available to the fact-checking route",
        );
    }
}
