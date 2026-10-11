//! Issue #1172 R9: the facts the committed Wikidata captures can reproduce
//! are derived, not written.
//!
//! The capitals of Japan, Russia, France, Germany, China, India, the USA, the
//! UK and Brazil and the author of The Lord of the Rings were retired from
//! `data/seed/facts.lino`. `seed::fact_derivation` rebuilds each from its
//! subject capture in `data/seed/fact-captures.lino` (projected from
//! `data/cache/wikidata/fact-claims` by `scripts/ground-fact-captures.py`) and
//! the realization rules of `data/seed/fact-realization.lino`: the value is
//! the subject's truthy claim, the labels and sources are the captures', and
//! each sentence is the relation's template with the Russian case forms, the
//! English article and the recorded preferred label variants. Twin of
//! `rust/tests/web/issue-1172-seeded-fact-retirement.test.mjs`.

use formal_ai::FormalAiEngine;
use formal_ai::seed::{FactRecord, facts};

/// The answers the retired written records gave, per subject and language.
const RETIRED_SUMMARIES: [(&str, &str, &str); 40] = [
    ("Q17", "en", "The capital of Japan is Tokyo."),
    ("Q17", "ru", "Столица Японии — Токио."),
    ("Q17", "hi", "जापान की राजधानी टोक्यो है।"),
    ("Q17", "zh", "日本的首都是东京。"),
    ("Q159", "en", "The capital of Russia is Moscow."),
    ("Q159", "ru", "Столица России — Москва."),
    ("Q159", "hi", "रूस की राजधानी मास्को है।"),
    ("Q159", "zh", "俄罗斯的首都是莫斯科。"),
    ("Q142", "en", "The capital of France is Paris."),
    ("Q142", "ru", "Столица Франции — Париж."),
    ("Q142", "hi", "फ्रांस की राजधानी पेरिस है।"),
    ("Q142", "zh", "法国的首都是巴黎。"),
    ("Q183", "en", "The capital of Germany is Berlin."),
    ("Q183", "ru", "Столица Германии — Берлин."),
    ("Q183", "hi", "जर्मनी की राजधानी बर्लिन है।"),
    ("Q183", "zh", "德国的首都是柏林。"),
    ("Q148", "en", "The capital of China is Beijing."),
    ("Q148", "ru", "Столица Китая — Пекин."),
    ("Q148", "hi", "चीन की राजधानी बीजिंग है।"),
    ("Q148", "zh", "中国的首都是北京。"),
    ("Q668", "en", "The capital of India is New Delhi."),
    ("Q668", "ru", "Столица Индии — Нью-Дели."),
    ("Q668", "hi", "भारत की राजधानी नई दिल्ली है।"),
    ("Q668", "zh", "印度的首都是新德里。"),
    (
        "Q30",
        "en",
        "The capital of the United States is Washington, D.C.",
    ),
    ("Q30", "ru", "Столица США — Вашингтон."),
    (
        "Q30",
        "hi",
        "संयुक्त राज्य अमेरिका की राजधानी वाशिंगटन, डी.सी. है।",
    ),
    ("Q30", "zh", "美国的首都是华盛顿哥伦比亚特区。"),
    ("Q145", "en", "The capital of the United Kingdom is London."),
    ("Q145", "ru", "Столица Великобритании — Лондон."),
    ("Q145", "hi", "यूनाइटेड किंगडम की राजधानी लंदन है।"),
    ("Q145", "zh", "英国的首都是伦敦。"),
    ("Q155", "en", "The capital of Brazil is Brasília."),
    ("Q155", "ru", "Столица Бразилии — Бразилиа."),
    ("Q155", "hi", "ब्राज़ील की राजधानी ब्रासीलिया है।"),
    ("Q155", "zh", "巴西的首都是巴西利亚。"),
    (
        "Q15228",
        "en",
        "The Lord of the Rings was written by J. R. R. Tolkien.",
    ),
    (
        "Q15228",
        "ru",
        "«Властелин колец» был написан Дж. Р. Р. Толкином.",
    ),
    (
        "Q15228",
        "hi",
        "द लॉर्ड ऑफ द रिंग्स को जे. आर. आर. टॉल्किन ने लिखा था।",
    ),
    ("Q15228", "zh", "《魔戒》由 J·R·R·托爾金 创作。"),
];

fn derived() -> Vec<FactRecord> {
    facts()
        .into_iter()
        .filter(|record| !record.subject_qid.is_empty())
        .collect()
}

