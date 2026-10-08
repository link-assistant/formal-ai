//! The generated system diagrams (issue #538, R382).
//!
//! `docs/diagrams/{system-overview,solver-handlers,cli-subcommands,http-routes}.md`
//! are rendered from live data by `formal_ai::agentic_coding::system_diagram`
//! (twin of `js/agentic/system_diagram.mjs`, whose driver
//! `scripts/generate-system-diagrams.mjs --check` is the CI drift gate). These
//! tests pin the Rust twin to the committed parts byte for byte, and pin that
//! every handler, promotion, route and CLI subcommand the live sources declare
//! appears in its part — the CLI list read from `rust/src/main.rs` is also held
//! to what the built `formal-ai --help` prints.

use std::path::{Path, PathBuf};

use formal_ai::agentic_coding::system_diagram::{
    CONFIG_PATH, DIAGRAM_DIR, cli_definition, render_system_diagrams, route_rows,
};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits above the crate")
        .to_path_buf()
}

fn read_repo(relative: &str) -> Option<String> {
    std::fs::read_to_string(repository_root().join(relative)).ok()
}

fn rendered() -> Vec<(String, String)> {
    render_system_diagrams(read_repo).expect("the live sources render")
}

fn part(file: &str) -> String {
    rendered()
        .into_iter()
        .find(|(name, _)| name == file)
        .map_or_else(|| panic!("no generated part {file}"), |(_, text)| text)
}

#[test]
fn committed_parts_are_the_generated_parts() {
    let parts = rendered();
    let files: Vec<&str> = parts.iter().map(|(file, _)| file.as_str()).collect();
    assert_eq!(
        files,
        [
            "system-overview.md",
            "solver-handlers.md",
            "cli-subcommands.md",
            "http-routes.md"
        ]
    );
    for (file, text) in &parts {
        let committed = read_repo(&format!("{DIAGRAM_DIR}/{file}"))
            .unwrap_or_else(|| panic!("{DIAGRAM_DIR}/{file} is committed"));
        assert_eq!(
            &committed, text,
            "{DIAGRAM_DIR}/{file} drifted; run node scripts/generate-system-diagrams.mjs --write"
        );
    }
}

#[test]
fn every_handler_appears_in_precedence_order() {
    let text = part("solver-handlers.md");
    let precedence = formal_ai::seed::handler_precedence();
    assert!(precedence.len() > 50, "{}", precedence.len());
    let mut last = 0;
    for name in precedence {
        let node = format!("h_{name}[\"");
        let at = text
            .find(&node)
            .unwrap_or_else(|| panic!("handler {name} is drawn"));
        assert!(at > last, "handler {name} is drawn in precedence order");
        last = at;
        assert!(text.contains(&format!("| `{name}` |")), "{name} is tabled");
    }
    for name in formal_ai::seed::browser_only_handlers() {
        assert!(
            text.contains(&format!("`{name}` | browser worker only |")),
            "{name} is marked browser-only"
        );
    }
}

#[test]
fn every_promotion_appears_in_rank_order() {
    let text = part("solver-handlers.md");
    let promotions = formal_ai::handler_promotion::promotions();
    assert!(!promotions.is_empty());
    let mut last = 0;
    for promotion in &promotions {
        let label = format!("[\"{}: {}\"]", promotion.rank, promotion.handler);
        let at = text
            .find(&label)
            .unwrap_or_else(|| panic!("promotion {} is drawn", promotion.handler));
        assert!(
            at > last,
            "promotion {} keeps rank order",
            promotion.handler
        );
        last = at;
    }
}

#[test]
fn every_route_and_path_appears() {
    let text = part("http-routes.md");
    let manifest = read_repo("data/meta/server-routes.lino").expect("route manifest");
    let ids: Vec<&str> = manifest
        .lines()
        .filter_map(|line| line.strip_prefix("  route "))
        .collect();
    let routes = route_rows(&manifest);
    assert_eq!(
        routes.iter().map(|row| row.id.as_str()).collect::<Vec<_>>(),
        ids
    );
    for route in &routes {
        assert!(
            text.contains(&format!("r_{}[\"", route.id)),
            "route {} is drawn",
            route.id
        );
        for path in &route.paths {
            assert!(text.contains(&format!("`{path}`")), "path {path} is tabled");
        }
    }
}

#[test]
fn every_subcommand_appears_and_matches_the_built_cli() {
    let text = part("cli-subcommands.md");
    let cli = cli_definition(&read_repo("rust/src/main.rs").expect("main.rs"));
    assert_eq!(cli.binary, "formal-ai");
    let names: Vec<&str> = cli
        .subcommands
        .iter()
        .map(|sub| sub.name.as_str())
        .collect();
    for name in &names {
        assert!(
            text.contains(&format!("| `formal-ai {name}` |")),
            "subcommand {name} is tabled"
        );
    }
    let help = std::process::Command::new(env!("CARGO_BIN_EXE_formal-ai"))
        .arg("--help")
        .output()
        .expect("the formal-ai binary runs");
    let help = String::from_utf8_lossy(&help.stdout);
    let listed: Vec<&str> = help
        .lines()
        .skip_while(|line| line.trim() != "Commands:")
        .skip(1)
        .take_while(|line| !line.trim().is_empty())
        .filter_map(|line| line.split_whitespace().next())
        .filter(|name| *name != "help")
        .collect();
    assert_eq!(names, listed, "the generated list is clap's own");
}

#[test]
fn an_edge_naming_a_vanished_subcommand_fails_the_renderer() {
    let broken = |relative: &str| {
        let text = read_repo(relative)?;
        Some(if relative == CONFIG_PATH {
            text.replace("subcommand chat", "subcommand vanished")
        } else {
            text
        })
    };
    assert_eq!(
        render_system_diagrams(broken).expect_err("a stale edge is refused"),
        "system_diagrams:edge:cli_chat:unknown_subcommand:vanished"
    );
}

#[test]
fn a_new_seed_handler_reaches_the_diagram() {
    let extended = |relative: &str| {
        let text = read_repo(relative)?;
        Some(if relative == "data/seed/handler-precedence.lino" {
            format!("{text}  handler diagram_probe_handler\n    rank 999999\n")
        } else {
            text
        })
    };
    let parts = render_system_diagrams(extended).expect("renders");
    let handlers = &parts
        .iter()
        .find(|(file, _)| file == "solver-handlers.md")
        .expect("handlers part")
        .1;
    assert!(handlers.contains("h_diagram_probe_handler[\"999999: diagram_probe_handler\"]"));
}
