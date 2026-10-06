//! Synthesis: typed programs enumerated over the instruction set, shortest
//! first; examples are the goal state (difference = failing examples), and
//! without examples the grounded words and the request's clause order rank
//! the programs.
#![allow(clippy::float_cmp, clippy::cast_precision_loss)]

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use super::catalog::{Catalog, OpId, Step, TypeId, is_list_type};
use super::grounding::Clause;
use super::interpreter::{Param, Runtime, infer_parameter, run_program};
use super::phrases::{ALTERNATIVES, COMPOSE, COUNTEREXAMPLE, MEANING_GOAL, SEARCH, fill};
use super::seed::{Hypothesis, meta_seed};
use super::text::Example;
use super::value::Value;
use super::{BOUNDS, Trace};

/// The most programs one enumeration visits. The JavaScript materialises
/// every program of a length and runs out of memory long before this; the
/// bound only keeps a request the JavaScript cannot answer from stalling.
pub const ENUMERATION_CEILING: usize = 4_000_000;

/// A program found from examples.
#[derive(Debug, Clone, PartialEq)]
pub struct ExampleProgram {
    /// The program's steps.
    pub steps: Vec<Step>,
    /// The inferred parameter.
    pub parameter: Param,
    /// How many other programs were as well evidenced.
    pub alternatives: usize,
    /// How many candidates were evaluated.
    pub evaluated: usize,
}

/// A program found from the request's meaning.
#[derive(Debug, Clone, PartialEq)]
pub struct MeaningProgram {
    /// The program's steps.
    pub steps: Vec<Step>,
    /// The type it takes.
    pub from_type: String,
    /// The parameter bound from the request.
    pub parameter: Param,
    /// Grounded word groups the program leaves unexplained.
    pub uncovered: usize,
}

/// The result of running the program on every example.
#[derive(Debug, Clone, PartialEq)]
pub struct Verification {
    /// Examples the program reproduces.
    pub passed: usize,
    /// Examples given.
    pub total: usize,
    /// The examples it does not reproduce, with what it gave.
    pub failures: Vec<(Example, Option<Value>)>,
}

/// The labels of a program's steps joined by the composition sign.
///
/// Mirrors `program.steps.map(metaStepLabel).join(" ∘ ")` in
/// js/worker/formal_ai_worker_meta_synthesis.js.
#[must_use]
pub fn program_label(catalog: &Catalog, steps: &[Step]) -> String {
    steps
        .iter()
        .map(|step| catalog.step_label(*step))
        .collect::<Vec<_>>()
        .join(COMPOSE)
}

/// The distinct operations of a program in first-use order.
fn distinct_ops(catalog: &Catalog, steps: &[Step], main: bool, out: &mut Vec<OpId>) {
    out.clear();
    let mut buffer = Vec::with_capacity(3);
    for step in steps {
        buffer.clear();
        if main {
            catalog.main_ops(*step, &mut buffer);
        } else {
            catalog.step_ops(*step, &mut buffer);
        }
        for op in &buffer {
            if !out.contains(op) {
                out.push(*op);
            }
        }
    }
}

/// Evidence for a program: the grounded score of every operation it uses.
///
/// Mirrors `metaEvidenceScore` in js/worker/formal_ai_worker_meta_synthesis.js.
#[must_use]
pub fn evidence_score(catalog: &Catalog, steps: &[Step], evidence: &[f64]) -> f64 {
    let mut ops = Vec::new();
    distinct_ops(catalog, steps, false, &mut ops);
    let mut score = 0.0;
    for op in ops {
        score += evidence.get(op as usize).copied().unwrap_or(0.0);
    }
    score
}

/// Run a program on every example; the verifier never trusts the
/// enumerator's own evaluation.
///
/// Mirrors `metaVerify` in js/worker/formal_ai_worker_meta_synthesis.js.
#[must_use]
pub fn verify(
    catalog: &Catalog,
    steps: &[Step],
    parameter: Param,
    examples: &[Example],
) -> Verification {
    let mut failures = Vec::new();
    for example in examples {
        let actual = run_program(catalog, steps, &example.input, parameter, Runtime::Worker);
        let same = actual
            .as_ref()
            .is_some_and(|value| value.to_json() == example.output.to_json());
        if !same {
            failures.push((example.clone(), actual));
        }
    }
    Verification {
        passed: examples.len() - failures.len(),
        total: examples.len(),
        failures,
    }
}

