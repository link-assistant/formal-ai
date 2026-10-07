//! Issue #918 (E71, R914-6): the browser-only rows of the handler-precedence
//! table whose native twin already runs on the memory surface.
//!
//! `memory_program` and `memory_program_gap` are `browser_only` in
//! `data/seed/handler-precedence.lino` because the worker holds the memory
//! store its solve table reads. The native runtime runs the same interpreter
//! on its own memory surface (`execute_memory_query_with_options`, behind the
//! CLI's `memory query` and the protocol's memory events). Both runtimes
//! compile a request against the reviewed templates and steps of
//! `data/seed/memory-programs.lino` and render the outcome through the seeded
//! `memory_program_*` responses, so neither holds vocabulary or prose.
//!
//! The answers below are byte-identical to the ones the browser worker gives
//! for the same request over the same store; the browser twin is
//! `rust/tests/web/issue-0918-browser-twins.test.mjs`, which also recomputes
//! the program id from the canonical program text both runtimes hash.
//!
//! `compound_interest` had diverging twins: the native row took its exchange
//! rate from the calculator and converted only to euros, while the worker
//! took a rate table of its own, also converted to rubles, and appended a
//! USD-to-USD "conversion" whenever a principal was spelled in dollars; their
//! number readers also disagreed on a decimal comma (`8,5%`). Both now read
//! the `policy compound_interest` block of `data/seed/handler-rules.lino` (the
//! periods per compounding meaning, the frequency labels, the default rates,
//! the final-amount marker) and the seeded `compound_interest_*` responses; a
//! conversion skips its source currency, and a lone comma before at most two
//! digits is a decimal comma in both. The worker's answers below are
//! unchanged except for the dropped USD-to-USD lines.

use formal_ai::execute_memory_query_with_options;
use formal_ai::memory::{MemoryEvent, MemoryStore};
use formal_ai::memory_program::{MemoryProgramAuthorization, MemoryProgramLimits};
use formal_ai::{ConversationTurn, SolverConfig, UniversalSolver};

const RENAME_REQUEST: &str =
    "List every fact I contributed about X and rename X to Y in all of them.";
const RENAME_EN: &str = "Memory program memory_program_41ef0c9602340676 matched 1 event(s), changed 1, and stopped at fixpoint after 2 iteration(s).";
const RENAME_EMPTY_RU: &str = "Программа памяти memory_program_41ef0c9602340676 сопоставила событий: 0, изменила: 0 и остановилась с результатом fixpoint после итераций: 1.";
const DESTRUCTIVE_EN: &str = "Destructive memory actions require explicit human confirmation. No memory event was erased or retracted.";
const GAP_EN: &str = "I could not compile this memory request without dropping a step. program_gap:no_complete_seeded_family";

fn fact_store() -> MemoryStore {
    MemoryStore::from_events(vec![MemoryEvent {
        id: "fact".to_owned(),
        kind: Some("fact".to_owned()),
        role: Some("user".to_owned()),
        content: Some("X".to_owned()),
        ..MemoryEvent::default()
    }])
}

fn run(prompt: &str, store: &mut MemoryStore) -> (String, String, bool) {
    let execution = execute_memory_query_with_options(
        prompt,
        store,
        None,
        MemoryProgramLimits::default(),
        MemoryProgramAuthorization::Write,
    )
    .unwrap_or_else(|| panic!("{prompt} must reach the memory surface"));
    (
        execution.answer.intent,
        execution.answer.answer,
        execution.changed,
    )
}

#[test]
fn a_seeded_rename_program_runs_to_fixpoint_with_the_seeded_wording() {
    let mut store = fact_store();
    let (intent, answer, changed) = run(RENAME_REQUEST, &mut store);
    assert_eq!(intent, "memory_program");
    assert_eq!(answer, RENAME_EN);
    assert!(changed);
    assert_eq!(store.events()[0].content.as_deref(), Some("Y"));
}

#[test]
fn the_russian_request_compiles_to_the_same_program_and_answers_in_russian() {
    let (intent, answer, changed) = run(
        "Перечисли все факты, которые я добавил о X, и переименуй X в Y во всех них.",
        &mut MemoryStore::default(),
    );
    assert_eq!(intent, "memory_program");
    assert_eq!(answer, RENAME_EMPTY_RU);
    assert!(!changed);
}

