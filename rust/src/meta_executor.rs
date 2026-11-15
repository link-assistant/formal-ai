//! Issue #1550 — the recursive procedure every answer goes through, in both
//! runtimes (the JavaScript twin is `js/agentic/crate/meta_executor.mjs`). At
//! every level it
//!
//! 1. splits the formalized request into tasks by the relations the shared
//!    grammar names (sequence, alternative, aggregation), in any supported
//!    language and never by an English conjunction written in the code;
//! 2. derives a test for each task as a first-class value that is executed;
//! 3. drafts every candidate a method can give (the turn's cache row, the
//!    direct derivation by the host solver, the composition of the selected
//!    sub-results);
//! 4. selects the smallest sufficient passing candidate by least action
//!    ([`crate::least_action::rank_by_least_action`], issue #491) and records
//!    why;
//! 5. composes the selected drafts by structure and checks the composition
//!    against the parent test;
//! 6. recurses into non-atomic tasks within the depth, breadth and budget
//!    bounds, all of which are visible in the trace.
//!
//! Every word and number the procedure reads is data in the structural rule
//! `task-decomposition` and the per-language `connective-*` vocabularies of
//! `data/seed/text-formalization.lino`. The events contain only integers and
//! strings derived from the request, so the JavaScript and native runtimes
//! emit the same sequence for the same request and host; the shared fixture
//! `rust/tests/fixtures/meta-executor-parity.json` pins it.

use std::collections::{BTreeMap, BTreeSet};

use crate::coding::contains_cjk;
use crate::formalization::{grammar_vocabulary, structural_rule_values};
use crate::least_action::{ActionCost, SolutionCandidate, rank_by_least_action};

#[path = "meta_executor/production.rs"]
mod production;
pub use production::{
    InProcessExecutor, adopted_composition, is_nested, last_turn_trace, nested, remember_trace,
    solve_answer, solve_turn,
};

const RULE: &str = "task-decomposition";

/// The event kinds, in the order a level emits them.
pub const EVENT_KINDS: [&str; 8] = [
    "meta_exec:bounds",
    "meta_exec:task",
    "meta_exec:test",
    "meta_exec:draft",
    "meta_exec:compose",
    "meta_exec:select",
    "meta_exec:unresolved",
    "meta_exec:result",
];

fn rule_values(kind: &str) -> Vec<String> {
    structural_rule_values(RULE, kind)
}

fn rule_value(kind: &str) -> String {
    let mut values = rule_values(kind);
    assert_eq!(values.len(), 1, "unique_task_decomposition_fact");
    values.remove(0)
}

fn rule_number(kind: &str) -> u32 {
    let text = rule_value(kind);
    assert!(
        !text.is_empty() && text.chars().all(|character| character.is_ascii_digit()),
        "task_decomposition_number"
    );
    text.parse().expect("task_decomposition_number")
}

/// The relations a level is split by, in precedence order.
#[must_use]
pub fn decomposition_relations() -> Vec<String> {
    rule_values("relation")
}

/// The recursion bounds of one run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bounds {
    pub depth: u32,
    pub breadth: u32,
    pub budget: u32,
}

/// Caller overrides of the seed's bounds; `None` keeps the seed's value.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BoundOverrides {
    pub depth: Option<u32>,
    pub breadth: Option<u32>,
    pub budget: Option<u32>,
}

/// The recursion bounds: the seed's values unless the caller overrides one.
///
/// # Panics
///
/// Panics when a bound of the seed's `task-decomposition` rule is missing or
/// not a number.
#[must_use]
pub fn decomposition_bounds(overrides: BoundOverrides) -> Bounds {
    Bounds {
        depth: overrides
            .depth
            .unwrap_or_else(|| rule_number("depth-bound")),
        breadth: overrides
            .breadth
            .unwrap_or_else(|| rule_number("breadth-bound")),
        budget: overrides
            .budget
            .unwrap_or_else(|| rule_number("budget-bound")),
    }
}

/// One event of a run: its kind and its `name=value` payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceEvent {
    pub kind: &'static str,
    pub payload: String,
}

fn emit(events: &mut Vec<TraceEvent>, kind: &'static str, fields: &[(&str, String)]) {
    let payload = fields
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join(" ");
    events.push(TraceEvent { kind, payload });
}

