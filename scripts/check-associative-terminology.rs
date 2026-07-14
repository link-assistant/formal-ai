#!/usr/bin/env rust-script
//! Associative-terminology hygiene lint for the links network.
//!
//! Issue #664: the product's associative surface is a *links network*, not a
//! "graph". `/v1/network` is the canonical endpoint; `/v1/graph` survives only
//! as a deprecated alias. This lint blocks *new* graph-named public API routes
//! and Rust modules/files so the terminology cleanup does not silently regress.
//!
//! What it flags:
//!   * a versioned public API route whose resource segment is graph-named,
//!     e.g. `/v1/knowledge-graph` (the deprecated `/v1/graph` alias is allowed);
//!   * a Rust `mod` declaration or `src/**/*.rs` file whose name is graph-named,
//!     e.g. `source_graph` (the Wikidata `knowledge_graph` engine is allowed).
//!
//! What it deliberately does NOT flag (allowlisted):
//!   * the deprecated `/v1/graph` (and `/api/formal-ai/v1/graph`) alias;
//!   * the Wikidata `knowledge_graph` engine module and "knowledge graph"
//!     citations;
//!   * the codecov coverage badge and other external graph-API citation URLs;
//!   * internal graph-theory identifiers (`substitution_graph`, `GraphNode`,
//!     …) — those are neither public routes nor module names, so a route/module
//!     scan never reaches them.
//!
//! Usage: rust-script scripts/check-associative-terminology.rs
//!
//! ```cargo
//! [dependencies]
//! walkdir = "2"
//! ```

use std::fs;
use std::path::Path;
#[cfg(not(test))]
use std::process::exit;
use walkdir::WalkDir;

/// Versioned public API prefixes whose next path segment names a resource.
const API_ROUTE_PREFIXES: &[&str] = &["/v1/", "/api/formal-ai/v1/"];

/// Graph-named routes that are intentionally retained (issue #664 keeps the
/// legacy alias so existing desktop / VS Code / e2e clients keep working).
const ROUTE_ALIAS_ALLOWLIST: &[&str] = &["/v1/graph", "/api/formal-ai/v1/graph"];

/// Graph-named Rust modules that are intentionally retained. `knowledge_graph`
/// is the Wikidata-style knowledge graph engine, not the product's link network.
const MODULE_ALLOWLIST: &[&str] = &["knowledge_graph"];

/// Hosts whose URLs legitimately contain a "graph" path segment: the codecov
/// coverage badge and external graph-API citations (e.g. Semantic Scholar's
/// Graph API, Wikidata "knowledge graph" references). Lines mentioning these
/// are exempt from the route scan so citations are never mistaken for our API.
const CITATION_HOSTS: &[&str] = &["codecov.io", "semanticscholar.org"];

/// Directories that hold generated data, immutable corpora, or historical
/// records — never authored by hand, so out of scope for a hygiene lint.
const EXCLUDE_PATTERNS: &[&str] = &[
    "/target/",
    "/.git/",
    "/node_modules/",
    "/data/cache/",
    "/tests/source/",
    "/docs/",
    "/experiments/",
    "/changelog.d/",
];

/// Generated bundles, vendored artifacts, and append-only records we never
/// author the terminology of by hand.
const EXCLUDE_FILE_FRAGMENTS: &[&str] = &[
    "/src/web/app.js",
    "/src/web/formal_ai_worker",
    "/src/web/worker/",
    "/package-lock.json",
    "/CHANGELOG.md",
    // This lint necessarily embeds graph-named routes/modules as documentation
    // and test fixtures (e.g. the `/v1/knowledge-graph` acceptance case), so it
    // must not scan itself; its own `#[cfg(test)]` suite guards its behavior.
    "/scripts/check-associative-terminology.rs",
];

/// Extensions whose files we scan for graph-named API routes.
const ROUTE_SCAN_EXTENSIONS: &[&str] = &["rs", "cjs", "mjs", "js", "jsx", "md", "json"];