#[test]
fn a_destructive_program_is_refused_without_confirmation() {
    let mut store = fact_store();
    let (intent, answer, changed) = run("Delete every fact I contributed about X.", &mut store);
    assert_eq!(intent, "memory_program_refused");
    assert_eq!(answer, DESTRUCTIVE_EN);
    assert!(!changed);
    assert_eq!(store.len(), 1);
}

#[test]
fn a_memory_request_no_seeded_family_covers_names_its_gap() {
    let (intent, answer, changed) = run(
        "Transpose every fact matrix in memory.",
        &mut MemoryStore::default(),
    );
    assert_eq!(intent, "memory_program_gap");
    assert_eq!(answer, GAP_EN);
    assert!(!changed);
}

const COMPOUND_PROMPT: &str = "Invest $1000 at 8% annual interest compounded monthly for 5 years";
const COMPOUND_REPORT: &str = "Compound interest calculation\n\nFormula: A = P(1 + r/n)^(n*t)\nP = 1000 USD\nr = 0.08 (8% annual)\nn = 12 (monthly)\nt = 5 years\n\nStep 1: periodic rate = r/n = 0.08/12 = 0.006666666666667\nStep 2: number of periods = n*t = 12*5 = 60\nStep 3: A = 1000 * (1 + 0.006666666666667)^60\nFinal amount: 1489.85 USD";
const COMPOUND_EUR: &str = "\n\nConversion: USD -> EUR\n1 USD in EUR = 0.92 EUR\n1489.85 USD * 0.92 = 1370.66 EUR\nRate detail: Exchange rate: 1 USD = 0.92 EUR (source: default (hardcoded))\nLive web freshness is not independently verified here; this uses the exchange-rate source available through the local calculator.";
const COMPOUND_RUB: &str = "\n\nConversion: USD -> RUB\n1 USD in RUB = 89.5 RUB\n1489.85 USD * 89.5 = 133341.57 RUB\nRate detail: Exchange rate: 1 USD = 89.5 RUB (source: default (hardcoded))";

fn offline_solver() -> UniversalSolver {
    UniversalSolver::new(SolverConfig {
        offline: true,
        ..SolverConfig::default()
    })
}

#[test]
fn a_compound_interest_report_converts_at_the_seeded_euro_and_ruble_rates() {
    let solver = offline_solver();
    for (prompt, conversion) in [
        (
            format!(
                "{COMPOUND_PROMPT} and convert the final amount to EUR using current exchange rates from the web."
            ),
            COMPOUND_EUR,
        ),
        (
            format!("{COMPOUND_PROMPT} and convert the final amount to rubles."),
            COMPOUND_RUB,
        ),
    ] {
        let answer = solver.solve(&prompt);
        assert_eq!(answer.intent, "calculation", "{prompt}");
        assert_eq!(
            answer.answer,
            format!("{COMPOUND_REPORT}{conversion}"),
            "{prompt}"
        );
    }
}

#[test]
fn a_principal_spelled_in_dollars_is_never_converted_to_dollars() {
    let answer =
        offline_solver().solve("invest 1000 dollars at 8,5% interest compounded daily for 2 years");
    assert_eq!(answer.intent, "calculation");
    assert_eq!(
        answer.answer,
        "Compound interest calculation\n\nFormula: A = P(1 + r/n)^(n*t)\nP = 1000 USD\nr = 0.085 (8.5% annual)\nn = 365 (daily)\nt = 2 years\n\nStep 1: periodic rate = r/n = 0.085/365 = 0.000232876712329\nStep 2: number of periods = n*t = 365*2 = 730\nStep 3: A = 1000 * (1 + 0.000232876712329)^730\nFinal amount: 1185.28 USD"
    );
}

#[test]
fn a_follow_up_converts_the_earlier_final_amount() {
    let history = [
        ConversationTurn::user(COMPOUND_PROMPT),
        ConversationTurn::assistant(COMPOUND_REPORT),
    ];
    let answer =
        offline_solver().solve_with_history("Convert the final amount to rubles", &history);
    assert_eq!(answer.intent, "calculation");
    assert_eq!(
        answer.answer,
        format!("Final amount conversion\nSource amount: 1489.85 USD{COMPOUND_RUB}")
    );
}
