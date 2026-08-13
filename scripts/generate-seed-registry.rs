#!/usr/bin/env rust-script
//! Generate the seed inventory every production path shares.
//!
//! Issue #991: `src/seed/embedded.rs` (27 manual conflict resolutions) and
//! `src/web/seed_loader.js` (14) each carried the same list of `data/seed/*.lino`
//! files, hand-ordered, so every branch that added a seed file appended to the
//! same lines of both. The list now lives once in
//! `data/meta/seed-registry.lino`, sorted by name and `merge=union`: two branches
//! adding a seed file produce two `seed` blocks, the union keeps both, and this
//! generator restores the order and rewrites the two generated files.
//!
//! Generated files:
//!   * `src/seed/embedded_registry.rs` -- `include_str!` constants,
//!     `seed_files()`, `RESPONSE_FILES`, `MEANING_FILES`. It is `include!`d by
//!     `src/seed/embedded.rs` rather than declared as a module, because rustfmt
//!     only formats files it reaches through a `mod` declaration: the generator
//!     therefore owns the layout outright and `cargo fmt --check` has nothing to
//!     say about it.
//!   * `src/web/seed-files.js` -- the list the browser worker fetches
//!
//! Usage:
//!   rust-script scripts/generate-seed-registry.rs           # verify
//!   rust-script scripts/generate-seed-registry.rs --write   # regenerate
//!
//! ```cargo
//! [dependencies]
//! ```

use std::collections::BTreeSet;
use std::fs;
#[cfg(not(test))]
use std::path::Path;

#[cfg(not(test))]
const REGISTRY: &str = "data/meta/seed-registry.lino";
const SEED_DIR: &str = "data/seed";
#[cfg(not(test))]
const RUST_TARGET: &str = "src/seed/embedded_registry.rs";
#[cfg(not(test))]
const WEB_TARGET: &str = "src/web/seed-files.js";

/// The column the generated Rust wraps at, matching rustfmt's default
/// `max_width` so the file reads like the rest of the crate even though rustfmt
/// never sees it.
const MAX_WIDTH: usize = 100;

/// One `data/seed/*.lino` file and the paths that consume it.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Seed {
    name: String,
    /// Embedded in the binary and part of the merged bundle `seed_files()`.
    bundle: bool,
    /// `meaning` joins `MEANING_FILES`, `response` joins `RESPONSE_FILES`.
    lexicon: BTreeSet<String>,
    /// Fetched by the browser worker at startup.
    web: bool,
}

impl Seed {
    /// A file gets an `include_str!` constant when some Rust path reads it.
    fn embedded(&self) -> bool {
        self.bundle || !self.lexicon.is_empty()
    }

    fn constant(&self) -> String {
        format!("{}_LINO", self.name.to_uppercase().replace('-', "_"))
    }

    fn path(&self) -> String {
        format!("{SEED_DIR}/{}.lino", self.name)
    }
}

/// A seed file the registry deliberately leaves out, and who owns it instead.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Unregistered {
    pattern: String,
    owner: String,
    reason: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Registry {
    seeds: Vec<Seed>,
    unregistered: Vec<Unregistered>,
}

fn unquote(value: &str) -> String {
    let trimmed = value.trim();
    trimmed
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or(trimmed)
        .to_string()
}