fn quoted_mask(characters: &[char]) -> Vec<bool> {
    let pairs: Vec<Vec<char>> = rule_values("quote-pair")
        .iter()
        .map(|pair| pair.chars().collect())
        .collect();
    assert!(
        pairs.iter().all(|pair| pair.len() == 2),
        "task_decomposition_quote_pair"
    );
    let mut mask = Vec::with_capacity(characters.len());
    let mut closer: Option<char> = None;
    for &character in characters {
        if let Some(close) = closer {
            mask.push(true);
            if character == close {
                closer = None;
            }
            continue;
        }
        if let Some(pair) = pairs.iter().find(|pair| pair[0] == character) {
            closer = Some(pair[1]);
            mask.push(true);
            continue;
        }
        mask.push(false);
    }
    mask
}

struct Marker {
    chars: Vec<String>,
    cjk: bool,
}

fn lower_char(character: char) -> String {
    character.to_lowercase().collect()
}

fn marker_matches(
    lowered: &[String],
    original: &[char],
    mask: &[bool],
    at: usize,
    marker: &Marker,
) -> bool {
    let end = at + marker.chars.len();
    if end > lowered.len() {
        return false;
    }
    for (offset, expected) in marker.chars.iter().enumerate() {
        if mask[at + offset] || &lowered[at + offset] != expected {
            return false;
        }
    }
    if marker.cjk {
        return true;
    }
    let edge_alphanumeric =
        |text: &String| text.chars().count() == 1 && text.chars().all(char::is_alphanumeric);
    if marker.chars.first().is_some_and(edge_alphanumeric)
        && at > 0
        && original[at - 1].is_alphanumeric()
    {
        return false;
    }
    if marker.chars.last().is_some_and(edge_alphanumeric)
        && end < original.len()
        && original[end].is_alphanumeric()
    {
        return false;
    }
    true
}

fn clean_part(characters: &[char], edges: &[String]) -> String {
    let text: String = characters.iter().collect();
    text.trim_matches(|character: char| {
        character.is_whitespace() || edges.iter().any(|edge| edge.chars().eq([character]))
    })
    .to_owned()
}

/// One level of the split.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Split {
    pub relation: String,
    pub parts: Vec<String>,
}

/// One level of the split: the first relation, in precedence order, whose
/// connectives cut the request into at least two non-empty parts. Text
/// inside a quotation is never cut.
#[must_use]
pub fn split_request(text: &str, language: &str) -> Option<Split> {
    let original: Vec<char> = text.chars().collect();
    let lowered: Vec<String> = original.iter().copied().map(lower_char).collect();
    let mask = quoted_mask(&original);
    let edges = rule_values("edge-punctuation");
    for relation in decomposition_relations() {
        let vocabulary = rule_value(&format!("{relation}-vocabulary"));
        let mut markers: Vec<Marker> = grammar_vocabulary(language, &vocabulary)
            .iter()
            .map(|word| Marker {
                chars: word.chars().map(lower_char).collect(),
                cjk: contains_cjk(word),
            })
            .filter(|marker| !marker.chars.is_empty())
            .collect();
        // Longest connective first, so "and then" is one sequence marker.
        markers.sort_by_key(|marker| std::cmp::Reverse(marker.chars.len()));
        let mut parts: Vec<&[char]> = Vec::new();
        let mut start = 0;
        let mut at = 0;
        while at < original.len() {
            if let Some(marker) = markers
                .iter()
                .find(|marker| marker_matches(&lowered, &original, &mask, at, marker))
            {
                parts.push(&original[start..at]);
                at += marker.chars.len();
                start = at;
            } else {
                at += 1;
            }
        }
        parts.push(&original[start..]);
        let cleaned: Vec<String> = parts
            .iter()
            .map(|part| clean_part(part, &edges))
            .filter(|part| !part.is_empty())
            .collect();
        if parts.len() > 1 && cleaned.len() >= 2 {
            return Some(Split {
                relation,
                parts: cleaned,
            });
        }
    }
    None
}

/// One task of the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskNode {
    pub id: String,
    pub text: String,
    pub depth: u32,
    pub relation: Option<String>,
    /// Why the task is atomic: `no_relation`, `depth_bound` or `breadth_bound`.
    pub atomic: Option<&'static str>,
    pub children: Vec<Self>,
}

/// The task tree of a request: a task splits while a relation applies and
/// the depth and breadth bounds allow; otherwise it is atomic and says why.
#[must_use]
pub fn plan_tasks(text: &str, language: &str, bounds: Bounds) -> TaskNode {
    plan_level(text, language, bounds, 0, "t0".to_owned())
}