/// A candidate kept for evaluation, ordered best first.
#[derive(Debug, Clone, Copy)]
struct Candidate {
    score: f64,
    sequence: usize,
    steps: [Option<Step>; 4],
}

impl Candidate {
    fn steps(&self) -> Vec<Step> {
        self.steps.iter().flatten().copied().collect()
    }
}

impl PartialEq for Candidate {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Candidate {}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Candidate {
    /// Greater is worse: lower evidence, then later in enumeration order.
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .score
            .total_cmp(&self.score)
            .then_with(|| self.sequence.cmp(&other.sequence))
    }
}

fn json_or_undefined(value: Option<&Value>) -> String {
    value.map_or_else(|| String::from("undefined"), Value::to_json)
}

/// Search for a program whose difference from the goal (failing examples)
/// is zero: shortest first, then most evidenced.
///
/// Mirrors `metaSynthesizeFromExamples` in js/worker/formal_ai_worker_meta_synthesis.js.
pub fn synthesize_from_examples(
    catalog: &Catalog,
    examples: &[Example],
    evidence: &[f64],
    trace: &mut Trace,
) -> Option<ExampleProgram> {
    let first = examples.first()?;
    let from_type = first.input.type_name();
    let to_type = first.output.type_name();
    trace.emit(
        "goal",
        meta_seed().note(
            "examples_goal",
            &[
                ("from", from_type),
                ("to", to_type),
                ("count", &examples.len().to_string()),
            ],
        ),
    );
    let mut evaluated = 0_usize;
    let mut rejected = 0_usize;
    let from = catalog.type_id(from_type);
    for length in 1..=BOUNDS.program_length {
        let keep = BOUNDS.candidate_budget.saturating_sub(evaluated);
        let mut typed = 0_usize;
        let mut heap: BinaryHeap<Candidate> = BinaryHeap::new();
        if let Some(from) = from {
            let mut ceiling = ENUMERATION_CEILING;
            catalog.for_each_program(from, length, &mut ceiling, &mut |steps, types| {
                let program_type = catalog.type_name(types.last().copied().unwrap_or_default());
                let fits = program_type == to_type
                    || (to_type == "list_any" && is_list_type(program_type));
                if !fits {
                    return;
                }
                let mut stored = [None; 4];
                for (slot, step) in stored.iter_mut().zip(steps) {
                    *slot = Some(*step);
                }
                let candidate = Candidate {
                    score: evidence_score(catalog, steps, evidence),
                    sequence: typed,
                    steps: stored,
                };
                typed += 1;
                if keep == 0 {
                    return;
                }
                heap.push(candidate);
                if heap.len() > keep {
                    heap.pop();
                }
            });
        }
        let ordered = heap.into_sorted_vec();
        let mut passing: Vec<(Vec<Step>, Param, f64)> = Vec::new();
        for candidate in &ordered {
            if evaluated >= BOUNDS.candidate_budget {
                break;
            }
            evaluated += 1;
            let steps = candidate.steps();
            let parameter = infer_parameter(catalog, &steps, examples);
            if parameter == Param::Undefined {
                continue;
            }
            let mut difference = 0_usize;
            let mut counterexample: Option<(&Example, Option<Value>)> = None;
            for example in examples {
                let actual =
                    run_program(catalog, &steps, &example.input, parameter, Runtime::Worker);
                let same = actual
                    .as_ref()
                    .is_some_and(|value| value.to_json() == example.output.to_json());
                if !same {
                    difference += 1;
                    if counterexample.is_none() {
                        counterexample = Some((example, actual));
                    }
                }
            }
            if difference == 0 {
                passing.push((steps, parameter, candidate.score));
            } else {
                rejected += 1;
                if rejected <= BOUNDS.rejections_traced
                    && candidate.score > 0.0
                    && let Some((example, actual)) = &counterexample
                {
                    trace.emit(
                        "counterexample",
                        fill(
                            COUNTEREXAMPLE,
                            &[
                                &program_label(catalog, &steps),
                                &example.input.to_json(),
                                &json_or_undefined(actual.as_ref()),
                                &example.output.to_json(),
                                &difference.to_string(),
                            ],
                        ),
                    );
                }
            }
        }
        trace.emit(
            "search",
            fill(
                SEARCH,
                &[
                    &length.to_string(),
                    &typed.to_string(),
                    &passing.len().to_string(),
                ],
            ),
        );
        if !passing.is_empty() {
            passing.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(Ordering::Equal));
            let best_evidence = passing[0].2;
            let ties: Vec<&(Vec<Step>, Param, f64)> = passing
                .iter()
                .filter(|item| item.2 == best_evidence)
                .collect();
            if ties.len() > 1 {
                let kept = ties
                    .iter()
                    .take(3)
                    .map(|item| program_label(catalog, &item.0))
                    .collect::<Vec<_>>()
                    .join(ALTERNATIVES);
                trace.emit(
                    "tie",
                    meta_seed().note(
                        "tie",
                        &[("count", &ties.len().to_string()), ("kept", &kept)],
                    ),
                );
            }
            let alternatives = ties.len() - 1;
            let (steps, parameter, _) = passing.swap_remove(0);
            return Some(ExampleProgram {
                steps,
                parameter,
                alternatives,
                evaluated,
            });
        }
    }
    trace.emit(
        "impasse",
        meta_seed().note(
            "no_program",
            &[
                ("length", &BOUNDS.program_length.to_string()),
                ("evaluated", &evaluated.to_string()),
            ],
        ),
    );
    None
}

