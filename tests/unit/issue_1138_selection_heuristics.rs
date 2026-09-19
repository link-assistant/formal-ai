//! Issue #1138 B12, plan 12 leaf 17: selection corpora in five languages.
//!
//! Ten moonshot-shaped prompts (#453) that must split into exactly two children
//! with a reported imbalance — five in the plan's wording and five held-out
//! paraphrases of the same three obligations — plus five TRIZ prompts that state
//! a trade-off explicitly, so the selection value is *derived* from clause
//! counts and never guessed.
//!
//! Written before the leaves that make them pass (plan 14 wave T).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::relative_meta_logic::SourceTier;
use formal_ai::selection_heuristics::{
    ActionCost, ApproachObservation, CandidateScore, ContradictionDerivation, SplitRefusal,
    balanced_split, combine_approaches, contradictions_in, split_refusal,
};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn lino_records(text: &str) -> Vec<Vec<&str>> {
    let mut records = Vec::new();
    let mut current = Vec::new();
    for line in text
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
    {
        if !line.starts_with(char::is_whitespace) && !current.is_empty() {
            records.push(std::mem::take(&mut current));
        }
        current.push(line);
    }
    if !current.is_empty() {
        records.push(current);
    }
    records
}

fn lino_field(record: &[&str], wanted: &str) -> String {
    record
        .iter()
        .filter_map(|line| line.trim().split_once(' '))
        .find_map(|(name, value)| {
            (name == wanted).then(|| value.trim().trim_matches('"').to_owned())
        })
        .unwrap_or_default()
}

/// The moonshot, in the plan's wording. Three obligations: an architecture that
/// teaches itself to a stated score band, a benchmark for it, and an explanation
/// of how the benchmark is run.
const MOONSHOT: [(&str, &str); 5] = [
    (
        "en",
        "Design an architecture that teaches itself to score steadily between 860 and 864 points in Atari Breakout, write a benchmark for it, and explain how the benchmark is run.",
    ),
    (
        "ru",
        "Разработай архитектуру, которая сама научилась бы стабильно выбивать от 860 до 864 очков в Atari Breakout, напиши для неё бенчмарк и объясни, как этот бенчмарк запускается.",
    ),
    (
        "hi",
        "एक ऐसी संरचना बनाओ जो खुद सीखकर Atari Breakout में लगातार 860 से 864 अंक ला सके, उसके लिए एक बेंचमार्क लिखो, और बताओ कि वह बेंचमार्क कैसे चलाया जाता है।",
    ),
    (
        "zh",
        "设计一个能够自己学会在 Atari Breakout 中稳定拿到 860 到 864 分的架构，为它写一个基准测试，并说明这个基准测试怎么运行。",
    ),
    (
        "es",
        "Diseña una arquitectura que aprenda por sí sola a sacar de forma estable entre 860 y 864 puntos en Atari Breakout, escribe un banco de pruebas para ella y explica cómo se ejecuta ese banco de pruebas.",
    ),
];

/// The same three obligations, worded differently. The split shape and the
/// imbalance must be identical, or the split is reading the wording.
const MOONSHOT_PARAPHRASE: [(&str, &str); 5] = [
    (
        "en",
        "I want a self-teaching architecture that reliably lands 860-864 in Breakout. Also produce a benchmark. Also document the way to run it.",
    ),
    (
        "ru",
        "Мне нужна самообучающаяся архитектура, надёжно набирающая 860-864 в Breakout. Ещё сделай бенчмарк. И опиши, как его запускать.",
    ),
    (
        "hi",
        "मुझे एक स्वयं सीखने वाली संरचना चाहिए जो Breakout में भरोसे से 860-864 लाए। साथ ही एक बेंचमार्क भी बनाओ। और उसे चलाने का तरीका भी लिखो।",
    ),
    (
        "zh",
        "我要一个能自学的架构，在 Breakout 里稳定拿到 860 到 864 分。另外做一个基准测试。再写清楚运行它的方法。",
    ),
    (
        "es",
        "Quiero una arquitectura que aprenda sola y saque de forma fiable 860-864 en Breakout. Haz además un banco de pruebas. Y documenta cómo ejecutarlo.",
    ),
];

/// An explicit trade-off with a stated lean, so the value is countable.
const TRADE_OFF: [(&str, &str); 5] = [
    (
        "en",
        "Give me the shortest answer that still covers every case, and if you must choose, favour completeness.",
    ),
    (
        "ru",
        "Дай самый короткий ответ, который всё же покрывает каждый случай, а если придётся выбирать — отдай предпочтение полноте.",
    ),
    (
        "hi",
        "सबसे छोटा उत्तर दो जो फिर भी हर मामले को कवर करे, और अगर चुनना पड़े तो पूर्णता को प्राथमिकता दो।",
    ),
    (
        "zh",
        "给我最短的答案，但仍然要覆盖每一种情况；如果必须取舍，优先保证完整。",
    ),
    (
        "es",
        "Dame la respuesta más corta que aun así cubra todos los casos y, si tienes que elegir, prioriza la exhaustividad.",
    ),
];