fn plan_level(text: &str, language: &str, bounds: Bounds, depth: u32, id: String) -> TaskNode {
    let mut node = TaskNode {
        id,
        text: text.to_owned(),
        depth,
        relation: None,
        atomic: None,
        children: Vec::new(),
    };
    match split_request(text, language) {
        None => node.atomic = Some("no_relation"),
        Some(_) if depth >= bounds.depth => node.atomic = Some("depth_bound"),
        Some(split) if u32::try_from(split.parts.len()).unwrap_or(u32::MAX) > bounds.breadth => {
            node.atomic = Some("breadth_bound");
        }
        Some(split) => {
            node.children = split
                .parts
                .iter()
                .enumerate()
                .map(|(index, part)| {
                    plan_level(
                        part,
                        language,
                        bounds,
                        depth + 1,
                        format!("{}.{}", node.id, index + 1),
                    )
                })
                .collect();
            node.relation = Some(split.relation);
        }
    }
    node
}

/// The text of every atomic task, left to right.
#[must_use]
pub fn task_leaves(node: &TaskNode) -> Vec<String> {
    if node.children.is_empty() {
        vec![node.text.clone()]
    } else {
        node.children.iter().flat_map(task_leaves).collect()
    }
}

/// The shape of a task tree without its words:
/// `sequence(leaf,aggregation(leaf,leaf))`.
#[must_use]
pub fn decomposition_shape(node: &TaskNode) -> String {
    if node.children.is_empty() {
        return "leaf".to_owned();
    }
    format!(
        "{}({})",
        node.relation.as_deref().unwrap_or_default(),
        node.children
            .iter()
            .map(decomposition_shape)
            .collect::<Vec<_>>()
            .join(",")
    )
}

/// The test of a task, as a value: the checks it executes and, for a
/// composite task, how many of its sub-tasks must pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskTest {
    pub task: String,
    pub checks: Vec<String>,
    pub requires: Option<String>,
}

/// Derive the test of one task from the grammar.
#[must_use]
pub fn derive_task_test(node: &TaskNode) -> TaskTest {
    let mut checks = vec![rule_value("leaf-check")];
    let composite = node
        .relation
        .as_deref()
        .filter(|_| !node.children.is_empty());
    if let Some(relation) = composite {
        checks.push(format!("{}:{relation}", rule_value("composite-check")));
    }
    let requires = composite.map(|relation| rule_value(&format!("{relation}-requires")));
    TaskTest {
        task: node.id.clone(),
        checks,
        requires,
    }
}

/// A draft one method gave for one task.
#[derive(Debug, Clone, PartialEq)]
pub struct Draft {
    pub intent: String,
    pub content: String,
    pub confidence: f32,
}

/// The verdict of a task test on one candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    pub passed: bool,
    pub failed: Option<String>,
    pub executed: Vec<String>,
}

/// The outcome of one task.
#[derive(Debug, Clone, PartialEq)]
pub struct TaskOutcome {
    pub task: String,
    /// `resolved` or `unresolved`.
    pub status: &'static str,
    pub reason: Option<&'static str>,
    pub candidate: Option<Candidate>,
    /// The direct derivation, whether or not it passed.
    pub fallback: Option<Candidate>,
}

impl TaskOutcome {
    #[must_use]
    pub fn is_resolved(&self) -> bool {
        self.status == "resolved"
    }
}

/// One candidate of one task, with its method and cost.
#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    pub method: &'static str,
    pub draft: Draft,
    pub steps: u32,
    pub code_units: u32,
    pub passed: bool,
}

fn children_satisfy(test: &TaskTest, child_outcomes: &[TaskOutcome]) -> bool {
    let resolved = child_outcomes
        .iter()
        .filter(|outcome| outcome.is_resolved())
        .count();
    if test.requires.as_deref() == Some("any") {
        resolved >= 1
    } else {
        resolved == child_outcomes.len()
    }
}

