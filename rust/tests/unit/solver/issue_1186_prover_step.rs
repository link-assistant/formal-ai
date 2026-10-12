//! Issue #1186 R4: the theorem-prover step of the formalization task.
//!
//! Each `prover` record of `data/seed/formal-targets.lino` turns its target's
//! rendering into a self-contained compile unit, and its binary runs on that
//! unit when found in PATH. These tests stub PATH with a scratch directory of
//! stand-in `lean`/`coqc` executables, so the run branch is pinned without
//! either prover installed. Twin of
//! `rust/tests/web/issue-1186-prover-seam.test.mjs`.

use formal_ai::{
    AppliedPredicate, ClauseExporter, ProverRun, QuantifiedClause, prover_check_slots,
    prover_command_in, prover_file_stem, prover_records, prover_runs_with, prover_unit,
};

const LEAN_UNIT: &str = "axiom U : Type\naxiom Student : U → Prop\naxiom Studies : U → Prop\naxiom Passes : U → U → Prop\naxiom exam : U\n\ntheorem formalized : ∀ (x : U), Student x ∧ Studies x → Passes x exam := by sorry\n";

const ROCQ_UNIT: &str = "Parameter U : Type.\nParameter Student : U -> Prop.\nParameter Studies : U -> Prop.\nParameter Passes : U -> U -> Prop.\nParameter exam : U.\n\nTheorem formalized : forall (x : U), Student x /\\ Studies x -> Passes x exam.\nProof.\nAdmitted.\n";

fn predicate(name: &str, object: Option<&str>) -> AppliedPredicate {
    AppliedPredicate {
        name: name.to_owned(),
        object: object.map(str::to_owned),
    }
}

fn probe_clause() -> QuantifiedClause {
    QuantifiedClause {
        quantifier: "forall".to_owned(),
        variable: "x".to_owned(),
        antecedent: vec![predicate("Student", None), predicate("Studies", None)],
        consequent: predicate("Passes", Some("exam")),
    }
}