/// What the meaning synthesis ranks programs against.
pub struct MeaningGoal<'a> {
    /// Each grounded word's tied operations.
    pub groups: &'a [Vec<String>],
    /// Evidence per operation (indexed by [`OpId`]).
    pub evidence: &'a [f64],
    /// A number the request states, bound to a parametric step.
    pub parameter: Option<f64>,
    /// Every grounded word's top hypotheses.
    pub words: &'a [Vec<Hypothesis>],
    /// Argument types the request's data words name.
    pub input_types: &'a [String],
    /// The request's clauses.
    pub clauses: &'a [Clause],
    /// Words tied across many operations.
    pub weak_words: &'a [Vec<Hypothesis>],
    /// The views tied plural nouns name.
    pub weak_views: &'a [Vec<Hypothesis>],
    /// True when the request quantifies universally ("every file").
    pub universal: bool,
    /// The bound's unit word, naming a filter's measure.
    pub measure_words: Option<&'a [Vec<Hypothesis>]>,
}

struct InternedClause {
    head: Vec<OpId>,
    others: Vec<OpId>,
    object_type: Option<String>,
}

type Interned = Vec<Vec<(OpId, f64)>>;

fn intern_all(catalog: &Catalog, ids: &[String]) -> Vec<OpId> {
    ids.iter()
        .map(|id| catalog.op(id).unwrap_or(OpId::MAX))
        .collect()
}

fn intern_words(catalog: &Catalog, words: &[Vec<Hypothesis>]) -> Interned {
    words
        .iter()
        .map(|hypotheses| {
            hypotheses
                .iter()
                .map(|hypothesis| {
                    (
                        catalog.op(&hypothesis.operation).unwrap_or(OpId::MAX),
                        hypothesis.score,
                    )
                })
                .collect()
        })
        .collect()
}

/// Coverage of the request: each grounded word credits once, with the best
/// of its hypotheses the program uses (a filter's measure excluded).
///
/// Mirrors `metaCoverageScore` in js/worker/formal_ai_worker_meta_synthesis.js.
fn coverage_score(main: &[OpId], words: &Interned) -> f64 {
    let mut score = 0.0;
    for hypotheses in words {
        let mut best = 0.0;
        for (op, value) in hypotheses {
            if main.contains(op) && *value > best {
                best = *value;
            }
        }
        score += best;
    }
    score
}

/// The composition departures of a program.
struct Order {
    violations: usize,
    mismatch: usize,
    holds_list: bool,
}