fn indent_of(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

fn parse_registry(source: &str) -> Registry {
    let mut registry = Registry::default();
    // Which indent-2 block the indented lines below belong to.
    let mut in_seed = false;

    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let indent = indent_of(line);
        let (key, value) = match trimmed.split_once(' ') {
            Some((key, value)) => (key, value.trim()),
            None => (trimmed, ""),
        };
        match (indent, key) {
            (2, "seed") => {
                registry.seeds.push(Seed {
                    name: unquote(value),
                    ..Seed::default()
                });
                in_seed = true;
            }
            (2, "unregistered") => {
                registry.unregistered.push(Unregistered {
                    pattern: unquote(value),
                    ..Unregistered::default()
                });
                in_seed = false;
            }
            (2, _) => in_seed = false,
            (4, _) if in_seed => {
                if let Some(seed) = registry.seeds.last_mut() {
                    match key {
                        "bundle" => seed.bundle = unquote(value) == "true",
                        "web" => seed.web = unquote(value) == "true",
                        "lexicon" => {
                            seed.lexicon.insert(unquote(value));
                        }
                        _ => {}
                    }
                }
            }
            (4, _) => {
                if let Some(entry) = registry.unregistered.last_mut() {
                    match key {
                        "owner" => entry.owner = unquote(value),
                        "reason" => entry.reason = unquote(value),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    registry
}

/// Merge the repeats a union merge leaves behind and restore the sorted order.
///
/// A union of two branches that each added a flag to the same seed leaves two
/// `seed` blocks with the same name. Both were true before the merge, so the
/// union of their flags is the answer; dropping either would lose a branch's
/// change.
fn canonicalize(mut registry: Registry) -> Registry {
    registry.seeds.sort_by(|left, right| left.name.cmp(&right.name));
    let mut merged: Vec<Seed> = Vec::with_capacity(registry.seeds.len());
    for seed in registry.seeds {
        match merged.last_mut() {
            Some(previous) if previous.name == seed.name => {
                previous.bundle |= seed.bundle;
                previous.web |= seed.web;
                previous.lexicon.extend(seed.lexicon);
            }
            _ => merged.push(seed),
        }
    }
    registry.seeds = merged;
    registry
        .unregistered
        .sort_by(|left, right| left.pattern.cmp(&right.pattern));
    registry.unregistered.dedup_by(|left, right| left.pattern == right.pattern);
    registry
}

/// `*` matches any run of characters; the patterns here name seed files, which
/// have no directory component, so there is nothing for it to stop at.
fn pattern_matches(pattern: &str, name: &str) -> bool {
    match pattern.split_once('*') {
        Some((head, tail)) => {
            let Some(rest) = name.strip_prefix(head) else {
                return false;
            };
            (0..=rest.len())
                .any(|split| rest.is_char_boundary(split) && pattern_matches(tail, &rest[split..]))
        }
        None => pattern == name,
    }
}

fn render_registry(registry: &Registry, header: &str) -> String {
    // No blank line between the header and the root: canonical Links Notation
    // ends the document at the first blank line after a comment block, so a
    // separator here makes `tests/unit/data_files.rs` reject the whole file.
    let mut lines = vec![header.trim_end().to_string(), "seed_registry".to_string()];
    for seed in &registry.seeds {
        lines.push(format!("  seed {}", seed.name));
        if seed.bundle {
            lines.push("    bundle true".to_string());
        }
        for lexicon in &seed.lexicon {
            lines.push(format!("    lexicon {lexicon}"));
        }
        if seed.web {
            lines.push("    web true".to_string());
        }
    }
    for entry in &registry.unregistered {
        lines.push(format!("  unregistered {}", entry.pattern));
        lines.push(format!("    owner \"{}\"", entry.owner));
        lines.push(format!("    reason \"{}\"", entry.reason));
    }
    format!("{}\n", lines.join("\n"))
}

/// The comment block above `seed_registry`, kept verbatim through a rewrite.
fn registry_header(source: &str) -> String {
    source
        .lines()
        .take_while(|line| line.trim().is_empty() || line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
}

const RUST_HEADER: &str = r#"// Embedded Links Notation seed files and the file registry.
//
// Generated by `rust-script scripts/generate-seed-registry.rs --write` from
// `data/meta/seed-registry.lino`. Do not edit by hand: issue #991 moved this
// inventory out of hand-written lists because two branches adding a seed file
// always appended to the same lines here, and the registry is `merge=union` so
// that never happens again.
//
// `src/seed/embedded.rs` pulls this in with `include!`, which keeps it outside
// rustfmt's reach -- the generator owns every byte, so a regenerated file is
// byte-identical whatever rustfmt would have preferred.
"#;

fn render_rust(registry: &Registry) -> String {
    let embedded: Vec<&Seed> = registry.seeds.iter().filter(|seed| seed.embedded()).collect();
    let mut out = String::from(RUST_HEADER);

    out.push_str("\n/// Raw embedded contents (used by `merged_bundle` and by tests).\n");
    for seed in &embedded {
        let one_line = format!(
            "pub const {}: &str = include_str!(\"../../{}\");",
            seed.constant(),
            seed.path()
        );
        if one_line.len() <= MAX_WIDTH {
            out.push_str(&one_line);
        } else {
            out.push_str(&format!(
                "pub const {}: &str =\n    include_str!(\"../../{}\");",
                seed.constant(),
                seed.path()
            ));
        }
        out.push('\n');
    }

    out.push_str(
        "\n/// Embedded copy of every Links Notation seed file. Returned in registry\n\
         /// order so callers can render the merged bundle deterministically.\n\
         #[must_use]\n\
         pub fn seed_files() -> Vec<(&'static str, &'static str)> {\n    vec![\n",
    );
    for seed in embedded.iter().filter(|seed| seed.bundle) {
        let one_line = format!("        (\"{}\", {}),", seed.path(), seed.constant());
        if one_line.len() <= MAX_WIDTH {
            out.push_str(&one_line);
            out.push('\n');
        } else {
            out.push_str(&format!(
                "        (\n            \"{}\",\n            {},\n        ),\n",
                seed.path(),
                seed.constant()
            ));
        }
    }
    out.push_str("    ]\n}\n");

    out.push_str(
        "\n/// The registered set of multilingual-response files, walked by\n\
         /// [`super::multilingual_responses`].\n\
         ///\n\
         /// Split so none breaches the seed file-size guard; each wraps its records\n\
         /// under a top-level `multilingual_responses` node and the parser walks all of\n\
         /// them, so an intent may live in whichever file keeps the sizes balanced.\n\
         pub const RESPONSE_FILES: &[&str] = &[\n",
    );
    for seed in embedded.iter().filter(|seed| seed.lexicon.contains("response")) {
        out.push_str(&format!("    {},\n", seed.constant()));
    }
    out.push_str("];\n");

    out.push_str(
        "\n/// The registered set of meaning-lexicon files, concatenated by\n\
         /// [`super::lexicon`].\n\
         ///\n\
         /// Split across several `.lino` files so none breaches the seed file-size\n\
         /// guard; each wraps its records under a top-level `meanings` node (the loader\n\
         /// walks all of them).\n\
         pub const MEANING_FILES: &[&str] = &[\n",
    );
    for seed in embedded.iter().filter(|seed| seed.lexicon.contains("meaning")) {
        out.push_str(&format!("    {},\n", seed.constant()));
    }
    out.push_str("];\n");
    out
}

fn render_web(registry: &Registry) -> String {
    let mut out = String::from(
        "// Generated by `rust-script scripts/generate-seed-registry.rs --write` from\n\
         // data/meta/seed-registry.lino. Do not edit by hand.\n\
         //\n\
         // Issue #991: this list used to live inside `seed_loader.js`, where every\n\
         // branch that added a seed file appended to the same lines. It is the browser\n\
         // half of the one inventory `src/seed/embedded.rs` is generated from, so the\n\
         // worker and the Rust engine can never drift apart about which files exist.\n\
         \n\
         self.FORMAL_AI_SEED_FILES = Object.freeze([\n",
    );
    for seed in registry.seeds.iter().filter(|seed| seed.web) {
        out.push_str(&format!("  \"seed/{}.lino\",\n", seed.name));
    }
    out.push_str("]);\n");
    out
}

/// Everything the registry has to satisfy, as human-readable failures.
fn problems(registry: &Registry, on_disk: &BTreeSet<String>) -> Vec<String> {
    let mut failures = Vec::new();
    let registered: BTreeSet<&str> = registry.seeds.iter().map(|seed| seed.name.as_str()).collect();

    for seed in &registry.seeds {
        if !on_disk.contains(&seed.name) {
            failures.push(format!(
                "`{}` is registered but `{}` does not exist",
                seed.name,
                seed.path()
            ));
        }
        if !seed.embedded() && !seed.web {
            failures.push(format!(
                "`{}` is registered with no consumer: give it `bundle`, `lexicon` or `web`, \
                 or move it to an `unregistered` entry",
                seed.name
            ));
        }
        for lexicon in &seed.lexicon {
            if lexicon != "meaning" && lexicon != "response" {
                failures.push(format!(
                    "`{}` declares lexicon `{lexicon}`; the lexicons are `meaning` and `response`",
                    seed.name
                ));
            }
        }
    }

    for entry in &registry.unregistered {
        if entry.owner.trim().is_empty() || entry.reason.trim().is_empty() {
            failures.push(format!(
                "unregistered `{}` names no owner or no reason; an exclusion without one \
                 is an omission, not a decision",
                entry.pattern
            ));
        }
        if !on_disk
            .iter()
            .any(|name| pattern_matches(&entry.pattern, name))
        {
            failures.push(format!(
                "unregistered `{}` matches no file in {SEED_DIR}; drop the stale exclusion",
                entry.pattern
            ));
        }
    }

    for name in on_disk {
        if registered.contains(name.as_str()) {
            continue;
        }
        if registry
            .unregistered
            .iter()
            .any(|entry| pattern_matches(&entry.pattern, name))
        {
            continue;
        }
        failures.push(format!(
            "`{SEED_DIR}/{name}.lino` exists but the registry neither registers nor excludes it; \
             add a `seed {name}` block naming which paths load it"
        ));
    }
    failures
}

#[cfg(not(test))]
fn seed_names_on_disk(root: &Path) -> BTreeSet<String> {
    let Ok(entries) = fs::read_dir(root.join(SEED_DIR)) else {
        return BTreeSet::new();
    };
    entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            name.strip_suffix(".lino").map(str::to_string)
        })
        .collect()
}

#[cfg(not(test))]
fn main() {
    let write = std::env::args().any(|argument| argument == "--write");
    let root = std::env::current_dir().expect("Failed to get current directory");
    let source = fs::read_to_string(root.join(REGISTRY)).unwrap_or_else(|error| {
        println!("::error::Could not read {REGISTRY}: {error}");
        std::process::exit(1);
    });

    let header = registry_header(&source);
    let registry = canonicalize(parse_registry(&source));
    let on_disk = seed_names_on_disk(&root);

    println!("\nChecking the seed registry in {REGISTRY}...\n");
    println!(
        "  {} registered, {} embedded, {} in the browser worker, {} on disk\n",
        registry.seeds.len(),
        registry.seeds.iter().filter(|seed| seed.embedded()).count(),
        registry.seeds.iter().filter(|seed| seed.web).count(),
        on_disk.len()
    );

    let failures = problems(&registry, &on_disk);
    if !failures.is_empty() {
        for failure in &failures {
            println!("::error::{failure}");
        }
        println!("\n{} seed registry violation(s). Update {REGISTRY}.\n", failures.len());
        std::process::exit(1);
    }

    let targets = [
        (REGISTRY, render_registry(&registry, &header)),
        (RUST_TARGET, render_rust(&registry)),
        (WEB_TARGET, render_web(&registry)),
    ];
    let mut stale = Vec::new();
    for (path, expected) in &targets {
        let actual = fs::read_to_string(root.join(path)).unwrap_or_default();
        if &actual == expected {
            continue;
        }
        if write {
            fs::write(root.join(path), expected).unwrap_or_else(|error| {
                println!("::error::Could not write {path}: {error}");
                std::process::exit(1);
            });
            println!("  wrote {path}");
        } else {
            stale.push(*path);
        }
    }

    if stale.is_empty() {
        println!("\nThe seed registry and every file generated from it agree.\n");
        return;
    }
    for path in &stale {
        println!(
            "::error::{path} does not match {REGISTRY}. Run \
             `rust-script scripts/generate-seed-registry.rs --write`"
        );
    }
    std::process::exit(1);
}

#[cfg(test)]
fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "# header\n\nseed_registry\n  \
        seed concepts\n    bundle true\n    web true\n  \
        seed meanings-units\n    bundle true\n    lexicon meaning\n  \
        unregistered closure-generated-*\n    owner \"scripts/close-total.py\"\n    \
        reason \"Derived from the other seed files.\"\n";

    fn sample() -> Registry {
        canonicalize(parse_registry(SAMPLE))
    }

    fn on_disk() -> BTreeSet<String> {
        ["concepts", "meanings-units", "closure-generated-01"]
            .into_iter()
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn a_registry_parses_into_seeds_and_exclusions() {
        let registry = sample();
        assert_eq!(registry.seeds.len(), 2);
        assert!(registry.seeds[0].bundle && registry.seeds[0].web);
        assert!(registry.seeds[1].lexicon.contains("meaning"));
        assert_eq!(registry.unregistered.len(), 1);
        assert!(problems(&registry, &on_disk()).is_empty());
    }

    #[test]
    fn a_union_merge_leaves_repeats_that_canonicalize_merges() {
        // Two branches each add a flag to the same seed. The union keeps both
        // blocks; dropping either would lose a branch's change.
        let unioned = format!("{SAMPLE}  seed concepts\n    lexicon response\n");
        let registry = canonicalize(parse_registry(&unioned));
        assert_eq!(registry.seeds.len(), 2, "the repeat is merged, not kept");
        assert!(registry.seeds[0].bundle, "the first branch's flag survives");
        assert!(
            registry.seeds[0].lexicon.contains("response"),
            "the second branch's flag survives"
        );
    }

    #[test]
    fn a_union_merge_leaves_the_order_scrambled_and_the_output_sorted() {
        let unioned = "seed_registry\n  seed zebra\n    web true\n  seed alpha\n    web true\n";
        let registry = canonicalize(parse_registry(unioned));
        let names: Vec<&str> = registry.seeds.iter().map(|seed| seed.name.as_str()).collect();
        assert_eq!(names, ["alpha", "zebra"]);
    }

    #[test]
    fn a_seed_file_nobody_registered_fails() {
        let mut disk = on_disk();
        disk.insert("brand-new".to_string());
        let failures = problems(&sample(), &disk);
        assert!(
            failures.iter().any(|failure| failure.contains("brand-new")),
            "{failures:?}"
        );
    }

    #[test]
    fn a_registered_file_that_does_not_exist_fails() {
        let mut disk = on_disk();
        disk.remove("concepts");
        let failures = problems(&sample(), &disk);
        assert!(
            failures.iter().any(|failure| failure.contains("does not exist")),
            "{failures:?}"
        );
    }

    #[test]
    fn a_stale_exclusion_fails() {
        let mut disk = on_disk();
        disk.remove("closure-generated-01");
        let failures = problems(&sample(), &disk);
        assert!(
            failures.iter().any(|failure| failure.contains("stale exclusion")),
            "{failures:?}"
        );
    }

    #[test]
    fn a_seed_no_path_consumes_fails() {
        let orphan = "seed_registry\n  seed concepts\n";
        let failures = problems(&canonicalize(parse_registry(orphan)), &on_disk());
        assert!(
            failures.iter().any(|failure| failure.contains("no consumer")),
            "{failures:?}"
        );
    }

    #[test]
    fn the_rendered_registry_has_no_blank_line_before_the_root() {
        // Canonical Links Notation ends the document at a blank line that
        // follows a comment block, so a rendered registry with a separator there
        // parses as an empty file and `tests/unit/data_files.rs` rejects it. The
        // sample carries the blank line the old renderer emitted, which makes
        // this a regression test for the file that shipped with it.
        let rendered = render_registry(&sample(), &registry_header(SAMPLE));

        assert!(rendered.starts_with("# header\nseed_registry\n"), "{rendered}");
        assert!(!rendered.contains("\n\n"), "{rendered}");
    }

    #[test]
    fn the_rendered_registry_round_trips() {
        let registry = sample();
        let rendered = render_registry(&registry, &registry_header(SAMPLE));
        assert_eq!(canonicalize(parse_registry(&rendered)), registry);
    }

    #[test]
    fn a_long_constant_wraps_at_the_generator_s_own_column() {
        let registry = canonicalize(parse_registry(
            "seed_registry\n  seed agent-info\n    bundle true\n  \
             seed agentic-tool-capabilities\n    bundle true\n",
        ));
        let rust = render_rust(&registry);
        assert!(rust.contains(
            "pub const AGENT_INFO_LINO: &str = include_str!(\"../../data/seed/agent-info.lino\");"
        ));
        assert!(rust.contains(
            "pub const AGENTIC_TOOL_CAPABILITIES_LINO: &str =\n    \
             include_str!(\"../../data/seed/agentic-tool-capabilities.lino\");"
        ));
        assert!(rust.lines().all(|line| line.len() <= MAX_WIDTH), "{rust}");
    }

    #[test]
    fn the_browser_list_holds_exactly_the_web_seeds() {
        let web = render_web(&sample());
        assert!(web.contains("\"seed/concepts.lino\","));
        assert!(
            !web.contains("meanings-units"),
            "a seed with no `web` flag is not fetched by the browser"
        );
    }

    #[test]
    fn a_pattern_matches_the_way_the_exclusions_read() {
        assert!(pattern_matches("closure-generated-*", "closure-generated-01"));
        assert!(!pattern_matches("closure-generated-*", "concepts"));
        assert!(pattern_matches("roles", "roles"));
        assert!(pattern_matches("google-trends-*", "google-trends-snapshot"));
    }
}
