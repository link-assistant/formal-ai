//! Formalize a request into obligations instead of matching phrases.
//!
//! Issue #1166 (E131). The executor used to decide what a request demands by
//! matching surface phrases: a CI-workflow clause was "the prompt mentions the
//! `ci_workflow_request` role", the program to write came from alias lists,
//! and the literal to print was collected once per mentioning clause — so the
//! Kotlin Hello World issue body, which names `Hello, World!` twice (once in
//! "print exactly: …", once in the Expected Output block), printed it twice
//! (#1156).
//!
//! This module parses a request into an [`ObligationGraph`]: one
//! [`ObligationNode`] per enumerated clause — the ledger node, so clause
//! spans, split behaviour, and `Underivable` reporting come from
//! `crate::obligation_ledger` and no new ledger type is introduced — plus a
//! semantic [`ObligationKind`] classification and, for output literals, the
//! anchored literal value. Two or more mentions of the same quoted literal
//! collapse to one node in the [`coreference_pass`], which is the structural
//! fix #1156 calls for: `unique_output_literal` returns the value once no
//! matter which clauses carry it.
//!
//! Classification reads the seed lexicon wherever a meaning or role exists
//! (`print_stdout`, `ci_workflow_request`, the program-request roles), so the
//! languages the seeds cover classify in any surface language; the smaller
//! style, naming, and badge vocabularies fall back to the multilingual token
//! tables below until a seed meaning owns them. A clause nothing classifies
//! keeps its ledger node and its `Underivable` expectation — nothing is ever
//! discarded (R710-R9).
//!
//! Callers replace phrase-match decision points with [`formalize_request`]:
//! `crate::coding::program_contract::explicit_stdout` and
//! `crate::agentic_coding::ci_workflow::requested_in` delegate here (issue
//! wiring), and `crate::solver_terminal` consults
//! [`request_carries_work_obligations`] before classifying a leading shell
//! token as a command line.

use crate::engine::{normalize_prompt, stable_id};
use crate::implementation_language;
use crate::normal_markov::quoted_segment_spans;
use crate::obligation_ledger::{ObligationExpectation, ObligationNode};
use crate::seed;

/// Bound handed to [`ObligationNode::build`], matching the agentic root.
const SPLIT_DEPTH_BOUND: u8 = crate::recursive_execution::DEFAULT_SPLIT_DEPTH_BOUND;

/// The semantic obligation kinds a request clause can carry.
///
/// The vocabulary of `data/meta/request-obligation-rules.lino` as it applies to
/// the request body; each classifier below cites the seed anchor it reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObligationKind {
    /// A clause demands a specific program output ("print exactly …").
    OutputLiteral,
    /// A clause demands a CI workflow ("add CI", "GitHub Actions workflow").
    CiWorkflow,
    /// A clause demands a program file in a named language.
    ProgramFile,
    /// A clause demands code style ("clear comments", "best practices").
    CodeStyle,
    /// A clause names the output file ("a meaningful name like …").
    FileNaming,
    /// A clause demands a CI badge.
    CiBadge,
}

impl ObligationKind {
    /// Stable slug for reports and seed rule rows.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::OutputLiteral => "output_literal",
            Self::CiWorkflow => "ci_workflow",
            Self::ProgramFile => "program_file",
            Self::CodeStyle => "code_style",
            Self::FileNaming => "file_naming",
            Self::CiBadge => "ci_badge",
        }
    }
}

/// Print-verb fallback vocabulary for languages whose `print_stdout` lexemes
/// the seed does not yet carry. A literal only counts as an output obligation
/// when it is *quoted*, so no command line can pick these up by accident.
const PRINT_VERB_FALLBACK: &[(&str, &[&str])] = &[
    ("en", &["print", "prints", "write", "writes", "output"]),
    ("ru", &["выведи", "вывод", "напечатай", "напиши", "печатать"]),
    ("hi", &["छापो", "छाप", "प्रिंट", "आउटपुट", "लिखो"]),
    ("zh", &["打印", "输出", "打印出"]),
    ("es", &["imprime", "imprimir", "escribe", "escribir", "salida"]),
];

/// Code-style clause vocabulary, pending a seed meaning for style demands.
const CODE_STYLE_MARKERS: &[&str] = &[
    "comment",
    "comments",
    "best practice",
    "best practices",
    "clean code",
    "readable",
    "комментар",
    "лучшие практики",
    "понятн",
    "टिप्पणी",
    "सर्वोत्तम अभ्यास",
    "注释",
    "最佳实践",
    "可读",
    "comentario",
    "comentarios",
    "buenas prácticas",
    "prácticas recomendadas",
];

