#!/usr/bin/env rust-script
//! Regenerate `data/seed/program-cst-grammars.lino` from the grammar crates
//! the resolved `meta-language` dependency actually ships (issue #1167, R2).
//!
//! The seed's grammar entries are generated, never hand-maintained: this
//! script reads `cargo metadata` for `rust/Cargo.toml`, takes the
//! `tree-sitter-*` dependencies of the resolved `meta-language` package as
//! the shipped grammar set (the crate exposes no public language list;
//! `language_parser.rs` only has `BuiltInLanguageParser`), maps crate names
//! to catalog slugs, and writes one `cst_grammar` block per slug in sorted
//! order.
//!
//! Which slugs deserve a block: every language
//! `data/meta/hello-world-languages.lino` declares **and** every slug a
//! previous revision of the seed already registered (so an upstream grammar
//! can grow the file but never silently drop an entry whose grammar still
//! ships), intersected with the shipped set — `c` and `php` carry entries
//! without a Hello World, and Hello World languages with no shipped grammar
//! (haskell, elixir, clojure, fsharp, ocaml, erlang, julia) honestly carry
//! none.
//!
//! Usage:
//!   rust-script scripts/generate-program-cst-grammars.rs            # --check
//!   rust-script scripts/generate-program-cst-grammars.rs --write
//!
//! `--check` (the default) exits non-zero naming the drift when the
//! committed file differs from the generated one, so CI can pin the file to
//! the dependency the same way the boundary ledger pins the tree. If a
//! public language list is wanted upstream, file it in
//! link-foundation/meta-language (general wording).

```cargo
[package]
edition = "2024"

[dependencies]
serde_json = "1"
```

use std::collections::BTreeSet;
use std::process::Command;

/// Crate name → catalog slug for every grammar crate meta-language has ever
/// shipped. `-ng` successors keep their language's slug; a crate with no row
/// here is not a program-language grammar this seed tracks (yet) and is
/// skipped with a printed note rather than guessed.
const CRATE_TO_SLUG: &[(&str, &str)] = &[
    ("tree-sitter-c", "c"),
    ("tree-sitter-c-sharp", "csharp"),
    ("tree-sitter-cpp", "cpp"),
    ("tree-sitter-go", "go"),
    ("tree-sitter-java", "java"),
    ("tree-sitter-javascript", "javascript"),
    ("tree-sitter-kotlin-ng", "kotlin"),
    ("tree-sitter-php", "php"),
    ("tree-sitter-python", "python"),
    ("tree-sitter-r", "r"),
    ("tree-sitter-ruby", "ruby"),
    ("tree-sitter-rust", "rust"),
    ("tree-sitter-scala", "scala"),
    ("tree-sitter-swift", "swift"),
    ("tree-sitter-typescript", "typescript"),
];

const SEED_PATH: &str = "data/seed/program-cst-grammars.lino";
const EMBEDDED_MIRROR: &str = "rust/embedded/data/seed/program-cst-grammars.lino";
const HELLO_WORLD_PATH: &str = "data/meta/hello-world-languages.lino";

fn main() {
    let write = std::env::args().any(|arg| arg == "--write");
    let shipped = shipped_grammar_slugs();
    let declared = hello_world_slugs();
    let registered = registered_slugs();
    let mut wanted: BTreeSet<String> = BTreeSet::new();
    wanted.extend(declared.iter().filter(|slug| shipped.contains(*slug)).cloned());
    wanted.extend(registered.iter().filter(|slug| shipped.contains(*slug)).cloned());
    let generated = render(&wanted);
    let committed = std::fs::read_to_string(SEED_PATH)
        .unwrap_or_else(|error| panic!("read {SEED_PATH}: {error}"));
    if committed == generated {
        println!(
            "program-cst-grammars: {} entries, matches the shipped grammar set",
            wanted.len()
        );
        return;
    }
    if !write {
        eprintln!(
            "program-cst-grammars drift: the committed file does not match the {} grammar \
             entries the resolved meta-language ships; run `rust-script \
             scripts/generate-program-cst-grammars.rs --write`",
            wanted.len()
        );
        std::process::exit(1);
    }
    std::fs::write(SEED_PATH, &generated)
        .unwrap_or_else(|error| panic!("write {SEED_PATH}: {error}"));
    std::fs::write(EMBEDDED_MIRROR, &generated)
        .unwrap_or_else(|error| panic!("write {EMBEDDED_MIRROR}: {error}"));
    println!(
        "program-cst-grammars: wrote {} entries to {SEED_PATH} and the embedded mirror",
        wanted.len()
    );
}

