//! Acceptance tests for issue #844: statement-level deduplication,
//! evidence-weighted importance, recursive source gathering with recheck, a
//! merged context instead of a list, and the identifier rung of the ladder.
//!
//! One test per acceptance criterion of the issue, in the issue's order.

use formal_ai::relative_meta_logic::{RelativeEvidence, SourceTier, Stance, TruthValue};
use formal_ai::summarization::{
    deduplicate, gather, is_valid_identifier, merge_into_context, rank, to_identifier,
    FetchedSource, GatheringPlan, IdentifierBudget, NamingConvention, SourceCache, SourceProvider,
    SourcedStatement, SummarizationConfig, SummarizationMode,
};
use formal_ai::world_model::{Context, Dependency, Statement as WorldStatement};

/// The same fact, said differently, by `count` distinct sources.
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

/// A provider that serves a fixed set of documents and counts its calls, so a
/// test can prove a warm cache reaches it zero times.
struct RecordedProvider {
    documents: Vec<FetchedSource>,
    calls: Vec<String>,
}

impl RecordedProvider {
    const fn new(documents: Vec<FetchedSource>) -> Self {
        Self {
            documents,
            calls: Vec::new(),
        }
    }
}

impl SourceProvider for RecordedProvider {
    fn fetch(&mut self, url: &str) -> Option<FetchedSource> {
        self.calls.push(url.to_string());
        self.documents
            .iter()
            .find(|document| document.url == url)
            .cloned()
    }
}

/// A provider whose every document links to a fresh one, forever. Only the depth
/// bound can stop a walk over it.
struct EndlessProvider {
    calls: usize,
}

impl SourceProvider for EndlessProvider {
    fn fetch(&mut self, url: &str) -> Option<FetchedSource> {
        self.calls += 1;
        let next = format!("{url}/next");
        Some(
            FetchedSource::new(
                url,
                SourceTier::Unoriginal,
                format!("Page {url} repeats the claim."),
            )
            .linking(next),
        )
    }
}

#[test]
fn n_sources_asserting_one_fact_yield_one_statement_with_a_justification_link() {
    let observations = many_sources(9, "The parser is fast.");
    let report = deduplicate(&observations);

    assert_eq!(
        report.statements.len(),
        1,
        "nine sources asserting one fact must yield one statement"
    );
    let node = &report.statements[0];
    assert_eq!(node.source_count(), 9);
    assert_eq!(node.variants.len(), 9);

    // Eight absorptions, each explainable: the representative it folded into,
    // the sentence, the source, and the signature that justified the merge.
    let links = report.justification(&node.id);
    assert_eq!(links.len(), 8, "one link per absorbed sentence");
    for link in &links {
        assert_eq!(link.representative, node.id);
        assert_eq!(link.absorbed, "The parser is fast.");
        assert_eq!(link.justification, node.signature.key());
        assert!(
            node.sources().contains(&link.source.as_str()),
            "the link's source must be one of the fact's sources"
        );
    }
    // Every source is reachable from the merged fact — the "traceable back to
    // its sources" half of the criterion.
    for index in 0..9 {
        let source = format!("source-{index}");
        assert!(node.sources().contains(&source.as_str()), "{source} lost");
    }
}

#[test]
fn wording_differences_merge_but_extra_content_does_not() {
    let observations = vec![
        SourcedStatement::from_sentence("The parser is fast.", "a", SourceTier::OriginalFirstParty),
        SourcedStatement::from_sentence("Parser is fast", "b", SourceTier::OriginalJournalism),
        SourcedStatement::from_sentence("the fast parser", "c", SourceTier::OriginalJournalism),
        SourcedStatement::from_sentence(
            "The parser is fast enough.",
            "d",
            SourceTier::IndependentCorroboration,
        ),
    ];
    let report = deduplicate(&observations);

    assert_eq!(
        report.statements.len(),
        2,
        "three wordings of one claim merge; the qualified claim stays apart: {:?}",
        report
            .statements
            .iter()
            .map(|node| node.signature.key())
            .collect::<Vec<_>>()
    );
    assert_eq!(report.statements[0].source_count(), 3);
    assert_eq!(report.statements[1].source_count(), 1);
}

