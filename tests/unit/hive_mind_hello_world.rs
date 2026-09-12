//! The Hello World ladder: the rungs Formal AI climbs before it is asked for
//! anything larger.
//!
//! The architect asked for unit, integration and end-to-end tests that use Hive
//! Mind to produce a hello world the way
//! `https://github.com/link-assistant/hive-mind/blob/main/create-test-repo.mjs`
//! does, over the top 10-20 languages, and for those runs to use *branches with
//! unique names instead of complete repositories*. The reason given was plain:
//! without a working hello world there is nothing to iterate from, and the
//! system never truly starts.
//!
//! These tests pin the parts of that ladder which live in this repository: the
//! language table, the seed programs, and the task generator that turns a row
//! into a contract the authoring action can run. The run against real GitHub is
//! the end-to-end rung and lives in the workflow; what is checked here is that
//! the run has something correct to execute.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Relative path of the language table.
const TABLE: &str = "data/meta/hello-world-languages.lino";

/// Relative path of the task generator.
const GENERATOR: &str = "scripts/hello-world-task.rs";

/// Directory holding one seed per language.
const SEEDS: &str = "examples/hello-world";

/// The exact line every program has to print.
const EXPECTED_OUTPUT: &str = "Hello, World!";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(relative: &str) -> String {
    let path = root().join(relative);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()))
        .replace("\r\n", "\n")
}

/// Value of a `key value` line inside the current `language` block, unwrapping
/// a quoted value. Canonical Links Notation has no escape for a quote inside a
/// quoted scalar, so the value simply ends at the closing quote -- which is why
/// program text lives in `examples/hello-world/` and not in the table.
fn parse_value(raw: &str) -> String {
    let raw = raw.trim();
    let Some(body) = raw.strip_prefix('"') else {
        return raw.to_string();
    };
    match body.split_once('"') {
        Some((value, _)) => value.to_string(),
        None => body.to_string(),
    }
}

/// One row of the table: `(slug, name, file, run)`.
fn languages() -> Vec<(String, String, String, String)> {
    let text = read(TABLE);
    let mut rows = Vec::new();
    let mut fields: Vec<(String, String)> = Vec::new();
    let mut inside = false;

    let push = |fields: &mut Vec<(String, String)>,
                rows: &mut Vec<(String, String, String, String)>| {
        let get = |key: &str| {
            fields
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
                .unwrap_or_default()
        };
        let slug = get("slug");
        if !slug.is_empty() {
            rows.push((slug, get("name"), get("file"), get("run")));
        }
        fields.clear();
    };

    for line in text.lines() {
        if line.trim() == "language" {
            if inside {
                push(&mut fields, &mut rows);
            }
            inside = true;
            continue;
        }
        if !inside {
            continue;
        }
        let Some(rest) = line.strip_prefix("    ") else {
            continue;
        };
        if rest.starts_with(' ') {
            continue;
        }
        if let Some((key, value)) = rest.split_once(char::is_whitespace) {
            fields.push((key.to_string(), parse_value(value)));
        }
    }
    if inside {
        push(&mut fields, &mut rows);
    }
    rows
}

/// The architect asked for the top 10-20 languages. Twenty are carried, so the
/// ladder cannot silently shrink to the three that happen to be easy.
#[test]
fn the_ladder_covers_the_top_twenty_languages() {
    let rows = languages();
    assert_eq!(
        rows.len(),
        20,
        "expected 20 languages in {TABLE}, found {}",
        rows.len()
    );
}

/// The ladder's table is data. Adding the twenty-first language must be a
/// change to `{TABLE}` and never a branch in Rust
/// (VISION.md, section 1), so neither the generator nor this test
/// names a language.
///
/// The scope is the ladder, not the whole tree: `src/coding/catalog/` still
/// holds its own language list as ~3000 lines of Rust, and its module comment
/// says adding a language means extending that constant. That is the same
/// mistake at a larger size and it is filed separately; widening this test to
/// cover it would only give the ladder a failure it cannot fix.
#[test]
fn the_ladder_itself_names_no_language_in_rust() {
    let slugs: Vec<String> = languages().into_iter().map(|row| row.0).collect();
    for source in [GENERATOR, "tests/unit/hive_mind_hello_world.rs"] {
        let text = read(source);
        // Only the behaviour is bound. Test fixtures below `mod tests` do name
        // a language or two to pin the contract's shape, and that is what a
        // fixture is for; what may not exist is a language list the program
        // consults at run time.
        let behaviour = text.split("mod tests {").next().unwrap_or(&text);
        let named: Vec<&String> = slugs
            .iter()
            .filter(|slug| behaviour.contains(&format!("\"{slug}\"")))
            .collect();
        assert!(
            named.is_empty(),
            "{source} names {} languages outside its fixtures; {TABLE} is the only place they belong: {named:?}",
            named.len()
        );
    }
}

