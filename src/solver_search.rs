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
use crate::solver::SolverConfig;
use crate::solver_handlers::finalize_simple;

/// Arithmetic operators the search can place between operands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Add,
    Sub,
    Mul,
}

impl Op {
    const fn symbol(self) -> char {
        match self {
            Self::Add => '+',
            Self::Sub => '-',
            Self::Mul => '*',
        }
    }

    const fn apply(self, lhs: i64, rhs: i64) -> i64 {
        match self {
            Self::Add => lhs.saturating_add(rhs),
            Self::Sub => lhs.saturating_sub(rhs),
            Self::Mul => lhs.saturating_mul(rhs),
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
    fn evaluate(&self, numbers: &[i64]) -> i64 {
        let mut acc = numbers[self.order[0]];
        for (index, op) in self.ops.iter().enumerate() {
            acc = op.apply(acc, numbers[self.order[index + 1]]);
        }
        acc
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

    log.append(
        "search:problem",
        format!(
            "reach target={} using numbers=[{}] with ops=[{}]",
            problem.target,
            join_numbers(&problem.numbers),
            join_ops(&problem.ops),
        ),
    );
    log.append("search:budget", config.compute_budget.to_string());
    record_generated_tests(log, &problem);

    let outcome = run_search(prompt, log, &problem, config.compute_budget);
    match outcome {
        Some(solution) => {
            log.append(
                "search:solution",
                format!("{} = {}", solution.expression, problem.target),
            );
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
        log.append("search:exhausted", "evaluations=0 budget=0".to_owned());
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
    log.append(
        "search:random",
        format!(
            "sampled={} best_diff={}",
            evaluations,
            best.as_ref().map_or(i64::MAX, |(_, diff)| *diff)
        ),
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
            log.append(
                "search:evolutionary",
                format!("generation={generation} best_diff=0"),
            );
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
            log.append(
                "search:evolutionary",
                format!(
                    "generation={generation} best_diff={}",
                    best.as_ref().map_or(i64::MAX, |(_, diff)| *diff)
                ),
            );
        }
    }

    log.append(
        "search:exhausted",
        format!(
            "evaluations={evaluations} best_diff={}",
            best.as_ref().map_or(i64::MAX, |(_, diff)| *diff)
        ),
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
    log.append(
        "search:candidate",
        format!(
            "phase={phase} evaluations={evaluations} expression={}",
            candidate.render(&problem.numbers)
        ),
    );
    SearchSolution {
        expression: candidate.render(&problem.numbers),
        evaluations,
    }
}

/// Fitness distance: 0 means every generated test passes (the composition uses
/// each number once, only allowed operators, and evaluates to the target).
fn score(candidate: &Candidate, problem: &SearchProblem) -> i64 {
    (candidate.evaluate(&problem.numbers) - problem.target).abs()
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
        "search:test",
        format!(
            "uses each of [{}] exactly once",
            join_numbers(&problem.numbers)
        ),
    );
    log.append(
        "search:test",
        format!("uses only operators [{}]", join_ops(&problem.ops)),
    );
    log.append(
        "search:test",
        format!("expression evaluates to {}", problem.target),
    );
}

fn build_answer(
    prompt: &str,
    log: &mut EventLog,
    problem: &SearchProblem,
    solution: &SearchSolution,
    budget: u32,
) -> SymbolicAnswer {
    let body = format!(
        concat!(
            "Found by budget-driven search: {expression} = {target}.\n",
            "No reusable part or rule matched, so the solver combined the given ",
            "numbers with the allowed operators and scored each candidate against ",
            "the generated equality tests as the fitness function.\n",
            "Search budget: {budget} candidate evaluations; a satisfying ",
            "composition was found after {evaluations} evaluations.\n",
            "Search path: {trace_id}",
        ),
        expression = solution.expression,
        target = problem.target,
        budget = budget,
        evaluations = solution.evaluations,
        trace_id = stable_id("search", prompt),
    );
    finalize_simple(
        prompt,
        log,
        "budget_search_solution",
        "response:search:solution",
        &body,
        0.9,
    )
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
/// language (en, ru, hi, zh). Returns `None` for any prompt that is not clearly
/// of this shape so the stage stays inert for the overwhelming majority of
/// impulses. Digits and the `+`/`-`/`*` symbols are language-neutral; only the
/// "combine numbers", search-verb, and target framings are localized.
fn parse_search_problem(prompt: &str) -> Option<SearchProblem> {
    // Unicode-aware lowercasing so Cyrillic framing keywords match regardless of
    // case (Devanagari and Han are caseless; ASCII is unaffected).
    let lower = prompt.to_lowercase();

    // Gate: require both a "combine numbers" framing and a search verb so plain
    // calculations ("3 + 5") never reach this path.
    if !contains_any(&lower, &NUMBERS_FRAMING) || !contains_any(&lower, &SEARCH_VERBS) {
        return None;
    }

    // Locate the target value as the integer nearest a target keyword. This
    // handles both operand-then-target order (en/ru/zh: "equals 26") and
    // target-then-keyword order (hi: "26 के बराबर").
    let integers = extract_integers_with_positions(&lower);
    if integers.len() < 3 {
        // Need at least two operands plus a distinct target.
        return None;
    }
    let keyword_positions = target_keyword_positions(&lower);
    if keyword_positions.is_empty() {
        return None;
    }
    let target_index = integers
        .iter()
        .enumerate()
        .min_by_key(|(_, (_, position))| distance_to_nearest(*position, &keyword_positions))
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

/// "Combine the numbers" framing across the supported languages. Matched as
/// substrings so inflected forms (`числа`/`чисел`, `संख्याओं`) still hit.
const NUMBERS_FRAMING: [&str; 4] = [
    "number", // en
    "числ",   // ru: числа / чисел / число / числами
    "संख्या",   // hi: संख्या / संख्याओं
    "数字",   // zh
];

/// Search verbs (find / combine / reach / make / express / arrange) across the
/// supported languages.
const SEARCH_VERBS: [&str; 24] = [
    // en
    "find",
    "combine",
    "reach",
    "make",
    "express",
    "arrange",
    // ru
    "найд",
    "комбин",
    "состав",
    "получ",
    "выраз",
    "достич",
    // hi
    "खोज",
    "बनाए",
    "संयोज",
    "प्राप्त",
    "व्यक्त",
    "पहुँच",
    // zh
    "找",
    "组合",
    "得到",
    "表达",
    "达到",
    "使",
];

/// Target keywords ("equals" and friends) across the supported languages. Their
/// positions anchor which integer is the target value.
const TARGET_KEYWORDS: [&str; 14] = [
    // en
    "equals",
    "equal to",
    "results in",
    "gives",
    "reach",
    "make",
    "get",
    // ru
    "равно",
    "равн",
    "получ",
    // hi
    "बराबर",
    // zh
    "等于",
    "得到",
    "达到",
];

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

/// Byte offsets at which any target keyword begins in `lower`.
fn target_keyword_positions(lower: &str) -> Vec<usize> {
    let mut positions = Vec::new();
    for keyword in TARGET_KEYWORDS {
        let mut from = 0;
        while let Some(offset) = lower[from..].find(keyword) {
            let absolute = from + offset;
            positions.push(absolute);
            from = absolute + keyword.len();
        }
    }
    positions
}

fn distance_to_nearest(position: usize, keyword_positions: &[usize]) -> usize {
    keyword_positions
        .iter()
        .map(|&keyword| position.abs_diff(keyword))
        .min()
        .unwrap_or(usize::MAX)
}

fn parse_ops(lower: &str) -> Vec<Op> {
    let mut ops = Vec::new();
    if lower.contains('+')
        || contains_any(
            lower,
            &[
                "plus",
                "add",
                "sum",
                "плюс",
                "сложе",
                "сумм",
                "जोड़",
                "योग",
                "加",
            ],
        )
    {
        ops.push(Op::Add);
    }
    if contains_any(
        lower,
        &[
            "minus",
            "subtract",
            "difference",
            "минус",
            "вычит",
            "разност",
            "घटा",
            "अंतर",
            "减",
        ],
    ) {
        ops.push(Op::Sub);
    }
    if lower.contains('*')
        || lower.contains('×')
        || contains_any(
            lower,
            &[
                "times",
                "multiply",
                "product",
                "умнож",
                "произвед",
                "गुणा",
                "乘",
            ],
        )
    {
        ops.push(Op::Mul);
    }
    if ops.is_empty() {
        // No operator named: allow the full toolbox.
        ops = vec![Op::Add, Op::Sub, Op::Mul];
    }
    ops
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
