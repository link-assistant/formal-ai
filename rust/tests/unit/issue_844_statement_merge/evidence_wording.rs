use super::{
    Language, SourceTier, SourcedStatement, deduplicate, many_sources, merge_into_context, rank,
    to_statements_in,
};

pub(super) fn the_evidence_wording_comes_from_the_seed_for_every_supported_language() {
    let mut sources = many_sources(8, "The parser is fast.");
    sources.push(SourcedStatement::from_sentence(
        "The parser is not fast.",
        "denier",
        SourceTier::OriginalJournalism,
    ));
    let report = deduplicate(&sources);
    let total = report.sources.len();
    let ranked = rank(&report);
    let contested = ranked
        .iter()
        .find(|item| item.is_contested())
        .expect("the thread disagrees with itself");

    // English is the default rendering, and it is seed data like the rest.
    assert_eq!(
        contested.evidence_summary_in(total, Language::English),
        "asserted by 8 of 9 sources, denied by 1"
    );
    assert_eq!(
        contested.evidence_summary(total),
        contested.evidence_summary_in(total, Language::English)
    );

    // Every other supported language has its own wording, still carrying both
    // counts — Russian, Hindi and Chinese, not English echoed back.
    for (language, marker) in [
        (Language::Russian, "источник"),
        (Language::Hindi, "स्रोत"),
        (Language::Chinese, "来源"),
    ] {
        let rendered = contested.evidence_summary_in(total, language);
        assert!(
            rendered.contains(marker) && rendered.contains('8') && rendered.contains('9'),
            "{language:?}: {rendered}"
        );
        assert_ne!(rendered, contested.evidence_summary(total), "{language:?}");
    }

    // The disputed-statement wrapper is seed data too, in the same languages.
    for (language, marker) in [
        (Language::English, "disputed"),
        (Language::Russian, "спорно"),
        (Language::Hindi, "विवादित"),
        (Language::Chinese, "有争议"),
    ] {
        let statements = to_statements_in(&ranked, total, language);
        assert!(
            statements.iter().any(|item| item.text.contains(marker)),
            "{language:?} disputed wrapper missing"
        );
    }

    // The gathering and recheck traces are machine slugs, not prose: `name=value`
    // fields joined by spaces, so they need no translation at all.
    let payload = merge_into_context("issue-844-trace", &sources)
        .recheck()
        .trace();
    for field in payload.split_whitespace() {
        assert!(
            field.split_once('=').is_some(),
            "trace fields must stay machine-readable: {field}"
        );
    }
    assert!(payload.contains("verdict=") && payload.contains("sources="));
}