/// Every language has a seed directory holding the program the table names, so
/// `seed:` in a generated contract always points at something that exists.
#[test]
fn every_language_has_a_seed_holding_its_program() {
    for (slug, _, file, _) in languages() {
        let path = root().join(SEEDS).join(&slug).join(&file);
        assert!(
            path.exists(),
            "{slug}: the contract seeds {SEEDS}/{slug} but {} is missing",
            path.display()
        );
        let program = fs::read_to_string(&path).expect("the seed program is readable");
        assert!(
            program.contains(EXPECTED_OUTPUT),
            "{slug}: {} does not print {EXPECTED_OUTPUT}",
            path.display()
        );
    }
}

/// The table records which programs were actually executed and which had no
/// toolchain, so a reader is never left to assume all twenty were run.
#[test]
fn the_table_records_what_was_executed() {
    let table = read(TABLE);
    assert!(
        table.contains("verified "),
        "{TABLE} does not say which programs were executed"
    );
    for slug in languages().into_iter().map(|row| row.0) {
        assert!(
            table.contains(&slug),
            "{TABLE} does not account for {slug} in its verification record"
        );
    }
}

/// Slugs name branches, so two languages may not share one.
#[test]
fn slugs_are_unique() {
    let rows = languages();
    let unique: BTreeSet<String> = rows.iter().map(|row| row.0.clone()).collect();
    assert_eq!(unique.len(), rows.len(), "duplicate slug in {TABLE}");
}

/// The generator emits a contract the resolver can parse, for every language.
/// Running it once per language is the unit rung of the ladder.
#[test]
fn the_generator_emits_a_usable_contract_for_every_language() {
    if Command::new("rust-script")
        .arg("--version")
        .output()
        .is_err()
    {
        eprintln!("rust-script is not installed; skipping the generator invocation");
        return;
    }
    for (slug, _, file, _) in languages() {
        let output = Command::new("rust-script")
            .arg(root().join(GENERATOR))
            .args(["--repo", root().to_str().expect("the root path is utf-8")])
            .args(["--language", &slug])
            .args(["--branch", &format!("hello-world/{slug}-test")])
            .output()
            .expect("the generator runs");
        assert!(
            output.status.success(),
            "{slug}: the generator failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let contract = String::from_utf8_lossy(&output.stdout);
        for required in [
            &format!("produces: {file}"),
            &format!("into: {SEEDS}/{slug}/{file}"),
            &format!("contains: {EXPECTED_OUTPUT}"),
            &format!("seed: {SEEDS}/{slug}"),
        ] {
            assert!(
                contract.contains(required.as_str()),
                "{slug}: the contract lacks `{required}`:\n{contract}"
            );
        }
    }
}

/// The branch a run writes to is unique, which is what the architect asked for
/// in place of creating a complete repository per test.
#[test]
fn runs_target_uniquely_named_branches_not_repositories() {
    let generator = read(GENERATOR);
    assert!(
        generator.contains("hello-world/"),
        "{GENERATOR} does not namespace its branches"
    );
    let plan = read("docs/case-studies/hive-mind-hello-world/PLAN.md");
    assert!(
        plan.contains("branch") && plan.contains("instead of"),
        "the plan does not record that branches replace whole repositories"
    );
}

/// The prompt carries no backslash escape anywhere in the table's run commands
/// or the generated task, because Formal AI writes `\n` into the artifact
/// literally and still reports success (issue #1116).
#[test]
fn nothing_the_agent_is_handed_carries_an_escape() {
    for (slug, _, file, run) in languages() {
        assert!(
            !run.contains('\\'),
            "{slug}: the run command carries an escape: {run}"
        );
        assert!(
            !file.contains('\\'),
            "{slug}: the file name carries an escape: {file}"
        );
    }
}
