//! Self-inspection records complete recursive source CST in human-readable Links Notation.
//!
//! The planner source embedded at build time is the canonical self-inspection target
//! (issue #538). The persisted source model owns all literals, comments, punctuation
//! and whitespace and reconstructs without its original source buffer. Fresh grammar
//! verification checks concrete structure independently. Legacy AST API names and
//! histogram adapters remain compatibility diagnostics; they are not source artifacts.

#[cfg(feature = "meta-language")]
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(feature = "meta-language")]
use meta_language::{LinkNetwork, LinkType, NetworkProjection, ParseConfiguration};

/// How many whole-source CST/AST parses [`ast_census`] has run in this process.
static AST_CENSUS_RUNS: AtomicU64 = AtomicU64::new(0);

/// The number of whole-source CST/AST parses this process has run so far.
///
/// A census parses an entire module through tree-sitter and walks every node, so
/// it is by far the most expensive primitive the solver can reach. Issue #1017
/// traced a thirty-second integration-test timeout to exactly one such parse
/// happening inside a request. Exposing the count lets a test state the
/// invariant structurally — *ordinary solving parses no module* — instead of
/// asserting on a wall-clock budget that differs per runner.
#[must_use]
pub fn ast_census_runs() -> u64 {
    AST_CENSUS_RUNS.load(Ordering::Relaxed)
}

/// Record one census run, and — only when `FORMAL_AI_TRACE_SLOW_INIT` is set
/// to a true spelling (issue #1181) — report what it cost and how big the
/// source was.
///
/// Off by default: this is diagnostic output for locating a slow first request,
/// in the same opt-in style as `FORMAL_AI_TRACE_REQUESTS`.
#[cfg(feature = "meta-language")]
fn record_census_run(source_len: usize, elapsed: std::time::Duration) {
    AST_CENSUS_RUNS.fetch_add(1, Ordering::Relaxed);
    if crate::cli_env::flag_enabled("FORMAL_AI_TRACE_SLOW_INIT") {
        crate::diagnostic!("debug";
            "[slow-init] ast_census: {source_len} bytes in {} ms (run #{})",
            elapsed.as_millis(),
            ast_census_runs()
        );
    }
}

/// The meta-language grammar label for Rust (matches
/// `data/seed/program-cst-grammars.lino`).
const RUST_GRAMMAR_LABEL: &str = "rust";

/// The self-inspection target: a real module of our own reasoning meta algorithm.
///
/// Embedded at compile time so the recipe is self-contained (no filesystem read at
/// run time — it works from the Agent CLI's sandbox workdir), while on-demand source content is always derived from those actual bytes.
pub const TARGET_MODULE_PATH: &str = "src/agentic_coding/planner.rs";

/// The planner source, embedded from disk at build time.
const TARGET_MODULE_SOURCE: &str = include_str!("planner.rs");

/// Historical API name for the complete source CST artifact written by the planner.
pub const AST_PATH: &str = "self-cst.lino";

/// A *differently worded* request for the self-AST recipe — the maintainer's
/// generality check: the router recognises the intent from the words, not a
/// hardcoded string.
#[must_use]
pub fn ast_task() -> &'static str {
    static TASK: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    TASK.get_or_init(|| super::wording::agentic_message("self_ast_task", &[]))
        .as_str()
}

/// Keywords that mark a user turn as the self-AST recipe. Kept distinct from the
/// diagram keywords (`mermaid`/`diagram`) so the two self-inspection recipes never
/// collide.
const AST_KEYWORDS: &[&str] = &[
    "cst/ast",
    "cst / ast",
    "abstract-syntax",
    "reason about itself",
    "concrete syntax",
];

