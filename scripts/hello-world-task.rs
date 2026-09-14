#!/usr/bin/env rust-script
//! Generate the Formal AI task contract for one Hello World language.
//!
//! The architect asked for tests that drive Hive Mind to write a hello world in
//! the top 10-20 languages, and for those tests to use *uniquely named branches
//! instead of whole repositories*, so an authoring run can be pointed at a
//! branch of this repository rather than at a scratch repository that is thrown
//! away. This script is the piece that turns one row of
//! `data/meta/hello-world-languages.lino` into the contract
//! `.github/actions/author-with-formal-ai/scripts/resolve-formal-ai-task.sh`
//! parses:
//!
//!   task: ...
//!   seed: ...
//!   produces: ...
//!   into: ...
//!   contains: ...
//!   message: ...
//!
//! The language table is data, so adding the twenty-first language is a data
//! change and never a branch in this file (VISION.md, section 1).
//!
//! Two rules the emitted contract has to respect, both paid for in defects:
//!
//! * Values are emitted bare. The parser reads `sed -n 's/^contains: *//p'`,
//!   which keeps whatever follows -- quoting `contains: "x"` makes it look for
//!   a literal quote (issue #1117).
//! * The `task:` line carries no backslash escapes. Formal AI emits `\n` and
//!   `\"` literally into the artifact and still reports success (issue #1116),
//!   so the prompt describes the program in words and the expected output is
//!   checked by `contains:`.
//!
//! Usage:
//!   rust-script scripts/hello-world-task.rs --language rust [--branch <name>]
//!   rust-script scripts/hello-world-task.rs --list
//!   rust-script --test scripts/hello-world-task.rs
//!
//! ```cargo
//! [package]
//! edition = "2021"
//! ```

#[cfg(not(test))]
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(not(test))]
use std::process::exit;

/// Relative path of the language table, from the repository root.
const TABLE_PATH: &str = "data/meta/hello-world-languages.lino";

/// The exact line every generated program has to print.
const EXPECTED_OUTPUT: &str = "Hello, World!";

/// One row of the language table.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Language {
    /// Display name, e.g. `C#`.
    name: String,
    /// Stable identifier used in branch names and on the command line.
    slug: String,
    /// File the program is written to, e.g. `hello.rb`.
    file: String,
    /// Command that runs the file from its own directory.
    run: String,
}

/// Read the value of a `key value` line, unwrapping a quoted value.
///
/// Canonical Links Notation has no escape for a quote inside a quoted scalar,
/// which is why no program text lives in the table: the value ends at the first
/// closing quote and nothing in it needs unescaping.
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

/// Parse the language table.
fn parse_table(text: &str) -> Vec<Language> {
    let mut languages: Vec<Language> = Vec::new();
    let mut fields: Vec<(String, String)> = Vec::new();
    let mut inside = false;

    let flush = |fields: &mut Vec<(String, String)>, languages: &mut Vec<Language>| {
        let get = |key: &str| -> String {
            fields
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
                .unwrap_or_default()
        };
        let slug = get("slug");
        if !slug.is_empty() {
            languages.push(Language {
                name: get("name"),
                slug,
                file: get("file"),
                run: get("run"),
            });
        }
        fields.clear();
    };

    for line in text.lines() {
        if line.trim() == "language" {
            if inside {
                flush(&mut fields, &mut languages);
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
        let Some((key, value)) = rest.split_once(char::is_whitespace) else {
            continue;
        };
        fields.push((key.to_string(), parse_value(value)));
    }
    if inside {
        flush(&mut fields, &mut languages);
    }
    languages
}

/// Branch a run writes to when none is given.
///
/// Unique per language and per run, because the architect asked for branches
/// with unique names in place of throwaway repositories: two runs for the same
/// language must not collide, and a failed branch stays readable next to the
/// one that replaced it.
fn default_branch(slug: &str, suffix: &str) -> String {
    format!("hello-world/{slug}-{suffix}")
}

/// The task contract for one language.
fn contract(language: &Language, branch: &str) -> String {
    let Language {
        name,
        slug,
        file,
        run,
        ..
    } = language;
    let directory = format!("examples/hello-world/{slug}");
    // Described in words, never as escaped source: issue #1116.
    let task = format!(
        "Write a {name} program in the file `{file}`. Running it with `{run}` \
         must print the single line {EXPECTED_OUTPUT} and nothing else. \
         Write only that one file."
    );
    format!(
        "task: {task}\n\
         seed: {directory}\n\
         produces: {file}\n\
         into: {directory}/{file}\n\
         contains: {EXPECTED_OUTPUT}\n\
         message: feat(hello-world): {name} prints {EXPECTED_OUTPUT}\n\
         branch: {branch}\n"
    )
}

/// Locate the repository root from this script's own position.
fn repo_root() -> PathBuf {
    Path::new(file!())
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(not(test))]
fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut root = repo_root();
    let mut language: Option<String> = None;
    let mut branch: Option<String> = None;
    let mut list = false;

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--language" | "-l" => {
                index += 1;
                language = args.get(index).cloned();
            }
            "--branch" | "-b" => {
                index += 1;
                branch = args.get(index).cloned();
            }
            "--repo" => {
                index += 1;
                if let Some(value) = args.get(index) {
                    root = PathBuf::from(value);
                }
            }
            "--list" => list = true,
            other => {
                eprintln!("hello-world-task: unknown option: {other}");
                exit(2);
            }
        }
        index += 1;
    }

    let path = root.join(TABLE_PATH);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("hello-world-task: cannot read {}: {error}", path.display());
            exit(1);
        }
    };
    let languages = parse_table(&text);

    if list {
        for entry in &languages {
            println!("{}\t{}", entry.slug, entry.name);
        }
        return;
    }

    let Some(slug) = language else {
        eprintln!("hello-world-task: --language is required (try --list)");
        exit(2);
    };
    let Some(entry) = languages.iter().find(|entry| entry.slug == slug) else {
        eprintln!("hello-world-task: no language with slug {slug} in {TABLE_PATH}");
        exit(1);
    };
    let suffix = env::var("HELLO_WORLD_BRANCH_SUFFIX").unwrap_or_else(|_| {
        // Wall-clock seconds are enough: one run generates one branch.
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs().to_string())
            .unwrap_or_else(|_| "0".to_string())
    });
    let branch = branch.unwrap_or_else(|| default_branch(&entry.slug, &suffix));
    print!("{}", contract(entry, &branch));
}