/// Two candidates that each win on a different cost dimension.
fn tied_pair() -> Vec<CandidateScore> {
    vec![
        CandidateScore {
            candidate_id: "concise".to_owned(),
            checks: (4, 4),
            cost: ActionCost {
                steps: 18,
                code_size: 120,
                resource_units: 0,
                leaf_count: 2,
            },
        },
        CandidateScore {
            candidate_id: "complete".to_owned(),
            checks: (4, 4),
            cost: ActionCost {
                steps: 4,
                code_size: 480,
                resource_units: 0,
                leaf_count: 2,
            },
        },
    ]
}

#[test]
fn a_moonshot_prompt_splits_into_exactly_two_children() {
    let mut failures: Vec<String> = Vec::new();
    for (language, prompt) in MOONSHOT {
        match balanced_split(prompt) {
            Some(split) => {
                let total = split.left_weight + split.right_weight;
                if total != 3 {
                    failures.push(format!(
                        "{language}: three obligations regrouped into {total} segments"
                    ));
                }
                if split.imbalance() != 1 {
                    failures.push(format!(
                        "{language}: a three-clause task splits 2/1, imbalance 1, got {}",
                        split.imbalance()
                    ));
                }
                if split.left.trim().is_empty() || split.right.trim().is_empty() {
                    failures.push(format!("{language}: a child is empty"));
                }
            }
            None => failures.push(format!("{language}: three obligations did not split")),
        }
    }
    assert!(
        failures.is_empty(),
        "the moonshot carries three obligations in every language and must become a \
         two-child node with a reported imbalance: {failures:?}"
    );
}

#[test]
fn a_held_out_paraphrase_produces_the_same_split_shape_and_imbalance() {
    let mut failures: Vec<String> = Vec::new();
    for ((language, original), (_, paraphrase)) in MOONSHOT.iter().zip(MOONSHOT_PARAPHRASE.iter()) {
        let (Some(first), Some(second)) = (balanced_split(original), balanced_split(paraphrase))
        else {
            failures.push(format!("{language}: one of the two wordings did not split"));
            continue;
        };
        if first.imbalance() != second.imbalance() {
            failures.push(format!(
                "{language}: imbalance {} vs {}",
                first.imbalance(),
                second.imbalance()
            ));
        }
        if (first.left_weight + first.right_weight) != (second.left_weight + second.right_weight) {
            failures.push(format!("{language}: different segment counts"));
        }
    }
    assert!(
        failures.is_empty(),
        "the same three obligations, differently worded, must produce the same shape; \
         otherwise the split is reading the wording rather than the obligations: \
         {failures:?}"
    );
}