/// File-naming clause phrases. A bare filename token is deliberately not a
/// naming obligation — `make test-hello-world.yml` stays a command line.
const FILE_NAMING_MARKERS: &[&str] = &[
    "name like",
    "named",
    "meaningful name",
    "назови",
    "название файла",
    "имя файла",
    "осмысленное имя",
    "नाम रखो",
    "सार्थक नाम",
    "命名",
    "名为",
    "文件名",
    "有意义的名字",
    "llama al archivo",
    "llámalo",
    "nombre del archivo",
    "nombre significativo",
];

/// CI-badge clause phrases.
const CI_BADGE_MARKERS: &[&str] = &["badge", "значок", "бейдж", "escudo", "insignia", "प्रतीक", "徽章"];

/// The formalized request: ledger nodes plus their classifications.
///
/// `nodes` are exactly the [`ObligationNode`]s the ledger enumerates for the
/// request, in clause order; `classifications` maps node ids to kinds;
/// `literal_by_node` maps an `OutputLiteral` node id to its anchored value.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ObligationGraph {
    nodes: Vec<ObligationNode>,
    classifications: Vec<(String, ObligationKind)>,
    literal_by_node: Vec<(String, String)>,
    language: Option<String>,
}

impl ObligationGraph {
    /// The enumerated obligation nodes, in clause order.
    #[must_use]
    pub fn nodes(&self) -> &[ObligationNode] {
        &self.nodes
    }

    /// The implementation language tag extracted from the request.
    #[must_use]
    pub fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }

    /// Distinct kinds present, in declaration order.
    #[must_use]
    pub fn obligation_kinds(&self) -> Vec<ObligationKind> {
        let mut kinds: Vec<ObligationKind> = self
            .classifications
            .iter()
            .map(|(_, kind)| *kind)
            .collect();
        kinds.sort_unstable();
        kinds.dedup();
        kinds
    }

    /// Whether the graph contains an obligation of the given kind.
    #[must_use]
    pub fn has_obligation(&self, kind: ObligationKind) -> bool {
        self.classifications
            .iter()
            .any(|(_, present)| *present == kind)
    }

    /// The nodes classified with `kind`, in clause order.
    #[must_use]
    pub fn obligations_of_kind(&self, kind: ObligationKind) -> Vec<&ObligationNode> {
        self.nodes
            .iter()
            .filter(|node| {
                self.classifications
                    .iter()
                    .any(|(node_id, present)| *node_id == node.node_id && *present == kind)
            })
            .collect()
    }

    /// The unique output literal this request requires, or `None`.
    ///
    /// The coreference pass has already merged every mention of the same
    /// quoted literal into one node, so two mentions still yield one value —
    /// the structural fix for the doubled output of #1156.
    #[must_use]
    pub fn unique_output_literal(&self) -> Option<&str> {
        let mut values = self
            .nodes
            .iter()
            .filter(|node| self.has_node_kind(&node.node_id, ObligationKind::OutputLiteral))
            .filter_map(|node| {
                self.literal_by_node
                    .iter()
                    .find(|(node_id, _)| *node_id == node.node_id)
                    .map(|(_, value)| value.as_str())
            });
        let first = values.next()?;
        values.next().is_none().then_some(first)
    }

    /// The literal anchored to `node`, when it is an output-literal node.
    #[must_use]
    pub fn literal_for(&self, node: &ObligationNode) -> Option<&str> {
        if !self.has_node_kind(&node.node_id, ObligationKind::OutputLiteral) {
            return None;
        }
        self.literal_by_node
            .iter()
            .find(|(node_id, _)| *node_id == node.node_id)
            .map(|(_, value)| value.as_str())
    }

    /// Nodes whose expectation stayed `Underivable` — the clauses the rule set
    /// could not read. They are reported, never discarded (R710-R9).
    #[must_use]
    pub fn underivable(&self) -> Vec<&ObligationNode> {
        self.nodes
            .iter()
            .filter(|node| {
                matches!(node.expectation, ObligationExpectation::Underivable { .. })
            })
            .collect()
    }

    /// Gap lines for `gap_answer` consumers: one per undischargeable clause,
    /// citing the node id and the clause's byte span.
    #[must_use]
    pub fn gap_report(&self) -> Vec<String> {
        self.underivable()
            .into_iter()
            .map(|node| {
                let reason = match &node.expectation {
                    ObligationExpectation::Underivable { reason } => reason.clone(),
                    _ => String::new(),
                };
                format!(
                    "obligation {} span {}:{} underivable {}",
                    node.node_id, node.span.0, node.span.1, reason
                )
            })
            .collect()
    }

    fn has_node_kind(&self, node_id: &str, kind: ObligationKind) -> bool {
        self.classifications
            .iter()
            .any(|(id, present)| id == node_id && *present == kind)
    }

    /// The graph as one Links Notation record, for event logs and parity
    /// fixtures: one line per node with its kind, span, and anchored literal.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::new();
        out.push_str("request_obligation_graph\n");
        if let Some(language) = &self.language {
            out.push_str(&format!("  language {language}\n"));
        }
        for node in &self.nodes {
            let kind = self
                .classifications
                .iter()
                .find(|(node_id, _)| *node_id == node.node_id)
                .map(|(_, kind)| kind.slug())
                .unwrap_or("unclassified");
            out.push_str(&format!(
                "  obligation {}\n    kind {kind}\n    span {}:{}\n",
                node.node_id, node.span.0, node.span.1
            ));
            if let Some((_, literal)) = self
                .literal_by_node
                .iter()
                .find(|(node_id, _)| *node_id == node.node_id)
            {
                out.push_str(&format!(
                    "    literal \"{}\"\n",
                    literal.replace('\\', "\\\\").replace('"', "\\\"")
                ));
            }
        }
        out
    }
}