/// How far a program departs from the request's composition: coordinated
/// clauses apply in order; within a clause the head acts last, after its
/// object's modifiers; the head consumes the type its object noun names.
///
/// Mirrors `metaClauseOrder` in js/worker/formal_ai_worker_meta_synthesis.js.
fn clause_order(
    catalog: &Catalog,
    steps: &[Step],
    types: &[TypeId],
    clauses: &[InternedClause],
) -> Order {
    let mut ops: Vec<Vec<OpId>> = Vec::with_capacity(steps.len());
    let mut inputs: Vec<&str> = Vec::with_capacity(steps.len() + 1);
    for (position, step) in steps.iter().enumerate() {
        let mut buffer = Vec::with_capacity(3);
        catalog.step_ops(*step, &mut buffer);
        buffer.retain(|op| !catalog.is_view(*op));
        ops.push(buffer);
        let before = catalog.type_name(types[position]);
        inputs.push(if step.rewrite {
            "text"
        } else if step.mapped {
            before.strip_prefix("list_").unwrap_or(before)
        } else {
            before
        });
    }
    let final_type = catalog.type_name(types.last().copied().unwrap_or_default());
    let holds_list = inputs.iter().any(|kind| is_list_type(kind)) || is_list_type(final_type);
    let mut violations = 0;
    let mut mismatch = 0;
    let mut previous_head: Option<usize> = None;
    for clause in clauses {
        let Some(head) = ops
            .iter()
            .position(|step_ops| step_ops.iter().any(|op| clause.head.contains(op)))
        else {
            continue;
        };
        if previous_head.is_some_and(|previous| head < previous) {
            violations += 1;
        }
        previous_head = Some(head);
        for step_ops in ops.iter().skip(head + 1) {
            if step_ops.iter().any(|op| clause.others.contains(op)) {
                violations += 1;
            }
        }
        if let Some(object_type) = &clause.object_type {
            let input = inputs[head];
            if input != object_type && !(is_list_type(object_type) && input == "list_any") {
                mismatch += 1;
            }
        }
    }
    Order {
        violations,
        mismatch,
        holds_list,
    }
}

