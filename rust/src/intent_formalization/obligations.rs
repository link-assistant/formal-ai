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
//! Classification reads the seed lexicon only (`print_stdout`,
//! `ci_workflow_request`, the program-request roles, and the
//! `output_obligation_verb`, `code_style_obligation`,
//! `file_naming_obligation` and `ci_badge_obligation` roles of
//! `data/seed/meanings-repository-workflow.lino`), so a new surface language
//! is a seed edit. A clause nothing classifies keeps its ledger node and its
//! `Underivable` expectation — nothing is ever discarded (R710-R9).
//!
//! Callers replace phrase-match decision points with [`formalize_request`]:
//! `crate::agentic_coding::ci_workflow::requested_in` delegates to
//! [`request_demands`], `crate::coding::program_contract::explicit_stdout`
//! reads [`bound_output_literals`], and `crate::solver_terminal` consults
//! [`request_carries_work_obligations`] before classifying a leading shell
//! token as a command line. Every executor that reads the graph records
//! [`obligation_gap_lines`] through [`record_obligation_gaps`] (R1166-4).

use crate::engine::{normalize_prompt, stable_id};
use crate::implementation_language;
use crate::links_format::push_lino_field;
use crate::normal_markov::quoted_segment_spans;
use crate::obligation_ledger::{ObligationExpectation, ObligationNode};
use crate::seed;

/// The record head of the obligation graph's Links Notation rendering, and
/// the prefix of its stable id.
const GRAPH_RECORD: &str = "request_obligation_graph";

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

/// Seed role whose surfaces mark an output-literal clause beside the
/// `print_stdout` meaning (`data/seed/meanings-repository-workflow.lino`).
/// A literal only counts as an output obligation when it is *quoted*, so no
/// command line can pick these surfaces up by accident.
const ROLE_OUTPUT_OBLIGATION_VERB: &str = "output_obligation_verb";

/// Seed role whose surfaces mark a code-style clause.
const ROLE_CODE_STYLE_OBLIGATION: &str = "code_style_obligation";

/// Seed role whose surfaces mark a file-naming clause. A bare filename token
/// is deliberately not one: `make test-hello-world.yml` stays a command line.
const ROLE_FILE_NAMING_OBLIGATION: &str = "file_naming_obligation";

/// Seed role whose surfaces mark a CI-badge clause.
const ROLE_CI_BADGE_OBLIGATION: &str = "ci_badge_obligation";

/// The `when` key of the contract rule that names the reason an output
/// literal no executor binding carries is reported under
/// (`data/meta/obligation-evidence-contract.lino`).
const WHEN_UNBOUND_OUTPUT: &str = "unbound_output_literal";