/// Whether `prompt` asks to store the CST/AST of the meta algorithm (issue #538).
#[must_use]
pub fn is_self_ast_task(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    // Require an AST/CST intent word plus a self-reference so ordinary "parse this"
    // requests do not route here.
    let names_ast = AST_KEYWORDS.iter().any(|keyword| lower.contains(keyword))
        || lower
            .split(|character: char| !character.is_alphanumeric())
            .any(|word| word == "cst")
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
#[cfg(feature = "meta-language")]
pub fn ast_census(source: &str) -> AstCensus {
    let started = std::time::Instant::now();
    let network = LinkNetwork::parse(source, RUST_GRAMMAR_LABEL, ParseConfiguration::default());
    let (named_node_count, node_kinds) = named_node_histogram(&network);
    let census = AstCensus {
        total_link_count: network.len(),
        named_node_count,
        text_preserved: network.reconstruct_text() == source,
        clean: network.verify_full_match(None).is_clean(),
        node_kinds,
    };
    record_census_run(source.len(), started.elapsed());
    census
}

/// The named-node census walk over a parsed `network`: the named-node count
/// and the node-kind histogram, sorted by kind.
///
/// The AST proper is the named syntax links in the abstract-syntax projection,
/// which drops lossless tokens and trivia; grouping them by `term()` (the
/// grammar node kind) yields the histogram. Mirrored by `namedNodeHistogram`
/// in `js/agentic/crate/rust_ast_census.mjs`.
#[cfg(feature = "meta-language")]
fn named_node_histogram(network: &LinkNetwork) -> (usize, Vec<AstNodeCount>) {
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
    (named_node_count, node_kinds)
}

/// Return an unavailable census when the optional parsing engine is disabled.
#[must_use]
#[cfg(not(feature = "meta-language"))]
pub const fn ast_census(_source: &str) -> AstCensus {
    AstCensus {
        total_link_count: 0,
        named_node_count: 0,
        text_preserved: false,
        clean: false,
        node_kinds: Vec::new(),
    }
}

/// Render complete recursive source CST for the given target and actual source.
/// The legacy function name is retained for callers; the artifact at [`AST_PATH`]
/// contains source content and is verified by reconstruction and fresh grammar.
/// Historical census records remain separate diagnostics.
#[must_use]
pub fn render_ast_document(target_path: &str, source: &str) -> String {
    source_document(target_path, source)
        .unwrap_or_else(|reason| format!("SourceModelUnavailable:{reason}"))
}

/// The formatting half of [`render_ast_document`]: the CST/AST-in-data
/// document for an already computed `census`.
///
/// Mirrored by `renderCensusDocument` in `js/agentic/self_ast.mjs`.
#[must_use]
pub fn render_census_document(target_path: &str, census: &AstCensus) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "source_census_diagnostic");
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
#[cfg(feature = "meta-language")]
pub fn reconstruct_source(source: &str) -> String {
    let network = LinkNetwork::parse(source, RUST_GRAMMAR_LABEL, ParseConfiguration::default());
    network.reconstruct_text()
}

/// Preserve the input when the optional parsing engine is disabled.
#[must_use]
#[cfg(not(feature = "meta-language"))]
pub fn reconstruct_source(source: &str) -> String {
    source.to_owned()
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

/// Serialize Rust `source` into the *network* dialect of lino — plan 16 L7's
/// full-fidelity rust → meta direction (issue #1138).
///
/// `to_lino` is the dependency's documented lossless serialization
/// (`from_lino(to_lino(n))` is isomorphic for any network, spans included), so
/// this is the whole projection: nothing is dropped and nothing is invented.
/// Legacy node-census formatting is diagnostic only; recursive source content is
/// artifact; this is the dialect the meta → rust leg reads back.
#[must_use]
#[cfg(feature = "meta-language")]
pub fn network_lino(source: &str) -> String {
    crate::readable_source::source_to_links(source, "Rust")
        .and_then(|document| crate::readable_source::write_source_links(&document))
        .unwrap_or_else(|reason| format!("SourceModelUnavailable:{reason}"))
}

/// Preserve the input when the optional parsing engine is disabled.
#[must_use]
#[cfg(not(feature = "meta-language"))]
pub fn network_lino(source: &str) -> String {
    let _ = source;
    "SourceModelUnavailable:meta-language engine disabled".to_owned()
}

/// Render Rust source back out of a network-serialization lino document —
/// plan 16 L7's meta → rust direction: `from_lino`, then `reconstruct_text`.
///
/// The Meta root carries three lino dialects (the signature census, the ES
/// token-tree pivot, and this network serialization), and only the third
/// renders Rust — so a document in any other dialect is an error naming the
/// expected dialect, never a mis-parse into invented source.
#[cfg(feature = "meta-language")]
pub fn render_network_source(document: &str) -> Result<String, String> {
    let model = crate::readable_source::read_source_links(document)
        .map_err(|reason| format!("ReadableRustSourceRequired:{reason}"))?;
    let content = crate::readable_source::source_content(&model)?;
    let fields = content
        .as_array()
        .ok_or_else(|| "SourceDocumentRequired".to_owned())?;
    if crate::readable_source::field_text(fields, "language")? != "Rust" {
        return Err("SourceLanguageMismatch:Rust".to_owned());
    }
    crate::readable_source::verify_source_links(&model)?;
    crate::readable_source::source_from_links(&model)
}

/// The engine-disabled shape of [`render_network_source`]: without the
/// optional parsing engine the leg cannot render, and the honest answer is
/// the error, not a guess.
#[cfg(not(feature = "meta-language"))]
pub fn render_network_source(document: &str) -> Result<String, String> {
    let _ = document;
    Err(
        "the meta-language engine (feature `meta-language`) is disabled, so meta → rust \
         cannot render"
            .to_owned(),
    )
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
    super::wording::agentic_message(
        "self_ast_final_answer",
        &[
            ("target", TARGET_MODULE_PATH),
            ("path", AST_PATH),
            ("document", document.trim_end()),
        ],
    )
}

/// Generate actual recursive content; diagnostics and cached counts cannot satisfy this API.
pub fn source_document(target_path: &str, source: &str) -> Result<String, String> {
    #[cfg(feature = "meta-language")]
    {
        let document = crate::readable_source::source_to_links(source, "Rust")?;
        let owned = crate::readable_source::owned_source_links(target_path, &document)?;
        crate::readable_source::write_source_links(&owned)
    }
    #[cfg(not(feature = "meta-language"))]
    {
        let _ = (target_path, source);
        Err("SourceModelEngineUnavailable".to_owned())
    }
}
