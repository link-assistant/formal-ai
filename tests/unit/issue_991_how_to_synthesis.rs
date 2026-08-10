//! Regression coverage for issue #991: dynamic multi-source "how to X"
//! synthesis and seven-day service-accessibility caching.
//!
//! Every test here replays the committed real-service captures under
//! `tests/fixtures/issue-991/` through the *production* path
//! ([`formal_ai::try_how_to_procedure_with_client`] and
//! [`formal_ai::how_to_guide::synthesize_how_to_guide`]) with the client held
//! offline. Nothing is stubbed: the bytes came from wikiHow, Stack Exchange, and
//! the Wikimedia wikis through the same code the live run uses, and their
//! digests are recorded in `capture-manifest.lino`.
//!
//! The refresh check against the real services is gated behind
//! `FORMAL_AI_LIVE_FETCH=1` so the normal suite stays offline and deterministic
//! while drift against the live services is still detectable on demand.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::event_log::EventLog;
use formal_ai::how_to_capture_manifest::{
    drift, parse_manifest, read_captures, verify_bodies, CAPTURE_MANIFEST_FILE,
};
use formal_ai::how_to_guide::{
    select_sources, synthesize_how_to_guide, GuideBounds, HowToGuide, ServicePreferences,
    MIN_ACCEPTED_STEPS,
};
use formal_ai::service_accessibility::{ServiceAccessibilityCache, ServiceStatus};
use formal_ai::source_fetch::{CachedSourceClient, CurlSourceTransport};

/// The committed capture tree, relative to the crate root.
const FIXTURE_DIR: &str = "tests/fixtures/issue-991";

/// The cross-runtime parity expectation written by
/// `examples/issue_991_how_to_parity.rs`.
const PARITY_FILE: &str = "expected-guides.json";

/// The QA task documented by the primary procedural source.
const DOCUMENTED_TASK: &str = "make pancakes";
/// The QA task only the corroborating services answer.
const TECHNICAL_TASK: &str = "reverse a string in python";
/// The QA task no service documents at all.
const UNDOCUMENTED_TASK: &str = "build a nonexistent quantum flux capacitor";

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_DIR)
}

