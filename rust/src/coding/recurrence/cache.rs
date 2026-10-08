//! The committed recurrence source cache, read as the browser worker reads it.
//!
//! The cache `js/source-cache/wikifunctions-recurrences.lino` holds recurrences the
//! example `generate_recurrence_source_cache` formalized from captured
//! Wikifunctions abstract definitions, their source testers and the Wikidata
//! aliases of the concept each one computes. The browser answers a function
//! request that names one of them through `trySourceRecurrenceSynthesis`
//! (`js/worker/formal_ai_worker_recurrence.js`); this module is its native
//! twin, so the offline solver reads the same procedure memory instead of
//! ending in a skill gap (issue #1173 R1173-3, routing probes p223-p225).
//!
//! The cache is procedure memory, not seed knowledge: deleting it and running
//! the example rebuilds it byte for byte, and `rust/embedded/` mirrors it
//! (`scripts/mirror-package-data.rs`).

use std::sync::OnceLock;

use unicode_general_category::{GeneralCategory, get_general_category};

use crate::coding::concept_discovery::CandidatePart;
use crate::coding::recurrence::{Expression, Operation, Recurrence};
use crate::coding::task_spec::{ArtifactShape, CodingTaskSpec, Example};
use crate::seed::parser::{LinoNode, parse_lino};

const RECURRENCE_SOURCE_CACHE_LINO: &str =
    include_str!("../../../embedded/js/source-cache/wikifunctions-recurrences.lino");

/// The candidate kind the composition renders a source recurrence as.
const RECURRENCE_CANDIDATE_KIND: &str = "wikifunctions_recurrence";

/// The evaluation depth past which a recurrence is not evaluated further.
const EVALUATION_DEPTH_LIMIT: usize = 10_000;

/// One `recurrence` record of the cache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedRecurrence {
    /// The Wikifunctions function the record formalizes (`Z13667`).
    pub zid: String,
    /// The callable name the record declares (`factorial`).
    pub identifier: String,
    /// The label and every alias text, label first, in cache order.
    pub forms: Vec<String>,
    /// The formalized recurrence, when its expression uses only the
    /// operations the evaluator knows.
    pub recurrence: Option<Recurrence>,
    /// The source testers, argument and expected value as written.
    pub source_tests: Vec<Example>,
    /// The Wikidata concept page the aliases came from.
    pub concept_source_url: String,
}

/// Every record of the committed cache, in cache order.
///
/// The JavaScript twin is `recurrenceSourceRecords`.
#[must_use]
pub fn recurrence_source_records() -> &'static [CachedRecurrence] {
    static RECORDS: OnceLock<Vec<CachedRecurrence>> = OnceLock::new();
    RECORDS.get_or_init(|| records_from(RECURRENCE_SOURCE_CACHE_LINO))
}

/// Read the `recurrence` records of a cache document.
#[must_use]
pub fn records_from(text: &str) -> Vec<CachedRecurrence> {
    let tree = parse_lino(text);
    tree.children
        .iter()
        .find(|node| node.name == "recurrence_source_cache")
        .map_or_else(Vec::new, |catalog| {
            catalog
                .children
                .iter()
                .filter(|node| node.name == "recurrence")
                .map(cached_recurrence)
                .collect()
        })
}

fn cached_recurrence(record: &LinoNode) -> CachedRecurrence {
    let label = record.find_child_value("label").to_owned();
    let forms = std::iter::once(label.clone())
        .chain(
            record
                .children
                .iter()
                .filter(|child| child.name == "alias")
                .map(|alias| alias.find_child_value("text").to_owned()),
        )
        .collect();
    let source_tests = record
        .children
        .iter()
        .filter(|child| child.name == "source_test")
        .map(|test| Example {
            arguments: vec![test.find_child_value("argument").to_owned()],
            expected: test.find_child_value("expected").to_owned(),
        })
        .collect();
    let parameter = record.find_child_value("parameter");
    let recurrence = expression_root(record)
        .and_then(expression_from)
        .filter(|_| !parameter.is_empty())
        .map(|expression| Recurrence {
            source_function_zid: record.id.clone(),
            source_implementation_zid: record
                .find_child_value("abstract_implementation")
                .to_owned(),
            source_label: label,
            parameter: parameter.to_owned(),
            expression,
            predecessor_offsets: record
                .children
                .iter()
                .filter(|child| child.name == "predecessor_offset")
                .filter_map(|child| child.id.parse().ok())
                .collect(),
            source_url: record.find_child_value("source_url").to_owned(),
            source_sha256: record.find_child_value("sha256").to_owned(),
            fetched_at: record.find_child_value("fetched_at").to_owned(),
            license: record.find_child_value("license").to_owned(),
            operator_sources: Vec::new(),
        });
    CachedRecurrence {
        zid: record.id.clone(),
        identifier: record.find_child_value("identifier").to_owned(),
        forms,
        recurrence,
        source_tests,
        concept_source_url: record.find_child_value("concept_source_url").to_owned(),
    }
}

