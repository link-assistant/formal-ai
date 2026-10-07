//! Issue #1186 R4/R6: the relative-meta-logic export step of the
//! formalization task.
//!
//! The relative-meta-logic crate is unpublished, so the clause is exported
//! through its `rml` command: a universal conditional clause is rendered as
//! the typed fragment `rml export lean` reads, the command (the executable
//! `FORMAL_AI_RML` names, else `rml` in PATH) is run when present, and its
//! exit status and output path are a `formalize:fragment` derivation event
//! that `formal-ai explain` prints. Mirrored by
//! `rust/tests/web/issue-1186-rml-export.test.mjs`, which also runs the real
//! exporter in CI (`RML_BIN`, `.github/workflows/layered-ci.yml`).

use formal_ai::derivation::Derivation;
use formal_ai::{
    AppliedPredicate, QuantifiedClause, RmlExport, SolverConfig, UniversalSolver, rml_source,
    run_rml_export_with,
};

fn predicate(name: &str, object: Option<&str>) -> AppliedPredicate {
    AppliedPredicate {
        name: name.to_owned(),
        object: object.map(str::to_owned),
    }
}

fn clause(quantifier: &str, object: Option<&str>) -> QuantifiedClause {
    QuantifiedClause {
        quantifier: quantifier.to_owned(),
        variable: "x".to_owned(),
        antecedent: vec![predicate("Student", None), predicate("Studies", None)],
        consequent: predicate("Passes", object),
    }
}

#[test]
fn a_universal_conditional_renders_as_the_rml_typed_fragment() {
    assert_eq!(
        rml_source(&clause("forall", None)).as_deref(),
        Some(
            "(U: (Type 0) U)\n(Student: (Pi (U x) Prop))\n(Studies: (Pi (U x) Prop))\n(Passes: (Pi (U x) Prop))\n(formalized: (Pi (U x) (Pi ((Student x) h1) (Pi ((Studies x) h2) (Passes x)))))\n"
        )
    );
}

#[test]
fn an_object_becomes_a_constant_of_the_domain() {
    assert_eq!(
        rml_source(&clause("forall", Some("exam"))).as_deref(),
        Some(
            "(U: (Type 0) U)\n(Student: (Pi (U x) Prop))\n(Studies: (Pi (U x) Prop))\n(Passes: (Pi (U x) (Pi (U xx) Prop)))\n(exam: U exam)\n(formalized: (Pi (U x) (Pi ((Student x) h1) (Pi ((Studies x) h2) (Passes x exam)))))\n"
        )
    );
}

#[test]
fn existential_and_negative_readings_are_outside_the_export_subset() {
    assert_eq!(rml_source(&clause("exists", None)), None);
    assert_eq!(rml_source(&clause("no", None)), None);
}

#[cfg(unix)]
#[test]
fn the_rml_run_records_its_exit_status_and_paths() {
    use std::os::unix::fs::PermissionsExt;
    let directory = std::env::temp_dir().join(format!("formal-ai-rml-test-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("a scratch directory");
    let command = directory.join("rml");
    std::fs::write(&command, "#!/bin/sh\nexit 3\n").expect("a stand-in rml");
    std::fs::set_permissions(&command, std::fs::Permissions::from_mode(0o755))
        .expect("an executable stand-in");
    let source = rml_source(&clause("forall", None)).expect("a universal clause has a source");
    let export = run_rml_export_with(&command, &source, &directory);
    let RmlExport::Ran {
        exit,
        source_path,
        output_path,
        ..
    } = export.clone()
    else {
        panic!("the stand-in ran: {export:?}");
    };
    assert_eq!(exit, Some(3));
    assert_eq!(
        std::fs::read_to_string(&source_path).expect("the source was written"),
        source
    );
    assert!(
        output_path.ends_with(".lean"),
        "the Lean output path is recorded: {output_path}"
    );
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn the_rml_step_is_a_derivation_fragment() {
    // The honesty sentence the step renders is pinned exactly by
    // `rust/tests/web/issue-1186-rml-export.test.mjs`; here the persisted
    // fragment `formal-ai explain` prints is read back.
    let solver = UniversalSolver::new(SolverConfig {
        offline: true,
        ..SolverConfig::default()
    });
    let solved = solver.solve("Formalize in first-order logic: Every baker who sings smiles");
    let root = std::env::current_dir().expect("working directory");
    let record = Derivation::load(&root, &solved.derivation_id()).expect("a persisted derivation");
    let fragments = record.formalized_fragments.join("\n");
    assert!(
        fragments.contains("stage=rml_export status=absent invoked=false")
            || fragments.contains("stage=rml_export status=ran invoked=true"),
        "the rml step is recorded for `formal-ai explain`: {fragments}"
    );
}
