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
            recipe
                .supporting_files
                .iter()
                .any(|file| file.path == "tests/verify-output.sh" && file.source.contains(path)),
            "isolated build/check binds the same destination"
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

#[test]
fn isolated_stdout_recipe_binds_catalog_stage_and_rejects_changed_source() {
    for (language, compiler) in [
        ("Kotlin", "kotlinc Main.kt -include-runtime -d Main.jar"),
        ("Scala", "scalac Main.scala"),
        ("Rust", "rustc main.rs -o main"),
    ] {
        let prompt =
            format!("Write a {language} program that prints exactly \"Held out 43!\" and run it.");
        let recipe = UniversalSolver::default()
            .solve(&prompt)
            .execution_recipe
            .expect("owned recipe");
        assert_eq!(recipe.commands, ["sh tests/verify-output.sh"]);
        assert!(recipe.supporting_files[0].source.contains(compiler));
        let resolve = |candidate: &formal_ai::engine::ExecutionRecipe, output: &str| {
            formal_ai::program_contract::program_recovery_command(
                &prompt,
                candidate,
                &recipe.commands[0],
                output,
            )
        };
        assert_eq!(
            resolve(
                &recipe,
                "Exit code 127\n__formal_ai_owned_process_failed_0__"
            ),
            Some(compiler.to_owned())
        );
        assert_eq!(
            resolve(&recipe, "__formal_ai_owned_process_failed_99__"),
            None
        );
        assert_eq!(
            resolve(
                &recipe,
                "__formal_ai_owned_process_failed_0__\nlater unrelated output"
            ),
            None
        );
        let mut changed = (*recipe).clone();
        changed.supporting_files[0].source.push_str("\n# changed");
        assert_eq!(
            resolve(&changed, "__formal_ai_owned_process_failed_0__"),
            None
        );
    }
}