#[cfg(test)]
fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    fn table() -> Vec<Language> {
        let path = repo_root().join(TABLE_PATH);
        parse_table(&fs::read_to_string(path).expect("the language table is committed"))
    }

    /// The architect asked for the top 10-20 languages; the table carries 20.
    #[test]
    fn the_table_covers_twenty_languages() {
        let languages = table();
        assert_eq!(languages.len(), 20, "expected 20 languages, got {languages:?}");
    }

    /// Every row is complete: a missing `run` would generate a contract whose
    /// verification cannot be performed.
    #[test]
    fn every_language_is_fully_specified() {
        for entry in table() {
            for (field, value) in [
                ("name", &entry.name),
                ("slug", &entry.slug),
                ("file", &entry.file),
                ("run", &entry.run),
            ] {
                assert!(!value.is_empty(), "{} has an empty {field}", entry.slug);
            }
        }
    }

    /// A quoted value ends at its closing quote and carries no escape, because
    /// canonical Links Notation has none. The program text that would need one
    /// lives in `examples/hello-world/` instead.
    #[test]
    fn quoted_values_end_at_the_closing_quote() {
        assert_eq!(parse_value(r#""python3 hello.py""#), "python3 hello.py");
        for entry in table() {
            assert!(
                !entry.run.contains('\\') && !entry.file.contains('\\'),
                "{} carries a backslash the notation cannot express",
                entry.slug
            );
        }
    }

    /// Every language has a committed program that prints the expected output,
    /// so a contract's `contains:` line and the seed agree.
    #[test]
    fn every_language_has_a_seed_that_prints_the_expected_output() {
        for entry in table() {
            let path = repo_root()
                .join("examples/hello-world")
                .join(&entry.slug)
                .join(&entry.file);
            let program = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
            assert!(
                program.contains(EXPECTED_OUTPUT),
                "{} does not print {EXPECTED_OUTPUT}: {}",
                entry.slug,
                path.display()
            );
        }
    }

    /// Slugs are unique, because they name branches.
    #[test]
    fn slugs_are_unique() {
        let mut slugs: Vec<String> = table().into_iter().map(|entry| entry.slug).collect();
        slugs.sort();
        let before = slugs.len();
        slugs.dedup();
        assert_eq!(before, slugs.len(), "duplicate slug in {TABLE_PATH}");
    }

    /// The contract carries every line the resolver requires.
    #[test]
    fn the_contract_carries_every_required_line() {
        let entry = table()
            .into_iter()
            .find(|entry| entry.slug == "rust")
            .expect("rust is in the table");
        let text = contract(&entry, "hello-world/rust-123");
        for required in ["task: ", "seed: ", "produces: ", "into: ", "message: "] {
            assert!(text.contains(required), "contract lacks {required}:\n{text}");
        }
        assert_eq!(
            text.lines().filter(|line| line.starts_with("produces: ")).count(),
            text.lines().filter(|line| line.starts_with("into: ")).count(),
            "the resolver rejects unpaired produces/into lines"
        );
    }

    /// Values are emitted bare. `contains: "Hello, World!"` would make the run
    /// search the artifact for a literal quote character (issue #1117).
    #[test]
    fn contract_values_are_not_quoted() {
        for entry in table() {
            let text = contract(&entry, "hello-world/x-1");
            for line in text.lines() {
                let Some((_, value)) = line.split_once(": ") else {
                    continue;
                };
                assert!(
                    !value.starts_with('"'),
                    "{} emitted a quoted value, which the resolver keeps verbatim: {line}",
                    entry.slug
                );
            }
        }
    }

    /// The prompt carries no backslash escape. Formal AI writes `\n` and `\"`
    /// into the artifact literally and still reports success (issue #1116), so
    /// the program is described rather than dictated.
    #[test]
    fn the_prompt_carries_no_backslash_escape() {
        for entry in table() {
            let text = contract(&entry, "hello-world/x-1");
            let task = text
                .lines()
                .find(|line| line.starts_with("task: "))
                .expect("every contract has a task line");
            assert!(
                !task.contains('\\'),
                "{} put an escape in the prompt: {task}",
                entry.slug
            );
        }
    }

    /// Branches are unique per language and per run: the architect asked for
    /// uniquely named branches in place of throwaway repositories.
    #[test]
    fn default_branches_are_unique_per_language_and_run() {
        assert_ne!(default_branch("rust", "1"), default_branch("rust", "2"));
        assert_ne!(default_branch("rust", "1"), default_branch("go", "1"));
        assert!(default_branch("rust", "1").starts_with("hello-world/"));
    }
}
