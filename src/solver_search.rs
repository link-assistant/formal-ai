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

use crate::engine::{stable_id, SymbolicAnswer};
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::links_format::format_lino_record;
use crate::seed;
use crate::solver::SolverConfig;
use crate::solver_handlers::finalize_simple;

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

    let outcome = run_search(prompt, log, &problem, config.compute_budget);
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
            ))
        }
        None => {
            // Budget exhausted (or zero): leave the `search:` evidence on the log
            // and decline so the honest unknown-reasoning reply takes over.
            None
        }
    }
}

/// The best composition found by the search, with the evaluation count that
/// produced it.
struct SearchSolution {
    expression: String,
    evaluations: u32,
}

fn run_search(
    prompt: &str,
    log: &mut EventLog,
    problem: &SearchProblem,
    budget: u32,
) -> Option<SearchSolution> {
    if budget == 0 {
        log.append("search:exhausted:evaluations", 0.to_string());
        log.append("search:exhausted:budget", 0.to_string());
        return None;
    }

    let mut prng = Prng::seeded(seed_from_prompt(prompt));
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

fn build_answer(
    prompt: &str,
    log: &mut EventLog,
    problem: &SearchProblem,
    solution: &SearchSolution,
    budget: u32,
) -> SymbolicAnswer {
    // The reply prose lives in the seed knowledge base (R379: "data is the
    // interface"), localized to the prompt's language with an English fallback,
    // and its `{...}` placeholders are filled with this run's values.
    let language = detect_language(prompt);
    let template = seed::response_for("budget_search_solution", language.slug())
        .or_else(|| seed::response_for("budget_search_solution", "en"))
        .unwrap_or_default();
    let body = template
        .replace("{expression}", &solution.expression)
        .replace("{target}", &problem.target.to_string())
        .replace("{budget}", &budget.to_string())
        .replace("{evaluations}", &solution.evaluations.to_string())
        .replace("{trace_id}", &stable_id("search", prompt));
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

/// Recognize an arithmetic-reachability search problem across every supported
/// language. Returns `None` for any prompt that is not clearly of this shape so
/// the stage stays inert for the overwhelming majority of impulses.
///
/// Recognition is grounded entirely in the seed lexicon (issue #386): the
/// "combine numbers" framing, the search verb, and the target marker are read by
/// semantic role from `data/seed/meanings-search.lino`, and the operator
/// vocabulary comes from `data/seed/meanings-calculator.lino` — no per-language
/// phrase table lives here. Only the digits and the notation symbols they anchor
/// are language-neutral and matched directly.
fn parse_search_problem(prompt: &str) -> Option<SearchProblem> {
    // Unicode-aware lowercasing so Cyrillic framing keywords match regardless of
    // case (Devanagari and Han are caseless; ASCII is unaffected). The seed
    // surfaces are authored lowercase, so a raw-substring match lines up.
    let lower = prompt.to_lowercase();
    let lexicon = seed::lexicon();

    // Gate: require both a "combine numbers" framing and a search verb so plain
    // calculations ("3 + 5") never reach this path. Both are matched as raw
    // substrings so inflected forms (числа/чисел, संख्याओं, найдите) still hit.
    if !lexicon.mentions_role_raw(seed::ROLE_REACHABILITY_OPERAND_FRAMING, &lower)
        || !lexicon.mentions_role_raw(seed::ROLE_REACHABILITY_SEARCH_CUE, &lower)
    {
        return None;
    }

    // Locate the target value as the integer nearest a target marker. This
    // handles both operand-then-target order (en/ru/zh: "equals 26") and
    // target-then-marker order (hi: "26 के बराबर").
    let integers = extract_integers_with_positions(&lower);
    if integers.len() < 3 {
        // Need at least two operands plus a distinct target.
        return None;
    }
    let marker_positions = target_marker_positions(&lower);
    if marker_positions.is_empty() {
        return None;
    }
    let target_index = integers
        .iter()
        .enumerate()
        .min_by_key(|(_, (_, position))| distance_to_nearest(*position, &marker_positions))
        .map(|(index, _)| index)?;

    let target = integers[target_index].0;
    let numbers: Vec<i64> = integers
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != target_index)
        .map(|(_, (value, _))| *value)
        .collect();
    if numbers.len() < 2 || numbers.len() > MAX_OPERANDS {
        return None;
    }

    let ops = parse_ops(&lower);
    Some(SearchProblem {
        numbers,
        target,
        ops,
    })
}

/// Upper bound on operand count so the search space and per-call cost stay
/// bounded regardless of the prompt.
const MAX_OPERANDS: usize = 6;

/// Byte offsets at which any target-marker surface begins in `lower`.
///
/// The surfaces ("equals", "равно", "बराबर", "等于", …) come from the seed
/// meaning carrying [`ROLE_REACHABILITY_TARGET_MARKER`](seed::ROLE_REACHABILITY_TARGET_MARKER),
/// so anchoring the target value never names a keyword in Rust (issue #386).
fn target_marker_positions(lower: &str) -> Vec<usize> {
    let mut positions = Vec::new();
    for marker in seed::lexicon().words_for_role(seed::ROLE_REACHABILITY_TARGET_MARKER) {
        let mut from = 0;
        while let Some(offset) = lower[from..].find(&marker) {
            let absolute = from + offset;
            positions.push(absolute);
            from = absolute + marker.len();
        }
    }
    positions
}

fn distance_to_nearest(position: usize, marker_positions: &[usize]) -> usize {
    marker_positions
        .iter()
        .map(|&marker| position.abs_diff(marker))
        .min()
        .unwrap_or(usize::MAX)
}

/// Determine the allowed operator set from the prompt, grounded in the seed
/// operator vocabulary.
///
/// Each operator declared in the seed (addition, subtraction, multiplication,
/// division, modulo) is admitted when its notation symbol appears in an
/// arithmetic context or any of its spelled surfaces (in any language) is
/// mentioned. Declaration order is preserved so a seeded search over the set
/// stays deterministic. When the prompt names no operator, the full seed toolbox
/// is allowed.
fn parse_ops(lower: &str) -> Vec<Op> {
    let operators = seed::lexicon().arithmetic_operators();
    let mut ops: Vec<Op> = operators
        .iter()
        .filter(|operator| {
            symbol_present(lower, operator.symbol)
                || operator.spelled.iter().any(|word| lower.contains(word))
        })
        .map(|operator| Op::new(operator.symbol))
        .collect();
    if ops.is_empty() {
        // No operator named: allow the full seed toolbox, in declaration order.
        ops = operators
            .iter()
            .map(|operator| Op::new(operator.symbol))
            .collect();
    }
    ops
}

/// Is `symbol` present in `lower` as an arithmetic operator?
///
/// A notation symbol counts only when it sits in an arithmetic context — adjacent
/// to a digit or set off by whitespace (or a string boundary). This keeps a
/// hyphen inside a word ("state-of-the-art") or a slash inside a path from being
/// read as subtraction or division, while the space- or digit-flanked symbols in
/// a reachability prompt ("+ and *", "3-5") are recognised. The rule is uniform
/// across every operator symbol, so nothing about which glyph is ambiguous lives
/// in code.
fn symbol_present(lower: &str, symbol: char) -> bool {
    let chars: Vec<char> = lower.chars().collect();
    let arithmetic_context =
        |neighbor: Option<char>| neighbor.is_none_or(|c| c.is_ascii_digit() || c.is_whitespace());
    for (index, &current) in chars.iter().enumerate() {
        if current != symbol {
            continue;
        }
        let before = index.checked_sub(1).map(|prev| chars[prev]);
        let after = chars.get(index + 1).copied();
        if arithmetic_context(before) || arithmetic_context(after) {
            return true;
        }
    }
    false
}

/// Extract non-negative integers with their byte offsets, in order of
/// appearance.
fn extract_integers_with_positions(span: &str) -> Vec<(i64, usize)> {
    let mut numbers = Vec::new();
    let mut current = String::new();
    let mut start = 0;
    for (offset, ch) in span.char_indices() {
        if ch.is_ascii_digit() {
            if current.is_empty() {
                start = offset;
            }
            current.push(ch);
        } else if !current.is_empty() {
            if let Ok(value) = current.parse::<i64>() {
                numbers.push((value, start));
            }
            current.clear();
        }
    }
    if !current.is_empty() {
        if let Ok(value) = current.parse::<i64>() {
            numbers.push((value, start));
        }
    }
    numbers
}