#[derive(Debug, Clone, PartialEq, Eq)]
struct RouteViolation {
    file: String,
    line: usize,
    route: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ModuleViolation {
    file: String,
    line: usize,
    name: String,
    kind: &'static str,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct CheckResult {
    routes: Vec<RouteViolation>,
    modules: Vec<ModuleViolation>,
}

impl CheckResult {
    fn is_clean(&self) -> bool {
        self.routes.is_empty() && self.modules.is_empty()
    }
}

fn normalized_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

fn relative_path(path: &Path, cwd: &Path) -> String {
    path.strip_prefix(cwd)
        .unwrap_or(path)
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

fn should_exclude(path: &Path) -> bool {
    let path_str = normalized_path(path);
    EXCLUDE_PATTERNS
        .iter()
        .any(|pattern| path_str.contains(pattern))
        || EXCLUDE_FILE_FRAGMENTS
            .iter()
            .any(|fragment| path_str.contains(fragment))
}

fn has_route_scan_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ROUTE_SCAN_EXTENSIONS.iter().any(|allowed| *allowed == ext))
}

fn is_rust_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext == "rs")
}

/// True when an identifier's `-`/`_`/`.`-separated segments include a standalone
/// `graph` word. This intentionally ignores `graph` embedded in a longer word
/// (`graphql`, `ideographic`, `paragraph`) and internal snake/camel identifiers
/// that only *contain* graph (`substitution_graph` is snake-cased graph theory,
/// but it is never a public route or module name, so we never test it here).
fn identifier_is_graph_named(name: &str) -> bool {
    name.split(['-', '_', '.'])
        .any(|part| part.eq_ignore_ascii_case("graph"))
}

fn line_mentions_citation(line: &str) -> bool {
    CITATION_HOSTS.iter().any(|host| line.contains(host))
}

/// The leading route-segment run of `s` (alphanumerics plus `.`, `_`, `-`),
/// stopping at `/`, `?`, `"`, whitespace, or any other delimiter.
fn leading_route_segment(s: &str) -> &str {
    let end = s
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-'))
        .unwrap_or(s.len());
    &s[..end]
}

fn collect_route_violations(file: &str, content: &str, out: &mut Vec<RouteViolation>) {
    for (index, line) in content.lines().enumerate() {
        // Citations (codecov badge, external graph-API references) are not our
        // API surface even when they carry a "graph" path segment.
        if line_mentions_citation(line) {
            continue;
        }

        for prefix in API_ROUTE_PREFIXES {
            let mut start = 0;
            while let Some(rel) = line[start..].find(prefix) {
                let prefix_at = start + rel;
                let segment_at = prefix_at + prefix.len();

                // `/api/formal-ai/v1/` also contains `/v1/`; let the longer,
                // fully-qualified prefix own that match so a route is reported
                // once in its canonical form.
                if *prefix == "/v1/" && line[..prefix_at].ends_with("formal-ai") {
                    start = segment_at;
                    continue;
                }

                let segment = leading_route_segment(&line[segment_at..]);
                if identifier_is_graph_named(segment) {
                    let route = format!("{prefix}{segment}");
                    if !ROUTE_ALIAS_ALLOWLIST.contains(&route.as_str()) {
                        let violation = RouteViolation {
                            file: file.to_string(),
                            line: index + 1,
                            route,
                        };
                        if !out.contains(&violation) {
                            out.push(violation);
                        }
                    }
                }

                start = segment_at;
            }
        }
    }
}

fn collect_module_declaration_violations(file: &str, content: &str, out: &mut Vec<ModuleViolation>) {
    for (index, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        let declaration = trimmed
            .strip_prefix("pub(crate) ")
            .or_else(|| trimmed.strip_prefix("pub "))
            .unwrap_or(trimmed);
        let Some(rest) = declaration.strip_prefix("mod ") else {
            continue;
        };
        let name = rest
            .split(|c: char| c == ';' || c == '{' || c.is_whitespace())
            .next()
            .unwrap_or("");
        if name.is_empty() {
            continue;
        }
        if identifier_is_graph_named(name) && !MODULE_ALLOWLIST.contains(&name) {
            out.push(ModuleViolation {
                file: file.to_string(),
                line: index + 1,
                name: name.to_string(),
                kind: "module declaration",
            });
        }
    }
}