/// Parse an issue body or prompt into an [`ObligationGraph`].
///
/// Every enumerated requirement produces one [`ObligationNode`] anchored to
/// the seed lexicon; mentions of the same quoted literal are merged by the
/// [`coreference_pass`] regardless of which clause they appear in.
#[must_use]
pub fn formalize_request(text: &str) -> ObligationGraph {
    let root = ObligationNode::build(text, SPLIT_DEPTH_BOUND);
    // The ledger hands back the whole tree; the enumerated requirements are
    // the clause-level children (or the root itself for a single clause).
    let clause_nodes: Vec<ObligationNode> = if root.children.is_empty() {
        vec![root]
    } else {
        root.children
    };

    let mut graph = ObligationGraph {
        language: implementation_language::requested(&normalize_prompt(
            &crate::solver_handlers::text_outside_quoted_segments(text),
        )),
        ..ObligationGraph::default()
    };
    for node in clause_nodes {
        if let Some((kind, literal)) = classify_clause(&node.clause) {
            let node_id = node.node_id.clone();
            if let Some(value) = literal {
                graph.literal_by_node.push((node_id.clone(), value));
            }
            graph.classifications.push((node_id, kind));
        }
        graph.nodes.push(node);
    }
    coreference_pass(graph)
}

/// Merge nodes that share the same output literal (R2, the #1156 fix).
///
/// Two or more mentions of the same quoted literal — "print exactly:
/// `Hello, World!`" and the Expected Output block — produce exactly one
/// output-literal node; the value appears in the graph once. Distinct
/// literals stay distinct nodes. A filename clause is never an
/// output-literal node, so it can never merge with one.
#[must_use]
pub fn coreference_pass(graph: ObligationGraph) -> ObligationGraph {
    let ObligationGraph {
        nodes,
        classifications,
        literal_by_node,
        language,
    } = graph;
    let kind_of = |node_id: &str| {
        classifications
            .iter()
            .find(|(id, kind)| id == node_id)
            .map(|(_, kind)| *kind)
    };
    let literal_of = |node_id: &str| {
        literal_by_node
            .iter()
            .find(|(id, value)| id == node_id)
            .map(|(_, value)| value.clone())
    };

    let mut seen_literals: Vec<String> = Vec::new();
    let mut kept_nodes = Vec::with_capacity(nodes.len());
    let mut kept_classifications = Vec::with_capacity(classifications.len());
    let mut kept_literals = Vec::with_capacity(literal_by_node.len());

    for node in nodes {
        let is_output = kind_of(&node.node_id) == Some(ObligationKind::OutputLiteral);
        let literal = literal_of(&node.node_id);
        if is_output
            && let Some(value) = &literal
            && seen_literals.contains(value)
        {
            // A duplicate mention of an already-anchored literal: the
            // coreference of this node is the node that carries the value.
            continue;
        }
        if is_output
            && let Some(value) = &literal
        {
            seen_literals.push(value.clone());
        }
        kept_classifications.extend(
            classifications
                .iter()
                .filter(|(node_id, _)| *node_id == node.node_id)
                .cloned(),
        );
        kept_literals.extend(
            literal_by_node
                .iter()
                .filter(|(node_id, _)| *node_id == node.node_id)
                .cloned(),
        );
        kept_nodes.push(node);
    }

    ObligationGraph {
        nodes: kept_nodes,
        classifications: kept_classifications,
        literal_by_node: kept_literals,
        language,
    }
}

