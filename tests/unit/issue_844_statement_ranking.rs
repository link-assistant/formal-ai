//! Authority-aware ranking regressions for issue #844.

use formal_ai::relative_meta_logic::SourceTier;
use formal_ai::summarization::{SourcedStatement, deduplicate, rank};

/// The same fact, said by `count` distinct independent sources.
fn many_sources(count: usize, text: &str) -> Vec<SourcedStatement> {
    (0..count)
        .map(|index| {
            SourcedStatement::from_sentence(
                text,
                format!("source-{index}"),
                SourceTier::IndependentCorroboration,
            )
        })
        .collect()
}

#[test]
fn ranking_reflects_observed_frequency_and_source_stance() {
    let mut observations = many_sources(8, "The parser is fast.");
    observations.push(SourcedStatement::from_sentence(
        "The manual is long.",
        "lonely",
        SourceTier::IndependentCorroboration,
    ));
    let report = deduplicate(&observations);
    let ranked = rank(&report);

    let widely = ranked
        .iter()
        .find(|item| item.statement.representative.text.contains("parser"))
        .expect("the widely asserted fact is ranked");
    let lonely = ranked
        .iter()
        .find(|item| item.statement.representative.text.contains("manual"))
        .expect("the lone fact is ranked");

    // Same kind, so the same static prior: only observed frequency can separate
    // them, which is the point of the criterion.
    assert_eq!(widely.score.prior, lonely.score.prior);
    assert!(
        widely.score.coverage > lonely.score.coverage,
        "coverage must track distinct asserting sources: {} vs {}",
        widely.score.coverage,
        lonely.score.coverage
    );
    assert!(widely.score.weight > lonely.score.weight);
    assert_eq!(
        ranked[0].statement.representative.text,
        widely.statement.representative.text
    );
    assert_eq!(
        widely.evidence_summary(report.sources.len()),
        "asserted by 8 of 9 sources"
    );

    // Stance: a denial demotes the claim even though its coverage is unchanged.
    let mut contested = many_sources(8, "The parser is fast.");
    contested.push(SourcedStatement::from_sentence(
        "The parser is not fast.",
        "denier",
        SourceTier::OriginalJournalism,
    ));
    let contested_report = deduplicate(&contested);
    let contested_ranked = rank(&contested_report);
    let demoted = contested_ranked
        .iter()
        .find(|item| item.statement.signature.polarity.slug() == "asserted")
        .expect("the asserted side survives");
    assert_eq!(demoted.score.coverage, widely.score.coverage);
    assert!(demoted.score.agreement < 100, "agreement must fall");
    assert!(
        demoted.score.weight < widely.score.weight,
        "a denied fact must rank below the same fact uncontested"
    );
    assert_eq!(
        demoted.evidence_summary(contested_report.sources.len()),
        "asserted by 8 of 9 sources, denied by 1"
    );
    assert!(demoted.is_contested());
}

#[test]
fn an_unoriginal_mirror_adds_no_probability() {
    let original = deduplicate(&[SourcedStatement::from_sentence(
        "The parser is fast.",
        "first-party",
        SourceTier::OriginalFirstParty,
    )]);
    let mut mirrored = vec![SourcedStatement::from_sentence(
        "The parser is fast.",
        "first-party",
        SourceTier::OriginalFirstParty,
    )];
    for index in 0..5 {
        mirrored.push(SourcedStatement::from_sentence(
            "The parser is fast.",
            format!("mirror-{index}"),
            SourceTier::Unoriginal,
        ));
    }
    let mirrored = deduplicate(&mirrored);

    let alone = rank(&original)[0].probability.get();
    let echoed = rank(&mirrored)[0].probability.get();
    assert!(
        (alone - echoed).abs() < f64::EPSILON,
        "five unoriginal mirrors must not move the posterior: {alone} vs {echoed}"
    );
}

#[test]
fn unoriginal_repetition_cannot_outrank_an_authoritative_source() {
    let mut observations = vec![SourcedStatement::from_sentence(
        "The release is signed.",
        "first-party",
        SourceTier::OriginalFirstParty,
    )];
    for index in 0..8 {
        observations.push(SourcedStatement::from_sentence(
            "The rumour is widespread.",
            format!("mirror-{index}"),
            SourceTier::Unoriginal,
        ));
    }

    let ranked = rank(&deduplicate(&observations));
    let authoritative = ranked
        .iter()
        .find(|item| item.statement.representative.text.contains("release"))
        .expect("the authoritative fact is ranked");
    let repeated = ranked
        .iter()
        .find(|item| item.statement.representative.text.contains("rumour"))
        .expect("the repeated mirror is ranked");

    assert_eq!(
        repeated.score.evidence, 0,
        "unoriginal assertions carry no ranking evidence"
    );
    assert_eq!(repeated.score.authority, 0);
    assert_eq!(authoritative.score.authority, 100);
    assert!(
        authoritative.score.weight > repeated.score.weight,
        "one first-party assertion must outrank eight zero-trust mirrors: \
         {:?} vs {:?}",
        authoritative.score,
        repeated.score
    );
    assert_eq!(
        ranked[0].statement.representative.text,
        authoritative.statement.representative.text
    );
}