#[test]
fn every_retired_answer_is_realized_verbatim_from_the_captures() {
    let realized: Vec<(String, String, String)> = derived()
        .iter()
        .flat_map(|record| {
            record.localized.iter().map(|localized| {
                (
                    record.subject_qid.clone(),
                    localized.language.clone(),
                    localized.summary.clone(),
                )
            })
        })
        .collect();
    let expected: Vec<(String, String, String)> = RETIRED_SUMMARIES
        .iter()
        .map(|(qid, language, summary)| {
            (
                (*qid).to_owned(),
                (*language).to_owned(),
                (*summary).to_owned(),
            )
        })
        .collect();
    assert_eq!(realized, expected);
}

#[test]
fn the_derived_records_read_their_values_from_the_capture_claims() {
    let records: Vec<(String, String, String, String)> = derived()
        .into_iter()
        .map(|record| {
            (
                record.slug,
                record.subject_qid,
                record.value_qid,
                record.wikidata.join(" "),
            )
        })
        .collect();
    let expected = [
        ("fact_capital_q17", "Q17", "Q1490", "Q17 Q1490"),
        ("fact_capital_q159", "Q159", "Q649", "Q159 Q649"),
        ("fact_capital_q142", "Q142", "Q90", "Q142 Q90"),
        ("fact_capital_q183", "Q183", "Q64", "Q183 Q64"),
        ("fact_capital_q148", "Q148", "Q956", "Q148 Q956"),
        ("fact_capital_q668", "Q668", "Q987", "Q668 Q987"),
        ("fact_capital_q30", "Q30", "Q61", "Q30 Q61"),
        ("fact_capital_q145", "Q145", "Q84", "Q145 Q84"),
        ("fact_capital_q155", "Q155", "Q2844", "Q155 Q2844"),
        (
            "fact_author_of_book_q15228",
            "Q15228",
            "Q892",
            "Q892 Q15228",
        ),
    ];
    assert_eq!(
        records,
        expected
            .iter()
            .map(|(slug, subject, value, wikidata)| {
                (
                    (*slug).to_owned(),
                    (*subject).to_owned(),
                    (*value).to_owned(),
                    (*wikidata).to_owned(),
                )
            })
            .collect::<Vec<_>>()
    );
    let cache =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/cache/wikidata/fact-claims");
    for record in derived() {
        let path = cache.join(format!("{}.json", record.subject_qid));
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        let document: serde_json::Value =
            serde_json::from_str(&text).expect("committed capture is JSON");
        let claims = document["entities"][record.subject_qid.as_str()]["claims"]
            .as_object()
            .expect("a subject capture carries its claim");
        let value = claims
            .values()
            .next()
            .and_then(|entries| entries[0]["mainsnak"]["datavalue"]["value"]["id"].as_str());
        assert_eq!(value, Some(record.value_qid.as_str()), "{}", record.slug);
    }
}

#[test]
fn no_written_record_states_a_fact_the_captures_derive() {
    let written = include_str!("../../../../data/seed/facts.lino");
    assert!(
        !written.contains("subject_qid"),
        "a record with a subject Q-id belongs to the captures"
    );
    assert_eq!(facts().len(), 17);
}

#[test]
fn the_label_index_keeps_inflected_and_recorded_surfaces() {
    let germany = derived()
        .into_iter()
        .find(|record| record.subject_qid == "Q183")
        .expect("Germany is derived");
    for surface in [
        "germany",
        "germany's",
        "германия",
        "германии",
        "германию",
        "德国",
        "德國",
    ] {
        assert!(
            germany.subject_aliases.iter().any(|alias| alias == surface),
            "{surface} resolves Germany: {:?}",
            germany.subject_aliases
        );
    }
    let book = derived()
        .into_iter()
        .find(|record| record.subject_qid == "Q15228")
        .expect("The Lord of the Rings is derived");
    for surface in [
        "lord of the rings",
        "the lord of the rings",
        "властелина колец",
        "властелином колец",
    ] {
        assert!(
            book.subject_aliases.iter().any(|alias| alias == surface),
            "{surface} resolves the book: {:?}",
            book.subject_aliases
        );
    }
}

#[test]
fn the_engine_answers_from_the_derived_records() {
    for (prompt, expected) in [
        (
            "What is the capital of Japan?",
            "The capital of Japan is Tokyo.",
        ),
        (
            "What is the capital of the USA?",
            "The capital of the United States is Washington, D.C.",
        ),
        ("столица германии", "Столица Германии — Берлин."),
    ] {
        let answer = FormalAiEngine.answer(prompt);
        assert_eq!(answer.intent, "fact_lookup", "{prompt}");
        assert_eq!(answer.answer, expected, "{prompt}");
    }
}