/// Execute a task test against one candidate. Returns every check executed
/// and the first one that failed.
#[must_use]
pub fn run_task_test(
    test: &TaskTest,
    candidate: Option<&Draft>,
    child_outcomes: &[TaskOutcome],
) -> Verdict {
    let unresolved = rule_values("unresolved-intent");
    let unresolved_suffixes = rule_values("unresolved-suffix");
    let leaf_check = rule_value("leaf-check");
    let mut executed = Vec::new();
    let mut failed = None;
    for check in &test.checks {
        executed.push(check.clone());
        let passed = if *check == leaf_check {
            candidate.is_some_and(|draft| {
                !unresolved.contains(&draft.intent)
                    && !unresolved_suffixes
                        .iter()
                        .any(|suffix| draft.intent.ends_with(suffix.as_str()))
                    && !draft.content.trim().is_empty()
            })
        } else {
            children_satisfy(test, child_outcomes)
        };
        if !passed && failed.is_none() {
            failed = Some(check.clone());
        }
    }
    Verdict {
        passed: failed.is_none(),
        failed,
        executed,
    }
}

/// Issue #1550: the executed leaf test of the turn's answer.
///
/// The `validation` payload names the executed checks, the candidate they ran
/// on and the verdict (`answered(greeting)=true`).
#[must_use]
pub fn answer_validation(intent: &str, content: &str) -> String {
    let test = TaskTest {
        task: intent.to_owned(),
        checks: vec![rule_value("leaf-check")],
        requires: None,
    };
    let draft = Draft {
        intent: intent.to_owned(),
        content: content.to_owned(),
        confidence: 0.0,
    };
    let verdict = run_task_test(&test, Some(&draft), &[]);
    format!(
        "{}({intent})={}",
        verdict.executed.join("+"),
        verdict.passed
    )
}

/// Issue #1550: the selection among the turn's candidates by their executed tests.
///
/// The `trace:simplification` payload names the candidate of the first
/// passing test and how many of the executed tests passed (`17=1/4`), or
/// `fallback` when none passed (`unknown=0/1`).
#[must_use]
pub fn selection_record(validations: &[String], fallback: &str) -> String {
    let passing: Vec<&str> = validations
        .iter()
        .filter_map(|validation| {
            let (subject, verdict) = validation.rsplit_once('=')?;
            verdict.parse::<bool>().ok().filter(|passed| *passed)?;
            let (_, candidate) = subject.strip_suffix(')')?.split_once('(')?;
            Some(candidate)
        })
        .collect();
    let chosen = passing.first().copied().unwrap_or(fallback);
    format!("{chosen}={}/{}", passing.len(), validations.len().max(1))
}