/// The merge is deliberately conservative: it compares term sets, and does not
/// stem. "ships" and "ship" are different terms, so the two sentences stay
/// separate rather than being merged on a guess. Pinned so that a later stemming
/// change is a visible decision instead of a silent behaviour drift.
#[test]
fn inflected_wordings_stay_separate_because_the_merge_does_not_stem() {
    let report = deduplicate(&[
        SourcedStatement::from_sentence(
            "The library ships a solver.",
            "a",
            SourceTier::OriginalFirstParty,
        ),
        SourcedStatement::from_sentence(
            "The library does not ship a solver.",
            "b",
            SourceTier::OriginalFirstParty,
        ),
    ]);

    assert_eq!(report.statements.len(), 2);
    assert!(
        report.contradictions.is_empty(),
        "an unstemmed pair is not recognized as a contradiction: {:?}",
        report.contradictions
    );
    // The denial is still recorded as a denial — only the pairing is missed.
    let denied = report
        .statements
        .iter()
        .find(|node| node.signature.polarity.slug() == "denied")
        .expect("the negation cue was read");
    assert_eq!(denied.signature.terms, ["library", "ship", "solver"]);
}

#[test]
fn a_merge_that_conflates_two_facts_can_be_split() {
    // Same term set, different claims: word order carries no meaning for the
    // signature, so these merge — and a reviewer with new evidence must be able
    // to undo exactly that.
    let observations = vec![
        SourcedStatement::from_sentence("Rust calls Python.", "a", SourceTier::OriginalFirstParty),
        SourcedStatement::from_sentence("Python calls Rust.", "b", SourceTier::OriginalFirstParty),
    ];
    let mut report = deduplicate(&observations);
    assert_eq!(report.statements.len(), 1, "the conflating merge happened");
    let merged_id = report.statements[0].id.clone();
    assert_eq!(report.justification(&merged_id).len(), 1);

    assert!(report.split(&merged_id), "the merge must be reversible");
    assert_eq!(report.statements.len(), 2, "one node per absorbed variant");
    assert!(
        report.statement(&merged_id).is_none(),
        "merged node is gone"
    );
    assert!(
        report.justification(&merged_id).is_empty(),
        "the merge links are gone with it"
    );
    let texts: Vec<&str> = report
        .statements
        .iter()
        .map(|node| node.representative.text.as_str())
        .collect();
    assert_eq!(texts, ["Rust calls Python.", "Python calls Rust."]);
    for node in &report.statements {
        assert_eq!(node.variants.len(), 1, "nothing is merged any more");
    }

    // A node that never absorbed anything cannot be split, and an unknown id is
    // rejected rather than silently accepted.
    let untouched = report.statements[0].id.clone();
    assert!(!report.split(&untouched));
    assert!(!report.split("statement_deadbeef"));
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
fn contradictions_become_contradicts_edges_and_are_reported_as_disagreement() {
    let observations = vec![
        SourcedStatement::from_sentence(
            "The release is reproducible.",
            "vendor",
            SourceTier::OriginalFirstParty,
        ),
        SourcedStatement::from_sentence(
            "The release is reproducible.",
            "review",
            SourceTier::OriginalJournalism,
        ),
        SourcedStatement::from_sentence(
            "The release is not reproducible.",
            "auditor",
            SourceTier::OriginalFirstParty,
        ),
    ];
    let merged = merge_into_context("issue-844-contradiction", &observations);

    assert_eq!(
        merged.report.contradictions.len(),
        1,
        "the affirmed/denied twins must be recognized"
    );
    assert_eq!(
        merged.report.contradictions[0].terms,
        ["release", "reproducible"]
    );

    // Both sides survive in the context, each with a probability the JTMS
    // fixpoint settled on: neither is certain, and they cannot both be probable.
    let asserted_id = WorldStatement::new("The release is reproducible.").id;
    let denied_id = WorldStatement::new("The release is not reproducible.").id;
    let truths: Vec<f64> = [&asserted_id, &denied_id]
        .into_iter()
        .map(|id| {
            merged
                .context
                .statement(id)
                .unwrap_or_else(|| panic!("{id} must be in the context"))
                .truth
                .get()
        })
        .collect();
    for truth in &truths {
        assert!(
            *truth > 0.0 && *truth < 1.0,
            "a contradicted statement still carries a probability, not a verdict: {truths:?}"
        );
    }
    assert!(
        truths.iter().filter(|truth| **truth > 0.5).count() <= 1,
        "a claim and its denial must not both come out probable: {truths:?}"
    );
    // Both sides are asserted by an original first-party source, so the evidence
    // gives no reason to prefer either: the fixpoint is maximal uncertainty.
    assert!(
        (truths[0] - truths[1]).abs() < f64::EPSILON && (truths[0] - 0.5).abs() < f64::EPSILON,
        "a first-party claim against a first-party denial settles at 0.5: {truths:?}"
    );

    // The edges are `Contradicts`, and they are mutual.
    let asserted = merged
        .context
        .statement(&asserted_id)
        .expect("asserted side");
    let denied = merged.context.statement(&denied_id).expect("denied side");
    assert!(asserted
        .dependencies
        .iter()
        .any(|edge| edge.stance == Stance::Contradicts && edge.on == denied_id));
    assert!(denied
        .dependencies
        .iter()
        .any(|edge| edge.stance == Stance::Contradicts && edge.on == asserted_id));

    // And the disagreement is reported rather than resolved by dropping a side.
    let disagreements = merged.disagreements();
    assert_eq!(disagreements.len(), 1, "{disagreements:?}");
    assert!(
        disagreements[0].contains("asserted by 2 of 3 sources, denied by 1")
            && disagreements[0].contains("contradicts"),
        "{}",
        disagreements[0]
    );
    let rendered = merged.summary(&SummarizationConfig::default());
    assert!(
        rendered.contains("disputed"),
        "a contested fact must be presented as disputed: {rendered}"
    );
}

/// The mechanism behind the assertion above, in isolation: two statements that
/// contradict each other and whose own evidence saturates their support turn the
/// relaxation into the exact swap `x ← 1 - x`. Bounded passes alone would return
/// whichever half of the oscillation the last pass landed on — including "both
/// sides probable". The cascade collapses the cycle to its mean instead.
#[test]
fn a_saturated_mutual_contradiction_settles_at_maximal_uncertainty() {
    let mut context = Context::new("issue-844-oscillator");
    let claim_id = WorldStatement::new("The release is reproducible.").id;
    let denial_id = WorldStatement::new("The release is not reproducible.").id;
    let first_party = |source: &str| {
        RelativeEvidence::new(
            source,
            SourceTier::OriginalFirstParty,
            Stance::Supports,
            TruthValue::TRUE,
        )
    };

    let report = context.extend_statements([
        WorldStatement::new("The release is reproducible.")
            .with_evidence(first_party("vendor"))
            .with_dependency(Dependency::contradicts(&denial_id)),
        WorldStatement::new("The release is not reproducible.")
            .with_evidence(first_party("auditor"))
            .with_dependency(Dependency::contradicts(&claim_id)),
    ]);

    assert!(
        report.converged,
        "the collapsed mean must be verified as a fixpoint: {report:#?}"
    );
    for id in [&claim_id, &denial_id] {
        let truth = context
            .statement(id)
            .expect("both sides are kept")
            .truth
            .get();
        assert!(
            (truth - 0.5).abs() < f64::EPSILON,
            "{id} settles at 0.5, got {truth}"
        );
    }
}

#[test]
fn recursive_gathering_terminates_by_fixpoint_over_a_citation_cycle() {
    let documents = vec![
        FetchedSource::new(
            "https://a.example/post",
            SourceTier::OriginalFirstParty,
            "The build is reproducible.",
        )
        .linking("https://b.example/post"),
        FetchedSource::new(
            "https://b.example/post",
            SourceTier::OriginalJournalism,
            "The build is reproducible.",
        )
        // Back to A: without a fixpoint this walk never ends.
        .linking("https://a.example/post"),
    ];
    let mut provider = RecordedProvider::new(documents);
    let mut cache = SourceCache::new();
    let plan = GatheringPlan::new("build", 10).seeded_with("https://a.example/post");

    let report = gather(&plan, &mut provider, &mut cache);

    assert!(report.converged, "the cycle must end at a fixpoint");
    assert!(!report.stopped_at_depth_bound, "not at the depth bound");
    assert_eq!(report.fetches.len(), 2, "each URL is fetched at most once");
    assert_eq!(
        provider.calls,
        ["https://a.example/post", "https://b.example/post"]
    );
    assert_eq!(report.depth_reached, 1);
}

#[test]
fn gathering_respects_the_depth_bound_on_an_endless_chain() {
    let mut provider = EndlessProvider { calls: 0 };
    let mut cache = SourceCache::new();
    let plan = GatheringPlan::new("endless", 2).seeded_with("https://endless.example");

    let report = gather(&plan, &mut provider, &mut cache);

    assert!(
        report.stopped_at_depth_bound,
        "an endless chain can only be stopped by the depth bound"
    );
    assert!(!report.converged);
    assert_eq!(report.depth_reached, 2, "seeds are depth 0");
    assert_eq!(report.fetches.len(), 3, "one document per depth 0..=2");
    assert_eq!(provider.calls, 3);

    // `max_depth = 0` fetches the seeds and nothing else.
    let mut shallow = EndlessProvider { calls: 0 };
    let shallow_report = gather(
        &GatheringPlan::new("endless", 0).seeded_with("https://endless.example"),
        &mut shallow,
        &mut SourceCache::new(),
    );
    assert_eq!(shallow_report.fetches.len(), 1);
    assert!(shallow_report.stopped_at_depth_bound);
}

#[test]
fn gathering_stops_once_the_unmet_difference_is_empty() {
    let documents = vec![
        FetchedSource::new(
            "https://docs.example/install",
            SourceTier::OriginalFirstParty,
            "Install it with cargo install formal-ai.",
        )
        .supplying("install")
        .supplying("purpose")
        .linking("https://blog.example/rehash"),
        FetchedSource::new(
            "https://blog.example/rehash",
            SourceTier::Unoriginal,
            "It is installed with cargo.",
        ),
    ];
    let mut provider = RecordedProvider::new(documents);
    let plan = GatheringPlan::new("formal-ai", 5)
        .requiring("install")
        .requiring("purpose")
        .seeded_with("https://docs.example/install");

    let report = gather(&plan, &mut provider, &mut SourceCache::new());

    assert!(report.is_closed(), "{:?}", report.open_attributes);
    assert!(report.converged);
    assert_eq!(
        provider.calls,
        ["https://docs.example/install"],
        "the linked rehash is not needed and is not fetched"
    );
}

#[test]
fn a_warm_cache_replays_the_same_gathering_without_fetching() {
    let documents = vec![
        FetchedSource::new(
            "https://a.example/post",
            SourceTier::OriginalFirstParty,
            "The build is reproducible.",
        )
        .linking("https://mirror.example/post"),
        // Byte-identical body under a different URL: content addressing must
        // store it once.
        FetchedSource::new(
            "https://mirror.example/post",
            SourceTier::Unoriginal,
            "The build is reproducible.",
        ),
    ];
    let plan = GatheringPlan::new("build", 3).seeded_with("https://a.example/post");
    let mut cache = SourceCache::new();

    let mut cold_provider = RecordedProvider::new(documents.clone());
    let cold = gather(&plan, &mut cold_provider, &mut cache);
    assert_eq!(cold_provider.calls.len(), 2);
    assert_eq!(cold.cache_hits, 0);
    assert_eq!(cache.url_count(), 2);
    assert_eq!(
        cache.body_count(),
        1,
        "two URLs serving the same bytes share one stored body"
    );

    let mut warm_provider = RecordedProvider::new(documents);
    let warm = gather(&plan, &mut warm_provider, &mut cache);
    assert!(
        warm_provider.calls.is_empty(),
        "a warm cache must reach the provider zero times: {:?}",
        warm_provider.calls
    );
    assert_eq!(warm.cache_hits, 2);
    assert_eq!(
        warm.trace(),
        cold.trace(),
        "the replay must be byte-identical"
    );
    let cold_texts: Vec<&str> = cold
        .observations
        .iter()
        .map(|item| item.statement.text.as_str())
        .collect();
    let warm_texts: Vec<&str> = warm
        .observations
        .iter()
        .map(|item| item.statement.text.as_str())
        .collect();
    assert_eq!(cold_texts, warm_texts);
}

/// The issue's worked example: a Stack Overflow question, its answers, and the
/// material they link to, gathered recursively, merged, ranked, rechecked,
/// shortened, and traceable back to the sources.
#[test]
fn the_stack_overflow_case_works_end_to_end() {
    let documents = vec![
        FetchedSource::new(
            "https://stackoverflow.com/q/1",
            SourceTier::OriginalJournalism,
            "How do I install formal-ai? The answers below disagree.",
        )
        .supplying("question")
        .linking("https://stackoverflow.com/a/1")
        .linking("https://stackoverflow.com/a/2"),
        FetchedSource::new(
            "https://stackoverflow.com/a/1",
            SourceTier::IndependentCorroboration,
            "Install it with cargo install formal-ai. The crate is published on crates.io.",
        )
        .supplying("install")
        .linking("https://docs.rs/formal-ai"),
        // The same fact in different words: different order, different function
        // words, same terms — so it merges instead of being restated.
        FetchedSource::new(
            "https://stackoverflow.com/a/2",
            SourceTier::IndependentCorroboration,
            "The install is cargo install formal-ai.",
        )
        .supplying("install"),
        FetchedSource::new(
            "https://docs.rs/formal-ai",
            SourceTier::OriginalFirstParty,
            "Install it with cargo install formal-ai. The crate is published on crates.io.",
        )
        .supplying("install"),
    ];
    let mut provider = RecordedProvider::new(documents);
    let plan = GatheringPlan::new("formal-ai installation", 3)
        .requiring("question")
        .requiring("install")
        .requiring("license")
        .seeded_with("https://stackoverflow.com/q/1");

    // 1. Recursive gathering: the question, its answers, and the linked docs.
    let gathered = gather(&plan, &mut provider, &mut SourceCache::new());
    assert_eq!(gathered.fetches.len(), 4, "{:?}", gathered.fetches);
    assert_eq!(
        gathered.depth_reached, 2,
        "docs.rs is two hops from the question"
    );
    assert!(gathered.converged, "the frontier ran dry");
    assert_eq!(
        gathered.open_attributes,
        ["license"],
        "an unanswered attribute is reported, not silently dropped"
    );

    // 2. Deduplicated and merged into a context.
    let merged = merge_into_context("issue-844-stack-overflow", &gathered.observations);
    assert_eq!(merged.total_sources(), 4);
    let install = merged
        .ranked
        .iter()
        .find(|item| {
            item.statement
                .representative
                .text
                .contains("cargo install formal-ai")
        })
        .expect("the install fact is present");
    assert!(
        install.statement.source_count() >= 3,
        "three sources say how to install it: {:?}",
        install.statement.sources()
    );

    // 3. Traceable to sources: every merge is explainable.
    let links = merged.report.justification(&install.statement.id);
    assert!(!links.is_empty(), "the absorptions are recorded");
    for link in &links {
        assert!(
            link.source.starts_with("https://"),
            "each link names its source: {link:?}"
        );
        assert_eq!(link.justification, install.statement.signature.key());
    }

    // 4. Rechecked before presenting: every survivor carries a grounding query.
    let recheck = merged.recheck();
    assert_eq!(recheck.checked.len(), merged.ranked.len());
    assert!(!recheck.survivors().is_empty());
    for item in &recheck.checked {
        assert!(
            item.query().contains("fact check source"),
            "the recheck must reuse the fact-checking path: {}",
            item.query()
        );
    }
    for item in recheck.survivors() {
        assert!(item.verdict.is_presentable());
    }

    // 5. Shortened, through the same ladder as any other summary. The thread is a
    //    question about installing, so the install command is the answer rather
    //    than the boilerplate a project summary would drop.
    let config = SummarizationConfig::default().keeping_boilerplate();
    let checked = merged.checked_summary(&config);
    assert!(
        checked.contains("cargo install formal-ai"),
        "the answer survives the gate: {checked}"
    );
    assert!(
        !checked.contains(" on crates. io"),
        "`crates.io` is one token, not the end of a sentence: {checked}"
    );
    let topic = merged.checked_summary(&config.clone().with_mode(SummarizationMode::Topic));
    assert!(
        topic.split_whitespace().count() <= 5,
        "the topic rung is 1-5 words: {topic:?}"
    );
    let identifier = merged.checked_summary(&config.with_mode(SummarizationMode::Identifier));
    assert!(
        is_valid_identifier(&identifier, NamingConvention::SnakeCase),
        "the identifier rung must be a legal name: {identifier:?}"
    );
}

#[test]
fn a_statement_no_trusted_source_asserts_is_withheld_but_kept() {
    let observations = vec![
        SourcedStatement::from_sentence(
            "The build is reproducible.",
            "vendor",
            SourceTier::OriginalFirstParty,
        ),
        SourcedStatement::from_sentence("A rumour circulates.", "echo", SourceTier::Unoriginal),
    ];
    let merged = merge_into_context("issue-844-recheck", &observations);
    let recheck = merged.recheck();

    let withheld = recheck.withheld();
    assert_eq!(withheld.len(), 1, "{:?}", recheck.trace());
    assert_eq!(withheld[0].text(), "A rumour circulates.");
    assert_eq!(withheld[0].verdict.slug(), "unsupported");
    assert!(!withheld[0].verdict.is_presentable());

    let survivors = recheck.survivors();
    assert_eq!(survivors.len(), 1);
    assert_eq!(survivors[0].text(), "The build is reproducible.");
    assert_eq!(survivors[0].verdict.slug(), "confirmed");

    // Withholding is presentation-only: the statement is still in the context,
    // still carrying its probability and its grounding query.
    assert!(merged.probability_of("A rumour circulates.").is_some());
    assert!(withheld[0].query().contains("fact check source"));
    let presented = merged.checked_summary(&SummarizationConfig::default());
    assert!(!presented.contains("rumour"), "{presented}");
    assert!(presented.contains("reproducible"), "{presented}");
}

#[test]
fn the_identifier_rung_produces_valid_identifiers_under_a_length_budget() {
    let budget = IdentifierBudget::default();
    let phrases = [
        "The parser is fast",
        "A deterministic summarizer merges statements from many sources",
        "3 ways to install it",
        "解析器很快",
        "don't repeat yourself",
    ];
    for phrase in phrases {
        for convention in [
            NamingConvention::SnakeCase,
            NamingConvention::ScreamingSnakeCase,
            NamingConvention::CamelCase,
            NamingConvention::PascalCase,
        ] {
            let identifier = to_identifier(phrase, convention, &budget);
            assert!(
                is_valid_identifier(&identifier, convention),
                "{phrase:?} as {convention:?} gave {identifier:?}"
            );
            assert!(
                identifier.chars().count() <= budget.max_length,
                "{identifier:?} overruns the {} character budget",
                budget.max_length
            );
            assert!(
                identifier
                    .split('_')
                    .filter(|part| !part.is_empty())
                    .count()
                    <= budget.max_words,
                "{identifier:?} overruns the {} word budget",
                budget.max_words
            );
        }
    }

    // A reserved word is escaped rather than emitted.
    assert_eq!(
        to_identifier("the type", NamingConvention::SnakeCase, &budget),
        "type_"
    );
    assert!(!is_valid_identifier("type", NamingConvention::SnakeCase));
    // Function words are dropped, so the head of the phrase survives.
    assert_eq!(
        to_identifier("the type of a match", NamingConvention::SnakeCase, &budget),
        "type_match"
    );
    // A tight budget cuts characters only after it has dropped words.
    let tight = IdentifierBudget::new(12, 4);
    let cut = to_identifier(
        "deterministic summarizer merges statements",
        NamingConvention::SnakeCase,
        &tight,
    );
    assert!(cut.chars().count() <= 12, "{cut:?}");
    assert!(
        is_valid_identifier(&cut, NamingConvention::SnakeCase),
        "{cut:?}"
    );
    // A commit subject is prose under its own budget: capitalized, no period.
    let subject = to_identifier(
        "the summarizer merges statements from many sources into one context",
        NamingConvention::CommitSubject,
        &IdentifierBudget::commit_subject(),
    );
    assert!(
        is_valid_identifier(&subject, NamingConvention::CommitSubject),
        "{subject:?}"
    );
    assert!(subject.chars().count() <= 50, "{subject:?}");
    assert!(
        subject.starts_with(|ch: char| ch.is_uppercase()),
        "{subject:?}"
    );
}

#[test]
fn the_identifier_rung_is_the_bottom_of_the_ladder() {
    assert_eq!(
        SummarizationMode::Topic.one_step_shorter(),
        SummarizationMode::Identifier,
        "the ladder extends downward past Topic"
    );
    assert_eq!(
        SummarizationMode::Identifier.one_step_shorter(),
        SummarizationMode::Identifier,
        "Identifier is the fixed point, so recursion terminates"
    );
    assert!(SummarizationMode::Identifier.is_label_only());
    assert!(SummarizationMode::Topic.is_label_only());
    assert!(!SummarizationMode::Short.is_label_only());
}

#[test]
fn the_merge_is_deterministic_and_independent_of_source_order() {
    let mut observations = many_sources(4, "The parser is fast.");
    observations.push(SourcedStatement::from_sentence(
        "The parser is not fast.",
        "sceptic",
        SourceTier::OriginalJournalism,
    ));
    observations.push(SourcedStatement::from_sentence(
        "The library is written in Rust.",
        "vendor",
        SourceTier::OriginalFirstParty,
    ));

    let forward = merge_into_context("issue-844-determinism", &observations);
    let again = merge_into_context("issue-844-determinism", &observations);
    assert_eq!(
        forward.links_notation(),
        again.links_notation(),
        "the same evidence must produce the same context, byte for byte"
    );
    assert_eq!(forward.recheck().trace(), again.recheck().trace());

    let mut reversed = observations.clone();
    reversed.reverse();
    let backward = merge_into_context("issue-844-determinism", &reversed);
    let order_of = |merged: &formal_ai::summarization::MergedContext| -> Vec<String> {
        merged
            .ranked
            .iter()
            .map(|item| item.statement.signature.key())
            .collect()
    };
    assert_eq!(
        order_of(&forward),
        order_of(&backward),
        "ranking must not depend on the order the sources arrived in"
    );
    for key in order_of(&forward) {
        let left = forward
            .ranked
            .iter()
            .find(|item| item.statement.signature.key() == key)
            .expect("present forward");
        let right = backward
            .ranked
            .iter()
            .find(|item| item.statement.signature.key() == key)
            .expect("present backward");
        assert_eq!(left.score, right.score, "{key}");
        // Exact equality is the requirement, not an approximation: the
        // posteriors are rounded to `TRUTH_VALUE_DECIMALS`, so two runs over the
        // same evidence must agree to the last digit.
        assert_eq!(
            left.probability.get().to_bits(),
            right.probability.get().to_bits(),
            "{key} probability: {} vs {}",
            left.probability.get(),
            right.probability.get(),
        );
    }
}