/// The single node under a record's `expression` container.
///
/// The JavaScript twin is `recurrenceExpressionNode`.
fn expression_root(record: &LinoNode) -> Option<&LinoNode> {
    let container = record
        .children
        .iter()
        .find(|child| child.name == "expression")?;
    match container.children.as_slice() {
        [root] => Some(root),
        _ => None,
    }
}

/// The recurrence expression a cache node spells.
///
/// `None` when a node names an operation the evaluator does not know or has
/// the wrong arity: the cases `renderRecurrenceExpression` and
/// `evaluateRecurrenceExpression` return `null` for.
fn expression_from(node: &LinoNode) -> Option<Expression> {
    let operands = || {
        node.children
            .iter()
            .map(expression_from)
            .collect::<Option<Vec<_>>>()
    };
    let apply = |operation: Operation, arity: usize| {
        operands()
            .filter(|operands| operands.len() == arity)
            .map(|operands| Expression::Apply(operation, operands))
    };
    match node.name.as_str() {
        "parameter" => Some(Expression::Parameter(node.id.clone())),
        "literal" => node.id.parse().ok().map(Expression::Literal),
        "recur" => operands()
            .filter(|operands| operands.len() == 1)
            .and_then(|operands| operands.into_iter().next())
            .map(|argument| Expression::Recur(Box::new(argument))),
        "conditional" => apply(Operation::Conditional, 3),
        "less_equal" => apply(Operation::LessEqual, 2),
        "equal" => apply(Operation::Equal, 2),
        "add" => apply(Operation::Add, 2),
        "multiply" => apply(Operation::Multiply, 2),
        "subtract" => apply(Operation::Subtract, 2),
        "subtract_one" => apply(Operation::SubtractOne, 1),
        _ => None,
    }
}

/// `value` folded for matching.
///
/// Lowercase, combining marks dropped, every run of characters that is not a
/// letter, a number or `!` turned into one space, trimmed. The JavaScript
/// twin is `recurrenceFold`, which decomposes to NFD before it drops the
/// marks. The crate carries no decomposition table, so a letter
/// written precomposed (`é`) keeps its mark here; a form written with
/// combining marks, as Devanagari is (`फ़` is `फ` and a nukta), folds exactly
/// as the browser folds it, and both sides of a match are folded alike.
#[must_use]
pub fn recurrence_fold(value: &str) -> String {
    let mut folded = String::with_capacity(value.len());
    let mut pending_space = false;
    for character in value.chars().flat_map(char::to_lowercase) {
        match get_general_category(character) {
            GeneralCategory::NonspacingMark
            | GeneralCategory::SpacingMark
            | GeneralCategory::EnclosingMark => {}
            category if keeps(category, character) => {
                if pending_space && !folded.is_empty() {
                    folded.push(' ');
                }
                pending_space = false;
                folded.push(character);
            }
            _ => pending_space = true,
        }
    }
    folded
}

const fn keeps(category: GeneralCategory, character: char) -> bool {
    matches!(
        category,
        GeneralCategory::UppercaseLetter
            | GeneralCategory::LowercaseLetter
            | GeneralCategory::TitlecaseLetter
            | GeneralCategory::ModifierLetter
            | GeneralCategory::OtherLetter
            | GeneralCategory::DecimalNumber
            | GeneralCategory::LetterNumber
            | GeneralCategory::OtherNumber
    ) || character == '!'
}

/// The record whose label or alias the prompt names.
///
/// The longest folded form of at least three characters wins, the earlier
/// record on a tie.
/// The JavaScript twin is `recurrenceSourceMatch`.
#[must_use]
pub fn recurrence_source_match(prompt: &str) -> Option<&'static CachedRecurrence> {
    let folded = recurrence_fold(prompt);
    let mut best: Option<(&CachedRecurrence, usize)> = None;
    for record in recurrence_source_records() {
        let matched = record
            .forms
            .iter()
            .map(|form| recurrence_fold(form))
            .filter(|form| form.chars().count() >= 3 && folded.contains(form.as_str()))
            .map(|form| form.chars().count())
            .max();
        if let Some(length) = matched
            && best.is_none_or(|(_, kept)| length > kept)
        {
            best = Some((record, length));
        }
    }
    best.map(|(record, _)| record)
}

/// A value the recurrence evaluator computes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecurrenceValue {
    /// An integer.
    Number(i64),
    /// The result of a comparison.
    Truth(bool),
}