/// Whether the request formalizes into work the terminal must not claim.
///
/// The leading-token path of terminal detection fires whenever a prompt
/// starts with a shell token, and many shell tokens are ordinary words in
/// every language ("make a small Kotlin app …"). The English marker words of
/// issue #1175 cover one language; this check covers the request's meaning: a
/// prompt that demands a quoted output literal *and* an authoring clause
/// (program, CI, style, naming, badge) is a sentence about building
/// something, not a command line. A bare `echo "Hello"` carries no authoring
/// clause and stays a command.
#[must_use]
pub fn request_carries_work_obligations(text: &str) -> bool {
    let graph = formalize_request(text);
    let authoring = graph.has_obligation(ObligationKind::ProgramFile)
        || graph.has_obligation(ObligationKind::CiWorkflow)
        || graph.has_obligation(ObligationKind::CodeStyle)
        || graph.has_obligation(ObligationKind::FileNaming)
        || graph.has_obligation(ObligationKind::CiBadge);
    (graph.has_obligation(ObligationKind::OutputLiteral) && authoring)
        || graph.has_obligation(ObligationKind::FileNaming)
        || graph.has_obligation(ObligationKind::CiBadge)
}

/// Classify one clause against the seed lexicon and the fallback tables.
///
/// Returns the kind plus, for `OutputLiteral`, the anchored quoted value.
/// `None` leaves the node unclassified — its ledger expectation (often
/// `Underivable`) still reports it.
fn classify_clause(clause: &str) -> Option<(ObligationKind, Option<String>)> {
    let outside = crate::solver_handlers::text_outside_quoted_segments(clause);
    let normalized = normalize_prompt(&outside);
    let lower = normalized.to_lowercase();
    let lexicon = seed::lexicon();

    // Output literal: a print-meaning clause that carries a quoted value.
    let print_evidence = lexicon
        .meaning("print_stdout")
        .is_some_and(|meaning| meaning.evidenced_in(&lower))
        || PRINT_VERB_FALLBACK
            .iter()
            .any(|(_, verbs)| verbs.iter().any(|verb| lower.contains(verb)));
    let quoted = quoted_segment_spans(clause);
    if print_evidence && !quoted.is_empty() {
        // The anchored literal is the last quoted segment of the clause: the
        // Expected Output block quotes the value after its introducing prose.
        let value = quoted
            .last()
            .map(|segment| segment.text.clone())
            .unwrap_or_default();
        if !value.is_empty() {
            return Some((ObligationKind::OutputLiteral, Some(value)));
        }
    }

    if lexicon.mentions_role(seed::ROLE_CI_WORKFLOW_REQUEST, &normalized) {
        return Some((ObligationKind::CiWorkflow, None));
    }
    if lexicon.mentions_role(seed::ROLE_PROGRAM_REQUEST, &normalized)
        || lexicon.mentions_role(seed::ROLE_CODING_REQUEST_VERB, &normalized)
    {
        return Some((ObligationKind::ProgramFile, None));
    }
    if CODE_STYLE_MARKERS.iter().any(|marker| lower.contains(marker)) {
        return Some((ObligationKind::CodeStyle, None));
    }
    if FILE_NAMING_MARKERS.iter().any(|marker| lower.contains(marker)) {
        return Some((ObligationKind::FileNaming, None));
    }
    if CI_BADGE_MARKERS.iter().any(|marker| lower.contains(marker)) {
        return Some((ObligationKind::CiBadge, None));
    }
    None
}

/// Stable graph id for a request: the same meaning yields the same id, so
/// parity fixtures can pin expected graphs without pinning clause order.
#[must_use]
pub fn graph_id(text: &str) -> String {
    stable_id("request_obligation_graph", text)
}
