use std::fs;

use super::repository_file;

/// Defect D14: 23 test cases that no CI job ran.
///
/// `scripts/*.rs` are rust-script programs with their own dependency
/// manifests, so `cargo test` never builds them. Two mechanisms did make some
/// of their inline `#[cfg(test)]` suites part of CI -- compiling the file into
/// the unit test crate with `#[path]`, and a gate whose `run` is
/// `rust-script --test scripts/<name>.rs` -- and four scripts had neither.
/// `check-hardcoded-language.rs`, `check-wasm-worker-size.rs`,
/// `publish-crate.rs` and `wait-for-crate.rs` carried 23 passing test cases
/// that nothing executed. All 23 pass today, which is exactly why nobody
/// noticed: the first one to break would have broken in silence.
///
/// The `--list` mode exists for this test. Asserting the selection here rather
/// than restating it means the shell and the suite cannot disagree about which
/// scripts are covered -- a hand-copied list in one of the two is how this
/// defect returns.
#[test]
fn every_standalone_script_test_suite_is_run_by_something() {
    use std::process::Command;

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");

    let compiled_into_the_test_crate: Vec<String> = {
        let mut found = Vec::new();
        let mut stack = vec![root.join("rust/tests")];
        while let Some(directory) = stack.pop() {
            for entry in fs::read_dir(&directory).expect("the test tree is readable") {
                let path = entry.expect("a readable directory entry").path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                if path.extension().is_none_or(|extension| extension != "rs") {
                    continue;
                }
                let source = fs::read_to_string(&path).expect("a readable test file");
                for line in source.lines().filter(|line| line.contains("#[path = \"")) {
                    if let Some(name) = line
                        .split("scripts/")
                        .nth(1)
                        .and_then(|tail| tail.split('"').next())
                    {
                        found.push(name.to_string());
                    }
                }
            }
        }
        found
    };

    // Both places a script's suite can be invoked with its own manifest: a gate
    // shard, and a workflow step that calls `rust-script --test` directly
    // (`question-necessity-ratchet.yml` does). Reading only the shards was a
    // false negative in this very test -- so was reading one invocation per
    // line, when `check-minimal-core-boundary.lino` chains two with `&&`.
    let run_by_a_gate: Vec<String> = {
        let mut found = Vec::new();
        let mut files = Vec::new();
        for directory in ["data/meta/ci-gates", ".github/workflows"] {
            for entry in fs::read_dir(root.join(directory)).expect("a readable directory") {
                let path = entry.expect("a readable directory entry").path();
                if path.is_file() {
                    files.push(path);
                }
            }
        }
        for path in files {
            let source = fs::read_to_string(&path).expect("a readable file");
            for invocation in source.split("rust-script --test scripts/").skip(1) {
                if let Some(name) = invocation.split(['"', ' ', '\n']).next() {
                    found.push(name.to_string());
                }
            }
        }
        found
    };

    // Both a registered --test owner and a native test-crate owner run the
    // inline tests of their #[path] modules. Match the standalone runner's two
    // consumer inventories so a native helper is not mistaken for an uncovered
    // standalone program (the changelog-files.rs regression).
    let included_by_a_gate: Vec<String> = {
        let mut found = Vec::new();
        for consumer in run_by_a_gate
            .iter()
            .chain(compiled_into_the_test_crate.iter())
        {
            let source = fs::read_to_string(root.join("scripts").join(consumer))
                .expect("a gate's test owner should be readable");
            for line in source.lines() {
                if let Some(included) = line
                    .split("#[path = \"")
                    .nth(1)
                    .and_then(|tail| tail.split('"').next())
                    .filter(|name| {
                        std::path::Path::new(name)
                            .extension()
                            .is_some_and(|extension| extension == "rs")
                    })
                {
                    found.push(
                        included
                            .rsplit("scripts/")
                            .next()
                            .unwrap_or(included)
                            .to_owned(),
                    );
                }
            }
        }
        found
    };

    let mut uncovered: Vec<String> = Vec::new();
    for entry in fs::read_dir(root.join("scripts")).expect("the scripts directory") {
        let path = entry.expect("a readable directory entry").path();
        if path.extension().is_none_or(|extension| extension != "rs") {
            continue;
        }
        let source = fs::read_to_string(&path).expect("a readable script");
        if !source.contains("cfg(test)") {
            continue;
        }
        let name = path
            .file_name()
            .expect("a file inside scripts/")
            .to_string_lossy()
            .into_owned();
        if compiled_into_the_test_crate.contains(&name)
            || run_by_a_gate.contains(&name)
            || included_by_a_gate.contains(&name)
        {
            continue;
        }
        uncovered.push(format!("scripts/{name}"));
    }
    uncovered.sort();

    let listed = Command::new(root.join("scripts/test-scripts.sh"))
        .arg("--list")
        .current_dir(root)
        .output()
        .expect("scripts/test-scripts.sh --list should run");
    assert!(
        listed.status.success(),
        "scripts/test-scripts.sh --list failed: {}",
        String::from_utf8_lossy(&listed.stderr)
    );
    let mut selected: Vec<String> = String::from_utf8_lossy(&listed.stdout)
        .lines()
        .map(str::to_string)
        .collect();
    selected.sort();

    assert_eq!(
        selected, uncovered,
        "scripts/test-scripts.sh must run exactly the inline suites the other two mechanisms \
         miss; a suite in neither list is a suite CI never runs"
    );

    let registry = repository_file("data/meta/ci-gates/test-script-suites.lino");
    assert!(
        registry.contains("run \"scripts/test-scripts.sh\""),
        "the sweep only counts if a registered gate runs it"
    );
    assert!(
        registry.contains("stage rust"),
        "the sweep needs the Rust toolchain rust-script builds with"
    );
}