fn file_name_violation(path: &Path, file: &str) -> Option<ModuleViolation> {
    let stem = path.file_stem().and_then(|stem| stem.to_str())?;
    if identifier_is_graph_named(stem) && !MODULE_ALLOWLIST.contains(&stem) {
        Some(ModuleViolation {
            file: file.to_string(),
            line: 0,
            name: stem.to_string(),
            kind: "source file",
        })
    } else {
        None
    }
}

fn check_directory(cwd: &Path) -> CheckResult {
    let mut result = CheckResult::default();

    for entry in WalkDir::new(cwd)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        if should_exclude(path) {
            continue;
        }

        let rust = is_rust_file(path);
        if rust {
            if let Some(violation) = file_name_violation(path, &relative_path(path, cwd)) {
                result.modules.push(violation);
            }
        }

        if !has_route_scan_extension(path) {
            continue;
        }

        match fs::read_to_string(path) {
            Ok(content) => {
                let file = relative_path(path, cwd);
                collect_route_violations(&file, &content, &mut result.routes);
                if rust {
                    collect_module_declaration_violations(&file, &content, &mut result.modules);
                }
            }
            Err(error) => {
                eprintln!("Warning: Could not read {}: {error}", path.display());
            }
        }
    }

    result
}

fn escape_annotation_property(value: &str) -> String {
    value
        .replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
        .replace(':', "%3A")
        .replace(',', "%2C")
}

fn escape_annotation_message(value: &str) -> String {
    value
        .replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
}

fn route_annotation(violation: &RouteViolation) -> String {
    let message = format!(
        "New graph-named public API route `{}`. The associative surface is a links network: add a `network`-based route (only the deprecated `/v1/graph` alias is exempt).",
        violation.route
    );
    format!(
        "::error file={},line={}::{}",
        escape_annotation_property(&violation.file),
        violation.line,
        escape_annotation_message(&message)
    )
}

fn module_annotation(violation: &ModuleViolation) -> String {
    let message = format!(
        "New graph-named {} `{}`. Name modules and files after the links network (e.g. `source_links`); only the Wikidata `knowledge_graph` engine is exempt.",
        violation.kind, violation.name
    );
    if violation.line == 0 {
        format!(
            "::error file={}::{}",
            escape_annotation_property(&violation.file),
            escape_annotation_message(&message)
        )
    } else {
        format!(
            "::error file={},line={}::{}",
            escape_annotation_property(&violation.file),
            violation.line,
            escape_annotation_message(&message)
        )
    }
}

#[cfg(not(test))]
fn print_violations(result: &CheckResult) {
    if !result.routes.is_empty() {
        println!("Found new graph-named public API routes:\n");
        for violation in &result.routes {
            println!("{}", route_annotation(violation));
            println!("  {}:{}: `{}`", violation.file, violation.line, violation.route);
        }
        println!();
    }

    if !result.modules.is_empty() {
        println!("Found new graph-named modules or source files:\n");
        for violation in &result.modules {
            println!("{}", module_annotation(violation));
            if violation.line == 0 {
                println!("  {}: `{}` ({})", violation.file, violation.name, violation.kind);
            } else {
                println!(
                    "  {}:{}: `{}` ({})",
                    violation.file, violation.line, violation.name, violation.kind
                );
            }
        }
        println!();
    }

    println!(
        "The associative surface is a links network, not a graph. Rename to `network`-based routes / `links`-based modules, or extend the allowlist in scripts/check-associative-terminology.rs if this is a genuine exception (deprecated alias, Wikidata knowledge graph, or a citation).\n"
    );
}