fn compose(node: &TaskNode, test: &TaskTest, child_outcomes: &[TaskOutcome]) -> Option<Candidate> {
    let resolved: Vec<&Candidate> = child_outcomes
        .iter()
        .filter_map(|outcome| outcome.candidate.as_ref().filter(|_| outcome.is_resolved()))
        .collect();
    if !children_satisfy(test, child_outcomes) || resolved.is_empty() {
        return None;
    }
    let chosen: &[&Candidate] = if test.requires.as_deref() == Some("any") {
        &resolved[..1]
    } else {
        resolved.as_slice()
    };
    let content = chosen
        .iter()
        .map(|candidate| candidate.draft.content.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");
    let confidence = chosen
        .iter()
        .map(|candidate| candidate.draft.confidence)
        .fold(f32::INFINITY, f32::min);
    let steps = rule_number("composition-cost")
        + chosen.iter().map(|candidate| candidate.steps).sum::<u32>();
    Some(Candidate {
        method: "composition",
        draft: Draft {
            intent: format!(
                "{}_composition",
                node.relation.as_deref().unwrap_or_default()
            ),
            content,
            confidence,
        },
        steps,
        code_units: 0,
        passed: false,
    })
}

const fn candidate_of(method: &'static str, steps: u32, draft: Draft) -> Candidate {
    Candidate {
        method,
        draft,
        steps,
        code_units: 0,
        passed: false,
    }
}

/// The methods the procedure asks the host for.
pub trait MetaHost {
    /// The direct derivation: the host solver on the task's text.
    fn draft(&mut self, text: &str, task: &TaskNode) -> Option<Draft>;
    /// A stored row of an earlier run of this procedure, if any.
    fn cached(&mut self, _text: &str, _task: &TaskNode) -> Option<Draft> {
        None
    }
}

struct RunState {
    events: Vec<TraceEvent>,
    budget: u32,
    memo: BTreeMap<String, Draft>,
    withheld: BTreeSet<String>,
}

fn count_chars(text: &str) -> u32 {
    u32::try_from(text.chars().count()).unwrap_or(u32::MAX)
}

fn emit_task(state: &mut RunState, node: &TaskNode) {
    emit(
        &mut state.events,
        "meta_exec:task",
        &[
            ("id", node.id.clone()),
            ("depth", node.depth.to_string()),
            (
                "relation",
                node.relation.clone().unwrap_or_else(|| "-".to_owned()),
            ),
            ("children", node.children.len().to_string()),
            ("atomic", node.atomic.unwrap_or("-").to_owned()),
            ("text", node.text.clone()),
        ],
    );
}

fn draft_candidates(
    node: &TaskNode,
    test: &TaskTest,
    child_outcomes: &[TaskOutcome],
    host: &mut dyn MetaHost,
    state: &mut RunState,
) -> (Vec<Candidate>, bool) {
    let mut candidates = Vec::new();
    let remembered = state
        .memo
        .get(&node.text)
        .cloned()
        .or_else(|| host.cached(&node.text, node));
    if let Some(draft) = remembered {
        candidates.push(candidate_of("cache", rule_number("cache-cost"), draft));
    }
    let mut budget_exhausted = false;
    if state.budget > 0 {
        state.budget -= 1;
        if let Some(draft) = host.draft(&node.text, node) {
            candidates.push(candidate_of(
                "derivation",
                rule_number("derivation-cost"),
                draft,
            ));
        }
    } else {
        budget_exhausted = true;
    }
    if !node.children.is_empty() {
        let composition = compose(node, test, child_outcomes);
        let resolved = child_outcomes
            .iter()
            .filter(|outcome| outcome.is_resolved())
            .count();
        emit(
            &mut state.events,
            "meta_exec:compose",
            &[
                ("task", node.id.clone()),
                ("relation", node.relation.clone().unwrap_or_default()),
                (
                    "children",
                    node.children
                        .iter()
                        .map(|child| child.id.as_str())
                        .collect::<Vec<_>>()
                        .join(","),
                ),
                ("resolved", format!("{resolved}/{}", child_outcomes.len())),
                (
                    "built",
                    if composition.is_some() {
                        "true"
                    } else {
                        "false"
                    }
                    .to_owned(),
                ),
            ],
        );
        if let Some(composition) = composition {
            candidates.push(composition);
        }
    }
    (candidates, budget_exhausted)
}

fn run_task(node: &TaskNode, host: &mut dyn MetaHost, state: &mut RunState) -> TaskOutcome {
    emit_task(state, node);
    if state.withheld.contains(&node.id) {
        emit(
            &mut state.events,
            "meta_exec:test",
            &[
                ("task", node.id.clone()),
                ("checks", "-".to_owned()),
                ("withheld", "true".to_owned()),
            ],
        );
        emit(
            &mut state.events,
            "meta_exec:unresolved",
            &[("task", node.id.clone()), ("reason", "no_test".to_owned())],
        );
        return TaskOutcome {
            task: node.id.clone(),
            status: "unresolved",
            reason: Some("no_test"),
            candidate: None,
            fallback: None,
        };
    }
    let test = derive_task_test(node);
    emit(
        &mut state.events,
        "meta_exec:test",
        &[
            ("task", node.id.clone()),
            ("checks", test.checks.join(",")),
            (
                "requires",
                test.requires.clone().unwrap_or_else(|| "-".to_owned()),
            ),
        ],
    );
    let child_outcomes: Vec<TaskOutcome> = node
        .children
        .iter()
        .map(|child| run_task(child, host, state))
        .collect();
    let (mut candidates, budget_exhausted) =
        draft_candidates(node, &test, &child_outcomes, host, state);
    select(
        node,
        &test,
        &child_outcomes,
        &mut candidates,
        budget_exhausted,
        state,
    )
}

fn select(
    node: &TaskNode,
    test: &TaskTest,
    child_outcomes: &[TaskOutcome],
    candidates: &mut [Candidate],
    budget_exhausted: bool,
    state: &mut RunState,
) -> TaskOutcome {
    let mut ranked = Vec::with_capacity(candidates.len());
    for candidate in candidates.iter_mut() {
        let verdict = run_task_test(test, Some(&candidate.draft), child_outcomes);
        candidate.passed = verdict.passed;
        candidate.code_units = count_chars(&candidate.draft.content);
        emit(
            &mut state.events,
            "meta_exec:draft",
            &[
                ("task", node.id.clone()),
                ("method", candidate.method.to_owned()),
                ("steps", candidate.steps.to_string()),
                ("code_units", candidate.code_units.to_string()),
                ("executed", verdict.executed.join(",")),
                ("passed", verdict.passed.to_string()),
                ("failed", verdict.failed.unwrap_or_else(|| "-".to_owned())),
            ],
        );
        ranked.push(SolutionCandidate {
            id: candidate.method.to_owned(),
            solves_entire_range: verdict.passed,
            cost: ActionCost {
                steps: candidate.steps,
                code_units: candidate.code_units,
                compute_ms: 0,
                memory_kb: 0,
            },
        });
    }
    if budget_exhausted {
        emit(
            &mut state.events,
            "meta_exec:draft",
            &[
                ("task", node.id.clone()),
                ("method", "derivation".to_owned()),
                ("skipped", "budget".to_owned()),
            ],
        );
    }
    let order = rank_by_least_action(&ranked);
    let fallback = candidates
        .iter()
        .find(|candidate| candidate.method != "composition" && candidate.method != "cache")
        .cloned();
    let Some(best) = order.first() else {
        let reason = if candidates.is_empty() {
            if budget_exhausted {
                "budget"
            } else {
                "no_draft"
            }
        } else if !node.children.is_empty() && !children_satisfy(test, child_outcomes) {
            "children"
        } else {
            "no_passing_draft"
        };
        emit(
            &mut state.events,
            "meta_exec:unresolved",
            &[("task", node.id.clone()), ("reason", reason.to_owned())],
        );
        return TaskOutcome {
            task: node.id.clone(),
            status: "unresolved",
            reason: Some(reason),
            candidate: None,
            fallback,
        };
    };
    let selected = candidates
        .iter()
        .find(|candidate| candidate.method == best.id && candidate.passed)
        .cloned()
        .expect("a ranked candidate is a passing candidate");
    emit(
        &mut state.events,
        "meta_exec:select",
        &[
            ("task", node.id.clone()),
            ("method", selected.method.to_owned()),
            ("reason", "least_action".to_owned()),
            ("steps", selected.steps.to_string()),
            ("code_units", selected.code_units.to_string()),
            ("passing", format!("{}/{}", order.len(), candidates.len())),
        ],
    );
    state.memo.insert(node.text.clone(), selected.draft.clone());
    TaskOutcome {
        task: node.id.clone(),
        status: "resolved",
        reason: None,
        candidate: Some(selected),
        fallback,
    }
}

/// One request of the procedure.
#[derive(Debug, Clone, Copy)]
pub struct Request<'a> {
    pub text: &'a str,
    pub language: &'a str,
}