/// The breaks that end the text introducing a quoted output operand.
const OUTPUT_INTRODUCTION_BREAKS: [char; 4] = ['\n', '.', ';', '\u{3002}'];

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
        let mut kinds: Vec<ObligationKind> =
            self.classifications.iter().map(|(_, kind)| *kind).collect();
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
            .filter(|node| matches!(node.expectation, ObligationExpectation::Underivable { .. }))
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
                    ObligationExpectation::Underivable { reason } => reason.as_str(),
                    _ => "",
                };
                gap_line(node, reason)
            })
            .collect()
    }

    /// Gap lines for the output-literal nodes whose value `bound` lacks.
    ///
    /// The graph reads an output clause broadly (the `print_stdout` meaning
    /// and the `output_obligation_verb` surfaces anywhere in the clause, the
    /// clause's last quoted segment); an executor binds only the literals
    /// [`bound_output_literals`] anchors. A literal the graph demands but no
    /// binding carries would otherwise vanish from the program silently, so
    /// it is reported under the contract's `unbound_output_literal` reason
    /// (R1166-4).
    #[must_use]
    pub fn unbound_output_report(&self, bound: &[String]) -> Vec<String> {
        let reason = crate::obligation_ledger::ExpectationRules::shipped()
            .rules
            .into_iter()
            .find(|rule| rule.when == WHEN_UNBOUND_OUTPUT)
            .map_or_else(|| WHEN_UNBOUND_OUTPUT.to_owned(), |rule| rule.reason);
        self.nodes
            .iter()
            .filter(|node| self.has_node_kind(&node.node_id, ObligationKind::OutputLiteral))
            .filter(|node| {
                self.literal_by_node
                    .iter()
                    .find(|(node_id, _)| *node_id == node.node_id)
                    .is_some_and(|(_, value)| !bound.contains(value))
            })
            .map(|node| gap_line(node, &reason))
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
        push_lino_field(&mut out, 0, GRAPH_RECORD, None);
        if let Some(language) = &self.language {
            push_lino_field(&mut out, 2, "language", Some(language.as_str()));
        }
        for node in &self.nodes {
            let kind = self
                .classifications
                .iter()
                .find(|(node_id, _)| *node_id == node.node_id)
                .map_or("unclassified", |(_, kind)| kind.slug());
            push_lino_field(&mut out, 2, "obligation", Some(node.node_id.as_str()));
            push_lino_field(&mut out, 4, "kind", Some(kind));
            let span = format!("{}:{}", node.span.0, node.span.1);
            push_lino_field(&mut out, 4, "span", Some(span.as_str()));
            if let Some((_, literal)) = self
                .literal_by_node
                .iter()
                .find(|(node_id, _)| *node_id == node.node_id)
            {
                let quoted = format!("\"{}\"", literal.replace('\\', "\\\\").replace('"', "\\\""));
                push_lino_field(&mut out, 4, "literal", Some(quoted.as_str()));
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
            .find(|(id, _kind)| id == node_id)
            .map(|(_, kind)| *kind)
    };
    let literal_of = |node_id: &str| {
        literal_by_node
            .iter()
            .find(|(id, _value)| id == node_id)
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
        if is_output && let Some(value) = &literal {
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
///
/// One clause can carry both halves ("make a Kotlin app that prints
/// `Hello`; add a GitHub Actions workflow" enumerates no cue to split on),
/// and the graph keeps one kind per node, so the authoring half is read from
/// every clause's own text rather than from the node classifications alone.
#[must_use]
pub fn request_carries_work_obligations(text: &str) -> bool {
    let graph = formalize_request(text);
    let authoring: Vec<ObligationKind> = graph
        .nodes
        .iter()
        .filter_map(|node| authoring_kind(&node.clause))
        .collect();
    let names_or_badges =
        |kind: ObligationKind| graph.has_obligation(kind) || authoring.contains(&kind);
    (graph.has_obligation(ObligationKind::OutputLiteral) && !authoring.is_empty())
        || names_or_badges(ObligationKind::FileNaming)
        || names_or_badges(ObligationKind::CiBadge)
}

/// Whether the formalized request demands an obligation of `kind` (R1166-3).
///
/// The executor's decision points read the request through its obligation
/// nodes rather than through a phrase match over the whole text: a node's own
/// classification counts, and so does the authoring kind of its clause text,
/// because one clause can carry both an output literal and an authoring
/// demand while the graph keeps one kind per node (the same reading
/// [`request_carries_work_obligations`] makes).
#[must_use]
pub fn request_demands(text: &str, kind: ObligationKind) -> bool {
    let graph = formalize_request(text);
    graph.has_obligation(kind)
        || graph
            .nodes
            .iter()
            .any(|node| authoring_kind(&node.clause) == Some(kind))
}

/// One gap line: the node id, its byte span, and the underivable reason.
fn gap_line(node: &ObligationNode, reason: &str) -> String {
    let span = format!("{}:{}", node.span.0, node.span.1);
    [
        ("obligation", node.node_id.as_str()),
        ("span", span.as_str()),
        ("underivable", reason),
    ]
    .iter()
    .map(|(name, value)| [*name, *value].join(" "))
    .collect::<Vec<_>>()
    .join(" ")
}

/// The output literals an executor binds from `text`, each once, in order.
///
/// This is the program executor's reading of the graph's output clauses
/// (R1166-3): every quoted segment is assigned to the enumerated obligation
/// clause that contains it, and it is an output operand when the text
/// introducing it — from the clause start or the previous quoted segment,
/// after the last sentence break — evidences the seed `print_stdout` meaning.
/// A print verb in an earlier clause never binds a literal in a later one,
/// and a value quoted in several clauses is bound once (the #1156
/// coreference). Literals the graph classifies more broadly but this reading
/// does not anchor are reported by
/// [`ObligationGraph::unbound_output_report`], never dropped.
///
/// A clause that quotes nothing can still name its output in the open
/// ("… prints Hello, World! and run it"): [`unquoted_output`] binds that
/// utterance, at the clause's position in request order (PR #1188 T18).
#[must_use]
pub fn bound_output_literals(text: &str) -> Vec<String> {
    let clauses = crate::obligation_ledger::clauses_with_spans(text);
    let clause_starts: Vec<usize> = clauses.iter().map(|(_, span)| span.0).collect();
    let lexicon = seed::lexicon();
    let print = lexicon.meaning("print_stdout");
    let mut previous_end: usize = 0;
    let mut bound: Vec<(usize, String)> = Vec::new();
    for literal in quoted_segment_spans(text) {
        let clause_start = clause_starts
            .iter()
            .copied()
            .filter(|start| *start <= literal.start)
            .max()
            .unwrap_or(0);
        let window = &text[previous_end.max(clause_start)..literal.start];
        previous_end = literal.end;
        let introduction = window
            .rsplit(OUTPUT_INTRODUCTION_BREAKS)
            .next()
            .unwrap_or(window);
        if print.is_some_and(|meaning| meaning.evidenced_in(&introduction.to_lowercase())) {
            bound.push((literal.start, literal.text));
        }
    }
    for (clause, span) in &clauses {
        if quoted_segment_spans(clause).is_empty()
            && let Some(output) = unquoted_output(clause)
        {
            bound.push((span.0, output));
        }
    }
    bound.sort_by_key(|(start, _)| *start);
    let mut outputs: Vec<String> = Vec::new();
    for (_, value) in bound {
        if !outputs.contains(&value) {
            outputs.push(value);
        }
    }
    outputs
}

/// The marks that end the words an unquoted output spans.
const UNQUOTED_OUTPUT_ENDS: [char; 9] = [
    '.', '!', '?', ';', ':', '\u{3002}', '\u{ff01}', '\u{ff1f}', '\u{0964}',
];

/// The marks peeled off the end of an unquoted output: the sentence's own
/// punctuation, not the utterance's. `!` and `?` stay, because an utterance
/// ("Hello, World!") carries them and a sentence about it rarely does.
const UNQUOTED_OUTPUT_PEEL: [char; 6] = ['.', ',', ';', ':', '\u{3002}', '\u{0964}'];

/// A word with its edge punctuation removed, lowercased.
fn bare_word(word: &str) -> String {
    word.trim_matches(|character: char| !character.is_alphanumeric())
        .to_lowercase()
}

/// The output a clause that quotes nothing names in the open, when it is an
/// utterance rather than a description of one (PR #1188 T18): the
/// [`unquoted_utterance`] the words after the clause's first `print_stdout`
/// word spell.
fn unquoted_output(clause: &str) -> Option<String> {
    let lexicon = seed::lexicon();
    let print = lexicon.meaning("print_stdout")?;
    let mut words = clause.split_whitespace();
    words
        .by_ref()
        .find(|word| print.evidenced_in(&bare_word(word)))?;
    unquoted_utterance(words)
}

/// The utterance `words` (the words after a print verb) open with, if they
/// open with one (PR #1188 T18).
///
/// The words up to the first seed clause separator
/// (`skill_procedure_clause_separator`: "and", "then", "и", "फिर", …) or
/// through the first word that ends a sentence. They are an utterance only
/// when they read as one:
///
/// * they open with a capital, the way a quoted utterance does mid-sentence
///   ("prints Hello, World!"), where a description opens with an ordinary
///   word ("prints prime numbers below 50");
/// * they do not describe a value ([`describes_a_value`]).
///
/// The coding task specification binds a program's stdout through the same
/// reading, so both bind the same output for one request.
pub(crate) fn unquoted_utterance<'a>(words: impl Iterator<Item = &'a str>) -> Option<String> {
    let lexicon = seed::lexicon();
    let separators = lexicon.words_for_role(seed::ROLE_SKILL_PROCEDURE_CLAUSE_SEPARATOR);
    let mut kept: Vec<&str> = Vec::new();
    for word in words {
        if separators.contains(&bare_word(word)) {
            break;
        }
        kept.push(word);
        if word.ends_with(UNQUOTED_OUTPUT_ENDS) {
            break;
        }
    }
    let joined = kept.join(" ");
    let output = joined.trim_end_matches(UNQUOTED_OUTPUT_PEEL);
    (output.chars().next().is_some_and(char::is_uppercase) && !describes_a_value(output))
        .then(|| output.to_owned())
}

/// Whether `output` describes a value the program must compute rather than
/// spelling the text it echoes (PR #1188 T18):
///
/// * a word is a seed function word (`statement_function_word`: "the", "of",
///   "for", …), which is how a described value is built ("the sum of a and
///   b", "a greeting", "`FizzBuzz` for 1 to 100");
/// * it mentions a coding structure (`coding_structure`: "the sum …",
///   "Fibonacci numbers …") or a program task (`program_task_alias`:
///   `FizzBuzz`), which name a computation, not text. The one task that *is*
///   its text is the greeting program: a task alias that is also a
///   `social_greeting` ("Hello, World!") stays an utterance.
pub(crate) fn describes_a_value(output: &str) -> bool {
    let lexicon = seed::lexicon();
    let lower = normalize_prompt(&output.to_lowercase());
    let function_words = lexicon.words_for_role(seed::ROLE_STATEMENT_FUNCTION_WORD);
    output
        .split_whitespace()
        .any(|word| function_words.contains(&bare_word(word)))
        || lexicon.mentions_role(seed::ROLE_CODING_STRUCTURE, &lower)
        || (lexicon.mentions_role(seed::ROLE_PROGRAM_TASK_ALIAS, &lower)
            && !lexicon.mentions_role(seed::ROLE_SOCIAL_GREETING, &lower))
}

/// The event kind an executor records one undischargeable clause under.
pub const OBLIGATION_GAP_KIND: &str = "obligation_gap";

/// Every gap line an executor reports for `text` (R1166-3, R1166-4).
///
/// The [`ObligationGraph::gap_report`] lines (clauses no rule can read)
/// followed by the [`ObligationGraph::unbound_output_report`] lines (output
/// literals no binding carries), so a completed run names each obligation it
/// could not discharge instead of omitting it.
#[must_use]
pub fn obligation_gap_lines(text: &str) -> Vec<String> {
    let graph = formalize_request(text);
    let mut lines = graph.gap_report();
    lines.extend(graph.unbound_output_report(&bound_output_literals(text)));
    lines
}

/// Record every undischargeable obligation of `text` in the executor's log.
///
/// One `obligation_gap` event per [`obligation_gap_lines`] line (node id,
/// byte span, reason). Returns the lines it recorded.
pub fn record_obligation_gaps(text: &str, log: &mut crate::event_log::EventLog) -> Vec<String> {
    let gaps = obligation_gap_lines(text);
    for gap in &gaps {
        log.append(OBLIGATION_GAP_KIND, gap.clone());
    }
    gaps
}

/// Classify one clause against the seed lexicon.
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
        || lexicon.mentions_role_raw(ROLE_OUTPUT_OBLIGATION_VERB, &lower);
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
    authoring_kind(clause).map(|kind| (kind, None))
}

/// The authoring obligation a clause demands — program, CI workflow, style,
/// naming or badge — read from its text outside quoted segments.
fn authoring_kind(clause: &str) -> Option<ObligationKind> {
    let normalized = normalize_prompt(&crate::solver_handlers::text_outside_quoted_segments(
        clause,
    ));
    let lower = normalized.to_lowercase();
    let lexicon = seed::lexicon();
    if lexicon.mentions_role(seed::ROLE_CI_WORKFLOW_REQUEST, &normalized) {
        return Some(ObligationKind::CiWorkflow);
    }
    if lexicon.mentions_role(seed::ROLE_PROGRAM_REQUEST, &normalized)
        || lexicon.mentions_role(seed::ROLE_CODING_REQUEST_VERB, &normalized)
    {
        return Some(ObligationKind::ProgramFile);
    }
    [
        (ROLE_CODE_STYLE_OBLIGATION, ObligationKind::CodeStyle),
        (ROLE_FILE_NAMING_OBLIGATION, ObligationKind::FileNaming),
        (ROLE_CI_BADGE_OBLIGATION, ObligationKind::CiBadge),
    ]
    .into_iter()
    .find(|(role, _)| lexicon.mentions_role_raw(role, &lower))
    .map(|(_, kind)| kind)
}

/// Stable graph id for a request: the same meaning yields the same id, so
/// parity fixtures can pin expected graphs without pinning clause order.
#[must_use]
#[allow(dead_code)] // drafted for the obligation parity fixtures, not yet read
pub fn graph_id(text: &str) -> String {
    stable_id(GRAPH_RECORD, text)
}
