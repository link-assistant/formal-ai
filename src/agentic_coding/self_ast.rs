//! Fourth agentic recipe — driving Formal AI to *store the CST/AST of its own
//! Rust meta algorithm in our data* (issue #538).
//!
//! Issue #538 asks, among its self-inspection axes, for the *"CST/AST of all our
//! Rust logic (meta algorithm) in our data"* so the system can *"reason about
//! itself"*. This module is the smallest real, tested slice of that axis: it
//! parses a real module of our own reasoning meta algorithm
//! ([the deterministic planner](super::planner)) through the **sole CST/AST engine
//! in this repo** — the link-foundation
//! [meta-language](https://github.com/link-foundation/meta-language) links network,
//! the same `LinkNetwork::parse` path used by `crate::coding::cst` — and emits
//! the resulting abstract-syntax node census as Links Notation.
//!
//! Nothing here is hardcoded to one answer: [`render_ast_document`] is a pure
//! function of whatever Rust source it is handed (see [`ast_census`]), so the same
//! recipe stores the CST/AST of *any* module. The recipe pins the planner as its
//! canonical self-inspection target the same way [`super::diagram`] pins the
//! planner's recipe table — the generality is proven by the unit tests parsing
//! several different sources, and by a byte-for-byte parity test on the committed
//! artifact. Neural inference stays a NON-GOAL: the census is a deterministic
//! function of the real tree-sitter parse.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use meta_language::{LinkNetwork, LinkType, NetworkProjection, ParseConfiguration};

/// The meta-language grammar label for Rust (matches
/// `data/seed/program-cst-grammars.lino`).
const RUST_GRAMMAR_LABEL: &str = "rust";

/// The self-inspection target: a real module of our own reasoning meta algorithm.
///
/// Embedded at compile time so the recipe is self-contained (no filesystem read at
/// run time — it works from the Agent CLI's sandbox workdir), and so the committed
/// artifact tracks the real planner source, regenerated like any other seed data.
pub const TARGET_MODULE_PATH: &str = "src/agentic_coding/planner.rs";

/// The planner source, embedded from disk at build time.
const TARGET_MODULE_SOURCE: &str = include_str!("planner.rs");

/// The workspace path the planner writes the generated CST/AST document to.
pub const AST_PATH: &str = "self-ast.lino";

/// A *differently worded* request for the self-AST recipe — the maintainer's
/// generality check: the router recognises the intent from the words, not a
/// hardcoded string.
pub const AST_TASK: &str =
    "Store the CST/AST of our Rust meta algorithm in our data so the system \
                            can reason about itself: parse the planner module and record its \
                            abstract-syntax node census in Links Notation.";

/// Keywords that mark a user turn as the self-AST recipe. Kept distinct from the
/// diagram keywords (`mermaid`/`diagram`) so the two self-inspection recipes never
/// collide.
const AST_KEYWORDS: [&str; 4] = [
    "cst/ast",
    "cst / ast",
    "abstract-syntax",
    "reason about itself",
];

/// Whether `prompt` asks to store the CST/AST of the meta algorithm (issue #538).
#[must_use]
pub fn is_self_ast_task(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    // Require an AST/CST intent word plus a self-reference so ordinary "parse this"
    // requests do not route here.
    let names_ast = AST_KEYWORDS.iter().any(|keyword| lower.contains(keyword))
        || (lower.contains("ast") && lower.contains("meta algorithm"));
    let self_reference = lower.contains("our")
        || lower.contains("itself")
        || lower.contains("meta algorithm")
        || lower.contains("planner");
    names_ast && self_reference
}

/// One row of the abstract-syntax node census: a tree-sitter node kind and how
/// many times it occurs in the parsed source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstNodeCount {
    /// The named grammar node kind (e.g. `function_item`, `identifier`).
    pub kind: String,
    /// How many named syntax links of this kind the parse produced.
    pub count: usize,
}

/// The full CST/AST evidence for a parsed Rust source, derived from the real
/// meta-language links network.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstCensus {
    /// Total links in the lossless network (a size signal, not the AST shape).
    pub total_link_count: usize,
    /// Named abstract-syntax nodes (the AST proper — trivia/tokens excluded).
    pub named_node_count: usize,
    /// Whether the network reconstructs the original source byte-for-byte.
    pub text_preserved: bool,
    /// Whether the parse verified cleanly (no error nodes).
    pub clean: bool,
    /// The node-kind histogram, sorted by kind for deterministic output.
    pub node_kinds: Vec<AstNodeCount>,
}

