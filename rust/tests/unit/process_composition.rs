//! Operand binding must generalize beyond default filenames and one output line.
use formal_ai::UniversalSolver;

#[test]
fn source_destination_is_bound_independently_of_literal_stdout() {
    for (language, path) in [
        ("Python", "tools/status_probe.py"),
        ("Rust", "src/probe.rs"),
        ("Kotlin", "src/Probe.kt"),
        ("Scala", "src/Probe.scala"),
    ] {
        let request = format!(
            "Create a program in {language} that prints exactly `report.py`. Save it in `{path}`."
        );
        let recipe = UniversalSolver::default()
            .solve(&request)
            .execution_recipe
            .expect("process recipe");
        assert_eq!(recipe.path, path, "{request}");
        assert!(
            recipe.source.contains("report.py"),
            "a filename-shaped value remains output"
        );
        assert!(
            recipe.commands.iter().any(|command| command.contains(path)),
            "build/check binds the same destination"
        );
    }
}

#[test]
fn ordered_literal_outputs_are_not_silently_dropped() {
    for language in ["Python", "Rust", "Kotlin", "Scala"] {
        let request = format!(
            "Create a program in {language}. Print exactly `North 11!`. Then print exactly `South 29!`."
        );
        let recipe = UniversalSolver::default()
            .solve(&request)
            .execution_recipe
            .expect("process recipe");
        assert!(recipe.source.contains("North 11!"));
        assert!(
            recipe.source.contains("South 29!"),
            "second output obligation lost in {language}"
        );
        let verifier = recipe
            .supporting_files
            .iter()
            .find(|file| file.path == "tests/verify-output.sh")
            .unwrap();
        assert!(
            verifier.source.contains("North 11!\nSouth 29!"),
            "verifier covers the ordered output contract"
        );
    }
}
