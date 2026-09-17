//! Budget-driven random and evolutionary search for the synthesis stage.
//!
//! `GOALS.md` (Universal Solver Goals): "When no reusable part exists, combine
//! reasoning, random search, and evolutionary search according to the available
//! compute budget instead of giving up." `docs/USER-JOURNEYS.md` F4 lists this
//! as a future journey. Deterministic reuse and rule reasoning run first; only
//! when they produce no candidate does this stage activate. It recognizes an
//! arithmetic-reachability problem ("combine the numbers … to reach TARGET"),
//! samples candidate compositions of the known parts, and evolves the
//! best-scoring ones against generated equality tests as the fitness function.
//!
//! Determinism (`VISION.md` contract): the pseudo-random stream is seeded from
//! the impulse content hash, so the same prompt produces the same search path
//! and the same answer across runs. The compute budget counts candidate
//! evaluations; on exhaustion the stage records its `search:` evidence and
//! declines, leaving the honest unknown-reasoning reply to take over.

use crate::engine::{SymbolicAnswer, stable_id};
use crate::event_log::EventLog;
use crate::execution_evidence::{Evidence, EvidenceSource, ObservationKind};
use crate::language::detect as detect_language;
use crate::links_format::format_lino_record;
use crate::seed;
use crate::selection_heuristics::{
    Experiment, ExperimentChooser, HypothesisSpace, RefutationSearch, SearchHypothesis,
    SearchVerdict,
};
use crate::solver::SolverConfig;
use crate::solver_handlers::finalize_simple;

mod portfolio;
mod problem;

use problem::parse_search_problem;

/// Ask the sources registry what the prompt's unresolved surfaces mean.
///
/// Issue #1138 B1: an offline run records its policy boundary, while an online
/// run preserves a source id and status for every miss and attributable hashes
/// for every hit. This search concern lives beside the solver's other bounded
/// search algorithms so the universal-loop orchestrator stays reviewable.
pub fn record_external_search(
    config: &SolverConfig,
    log: &mut EventLog,
    prompt: &str,
    language: crate::language::Language,
) -> Vec<crate::concept_lookup::ConceptSense> {
    if config.offline {
        log.append("search:external", "skipped:offline".to_owned());
        return Vec::new();
    }
    log.append("search:external", prompt.to_owned());
    let normalized = crate::engine::normalize_prompt(prompt);
    let code = language.slug();
    let surfaces = crate::concept_lookup::unknown_surfaces(&normalized, code);
    let bounds = crate::source_walk::LookupBounds::default();
    let cache_root = crate::coding::synthesis_runtime::source_cache_root();
    let client = crate::source_fetch::CachedSourceClient::new(
        &cache_root,
        crate::source_fetch::CurlSourceTransport,
    )
    .with_online(crate::coding::synthesis_runtime::live_fetch_enabled());
    let preferences = crate::solver_handler_how_synthesis::service_preferences_from_env(log);
    let mut availability =
        crate::service_accessibility::ServiceAccessibilityCache::load(&cache_root);
    let now = crate::service_accessibility::unix_now();
    let mut senses = Vec::new();
    for surface in surfaces.iter().take(bounds.max_services) {
        let outcome = crate::concept_lookup::lookup_surface(
            surface,
            code,
            &client,
            &preferences,
            &bounds,
            &mut availability,
            now,
        );
        if outcome.items.is_empty() {
            log.append(
                "concept_lookup:miss",
                crate::trace_record::payload(&[
                    ("surface", surface.clone()),
                    (
                        "consulted",
                        outcome
                            .outcomes
                            .iter()
                            .map(|row| format!("{} {}", row.source_id, row.status))
                            .collect::<Vec<_>>()
                            .join(" "),
                    ),
                ]),
            );
        }
        for sense in &outcome.items {
            log.append(
                "concept_lookup:hit",
                crate::trace_record::payload(&[
                    ("surface", surface.clone()),
                    ("source", sense.source_id.clone()),
                    ("sha256", sense.sha256.clone()),
                    ("license", sense.license_name.clone()),
                ]),
            );
        }
        senses.extend(outcome.items);
    }
    senses
}