/// Whether the gated refresh check may reach the real services.
fn live_fetch_requested() -> bool {
    matches!(
        std::env::var("FORMAL_AI_LIVE_FETCH")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

/// Synthesise a guide from the committed captures, with the client offline.
fn offline_guide(task: &str, preferences: &ServicePreferences) -> HowToGuide {
    let fixture = fixture_dir();
    let client = CachedSourceClient::new(&fixture, CurlSourceTransport).with_online(false);
    let mut availability = ServiceAccessibilityCache::new(
        std::env::temp_dir().join(format!("formal-ai-issue-991-{}", task.replace(' ', "-"))),
    );
    synthesize_how_to_guide(
        task,
        &client,
        preferences,
        &GuideBounds::default(),
        &mut availability,
        u64::MAX / 2,
    )
}

#[test]
fn enabled_registry_services_contribute_and_opt_outs_stay_authoritative() {
    let preferences = ServicePreferences::default();
    let selected: Vec<String> =
        select_sources(DOCUMENTED_TASK, &preferences, &GuideBounds::default())
            .into_iter()
            .map(|record| record.id)
            .collect();
    assert!(
        selected.contains(&String::from("wikihow")),
        "the seed registry's primary procedural service must be consulted: {selected:?}"
    );
    assert!(
        selected.len() > 1,
        "more than one registry service must be able to contribute: {selected:?}"
    );

    // The settings opt-out is authoritative: a disabled service is not consulted
    // at all, and the guide says so instead of silently dropping it.
    let without = ServicePreferences::default().with("externalServiceWikihow", false);
    let disabled: Vec<String> = select_sources(DOCUMENTED_TASK, &without, &GuideBounds::default())
        .into_iter()
        .map(|record| record.id)
        .collect();
    assert!(
        !disabled.contains(&String::from("wikihow")),
        "a disabled service must never be consulted: {disabled:?}"
    );

    let guide = offline_guide(DOCUMENTED_TASK, &without);
    let outcome = guide
        .outcomes
        .iter()
        .find(|outcome| outcome.source_id == "wikihow")
        .expect("the disabled service is still reported");
    assert_eq!(outcome.status, "disabled");
    assert_eq!(outcome.detail, "externalServiceWikihow");
    assert!(
        guide.steps.iter().all(|step| step.source_id != "wikihow"),
        "a disabled service must contribute no steps"
    );
}

#[test]
fn every_accepted_step_carries_exact_provenance_within_declared_bounds() {
    let guide = offline_guide(DOCUMENTED_TASK, &ServicePreferences::default());
    assert!(
        guide.is_sufficient(),
        "the committed captures document this task: {}",
        guide.trace()
    );

    let captures = read_captures(fixture_dir()).expect("read committed captures");
    let digests: BTreeMap<&str, &str> = captures
        .iter()
        .map(|entry| (entry.url.as_str(), entry.sha256.as_str()))
        .collect();
    for step in &guide.steps {
        // Provenance is exact, not approximate: the URL is one that was really
        // captured and the digest is the digest of those very bytes.
        assert_eq!(
            digests.get(step.source_url.as_str()).copied(),
            Some(step.sha256.as_str()),
            "step provenance must point at a committed capture: {}",
            step.provenance()
        );
        assert!(
            !step.fetched_at.is_empty(),
            "a step records when it was fetched"
        );
        assert!(
            !step.license_name.is_empty() && step.license_url.starts_with("http"),
            "a step records the license it is quoted under: {}",
            step.provenance()
        );
        assert!(step.depth <= guide.bounds.max_depth, "depth bound holds");
    }
    assert!(
        guide.steps.len() <= guide.bounds.max_steps,
        "step bound holds"
    );
    for outcome in &guide.outcomes {
        assert!(
            outcome.pages <= guide.bounds.max_pages_per_service,
            "page bound holds for {}: {}",
            outcome.source_id,
            outcome.trace_payload()
        );
    }
    assert!(
        guide
            .outcomes
            .iter()
            .filter(|outcome| outcome.pages > 0)
            .count()
            <= guide.bounds.max_services,
        "service bound holds"
    );
}

#[test]
fn search_results_are_captured_recursively_within_the_depth_bound() {
    // The technical task is not a wikiHow page: the procedure is reached by
    // following a search result into a question's answers, which is the
    // recursion the issue asks for. Depth 1 steps are the proof it happened.
    let guide = offline_guide(TECHNICAL_TASK, &ServicePreferences::default());
    assert!(
        guide.is_sufficient(),
        "the corroborating services answer this task: {}",
        guide.trace()
    );
    assert!(
        guide.steps.iter().any(|step| step.depth > 0),
        "at least one step must come from a recursively captured page: {}",
        guide.trace()
    );
    assert!(
        guide
            .steps
            .iter()
            .all(|step| step.depth <= guide.bounds.max_depth),
        "the recursion stays inside the declared depth bound"
    );

    // Relevance is judged before a search result is followed, so an unrelated
    // hit is reported rather than mined for steps.
    let empty = offline_guide(UNDOCUMENTED_TASK, &ServicePreferences::default());
    assert!(
        empty
            .outcomes
            .iter()
            .any(|outcome| outcome.detail.contains("no_relevant_result")),
        "irrelevant search results must be reported: {}",
        empty.trace()
    );
}

#[test]
fn insufficient_evidence_is_reported_rather_than_invented() {
    let guide = offline_guide(UNDOCUMENTED_TASK, &ServicePreferences::default());
    assert!(
        guide.steps.len() < MIN_ACCEPTED_STEPS,
        "no service documents this task: {}",
        guide.trace()
    );
    assert!(!guide.is_sufficient());
    assert!(
        guide.trace().contains("how_to:insufficient_evidence"),
        "the refusal is explicit: {}",
        guide.trace()
    );
    assert!(
        guide.markdown().contains("Insufficient evidence"),
        "the reader is told, not given invented steps"
    );
    // The refusal still accounts for every service that was considered.
    assert!(
        !guide.outcomes.is_empty(),
        "the services consulted are reported even when nothing was found"
    );

    // And the handler keeps answering: the pre-existing discovery plan is a
    // strict fallback, so the runtime never loses an answer it used to give.
    let mut log = EventLog::new();
    let client = CachedSourceClient::new(fixture_dir(), CurlSourceTransport).with_online(false);
    let mut availability =
        ServiceAccessibilityCache::new(std::env::temp_dir().join("formal-ai-issue-991-fallback"));
    let prompt = "how to build a nonexistent quantum flux capacitor?";
    let answer = formal_ai::try_how_to_procedure_with_client(
        prompt,
        prompt,
        &mut log,
        &client,
        &ServicePreferences::default(),
        &mut availability,
    );
    assert!(answer.is_some(), "the discovery plan still answers");
}

#[test]
fn per_service_accessibility_is_remembered_for_at_least_seven_days() {
    let directory = std::env::temp_dir().join("formal-ai-issue-991-accessibility");
    let _ = fs::remove_dir_all(&directory);
    let mut cache = ServiceAccessibilityCache::new(&directory);
    let seven_days = 7 * 24 * 60 * 60;
    let record = cache.observe("wikihow", ServiceStatus::Unreachable, "http_503", 1_000);
    assert!(
        record.ttl_seconds >= seven_days,
        "the accessibility TTL must be at least seven days, got {}",
        record.ttl_seconds
    );

    // Inside the TTL the recorded failure is authoritative and the service is
    // skipped rather than re-probed on every request.
    assert!(cache.known_unreachable("wikihow", 1_000 + 6 * 86_400));
    assert!(!cache.needs_refresh("wikihow", 1_000 + 6 * 86_400));
    // Past the TTL it must be refreshed instead of trusted forever.
    assert!(cache.needs_refresh("wikihow", 1_000 + 8 * 86_400));
    assert!(!cache.known_unreachable("wikihow", 1_000 + 8 * 86_400));
    // Explicit invalidation forgets it immediately.
    assert!(cache.invalidate("wikihow").is_some());
    assert!(cache.needs_refresh("wikihow", 1_000));
    cache.observe("wikihow", ServiceStatus::Reachable, "captured", 1_000);
    assert_eq!(cache.invalidate_all(), 1);

    // The record survives a round trip through its committed Links Notation and
    // is projected into environment associative memory.
    cache.observe("stackexchange", ServiceStatus::Reachable, "captured", 2_000);
    cache.save().expect("persist the accessibility record");
    let reloaded = ServiceAccessibilityCache::load(&directory);
    let stored = reloaded
        .record("stackexchange")
        .expect("the record survives the round trip");
    assert!(stored.status.is_reachable());
    assert!(stored.ttl_seconds >= seven_days);
    let memory = reloaded.associative_memory();
    assert!(
        !memory.is_empty(),
        "accessibility must be visible in environment associative memory"
    );
    let _ = fs::remove_dir_all(&directory);
}

#[test]
fn the_committed_captures_replay_offline_and_match_their_digests() {
    let fixture = fixture_dir();
    let recorded = parse_manifest(
        &fs::read_to_string(fixture.join(CAPTURE_MANIFEST_FILE)).expect("committed manifest"),
    );
    assert!(!recorded.is_empty(), "the QA captures are committed");
    for entry in &recorded {
        assert_eq!(entry.sha256.len(), 64, "every capture records its digest");
        assert!(
            !entry.fetched_at.is_empty(),
            "every capture records its timestamp"
        );
        assert!(
            !entry.license_name.is_empty(),
            "every capture records the license it is quoted under: {}",
            entry.trace_payload()
        );
    }

    // The bytes on disk still hash to what the manifest claims, and the manifest
    // still describes exactly the captures that are committed.
    let current = read_captures(&fixture).expect("read the committed captures");
    let invalid = verify_bodies(&fixture, &current).expect("verify capture bodies");
    assert!(
        invalid.is_empty(),
        "capture bodies must match their digests: {invalid:?}"
    );
    let differences = drift(&recorded, &current);
    assert!(
        differences.is_empty(),
        "the committed manifest must describe the committed captures: {:?}",
        differences
            .iter()
            .map(formal_ai::how_to_capture_manifest::CaptureDrift::trace_payload)
            .collect::<Vec<_>>()
    );
}

#[test]
fn the_native_and_browser_runtimes_synthesise_the_same_guide() {
    // `examples/issue_991_how_to_parity.rs` writes this expectation from the
    // production Rust path; `tests/web/issue-991-how-to-synthesis.test.mjs`
    // asserts the browser worker reproduces it over the same capture bytes.
    // Asserting it here too makes the file a regression on both runtimes rather
    // than a snapshot only one of them can move.
    let expected = fs::read_to_string(fixture_dir().join(PARITY_FILE)).expect("parity expectation");
    for task in [DOCUMENTED_TASK, TECHNICAL_TASK, UNDOCUMENTED_TASK] {
        let guide = offline_guide(task, &ServicePreferences::default());
        assert!(
            expected.contains(&format!("\"{task}\"")),
            "the parity expectation must cover {task}"
        );
        for step in &guide.steps {
            let quoted = step
                .text
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n");
            assert!(
                expected.contains(&format!("\"text\": \"{quoted}\"")),
                "the Rust guide for {task} drifted from the shared parity expectation; \
                 re-run `cargo run --example issue_991_how_to_parity` and review the diff: {}",
                step.provenance()
            );
        }
    }
}

/// Every phrase the reader sees is seed data, not a literal in the renderer
/// (R379, `docs/design/no-hardcoded-natural-language.md`). Deleting or
/// mistyping one of the `how_to_guide_*` intents would silently render an empty
/// heading, so the seed lookup is asserted directly, in every seeded language.
#[test]
fn the_reader_facing_guide_is_rendered_from_seeded_prose() {
    const CHROME_INTENTS: [&str; 11] = [
        "how_to_guide_heading",
        "how_to_guide_step",
        "how_to_guide_insufficient_evidence",
        "how_to_guide_sources_heading",
        "how_to_guide_source_outcome",
        "how_to_guide_citation",
        "how_to_guide_conflicts_heading",
        "how_to_guide_conflict",
        "how_to_guide_copies_heading",
        "how_to_guide_copy",
        "how_to_guide_bounds",
    ];
    for intent in CHROME_INTENTS {
        for language in ["en", "ru", "hi", "zh"] {
            let text = formal_ai::seed::response_for(intent, language)
                .unwrap_or_else(|| panic!("{intent} must be seeded for {language}"));
            assert!(
                !text.trim().is_empty(),
                "{intent} must carry text for {language}"
            );
        }
    }

    let guide = offline_guide(DOCUMENTED_TASK, &ServicePreferences::default());
    let english = guide.markdown();
    assert_eq!(english, guide.markdown_in("en"));
    let russian = guide.markdown_in("ru");
    assert_ne!(
        russian, english,
        "another seeded language must change the chrome"
    );
    // The evidence itself is language-neutral: the same steps, digests, and
    // bounds are reported either way.
    for step in &guide.steps {
        let digest = &step.sha256[..12];
        assert!(english.contains(digest) && russian.contains(digest));
    }
    assert!(
        russian.contains(&guide.bounds.trace_payload()),
        "the declared bounds are reported in every language"
    );
}

#[test]
fn the_gated_refresh_check_detects_drift_against_the_real_services() {
    let fixture = fixture_dir();
    let recorded = parse_manifest(
        &fs::read_to_string(fixture.join(CAPTURE_MANIFEST_FILE)).expect("committed manifest"),
    );
    if !live_fetch_requested() {
        // Offline, the check still has to prove it *can* see drift: a mutated
        // manifest must be reported, otherwise a silent no-op would pass as a
        // clean refresh in CI.
        let mut mutated = recorded;
        mutated[0].sha256 = String::from("0").repeat(64);
        let current = read_captures(&fixture).expect("read the committed captures");
        let differences = drift(&mutated, &current);
        assert!(
            !differences.is_empty(),
            "the drift check must report a changed digest"
        );
        return;
    }

    // With FORMAL_AI_LIVE_FETCH=1 the same production path refreshes the
    // captures from the real services and any difference is reported.
    let client = CachedSourceClient::new(&fixture, CurlSourceTransport).with_online(true);
    let mut availability = ServiceAccessibilityCache::load(&fixture);
    for task in [DOCUMENTED_TASK, TECHNICAL_TASK, UNDOCUMENTED_TASK] {
        synthesize_how_to_guide(
            task,
            &client,
            &ServicePreferences::default(),
            &GuideBounds::default(),
            &mut availability,
            formal_ai::service_accessibility::unix_now(),
        );
    }
    let current = read_captures(&fixture).expect("read the refreshed captures");
    let invalid = verify_bodies(&fixture, &current).expect("verify refreshed bodies");
    assert!(
        invalid.is_empty(),
        "refreshed bodies must match their digests: {invalid:?}"
    );
    let differences = drift(&recorded, &current);
    assert!(
        differences.is_empty(),
        "the real services drifted from the committed captures; re-run \
         `FORMAL_AI_LIVE_FETCH=1 cargo run --example issue_991_how_to_capture`: {:?}",
        differences
            .iter()
            .map(formal_ai::how_to_capture_manifest::CaptureDrift::trace_payload)
            .collect::<Vec<_>>()
    );
}