/// The `tree-sitter-*` dependency names of the resolved `meta-language`
/// packages, mapped to catalog slugs. `cargo metadata` resolves the graph
/// without building anything.
fn shipped_grammar_slugs() -> BTreeSet<String> {
    let output = Command::new("cargo")
        .args([
            "metadata",
            "--format-version",
            "1",
            "--manifest-path",
            "rust/Cargo.toml",
        ])
        .output()
        .expect("cargo metadata runs (resolution only, no build)");
    if !output.status.success() {
        panic!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let metadata: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("cargo metadata emits JSON");
    let packages = metadata["packages"]
        .as_array()
        .expect("metadata lists packages");
    let mut slugs = BTreeSet::new();
    for package in packages {
        if package["name"] != "meta-language" {
            continue;
        }
        let Some(dependencies) = package["dependencies"].as_array() else {
            continue;
        };
        for dependency in dependencies {
            let Some(name) = dependency["name"].as_str() else {
                continue;
            };
            match CRATE_TO_SLUG.iter().find(|(crate_name, _)| *crate_name == name) {
                Some((_, slug)) => {
                    slugs.insert((*slug).to_owned());
                }
                None if name.starts_with("tree-sitter") => {
                    println!("note: shipped grammar crate {name} has no slug mapping yet");
                }
                _ => {}
            }
        }
    }
    slugs
}

/// The slugs `data/meta/hello-world-languages.lino` declares, read with the
/// same indentation-tree walk the seed loaders use: records nest one wrapper
/// level below the head, so `slug` values sit under each `language` child.
fn hello_world_slugs() -> BTreeSet<String> {
    let text = std::fs::read_to_string(HELLO_WORLD_PATH)
        .unwrap_or_else(|error| panic!("read {HELLO_WORLD_PATH}: {error}"));
    text.lines()
        .filter(|line| line.trim_start().starts_with("slug "))
        .filter_map(|line| line.trim().strip_prefix("slug "))
        .map(str::trim)
        .map(str::to_owned)
        .collect()
}

/// The slugs the committed seed already registers, so regeneration can grow
/// the file but never drop a still-shipped entry.
fn registered_slugs() -> BTreeSet<String> {
    let text = std::fs::read_to_string(SEED_PATH)
        .unwrap_or_else(|error| panic!("read {SEED_PATH}: {error}"));
    text.lines()
        .filter_map(|line| line.trim().strip_prefix("cst_grammar "))
        .map(|slug| slug.trim_matches('"').to_owned())
        .collect()
}

/// The whole file: a stable header (the description prose names the
/// generation rule) followed by one uniform block per slug, sorted.
fn render(slugs: &BTreeSet<String>) -> String {
    let mut out = String::new();
    out.push_str("program_cst_grammars\n");
    out.push_str(
        "  description \"CST/AST engine metadata for generated programs. The sole engine is the \
         link-foundation meta-language links network (meta_language::LinkNetwork), a single \
         mutable links-network representation that ships real tree-sitter grammars for every \
         target language (C, C++, C#, Go, Java, JavaScript, Kotlin, PHP, Python, R, Ruby, Rust, \
         Scala, Swift, TypeScript). Coding handlers reason from a semantic plan, render concrete \
         source, and then validate that source through meta-language before accepting it — code \
         text is never treated as opaque output. The entries below are generated by \
         scripts/generate-program-cst-grammars.rs from the grammar crates the resolved \
         meta-language dependency actually ships, intersected with the languages \
         data/meta/hello-world-languages.lino declares plus every slug a previous revision \
         already registered; --check fails when the committed file drifts from that set.\"\n",
    );
    out.push_str("  component \"meta-language\"\n");
    out.push_str(
        "  component_repository \"https://github.com/link-foundation/meta-language\"\n",
    );
    for slug in slugs {
        out.push_str(&format!("  cst_grammar \"{slug}\"\n"));
        out.push_str(&format!("    program_language \"{slug}\"\n"));
        out.push_str("    engine \"meta_language\"\n");
        out.push_str(&format!("    meta_language_label \"{slug}\"\n"));
        out.push_str(
            "    source_repository \"https://github.com/link-foundation/meta-language\"\n",
        );
    }
    out
}