/// One arithmetic operator the search can place between operands, identified by
/// its language-neutral notation `symbol`.
///
/// The operator set is derived from the seed lexicon
/// ([`seed::Lexicon::arithmetic_operators`]) rather than a hardcoded list, so
/// division and modulo are supported the moment the seed lists them and no
/// per-language operator table lives in Rust (issue #386). The arithmetic each
/// symbol denotes is intrinsic to the notation, so `apply` matches on the symbol
/// alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Op {
    symbol: char,
}

impl Op {
    const fn new(symbol: char) -> Self {
        Self { symbol }
    }

    const fn symbol(self) -> char {
        self.symbol
    }

    /// Apply the operator, or `None` when it is undefined for the operands (an
    /// integer division or modulo by zero). A `None` result scores as maximally
    /// unfit, so the search never proposes an undefined composition.
    fn apply(self, lhs: i64, rhs: i64) -> Option<i64> {
        match self.symbol {
            '+' => Some(lhs.saturating_add(rhs)),
            '-' => Some(lhs.saturating_sub(rhs)),
            '*' => Some(lhs.saturating_mul(rhs)),
            '/' => (rhs != 0).then(|| lhs / rhs),
            '%' => (rhs != 0).then(|| lhs % rhs),
            // An operator symbol the arithmetic evaluator does not model.
            _ => None,
        }
    }
}

/// A recognized arithmetic-reachability problem: reach `target` by combining the
/// `numbers` (each used once, in some order) with the allowed `ops`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct SearchProblem {
    numbers: Vec<i64>,
    target: i64,
    ops: Vec<Op>,
}

/// A candidate composition: an ordering of the operand indices plus the operator
/// placed before each operand after the first. Evaluated left to right.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Candidate {
    order: Vec<usize>,
    ops: Vec<Op>,
}

impl Candidate {
    /// Evaluate left to right, or `None` when any step is undefined (division or
    /// modulo by zero).
    fn evaluate(&self, numbers: &[i64]) -> Option<i64> {
        let mut acc = numbers[self.order[0]];
        for (index, op) in self.ops.iter().enumerate() {
            acc = op.apply(acc, numbers[self.order[index + 1]])?;
        }
        Some(acc)
    }

    fn render(&self, numbers: &[i64]) -> String {
        let mut out = numbers[self.order[0]].to_string();
        for (index, op) in self.ops.iter().enumerate() {
            out.push(' ');
            out.push(op.symbol());
            out.push(' ');
            out.push_str(&numbers[self.order[index + 1]].to_string());
        }
        out
    }
}

/// Deterministic `splitmix64` stream seeded from the impulse content hash, so
/// "random guessing" stays reproducible per the `SolverConfig` contract.
struct Prng {
    state: u64,
}

impl Prng {
    const fn seeded(seed: u64) -> Self {
        // Avoid the degenerate all-zero seed which would keep the mixer stuck.
        Self {
            state: seed ^ 0x9e37_79b9_7f4a_7c15,
        }
    }

    const fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn below(&mut self, bound: usize) -> usize {
        if bound == 0 {
            0
        } else {
            // The remainder is strictly less than `bound` (a `usize`), so the
            // conversion back is always exact; `try_from` keeps clippy happy on
            // 32-bit targets without an escape-hatch cast.
            usize::try_from(self.next_u64() % bound as u64).unwrap_or(0)
        }
    }
}