/// Without examples, the goal is a program that uses one operation from
/// every grounded word's hypothesis group, preferring a type-preserving one.
///
/// Mirrors `metaSynthesizeFromMeaning` in js/worker/formal_ai_worker_meta_synthesis.js.
pub fn synthesize_from_meaning(
    catalog: &Catalog,
    goal: &MeaningGoal<'_>,
    trace: &mut Trace,
) -> Option<MeaningProgram> {
    let shown_groups = goal
        .groups
        .iter()
        .map(|group| ["{", &group.join("|"), "}"].concat())
        .collect::<Vec<_>>()
        .join(" ");
    trace.emit("goal", fill(MEANING_GOAL, &[&shown_groups]));
    let seed = super::seed::meta_seed();
    let groups: Vec<Vec<OpId>> = goal
        .groups
        .iter()
        .map(|group| intern_all(catalog, group))
        .collect();
    let clauses: Vec<InternedClause> = goal
        .clauses
        .iter()
        .map(|clause| InternedClause {
            head: intern_all(catalog, &clause.head),
            others: intern_all(catalog, &clause.others),
            object_type: clause.object_type.clone(),
        })
        .collect();
    let words = intern_words(catalog, goal.words);
    let weak_words = intern_words(catalog, goal.weak_words);
    let weak_views = intern_words(catalog, goal.weak_views);
    let measure_words = goal.measure_words.map(|words| intern_words(catalog, words));
    let all_words: Interned = words.iter().chain(weak_words.iter()).cloned().collect();
    let mut strongest: Vec<(OpId, f64)> = Vec::new();
    for hypotheses in &all_words {
        for (op, score) in hypotheses {
            if let Some(entry) = strongest.iter_mut().find(|(id, _)| id == op) {
                entry.1 = entry.1.max(*score);
            } else {
                strongest.push((*op, score.max(0.0)));
            }
        }
    }
    let strongest_of = |op: OpId| {
        strongest
            .iter()
            .find(|(id, _)| *id == op)
            .map_or(0.0, |(_, score)| *score)
    };
    // The last clause's head is the outermost operation, so its result type
    // is the program's.
    let last_head_ids: &[String] = goal
        .clauses
        .last()
        .map(|clause| clause.head.as_slice())
        .or_else(|| goal.groups.first().map(Vec::as_slice))
        .unwrap_or_default();
    let last_head = intern_all(catalog, last_head_ids);
    let head_types: Vec<String> = last_head_ids
        .iter()
        .filter_map(|id| seed.primitive(id))
        .map(|primitive| primitive.to.clone())
        .filter(|to| !to.is_empty())
        .collect();
    let allowed_parametric = usize::from(goal.parameter.is_some());
    let evidence_of = |op: OpId| goal.evidence.get(op as usize).copied().unwrap_or(0.0);
    let mut best: Option<(MeaningProgram, [f64; 14])> = None;
    let mut used = Vec::new();
    let mut main = Vec::new();
    let mut buffer = Vec::new();
    for from_name in ["text", "list_number", "list_text", "number", "path"] {
        let Some(from) = catalog.type_id(from_name) else {
            continue;
        };
        for length in 1..BOUNDS.program_length {
            let mut ceiling = ENUMERATION_CEILING;
            catalog.for_each_program(from, length, &mut ceiling, &mut |steps, types| {
                let parametric = steps
                    .iter()
                    .filter(|step| catalog.is_parametric(**step))
                    .count();
                if parametric > allowed_parametric {
                    return;
                }
                // A measure's parts serve the words they ground ("80 characters").
                distinct_ops(catalog, steps, false, &mut used);
                for step in steps.iter().filter(|step| step.filter.is_some()) {
                    let prim = catalog.prim(*step);
                    if prim.id == "value" {
                        continue;
                    }
                    for part in &prim.parts {
                        if !used.contains(part) {
                            used.push(*part);
                        }
                    }
                }
                if !last_head.iter().any(|op| used.contains(op)) {
                    return;
                }
                let uncovered = groups
                    .iter()
                    .filter(|group| !group.iter().any(|op| used.contains(op)))
                    .count();
                // A filter's measure is internal to the filter: it is not
                // charged as an ungrounded operation.
                let mut ungrounded = 0_usize;
                for step in steps {
                    buffer.clear();
                    catalog.main_ops(*step, &mut buffer);
                    ungrounded += buffer.iter().filter(|op| evidence_of(**op) == 0.0).count();
                }
                distinct_ops(catalog, steps, true, &mut main);
                let mut parts: Vec<OpId> = Vec::new();
                for step in steps.iter().filter(|step| step.filter.is_some()) {
                    let prim = catalog.prim(*step);
                    let own = if prim.parts.is_empty() {
                        vec![prim.op]
                    } else {
                        prim.parts.clone()
                    };
                    for part in own {
                        if !parts.contains(&part) {
                            parts.push(part);
                        }
                    }
                }
                // A word the main program already explains does not name the
                // measure; a word whose meaning it owes to another word may.
                let mut measure_evidence = 0.0;
                for hypotheses in measure_words.as_ref().unwrap_or(&all_words) {
                    if measure_words.is_none()
                        && hypotheses.iter().any(|(op, score)| {
                            main.contains(op) && *score >= strongest_of(*op) * 0.99
                        })
                    {
                        continue;
                    }
                    measure_evidence += hypotheses
                        .iter()
                        .filter(|(op, _)| parts.contains(op))
                        .map(|(_, score)| *score)
                        .fold(0.0, f64::max);
                }
                let order = clause_order(catalog, steps, types, &clauses);
                let program_type = catalog.type_name(types.last().copied().unwrap_or_default());
                let stated_input = usize::from(
                    !goal.input_types.is_empty()
                        && !goal.input_types.iter().any(|kind| kind == from_name),
                );
                // A rewrite writes the head's result back to the file: it fits there.
                let rewrites_head = steps
                    .iter()
                    .any(|step| step.rewrite && last_head.contains(&catalog.prim(*step).op));
                let head_fits = rewrites_head
                    || head_types.iter().any(|kind| {
                        kind == program_type || (kind == "list_any" && is_list_type(program_type))
                    });
                // "every file" asks for iteration: a program never holding a
                // list does not quantify over anything.
                let unquantified = goal.universal && !order.holds_list;
                let rank = [
                    ungrounded as f64,
                    uncovered as f64,
                    -coverage_score(&main, &words),
                    -measure_evidence,
                    order.violations as f64,
                    order.mismatch as f64,
                    if unquantified { 1.0 } else { 0.0 },
                    stated_input as f64,
                    if head_fits { 0.0 } else { 1.0 },
                    -coverage_score(&main, &weak_views),
                    if program_type == from_name { 0.0 } else { 1.0 },
                    steps.len() as f64,
                    -coverage_score(&main, &weak_words),
                    parts.len() as f64,
                ];
                let better = best.as_ref().is_none_or(|(_, current)| {
                    rank.iter()
                        .zip(current.iter())
                        .find(|(a, b)| a != b)
                        .is_some_and(|(a, b)| a < b)
                });
                if better {
                    best = Some((
                        MeaningProgram {
                            steps: steps.to_vec(),
                            from_type: from_name.to_owned(),
                            parameter: if parametric > 0 {
                                goal.parameter.map_or(Param::Null, Param::Number)
                            } else {
                                Param::Null
                            },
                            uncovered,
                        },
                        rank,
                    ));
                }
            });
        }
    }
    match best {
        None => {
            trace.emit("impasse", meta_seed().note("no_head_program", &[]));
            None
        }
        Some((program, _)) => {
            if program.uncovered > 0 {
                trace.emit(
                    "evidence",
                    meta_seed().note("uncovered", &[("count", &program.uncovered.to_string())]),
                );
            }
            Some(program)
        }
    }
}