/// A scratch directory unique to this test process and `name`.
fn scratch(name: &str) -> std::path::PathBuf {
    let directory = std::env::temp_dir().join(format!("formal-ai-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("a scratch directory");
    directory
}

#[test]
fn the_seed_declares_a_lean_and_a_rocq_prover() {
    let declared: Vec<(&str, &str, &str, &str)> = prover_records()
        .iter()
        .map(|prover| {
            (
                prover.id.as_str(),
                prover.target.as_str(),
                prover.binary.as_str(),
                prover.extension.as_str(),
            )
        })
        .collect();
    assert_eq!(
        declared,
        vec![
            ("lean", "lean", "lean", "lean"),
            ("rocq", "rocq", "coqc", "v")
        ]
    );
    assert_eq!(ClauseExporter::active(), ClauseExporter::SeedTemplates);
    assert_eq!(ClauseExporter::active().name(), "seed_templates");
}

#[test]
fn each_compile_unit_declares_the_domain_predicates_and_objects() {
    let units: Vec<Option<String>> = prover_records()
        .iter()
        .map(|prover| prover_unit(&probe_clause(), prover))
        .collect();
    assert_eq!(
        units,
        vec![Some(LEAN_UNIT.to_owned()), Some(ROCQ_UNIT.to_owned())]
    );
}

#[test]
fn an_empty_path_finds_no_prover() {
    let directory = scratch("prover-empty");
    let runs = prover_runs_with(&probe_clause(), std::ffi::OsStr::new(""), &directory);
    assert_eq!(
        runs.iter().map(|(_, run)| run.clone()).collect::<Vec<_>>(),
        vec![ProverRun::Absent, ProverRun::Absent]
    );
    let slots = prover_check_slots(&runs, "en");
    assert_eq!(
        slots,
        vec![
            (
                "prover_summary".to_owned(),
                "No theorem prover was invoked.".to_owned()
            ),
            (
                "lean_check".to_owned(),
                "lean was not found in PATH, so the lean text was not compiled.".to_owned()
            ),
            (
                "rocq_check".to_owned(),
                "coqc was not found in PATH, so the coqc text was not compiled.".to_owned()
            ),
        ]
    );
    let _ = std::fs::remove_dir_all(&directory);
}

#[cfg(unix)]
#[test]
fn provers_in_a_stubbed_path_run_on_their_units_and_report_their_exit_status() {
    use std::os::unix::fs::PermissionsExt;
    let bin = scratch("prover-path");
    let units = scratch("prover-units");
    for (binary, exit) in [("lean", 0), ("coqc", 1)] {
        let script = bin.join(binary);
        std::fs::write(
            &script,
            format!(
                "#!/bin/sh\ncp \"$1\" \"{}/{binary}.seen\"\nexit {exit}\n",
                bin.display()
            ),
        )
        .expect("a stand-in prover");
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755))
            .expect("an executable stand-in");
    }
    assert_eq!(
        prover_command_in(bin.as_os_str(), "lean"),
        Some(bin.join("lean"))
    );
    assert_eq!(prover_command_in(bin.as_os_str(), "rml"), None);

    let runs = prover_runs_with(&probe_clause(), bin.as_os_str(), &units);
    let lean_path = units.join(format!("{}.lean", prover_file_stem(LEAN_UNIT)));
    let rocq_path = units.join(format!("{}.v", prover_file_stem(ROCQ_UNIT)));
    assert_eq!(
        runs.iter().map(|(_, run)| run.clone()).collect::<Vec<_>>(),
        vec![
            ProverRun::Ran {
                command: bin.join("lean").display().to_string(),
                exit: Some(0),
                source_path: lean_path.display().to_string(),
            },
            ProverRun::Ran {
                command: bin.join("coqc").display().to_string(),
                exit: Some(1),
                source_path: rocq_path.display().to_string(),
            },
        ]
    );
    assert_eq!(
        std::fs::read_to_string(bin.join("lean.seen")).expect("lean saw its unit"),
        LEAN_UNIT
    );
    assert_eq!(
        std::fs::read_to_string(bin.join("coqc.seen")).expect("coqc saw its unit"),
        ROCQ_UNIT
    );
    assert_eq!(
        prover_check_slots(&runs, "en"),
        vec![
            (
                "prover_summary".to_owned(),
                "A theorem prover was invoked: lean, coqc.".to_owned()
            ),
            (
                "lean_check".to_owned(),
                format!(
                    "lean was found in PATH and ran on {}; it exited with status 0.",
                    lean_path.display()
                )
            ),
            (
                "rocq_check".to_owned(),
                format!(
                    "coqc was found in PATH and ran on {}; it exited with status 1.",
                    rocq_path.display()
                )
            ),
        ]
    );
    assert_eq!(
        prover_check_slots(&runs[..1], "ru"),
        vec![
            (
                "prover_summary".to_owned(),
                "Вызван теорем-прувер: lean.".to_owned()
            ),
            (
                "lean_check".to_owned(),
                format!(
                    "lean найден в PATH и запущен на {}; код завершения 0.",
                    lean_path.display()
                )
            ),
        ]
    );
    let _ = std::fs::remove_dir_all(&bin);
    let _ = std::fs::remove_dir_all(&units);
}

#[test]
fn the_unit_file_name_is_its_fnv1a_hash() {
    // FNV-1a 64 of the empty string is its offset basis; the JavaScript host
    // (`proverFileStem` in js/server/prover-host.mjs) names files the same way.
    assert_eq!(prover_file_stem(""), "clause_cbf29ce484222325");
}

/// Cyrillic predicate and object names in a Lean compile unit must be wrapped
/// in guillemets so Lean 4 accepts the file (issue #1188 R9172).
#[test]
fn russian_clause_lean_unit_uses_guillemets_for_cyrillic_names() {
    let clause = QuantifiedClause {
        quantifier: "forall".to_owned(),
        variable: "x".to_owned(),
        antecedent: vec![predicate("студент", None), predicate("учится", None)],
        consequent: predicate("сдаёт", Some("экзамен")),
    };
    let lean_prover = prover_records()
        .iter()
        .find(|p| p.id == "lean")
        .expect("lean prover is seeded");
    let unit = prover_unit(&clause, lean_prover).expect("lean unit renders");
    assert!(
        unit.contains("axiom «студент» : U → Prop"),
        "Cyrillic predicate must be quoted in Lean axiom: {unit}"
    );
    assert!(
        unit.contains("axiom «учится» : U → Prop"),
        "Cyrillic predicate must be quoted in Lean axiom: {unit}"
    );
    assert!(
        unit.contains("axiom «сдаёт» : U → U → Prop"),
        "Cyrillic two-place predicate must be quoted: {unit}"
    );
    assert!(
        unit.contains("axiom «экзамен» : U"),
        "Cyrillic object constant must be quoted: {unit}"
    );
    assert!(
        unit.contains("«студент» x ∧ «учится» x → «сдаёт» x «экзамен»"),
        "theorem body must use quoted identifiers: {unit}"
    );
}