/// Seed the pseudo-random stream from the FNV-1a hash of the prompt so the same
/// impulse yields the same search path across runs.
fn seed_from_prompt(prompt: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in prompt.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// Entry point wired into step 7 of the universal loop. Returns `Some` only when
/// the prompt is a recognized search problem and a solution is found within the
/// configured compute budget. Every attempt records `search:` evidence on the
/// shared log; declining leaves that evidence attached for the unknown-reasoning
/// fallback.
pub fn try_budget_search(
    prompt: &str,
    log: &mut EventLog,
    config: SolverConfig,
) -> Option<SymbolicAnswer> {
    let problem = parse_search_problem(prompt)?;

    // Trace payloads stay structured (one atomic field per event, no prose) so
    // they read as machine data, matching the rest of the engine's event log.
    log.append("search:problem:target", problem.target.to_string());
    log.append("search:problem:numbers", join_numbers(&problem.numbers));
    log.append("search:problem:ops", join_ops(&problem.ops));
    log.append("search:budget", config.compute_budget.to_string());
    record_generated_tests(log, &problem);

    let (outcome, comparison_artifact) = if config.draft_count <= 1 {
        let outcome = match run_refutation_search(log, &problem, config.compute_budget) {
            RefutationOutcome::Solved(solution) => Some(solution),
            RefutationOutcome::Exhausted => None,
            RefutationOutcome::NotApplicable => run_search(
                seed_from_prompt(prompt),
                log,
                &problem,
                config.compute_budget,
            ),
        };
        (outcome, None)
    } else {
        portfolio::run_draft_portfolio(prompt, log, &problem, config)
    };
    match outcome {
        Some(solution) => {
            log.append(
                "search:solution",
                format!("{} = {}", solution.expression, problem.target),
            );
            record_skill_proposal(prompt, log, &problem, &solution);
            Some(build_answer(
                prompt,
                log,
                &problem,
                &solution,
                config.compute_budget,
                comparison_artifact.as_deref(),
            ))
        }
        None => {
            // Budget exhausted (or zero): leave the `search:` evidence on the log
            // and decline so the honest unknown-reasoning reply takes over.
            None
        }
    }
}

/// A finite candidate family small enough to refute completely is searched by
/// elimination before the stochastic fallback. Larger spaces stay on the
/// sampler: pretending an arbitrary prefix is the whole hypothesis space would
/// make `AllRefuted` a false conclusion.
const MAX_REFUTATION_HYPOTHESES: usize = 128;

enum RefutationOutcome {
    Solved(SearchSolution),
    Exhausted,
    NotApplicable,
}

fn run_refutation_search(
    log: &mut EventLog,
    problem: &SearchProblem,
    budget: u32,
) -> RefutationOutcome {
    let Some(space_size) = candidate_space_size(problem) else {
        return RefutationOutcome::NotApplicable;
    };
    if space_size == 0
        || space_size > MAX_REFUTATION_HYPOTHESES
        || usize::try_from(budget)
            .ok()
            .is_none_or(|budget| budget < space_size)
    {
        log.append(
            "search:heuristic:experiment",
            "refutation_search:not_applicable".to_owned(),
        );
        return RefutationOutcome::NotApplicable;
    }

    let candidates = enumerate_candidates(problem);
    if candidates.len() != space_size {
        return RefutationOutcome::NotApplicable;
    }
    let hypothesis_ids = candidates
        .iter()
        .map(|candidate| stable_id("search_hypothesis", &candidate.render(&problem.numbers)))
        .collect::<Vec<_>>();
    let experiments = hypothesis_ids
        .iter()
        .enumerate()
        .map(|(index, hypothesis_id)| Experiment {
            experiment_id: stable_id("search_experiment", hypothesis_id),
            probe: candidates[index].render(&problem.numbers),
            predicts_yes: vec![hypothesis_id.clone()],
            predicts_no: hypothesis_ids
                .iter()
                .filter(|candidate_id| *candidate_id != hypothesis_id)
                .cloned()
                .collect(),
        })
        .collect::<Vec<_>>();
    let mut space = HypothesisSpace {
        space_id: stable_id("search_space", &join_numbers(&problem.numbers)),
        hypotheses: hypothesis_ids
            .iter()
            .zip(candidates.iter())
            .map(|(hypothesis_id, candidate)| SearchHypothesis {
                hypothesis_id: hypothesis_id.clone(),
                rule: candidate.render(&problem.numbers),
                alive: true,
                refuted_by: None,
            })
            .collect(),
        experiments,
        observations: Vec::new(),
    };
    log.append(
        "search:heuristic:experiment",
        "refutation_search".to_owned(),
    );
    log.append("search:hypotheses", space_size.to_string());

    let mut evaluations = 0_u32;
    while evaluations < budget {
        let experiment = RefutationSearch
            .next_experiment(&space)
            .or_else(|| {
                let survivor = space.alive().first()?.hypothesis_id.as_str();
                space
                    .experiments
                    .iter()
                    .find(|experiment| experiment.predicts_yes.iter().any(|id| id == survivor))
            })
            .cloned();
        let Some(experiment) = experiment else {
            break;
        };
        let Some(index) = space
            .hypotheses
            .iter()
            .position(|hypothesis| experiment.predicts_yes.contains(&hypothesis.hypothesis_id))
        else {
            return RefutationOutcome::NotApplicable;
        };
        let candidate = &candidates[index];
        let value = candidate.evaluate(&problem.numbers);
        let succeeded = value == Some(problem.target);
        evaluations += 1;
        let rendered_value =
            value.map_or_else(|| "undefined".to_owned(), |value| value.to_string());
        let mut evidence = Evidence::observed(
            experiment.probe.clone(),
            vec![experiment.probe.clone()],
            Some(i64::from(!succeeded)),
            rendered_value.as_bytes(),
            ObservationKind::SymbolicCheck,
            EvidenceSource::Engine,
        );
        evidence.for_need.clone_from(&space.space_id);
        "refutation_search".clone_into(&mut evidence.produced_by);
        log.append("search:experiment", experiment.experiment_id.clone());
        log.append("evidence", evidence.to_links_notation());
        space.observe(&experiment.experiment_id, evidence);
        if succeeded {
            return RefutationOutcome::Solved(finish_solution(
                log,
                problem,
                candidate,
                evaluations,
                "refutation",
            ));
        }
    }

    match space.verdict() {
        SearchVerdict::AllRefuted => {
            log.append("search:refutation:verdict", "all_refuted".to_owned());
            log.append("search:exhausted:evaluations", evaluations.to_string());
            RefutationOutcome::Exhausted
        }
        SearchVerdict::Concluded { hypothesis_id } => {
            log.append(
                "search:refutation:verdict",
                format!("unverified_survivor:{hypothesis_id}"),
            );
            RefutationOutcome::NotApplicable
        }
        SearchVerdict::NotConfirmedNotRefuted { survivors, blocker } => {
            log.append(
                "search:refutation:verdict",
                format!("{blocker}:{}", survivors.join(",")),
            );
            RefutationOutcome::NotApplicable
        }
    }
}

fn candidate_space_size(problem: &SearchProblem) -> Option<usize> {
    let permutations = (1..=problem.numbers.len()).try_fold(1_usize, usize::checked_mul)?;
    let operator_slots = problem.numbers.len().saturating_sub(1);
    let operator_choices = (0..operator_slots)
        .try_fold(1_usize, |product, _| product.checked_mul(problem.ops.len()))?;
    permutations.checked_mul(operator_choices)
}

fn enumerate_candidates(problem: &SearchProblem) -> Vec<Candidate> {
    let mut orders = Vec::new();
    let mut current = Vec::new();
    let mut used = vec![false; problem.numbers.len()];
    enumerate_orders(problem.numbers.len(), &mut current, &mut used, &mut orders);
    let mut operator_rows = Vec::new();
    enumerate_operator_rows(
        &problem.ops,
        problem.numbers.len().saturating_sub(1),
        &mut Vec::new(),
        &mut operator_rows,
    );
    orders
        .into_iter()
        .flat_map(|order| {
            operator_rows.iter().cloned().map(move |ops| Candidate {
                order: order.clone(),
                ops,
            })
        })
        .collect()
}

fn enumerate_orders(
    len: usize,
    current: &mut Vec<usize>,
    used: &mut [bool],
    output: &mut Vec<Vec<usize>>,
) {
    if current.len() == len {
        output.push(current.clone());
        return;
    }
    for index in 0..len {
        if used[index] {
            continue;
        }
        used[index] = true;
        current.push(index);
        enumerate_orders(len, current, used, output);
        current.pop();
        used[index] = false;
    }
}

fn enumerate_operator_rows(
    operators: &[Op],
    slots: usize,
    current: &mut Vec<Op>,
    output: &mut Vec<Vec<Op>>,
) {
    if current.len() == slots {
        output.push(current.clone());
        return;
    }
    for operator in operators {
        current.push(*operator);
        enumerate_operator_rows(operators, slots, current, output);
        current.pop();
    }
}

/// The best composition found by the search, with the evaluation count that
/// produced it.
#[derive(Debug, Clone)]
struct SearchSolution {
    expression: String,
    evaluations: u32,
    candidate: Candidate,
}

fn run_search(
    seed: u64,
    log: &mut EventLog,
    problem: &SearchProblem,
    budget: u32,
) -> Option<SearchSolution> {
    if budget == 0 {
        log.append("search:exhausted:evaluations", 0.to_string());
        log.append("search:exhausted:budget", 0.to_string());
        return None;
    }

    let mut prng = Prng::seeded(seed);
    let mut evaluations: u32 = 0;
    let mut best: Option<(Candidate, i64)> = None;

    // Random search: sample compositions of the known parts, seeded from the
    // impulse hash. Half the budget seeds the evolutionary phase below.
    let random_budget = budget.div_ceil(2);
    let mut population: Vec<(Candidate, i64)> = Vec::new();
    while evaluations < random_budget {
        let candidate = random_candidate(&mut prng, problem);
        let diff = score(&candidate, problem);
        evaluations += 1;
        if diff == 0 {
            return Some(finish_solution(
                log,
                problem,
                &candidate,
                evaluations,
                "random",
            ));
        }
        remember_best(&mut best, &candidate, diff);
        insert_population(&mut population, candidate, diff, POPULATION);
    }
    log.append("search:random:sampled", evaluations.to_string());
    log.append(
        "search:random:best_diff",
        best.as_ref()
            .map_or(i64::MAX, |(_, diff)| *diff)
            .to_string(),
    );

    // Evolutionary search: mutate and cross over the best-scoring candidates,
    // scored against the generated equality tests as the fitness function.
    let mut generation: u32 = 0;
    while evaluations < budget {
        generation += 1;
        let child = breed(&mut prng, &population, problem);
        let diff = score(&child, problem);
        evaluations += 1;
        if diff == 0 {
            log.append("search:evolutionary:generation", generation.to_string());
            log.append("search:evolutionary:best_diff", 0.to_string());
            return Some(finish_solution(
                log,
                problem,
                &child,
                evaluations,
                "evolutionary",
            ));
        }
        remember_best(&mut best, &child, diff);
        insert_population(&mut population, child, diff, POPULATION);
        if generation.is_multiple_of(GENERATION_LOG_STRIDE) {
            log.append("search:evolutionary:generation", generation.to_string());
            log.append(
                "search:evolutionary:best_diff",
                best.as_ref()
                    .map_or(i64::MAX, |(_, diff)| *diff)
                    .to_string(),
            );
        }
    }

    log.append("search:exhausted:evaluations", evaluations.to_string());
    log.append(
        "search:exhausted:best_diff",
        best.as_ref()
            .map_or(i64::MAX, |(_, diff)| *diff)
            .to_string(),
    );
    None
}

/// Top candidates kept between evolutionary generations.
const POPULATION: usize = 8;
/// Emit a `search:evolutionary` progress event every N generations so the trace
/// stays inspectable without flooding the log.
const GENERATION_LOG_STRIDE: u32 = 16;

fn finish_solution(
    log: &mut EventLog,
    problem: &SearchProblem,
    candidate: &Candidate,
    evaluations: u32,
    phase: &'static str,
) -> SearchSolution {
    log.append("search:candidate:phase", phase.to_owned());
    log.append("search:candidate:evaluations", evaluations.to_string());
    log.append(
        "search:candidate:expression",
        candidate.render(&problem.numbers),
    );
    SearchSolution {
        expression: candidate.render(&problem.numbers),
        evaluations,
        candidate: candidate.clone(),
    }
}

/// Fitness distance: 0 means every generated test passes (the composition uses
/// each number once, only allowed operators, and evaluates to the target). A
/// composition that is undefined (division or modulo by zero) scores as
/// maximally unfit so the search never proposes it.
fn score(candidate: &Candidate, problem: &SearchProblem) -> i64 {
    candidate
        .evaluate(&problem.numbers)
        .map_or(i64::MAX, |value| (value - problem.target).abs())
}

fn remember_best(best: &mut Option<(Candidate, i64)>, candidate: &Candidate, diff: i64) {
    if best.as_ref().is_none_or(|(_, current)| diff < *current) {
        *best = Some((candidate.clone(), diff));
    }
}

fn insert_population(
    population: &mut Vec<(Candidate, i64)>,
    candidate: Candidate,
    diff: i64,
    capacity: usize,
) {
    if population
        .iter()
        .any(|(existing, _)| existing == &candidate)
    {
        return;
    }
    population.push((candidate, diff));
    population.sort_by_key(|entry| entry.1);
    population.truncate(capacity);
}

fn random_candidate(prng: &mut Prng, problem: &SearchProblem) -> Candidate {
    let order = random_permutation(prng, problem.numbers.len());
    let ops = (0..problem.numbers.len().saturating_sub(1))
        .map(|_| problem.ops[prng.below(problem.ops.len())])
        .collect();
    Candidate { order, ops }
}

fn random_permutation(prng: &mut Prng, len: usize) -> Vec<usize> {
    let mut order: Vec<usize> = (0..len).collect();
    // Fisher-Yates using the deterministic stream.
    for i in (1..len).rev() {
        let j = prng.below(i + 1);
        order.swap(i, j);
    }
    order
}

/// Produce one child by crossover (operators from a second parent) followed by a
/// single mutation (swap two operands or flip one operator).
fn breed(prng: &mut Prng, population: &[(Candidate, i64)], problem: &SearchProblem) -> Candidate {
    if population.is_empty() {
        return random_candidate(prng, problem);
    }
    let parent_a = &population[prng.below(population.len())].0;
    let parent_b = &population[prng.below(population.len())].0;

    let mut order = parent_a.order.clone();
    // Crossover: inherit each operator from whichever parent the stream picks.
    let mut ops: Vec<Op> = parent_a
        .ops
        .iter()
        .zip(parent_b.ops.iter())
        .map(|(a, b)| if prng.next_u64() & 1 == 0 { *a } else { *b })
        .collect();

    // Mutation.
    if !ops.is_empty() && prng.next_u64() & 1 == 0 {
        let slot = prng.below(ops.len());
        ops[slot] = problem.ops[prng.below(problem.ops.len())];
    } else if order.len() >= 2 {
        let i = prng.below(order.len());
        let j = prng.below(order.len());
        order.swap(i, j);
    }

    Candidate { order, ops }
}

fn record_generated_tests(log: &mut EventLog, problem: &SearchProblem) {
    // Step 6 of the loop generates a test per requirement before an answer is
    // committed; these are the fitness constraints the search must satisfy.
    log.append(
        "search:test:each_number_once",
        join_numbers(&problem.numbers),
    );
    log.append("search:test:only_operators", join_ops(&problem.ops));
    log.append("search:test:evaluates_to", problem.target.to_string());
}

#[allow(clippy::literal_string_with_formatting_args)]
fn build_answer(
    prompt: &str,
    log: &mut EventLog,
    problem: &SearchProblem,
    solution: &SearchSolution,
    budget: u32,
    comparison_artifact: Option<&str>,
) -> SymbolicAnswer {
    // The reply prose lives in the seed knowledge base (R379: "data is the
    // interface"), localized to the prompt's language with an English fallback,
    // and its `{...}` placeholders are filled with this run's values.
    let language = detect_language(prompt);
    let template =
        seed::localized_response("budget_search_solution", language.slug()).unwrap_or_default();
    // The `{...}` tokens are seed template placeholders, not Rust format args;
    // clippy's nursery lint mistakes `{budget}` for a captured binding because a
    // local `budget` is in scope, so it is silenced for this literal substitution.
    let substitutions = [
        ("{expression}", solution.expression.clone()),
        ("{target}", problem.target.to_string()),
        ("{budget}", budget.to_string()),
        ("{evaluations}", solution.evaluations.to_string()),
        ("{trace_id}", stable_id("search", prompt)),
    ];
    let mut body = substitutions
        .iter()
        .fold(template, |acc, (placeholder, value)| {
            acc.replace(placeholder, value)
        });
    if let Some(artifact) = comparison_artifact {
        body.push_str("\n\n```links\n");
        body.push_str(artifact);
        body.push_str("```\n");
    }
    finalize_simple(
        prompt,
        log,
        "budget_search_solution",
        "response:search:solution",
        &body,
        0.9,
    )
}

/// Emit a proposal-only auto-learning event when the search succeeds.
///
/// A satisfying composition is a demonstrated capability the next request could
/// reuse, so — like the skill-accumulation ledger ([`crate::skill_ledger`]) and
/// the meta self-improvement loop — the stage records it as a *proposed*
/// candidate skill (R21/R340). It is trace-only and human-gated: the promotion
/// gate (a regression test **and** a benchmark delta) is unmet at trace time, so
/// `status=proposed` and `promotable=false`. Nothing is auto-promoted and neither
/// routing nor the answer changes (C3/R13); the compact `search:skill:promotable`
/// count is always `0`, the auditable proof of that.
fn record_skill_proposal(
    prompt: &str,
    log: &mut EventLog,
    problem: &SearchProblem,
    solution: &SearchSolution,
) {
    let skill_id = stable_id(
        "search_skill",
        &format!(
            "reachability:{}:{}",
            problem.numbers.len(),
            solution.expression
        ),
    );
    let record = format_lino_record(
        &skill_id,
        &[
            ("record_type", "candidate_skill".to_owned()),
            ("skill_id", skill_id.clone()),
            ("method", "budget_search".to_owned()),
            (
                "route",
                format!("reachability:{}-operand", problem.numbers.len()),
            ),
            ("source_span", prompt.to_owned()),
            ("status", "proposed".to_owned()),
            ("has_tests", "false".to_owned()),
            ("has_benchmark_delta", "false".to_owned()),
            ("promotable", "false".to_owned()),
        ],
    );
    log.append("search:skill", record);
    // Always 0: no skill is ever auto-promoted without review.
    log.append("search:skill:promotable", "0".to_owned());
}

fn join_numbers(numbers: &[i64]) -> String {
    numbers
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn join_ops(ops: &[Op]) -> String {
    ops.iter()
        .map(|op| op.symbol().to_string())
        .collect::<Vec<_>>()
        .join(" ")
}