/// Parse `source` as Rust through the meta-language links network and take a
/// census of its abstract-syntax nodes.
///
/// This is the general primitive: it works on any Rust source, not just the pinned
/// target, so the recipe truly stores "the CST/AST of our Rust logic," one module
/// at a time. The census is a deterministic function of the parse.
#[must_use]
pub fn ast_census(source: &str) -> AstCensus {
    let network = LinkNetwork::parse(source, RUST_GRAMMAR_LABEL, ParseConfiguration::default());

    // The AST proper: named syntax links in the abstract-syntax projection, which
    // drops lossless tokens and trivia. Grouping by term() (the grammar node kind)
    // yields the node-kind census.
    let mut histogram: BTreeMap<String, usize> = BTreeMap::new();
    for link in network.projected_links(NetworkProjection::AbstractSyntax) {
        let metadata = link.metadata();
        if metadata.link_type() != Some(LinkType::Syntax) || !metadata.is_named() {
            continue;
        }
        if let Some(kind) = metadata.term() {
            *histogram.entry(kind.to_owned()).or_insert(0) += 1;
        }
    }

    let named_node_count = histogram.values().sum();
    let node_kinds = histogram
        .into_iter()
        .map(|(kind, count)| AstNodeCount { kind, count })
        .collect();

    AstCensus {
        total_link_count: network.len(),
        named_node_count,
        text_preserved: network.reconstruct_text() == source,
        clean: network.verify_full_match(None).is_clean(),
        node_kinds,
    }
}

/// Render the CST/AST-in-data document (Links Notation) for a given target and
/// source. Deterministic and ends with exactly one trailing newline.
///
/// The output is what the Agent CLI writes to [`AST_PATH`] and what
/// `data/meta/self-ast.lino` is committed as, asserted byte-for-byte in the
/// issue-#538 tests.
#[must_use]
pub fn render_ast_document(target_path: &str, source: &str) -> String {
    let census = ast_census(source);
    let mut out = String::new();
    let _ = writeln!(out, "self_ast");
    let _ = writeln!(out, "  target {target_path}");
    let _ = writeln!(out, "  language rust");
    // The sole CST/AST engine in this repo (see src/coding/cst.rs).
    let _ = writeln!(out, "  engine meta_language");
    let _ = writeln!(out, "  component meta-language");
    let _ = writeln!(out, "  grammar_label {RUST_GRAMMAR_LABEL}");
    let _ = writeln!(out, "  projection abstract_syntax");
    let _ = writeln!(out, "  text_preserved {}", census.text_preserved);
    let _ = writeln!(out, "  clean {}", census.clean);
    let _ = writeln!(out, "  total_link_count {}", census.total_link_count);
    let _ = writeln!(out, "  named_node_count {}", census.named_node_count);
    let _ = writeln!(out, "  distinct_node_kinds {}", census.node_kinds.len());
    let _ = writeln!(out, "  node_kinds");
    for AstNodeCount { kind, count } in &census.node_kinds {
        let _ = writeln!(out, "    {kind} {count}");
    }
    format!("{}\n", out.trim_end())
}

/// Reconstruct Rust `source` back out of its meta-language links network — the
/// *links → source* direction of the round-trip.
///
/// Issue #558 asks the system to translate its own source code to the links/meta
/// language *and back*. The forward direction is [`ast_census`] (source → links);
/// this is the reverse: parse the source into the links network, then render the
/// network back to text with `reconstruct_text`. Because the links network is a
/// lossless representation of the parse, the reconstruction is byte-for-byte the
/// original for any well-formed input — which [`round_trips`] verifies.
#[must_use]
pub fn reconstruct_source(source: &str) -> String {
    let network = LinkNetwork::parse(source, RUST_GRAMMAR_LABEL, ParseConfiguration::default());
    network.reconstruct_text()
}

/// Whether `source` survives a full source → links → source round-trip unchanged.
///
/// This is the concrete, testable form of issue #558's "translate the source code
/// to the meta language and back" requirement: it is `true` exactly when the
/// links representation loses nothing, so a downstream self-modification could be
/// expressed as an edit to the links and rendered back to compilable Rust.
#[must_use]
pub fn round_trips(source: &str) -> bool {
    reconstruct_source(source) == source
}

/// Render the CST/AST document for the pinned self-inspection target (the planner).
#[must_use]
pub fn render_document() -> String {
    render_ast_document(TARGET_MODULE_PATH, TARGET_MODULE_SOURCE)
}

/// The census of the pinned self-inspection target (the planner module). Exposed so
/// tests can assert the target is real, well-formed logic without re-embedding the
/// source.
#[must_use]
pub fn target_census() -> AstCensus {
    ast_census(TARGET_MODULE_SOURCE)
}

/// The pinned self-inspection target's source, embedded at build time. Exposed so
/// the self-healing loop can round-trip the module it maps a failure onto without
/// re-reading the filesystem at run time.
#[must_use]
pub const fn target_source() -> &'static str {
    TARGET_MODULE_SOURCE
}

/// The self-contained final answer: a natural-language summary plus the generated
/// CST/AST document inline.
#[must_use]
pub fn final_answer(document: &str) -> String {
    format!(
        "Stored the CST/AST of our Rust meta algorithm ({TARGET_MODULE_PATH}) in our data, parsed \
         through the meta-language links network (the sole CST/AST engine here) and recorded as an \
         abstract-syntax node census — a real step toward the system reasoning about itself.\n\n\
         Generated document ({AST_PATH}):\n\n{document}",
        document = document.trim_end(),
    )
}