#[cfg(not(test))]
fn main() {
    println!("\nChecking associative terminology (links network, not graph) for public API routes and modules...\n");

    let cwd = std::env::current_dir().expect("Failed to get current directory");
    let result = check_directory(&cwd);

    if result.is_clean() {
        println!("No new graph-named public API routes or modules found\n");
        exit(0);
    } else {
        print_violations(&result);
        exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("check-associative-terminology-{name}-{nanos}"));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn write(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    #[test]
    fn identifies_standalone_graph_segments_only() {
        assert!(identifier_is_graph_named("graph"));
        assert!(identifier_is_graph_named("knowledge-graph"));
        assert!(identifier_is_graph_named("source_graph"));
        assert!(identifier_is_graph_named("v1.graph"));
        // Embedded-in-a-word and unrelated names must not match.
        assert!(!identifier_is_graph_named("graphql"));
        assert!(!identifier_is_graph_named("ideographic"));
        assert!(!identifier_is_graph_named("paragraph"));
        assert!(!identifier_is_graph_named("network"));
        assert!(!identifier_is_graph_named("self_source_links"));
    }

    #[test]
    fn deprecated_graph_alias_is_allowed() {
        let mut routes = Vec::new();
        collect_route_violations(
            "src/server.rs",
            "(\"GET\", \"/v1/graph\" | \"/api/formal-ai/v1/graph\") => handle,",
            &mut routes,
        );
        assert!(routes.is_empty(), "the deprecated alias must be allowed: {routes:?}");
    }

    #[test]
    fn network_route_is_allowed() {
        let mut routes = Vec::new();
        collect_route_violations(
            "src/server.rs",
            "(\"GET\", \"/v1/network\" | \"/api/formal-ai/v1/network\") => handle,",
            &mut routes,
        );
        assert!(routes.is_empty());
    }

    #[test]
    fn new_knowledge_graph_route_is_flagged() {
        let mut routes = Vec::new();
        collect_route_violations(
            "src/server.rs",
            "(\"GET\", \"/v1/knowledge-graph\") => handle,",
            &mut routes,
        );
        assert_eq!(
            routes,
            vec![RouteViolation {
                file: "src/server.rs".to_string(),
                line: 1,
                route: "/v1/knowledge-graph".to_string(),
            }]
        );
    }

    #[test]
    fn new_fully_qualified_graph_route_is_flagged() {
        let mut routes = Vec::new();
        collect_route_violations(
            "src/server.rs",
            "(\"GET\", \"/api/formal-ai/v1/knowledge-graph\") => handle,",
            &mut routes,
        );
        assert_eq!(
            routes,
            vec![RouteViolation {
                file: "src/server.rs".to_string(),
                line: 1,
                route: "/api/formal-ai/v1/knowledge-graph".to_string(),
            }]
        );
    }

    #[test]
    fn codecov_badge_and_citations_are_exempt() {
        let mut routes = Vec::new();
        // The codecov coverage badge carries a /graph/ path segment.
        collect_route_violations(
            "README.md",
            "[![codecov](https://codecov.io/gh/link-assistant/formal-ai/branch/main/graph/badge.svg)]",
            &mut routes,
        );
        // Semantic Scholar's Graph API is an external citation, not our route —
        // even though it literally contains `/graph/v1/`.
        collect_route_violations(
            "src/web/tests/connectivity.js",
            "\"https://api.semanticscholar.org/graph/v1/paper/search?query=formal-ai\"",
            &mut routes,
        );
        // A citation host guards even an otherwise-flaggable route on the line.
        collect_route_violations(
            "docs.md",
            "see codecov.io mirror at /v1/knowledge-graph for coverage",
            &mut routes,
        );
        assert!(routes.is_empty(), "citations must be exempt: {routes:?}");
    }

    #[test]
    fn graph_named_module_declaration_is_flagged() {
        let mut modules = Vec::new();
        collect_module_declaration_violations(
            "src/lib.rs",
            "pub mod source_graph;\nmod knowledge_graph;\nmod source_links;\n",
            &mut modules,
        );
        assert_eq!(
            modules,
            vec![ModuleViolation {
                file: "src/lib.rs".to_string(),
                line: 1,
                name: "source_graph".to_string(),
                kind: "module declaration",
            }]
        );
    }

    #[test]
    fn graph_named_source_file_is_flagged() {
        assert_eq!(
            file_name_violation(Path::new("src/agentic_coding/source_graph.rs"), "src/agentic_coding/source_graph.rs"),
            Some(ModuleViolation {
                file: "src/agentic_coding/source_graph.rs".to_string(),
                line: 0,
                name: "source_graph".to_string(),
                kind: "source file",
            })
        );
        assert_eq!(
            file_name_violation(Path::new("src/engine/knowledge_graph.rs"), "src/engine/knowledge_graph.rs"),
            None,
            "the Wikidata knowledge_graph engine is allowlisted",
        );
        assert_eq!(
            file_name_violation(Path::new("src/self_source_links.rs"), "src/self_source_links.rs"),
            None,
        );
    }

    #[test]
    fn check_directory_flags_fixture_and_respects_scope() {
        let repo = temp_dir("fixture");
        // The acceptance fixture: a brand-new graph-named public API route.
        write(
            &repo.join("src/server.rs"),
            "match route {\n    \"/v1/network\" => canonical(),\n    \"/v1/graph\" => deprecated_alias(),\n    \"/v1/knowledge-graph\" => bad_new_route(),\n}\n",
        );
        // A graph-named module declaration and source file.
        write(&repo.join("src/lib.rs"), "pub mod source_graph;\n");
        write(&repo.join("src/source_graph.rs"), "// body\n");
        // Excluded scopes must be ignored even when they carry graph routes.
        write(
            &repo.join("tests/source/frozen.rs"),
            "let r = \"/v1/knowledge-graph\";\n",
        );
        write(&repo.join("docs/notes.md"), "planned: /v1/link-graph\n");

        let result = check_directory(&repo);

        assert_eq!(
            result.routes,
            vec![RouteViolation {
                file: "src/server.rs".to_string(),
                line: 4,
                route: "/v1/knowledge-graph".to_string(),
            }],
            "only the in-scope new route should be flagged",
        );
        let mut module_names: Vec<&str> = result.modules.iter().map(|m| m.name.as_str()).collect();
        module_names.sort_unstable();
        assert_eq!(module_names, vec!["source_graph", "source_graph"]);
        assert!(!result.is_clean());
    }

    #[test]
    fn route_annotation_uses_github_actions_format() {
        let violation = RouteViolation {
            file: "src/server.rs".to_string(),
            line: 4,
            route: "/v1/knowledge-graph".to_string(),
        };
        assert_eq!(
            route_annotation(&violation),
            "::error file=src/server.rs,line=4::New graph-named public API route `/v1/knowledge-graph`. The associative surface is a links network: add a `network`-based route (only the deprecated `/v1/graph` alias is exempt)."
        );
    }

    #[test]
    fn module_annotation_uses_github_actions_format() {
        // A `mod` declaration carries a line number.
        let declaration = ModuleViolation {
            file: "src/lib.rs".to_string(),
            line: 12,
            name: "source_graph".to_string(),
            kind: "module declaration",
        };
        assert_eq!(
            module_annotation(&declaration),
            "::error file=src/lib.rs,line=12::New graph-named module declaration `source_graph`. Name modules and files after the links network (e.g. `source_links`); only the Wikidata `knowledge_graph` engine is exempt."
        );

        // A source file has no meaningful line; the annotation omits `line=`.
        let source_file = ModuleViolation {
            file: "src/agentic_coding/source_graph.rs".to_string(),
            line: 0,
            name: "source_graph".to_string(),
            kind: "source file",
        };
        assert_eq!(
            module_annotation(&source_file),
            "::error file=src/agentic_coding/source_graph.rs::New graph-named source file `source_graph`. Name modules and files after the links network (e.g. `source_links`); only the Wikidata `knowledge_graph` engine is exempt."
        );
    }
}
