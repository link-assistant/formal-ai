//! Issue #901: automate TRIZ principles and general paradox resolution.
//!
//! The ask, in three parts: (1) contradictions are links carrying a 0-1
//! value chosen from the requirements, which `rust/src/selection_heuristics.rs`
//! already computes for candidate sets; (2) collect the *other* common
//! ways paradoxes and contradictions dissolve — different dimensions,
//! alternation, points on the range — as data; (3) replay it on a top-20
//! corpus of commonly discussed invention tasks and expose it from a
//! prompt. These tests pin the seed additions (twelve resolution
//! families, the twenty-task benchmark, the cues) and the user-facing
//! entry in `rust/src/triz_solver.rs`.

use formal_ai::event_log::EventLog;
use formal_ai::triz_solver::{handle_triz, triz_benchmark_tasks, triz_families};
use formal_ai::web_engine_core::normalize_prompt;

/// The handler answer for a raw prompt.
fn handled(prompt: &str) -> Option<formal_ai::engine::SymbolicAnswer> {
    handle_triz(prompt, &normalize_prompt(prompt), &mut EventLog::new())
}

/// The twelve seeded resolution families, one `- name: mechanism` line each,
/// in seed order (`data/seed/triz-principles.lino`).
const TRIZ_FAMILIES: &str = concat!(
    "- Range selection: Treat the contradiction as a link carrying a value from 0 to 1 (or -1 to 1) and choose the operating point that fits the stated requirements; the requirement's clauses vote, so 10 %, 50 % or 80 % is derived, never assumed.\n",
    "- Dimension change: Move to a dimension the contradiction does not live in -- geometry, field, state, scale -- so both poles hold there.\n",
    "- Separation in time: Give each pole its own interval -- alternate, schedule, duty-cycle -- instead of satisfying both at once.\n",
    "- Separation in space: Let each pole own a zone of the system; the contradiction only exists where both were forced into one place.\n",
    "- Separation upon condition: Make the system switch its own property when the condition flips, so each pole is satisfied exactly when it applies.\n",
    "- Escalate to the supersystem: Merge the conflicting elements into a larger whole that carries both properties at different roles.\n",
    "- Descend to subsystems: Split the element so different parts carry different poles; laminate, layer, distribute.\n",
    "- Phase and state transition: Change phase, state, or field regime so the conflicting property pair ceases to exist.\n",
    "- Accumulate then release: Buffer the scarce resource over time and spend the buffer at the peak, dissolving the peak-vs-average contradiction.\n",
    "- Invert the problem: Solve the anti-problem: make the harmful effect useful, or let the enemy of the goal do the work.\n",
    "- Bypass and redefine: Change the requirement frame: ask what the constraint serves, then satisfy the purpose without the constrained means.\n",
    "- Partial-spectrum solution: Replace the binary choice with a graceful spectrum -- approximate, degrade softly, mix ratios -- and pick the point the requirements name.",
);

/// The range-selection note every TRIZ answer ends with.
const TRIZ_LINK_NOTE: &str = "A contradiction is a link whose value (0-1) is chosen from the requirement's own clauses — basis points in the selection heuristic, with no default 50 %. Say which side the requirements favor and the point on the range follows.";

/// No benchmark task shares a word with the Russian prompt, so the first
/// three corpus entries are cited as canonical shape examples.
const RUSSIAN_PRECEDENTS: &str = concat!(
    "- Coat tiny pills evenly without wasting coating material or sticking them together. (pharmaceutical manufacturing: coverage vs waste) — methods: family_dimension_change, family_phase_transition\n",
    "- Separate wheat grains from stones of the same size and weight class. (agriculture: purity vs throughput) — methods: family_condition_separation, family_dimension_change\n",
    "- Mow wet grass without the clippings clogging the mower deck. (consumer equipment: cutting quality vs clogging) — methods: family_dimension_change, family_time_separation",
);

/// The umbrella task is the only benchmark statement the prompt overlaps.
const UMBRELLA_PRECEDENTS: &str = "- An umbrella big enough in rain yet small enough in a crowded bus. (consumer products: coverage vs packed size) — methods: family_space_separation, family_dimension_change";