/// Evaluate `expression` with `parameter` bound to `value`.
///
/// A recursive call evaluates `root` again. `None` when an operand is
/// missing, a recursive argument is not a natural number, the arithmetic
/// overflows or the depth limit is passed.
/// The JavaScript twin is `evaluateRecurrenceExpression`.
#[must_use]
pub fn evaluate_recurrence_expression(
    expression: &Expression,
    root: &Expression,
    parameter: &str,
    value: i64,
    depth: usize,
) -> Option<RecurrenceValue> {
    if depth > EVALUATION_DEPTH_LIMIT {
        return None;
    }
    let evaluate =
        |node: &Expression| evaluate_recurrence_expression(node, root, parameter, value, depth + 1);
    let number = |node: &Expression| match evaluate(node)? {
        RecurrenceValue::Number(number) => Some(number),
        RecurrenceValue::Truth(_) => None,
    };
    match expression {
        Expression::Parameter(name) => {
            (name == parameter).then_some(RecurrenceValue::Number(value))
        }
        Expression::Literal(literal) => Some(RecurrenceValue::Number(*literal)),
        Expression::Recur(argument) => {
            let argument = number(argument).filter(|argument| *argument >= 0)?;
            evaluate_recurrence_expression(root, root, parameter, argument, depth + 1)
        }
        Expression::Apply(operation, operands) => match (operation, operands.as_slice()) {
            (Operation::Conditional, [condition, then, otherwise]) => {
                let holds = match evaluate(condition)? {
                    RecurrenceValue::Truth(holds) => holds,
                    RecurrenceValue::Number(number) => number != 0,
                };
                evaluate(if holds { then } else { otherwise })
            }
            (Operation::LessEqual, [left, right]) => {
                Some(RecurrenceValue::Truth(number(left)? <= number(right)?))
            }
            (Operation::Equal, [left, right]) => {
                Some(RecurrenceValue::Truth(evaluate(left)? == evaluate(right)?))
            }
            (Operation::Add, [left, right]) => number(left)?
                .checked_add(number(right)?)
                .map(RecurrenceValue::Number),
            (Operation::Multiply, [left, right]) => number(left)?
                .checked_mul(number(right)?)
                .map(RecurrenceValue::Number),
            (Operation::Subtract, [left, right]) => number(left)?
                .checked_sub(number(right)?)
                .map(RecurrenceValue::Number),
            (Operation::SubtractOne, [operand]) => {
                number(operand)?.checked_sub(1).map(RecurrenceValue::Number)
            }
            _ => None,
        },
    }
}

/// Whether a record's recurrence reproduces its source tests.
///
/// It must carry at least one, and every one must hold.
/// The JavaScript twin is `verifiedRecurrenceRecord`.
#[must_use]
pub fn verified_recurrence_record(record: &CachedRecurrence) -> bool {
    let Some(recurrence) = &record.recurrence else {
        return false;
    };
    !record.source_tests.is_empty()
        && record.source_tests.iter().all(|test| {
            let (Some(argument), Ok(expected)) = (
                test.arguments
                    .first()
                    .and_then(|value| value.parse::<i64>().ok()),
                test.expected.parse::<i64>(),
            ) else {
                return false;
            };
            evaluate_recurrence_expression(
                &recurrence.expression,
                &recurrence.expression,
                &recurrence.parameter,
                argument,
                0,
            ) == Some(RecurrenceValue::Number(expected))
        })
}

/// The Python function a verified record renders, named by its `identifier`.
///
/// The JavaScript twin is `renderRecurrenceExpression` with the
/// `def … return …` line `trySourceRecurrenceSynthesis` wraps it in; both go
/// through [`Recurrence::render_python`].
#[must_use]
pub fn render_cached_recurrence(record: &CachedRecurrence) -> Option<String> {
    let recurrence = record.recurrence.as_ref()?;
    (!record.identifier.is_empty()).then(|| recurrence.render_python(&record.identifier))
}

/// The source candidate a Python function request gets from the cache.
///
/// It is the recurrence the request names, verified against its source
/// tests, offered to composition as a `wikifunctions_recurrence` part that
/// carries those tests, so the draft is executed against them before it is
/// selected.
/// The JavaScript twin is the matching and verifying half of
/// `trySourceRecurrenceSynthesis`; the browser evaluates the AST where the
/// native composition also runs the rendered program under `python3`.
#[must_use]
pub fn source_recurrence_candidate(spec: &CodingTaskSpec) -> Option<CandidatePart> {
    if spec.language != "python" || spec.artifact_shape != ArtifactShape::Function {
        return None;
    }
    let record = recurrence_source_match(&spec.requirement_sentences.join(" "))?;
    if !verified_recurrence_record(record) {
        return None;
    }
    let recurrence = record.recurrence.as_ref()?;
    let code = render_cached_recurrence(record)?;
    Some(CandidatePart {
        id: recurrence.source_implementation_zid.clone(),
        kind: RECURRENCE_CANDIDATE_KIND.to_owned(),
        label: recurrence.source_label.clone(),
        language: Some(spec.language.clone()),
        code: Some(code),
        callable_name: Some(record.identifier.clone()),
        source_tests: record.source_tests.clone(),
        license: recurrence.license.clone(),
        source_url: recurrence.source_url.clone(),
        sha256: recurrence.source_sha256.clone(),
        fetched_at: recurrence.fetched_at.clone(),
        score: 1.0,
    })
}