#[test]
fn an_explicit_trade_off_requirement_moves_the_selection_value() {
    let mut failures: Vec<String> = Vec::new();
    for (language, requirement) in TRADE_OFF {
        let found = contradictions_in(&tied_pair(), requirement);
        let Some(link) = found.first() else {
            failures.push(format!("{language}: no contradiction detected"));
            continue;
        };
        match &link.derivation {
            ContradictionDerivation::RequirementClauses {
                a_clauses,
                b_clauses,
            } => {
                if b_clauses <= a_clauses {
                    failures.push(format!(
                        "{language}: `favour completeness` must add a clause to \
                         completeness, got {a_clauses}/{b_clauses}"
                    ));
                }
            }
            other => failures.push(format!("{language}: value not derived, got {other:?}")),
        }
        if link.selection_basis_points <= 5_000 {
            failures.push(format!(
                "{language}: the stated lean must move the value past the midpoint, got {}",
                link.selection_basis_points
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "the selection value is derived from the requirement's own clauses in every \
         language, never guessed and never defaulted to 50 %: {failures:?}"
    );
}

#[test]
fn the_twenty_task_triz_corpus_derives_each_declared_selection_relation() {
    let text = fs::read_to_string(repo_root().join("data/benchmarks/selection-triz.lino"))
        .expect("selection-triz.lino should be readable");
    let records = lino_records(&text);
    let cases = records
        .iter()
        .filter(|record| lino_field(record, "record_type") == "triz_selection_case")
        .collect::<Vec<_>>();
    assert_eq!(cases.len(), 20, "#901 requires a twenty-task corpus");
    let languages = cases
        .iter()
        .map(|record| lino_field(record, "language"))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        languages,
        BTreeSet::from(["en", "es", "hi", "ru", "zh"].map(str::to_owned))
    );
    let mut per_language = BTreeMap::new();
    let mut prompts = BTreeSet::new();
    let mut failures = Vec::new();
    for record in cases {
        let language = lino_field(record, "language");
        *per_language.entry(language.clone()).or_insert(0_usize) += 1;
        let prompt = lino_field(record, "prompt");
        assert!(
            prompts.insert(prompt.clone()),
            "every TRIZ task is distinct"
        );
        let expected = lino_field(record, "expected");
        let link = contradictions_in(&tied_pair(), &prompt)
            .into_iter()
            .next()
            .expect("the tied candidates form one technical contradiction");
        let observed = match link.derivation {
            ContradictionDerivation::Underivable { .. } => "unresolved",
            ContradictionDerivation::RequirementClauses { .. }
                if link.selection_basis_points < 5_000 =>
            {
                "toward_brevity"
            }
            ContradictionDerivation::RequirementClauses { .. }
                if link.selection_basis_points > 5_000 =>
            {
                "toward_completeness"
            }
            ContradictionDerivation::RequirementClauses { .. } => "midpoint",
            ContradictionDerivation::SeededRecord { .. } => "seeded",
        };
        if observed != expected {
            failures.push(format!(
                "{language}: {prompt:?} expected {expected}, observed {observed} at {}",
                link.selection_basis_points
            ));
        }
    }
    assert_eq!(
        per_language,
        BTreeMap::from([
            (String::from("en"), 4),
            (String::from("es"), 4),
            (String::from("hi"), 4),
            (String::from("ru"), 4),
            (String::from("zh"), 4),
        ]),
        "each language contributes four independent trade-off shapes"
    );
    assert!(
        failures.is_empty(),
        "TRIZ selection relations must be derived from the requirement clauses: {failures:?}"
    );
}

#[test]
fn a_single_clause_moonshot_is_reported_as_underivable_not_atomic() {
    // #453's original prompt, in five languages. One clause, so there is no
    // boundary to cut at; until plan 01's lookup of published decomposition
    // approaches lands, the honest report is `Underivable` with the named
    // blocker — never "the task is atomic".
    let singles = [
        ("en", "Write a strong artificial intelligence"),
        ("ru", "Напиши сильный искусственный интеллект"),
        ("hi", "एक सशक्त कृत्रिम बुद्धिमत्ता लिखो"),
        ("zh", "写一个强人工智能"),
        ("es", "Escribe una inteligencia artificial fuerte"),
    ];
    let mut failures: Vec<String> = Vec::new();
    for (language, prompt) in singles {
        if balanced_split(prompt).is_some() {
            failures.push(format!("{language}: a single clause must not split"));
            continue;
        }
        match split_refusal(prompt) {
            Some(SplitRefusal::Underivable { blocker }) if !blocker.trim().is_empty() => {}
            Some(SplitRefusal::SingleClause { reason }) => failures.push(format!(
                "{language}: a moonshot is underivable, not merely single-clause: {reason}"
            )),
            Some(SplitRefusal::Underivable { .. }) => {
                failures.push(format!("{language}: the blocker must be named"));
            }
            None => failures.push(format!("{language}: a refused split must report why")),
        }
    }
    assert!(
        failures.is_empty(),
        "a single-clause moonshot is reported as underivable with its blocker, which is \
         the honest #453 limit until plan 01 lands: {failures:?}"
    );
}

#[test]
fn equivalent_approaches_merge_without_losing_their_first_historical_source() {
    let combined = combine_approaches(&[
        ApproachObservation {
            approach: String::from("Search the trusted source registry first."),
            source: String::from("history:turn-2"),
            tier: SourceTier::IndependentCorroboration,
            semantic_terms: vec![String::from("method:trusted_source_walk")],
        },
        ApproachObservation {
            approach: String::from("Сначала обойди реестр доверенных источников."),
            source: String::from("history:turn-5"),
            tier: SourceTier::OriginalFirstParty,
            semantic_terms: vec![String::from("method:trusted_source_walk")],
        },
        ApproachObservation {
            approach: String::from("Generate and refute a finite hypothesis set."),
            source: String::from("history:turn-7"),
            tier: SourceTier::IndependentCorroboration,
            semantic_terms: vec![String::from("method:refutation_search")],
        },
    ]);
    let projection = combined
        .iter()
        .map(|approach| {
            (
                approach.approach.as_str(),
                approach.first_source.as_str(),
                approach
                    .sources
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        projection,
        [
            (
                "Search the trusted source registry first.",
                "history:turn-2",
                vec!["history:turn-2", "history:turn-5"],
            ),
            (
                "Generate and refute a finite hypothesis set.",
                "history:turn-7",
                vec!["history:turn-7"],
            ),
        ]
    );
    assert!(
        combined
            .iter()
            .all(|approach| approach.approach_id.starts_with("statement_")),
        "every distinct approach remains content-addressable"
    );
}