/// Pill coating overlaps most (`pills`, `evenly`, `without`), chimney height
/// next (`engineering`, `without`), then the one-word ties by task id.
const PILL_PRECEDENTS: &str = concat!(
    "- Coat tiny pills evenly without wasting coating material or sticking them together. (pharmaceutical manufacturing: coverage vs waste) — methods: family_dimension_change, family_phase_transition\n",
    "- Disperse plant exhaust without building an ever taller chimney. (civil engineering: dispersion vs structure cost) — methods: family_phase_transition, family_supersystem\n",
    "- Protect a steel bridge from rust without repainting it every few years. (infrastructure: protection interval vs maintenance cost) — methods: family_inverse_problem, family_supersystem",
);

/// The exact TRIZ answer: the English `triz_resolution_map` template with
/// the family map, the cited precedents, and the link note filled in.
fn expected_answer(precedents: &str) -> String {
    format!(
        "A contradiction is a link with a value, not a wall. Ways through:\n{TRIZ_FAMILIES}\nPrecedent from the benchmark corpus:\n{precedents}\n{TRIZ_LINK_NOTE}"
    )
}

#[test]
fn the_seed_carries_twelve_families_and_twenty_benchmark_tasks() {
    let families = triz_families();
    assert!(families.len() >= 12, "got {}", families.len());
    // The two shapes #901 names explicitly must be present.
    assert!(
        families
            .iter()
            .any(|family| family.method_id == "family_range_selection")
    );
    assert!(
        families
            .iter()
            .any(|family| family.method_id == "family_dimension_change")
    );
    let tasks = triz_benchmark_tasks();
    assert_eq!(tasks.len(), 20, "the top-20 corpus");
    for task in &tasks {
        assert!(
            !task.contradiction.is_empty(),
            "{} states its contradiction",
            task.task_id
        );
        assert!(
            !task.methods.is_empty(),
            "{} names its methods",
            task.task_id
        );
    }
}

#[test]
fn a_russian_contradiction_prompt_gets_the_family_map() {
    let answer =
        handled("Как разрешить противоречие: деталь должна быть жёсткой и одновременно гибкой?")
            .expect("a contradiction question must be handled");
    assert_eq!(answer.answer, expected_answer(RUSSIAN_PRECEDENTS));
    assert_eq!(answer.intent, "triz_resolution");
    assert!(
        answer.answer.contains("Range selection") && answer.answer.contains("Dimension change"),
        "the family map must be in the answer: {}",
        answer.answer
    );
    assert!(
        answer.answer.contains("no default 50 %"),
        "the 0-1 link value note must be stated: {}",
        answer.answer
    );
    assert!(
        answer.answer.contains("methods: family_"),
        "precedents must cite their methods: {}",
        answer.answer
    );
}

#[test]
fn an_umbrella_prompt_finds_the_umbrella_precedent() {
    let answer = handled(
        "Inventive problem: an umbrella must be big enough in rain yet small enough in a crowded bus.",
    )
    .expect("handled");
    assert_eq!(answer.answer, expected_answer(UMBRELLA_PRECEDENTS));
    assert!(
        answer.answer.contains("umbrella") && answer.answer.contains("crowded bus"),
        "the matched benchmark task must be cited: {}",
        answer.answer
    );
    assert!(
        answer.answer.contains("family_space_separation"),
        "the umbrella task's methods must be named: {}",
        answer.answer
    );
}

#[test]
fn the_triz_acronym_and_english_paradox_cues_trigger() {
    assert!(handled("Apply TRIZ to smartwatch battery vs thickness").is_some());
    assert!(handled("How do we resolve this paradox: transparency vs strength?").is_some());
}

#[test]
fn ordinary_prompts_decline() {
    assert!(handled("What is the capital of France?").is_none());
    assert!(handled("Напомни мне встречу в 20:00").is_none());
}

#[test]
fn the_handler_logs_its_cue_and_precedents() {
    let prompt = "Engineering contradiction: coat tiny pills evenly without waste";
    let mut log = EventLog::new();
    let answer = handle_triz(prompt, &normalize_prompt(prompt), &mut log).expect("handled");
    assert_eq!(answer.answer, expected_answer(PILL_PRECEDENTS));
    assert!(log.first_of("triz_solver:cued").is_some());
    assert!(
        log.first_of("triz_solver:precedent").is_some(),
        "a matched precedent must be logged"
    );
    assert!(
        answer.answer.contains("pill"),
        "the pill-coating task is the overlapping precedent: {}",
        answer.answer
    );
}