/// Options of one run: bound overrides and the task ids whose test is removed.
#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    pub bounds: BoundOverrides,
    pub withheld_tests: Vec<String>,
}

/// The whole run: bounds, task tree, root outcome and the event sequence.
#[derive(Debug, Clone, PartialEq)]
pub struct ProcedureRun {
    pub bounds: Bounds,
    pub root: TaskNode,
    pub outcome: TaskOutcome,
    pub events: Vec<TraceEvent>,
}

/// Run the whole procedure on one request.
///
/// # Panics
///
/// Panics when the seed's `task-decomposition` rule is malformed (a missing,
/// repeated or non-numeric fact), which the shared grammar tests rule out.
#[must_use]
pub fn execute_procedure(
    request: Request<'_>,
    host: &mut dyn MetaHost,
    options: &RunOptions,
) -> ProcedureRun {
    let bounds = decomposition_bounds(options.bounds);
    let mut state = RunState {
        events: Vec::new(),
        budget: bounds.budget,
        memo: BTreeMap::new(),
        withheld: options.withheld_tests.iter().cloned().collect(),
    };
    emit(
        &mut state.events,
        "meta_exec:bounds",
        &[
            ("depth", bounds.depth.to_string()),
            ("breadth", bounds.breadth.to_string()),
            ("budget", bounds.budget.to_string()),
        ],
    );
    let root = plan_tasks(request.text, request.language, bounds);
    let outcome = run_task(&root, host, &mut state);
    emit(
        &mut state.events,
        "meta_exec:result",
        &[
            ("task", root.id.clone()),
            ("status", outcome.status.to_owned()),
            (
                "method",
                outcome
                    .candidate
                    .as_ref()
                    .map_or("-", |candidate| candidate.method)
                    .to_owned(),
            ),
            ("budget_left", state.budget.to_string()),
        ],
    );
    ProcedureRun {
        bounds,
        root,
        outcome,
        events: state.events,
    }
}
