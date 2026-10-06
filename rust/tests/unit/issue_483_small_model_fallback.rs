//! Issue #483: small-model formalization fallback behind an
//! experimental flag.
//!
//! The contract, per the issue: off by default and not loaded unless
//! the user enables it in settings; only models that fit the user's
//! hardware displayed; sorted by public ratings; downloaded on demand
//! only — nothing in the application package or web UI; the model picks
//! the best match from offered options; confirmed by unit tests; and
//! LLMs never at the steering wheel (formal first). These integration
//! tests pin rust/src/small_model_fallback.rs end to end (the module
//! also carries in-crate unit tests).

use formal_ai::event_log::EventLog;
use formal_ai::small_model_fallback::{
    FormalizationCandidate, SmallModelOptions, catalog, confirm_proposal, download_manifest,
    eligible, propose_best_match, recommend,
};

fn grounded_options() -> Vec<FormalizationCandidate> {
    vec![
        FormalizationCandidate {
            qid: "Q828224".into(),
            label: "kilometre".into(),
            description: "unit of length, 1000 metres".into(),
        },
        FormalizationCandidate {
            qid: "Q11573".into(),
            label: "metre".into(),
            description: "SI unit of length".into(),
        },
        FormalizationCandidate {
            qid: "Q218593".into(),
            label: "inch".into(),
            description: "unit of length".into(),
        },
    ]
}

#[test]
fn off_by_default_and_nothing_loads() {
    let options = SmallModelOptions::default();
    assert!(!options.enabled, "the experimental flag defaults to off");
    assert!(eligible(&options).is_empty());
    assert!(recommend(&options).is_none());
    let mut log = EventLog::new();
    assert!(
        propose_best_match(&options, "metre", &grounded_options(), &mut log).is_none(),
        "a disabled feature proposes nothing, not even a default model"
    );
}

#[test]
fn only_hardware_fitting_models_are_shown_sorted_by_public_rating() {
    let options = SmallModelOptions::enabled_for(1024);
    let shown = eligible(&options);
    assert!(!shown.is_empty());
    assert!(
        shown.iter().all(|model| model.ram_required_mb <= 1024),
        "nothing above the machine's RAM is offered"
    );
    assert!(
        !shown.iter().any(|model| model.model_id == "phi_3_5_mini"),
        "a 6144 MB model must not appear on a 1024 MB machine"
    );
    let ratings: Vec<u8> = shown.iter().map(|model| model.rating).collect();
    let mut sorted = ratings.clone();
    sorted.sort_unstable_by(|a, b| b.cmp(a));
    assert_eq!(ratings, sorted, "best public rating first");
    // The best-rated fitting model is the recommendation.
    assert_eq!(
        recommend(&options).map(|model| model.rating),
        Some(ratings[0])
    );
}

#[test]
fn nothing_is_packaged_or_downloaded_in_advance() {
    for model in catalog() {
        assert!(
            !model.packaged,
            "{} must not ship in the package or web UI",
            model.model_id
        );
        let manifest = download_manifest(&model);
        assert_eq!(manifest.model_id, model.model_id);
        assert!(
            manifest.disk_required_mb > 0,
            "an on-demand download has a stated size"
        );
    }
}

#[test]
fn the_model_selects_the_best_match_from_offered_options() {
    let options = SmallModelOptions::enabled_for(1024);
    let mut log = EventLog::new();
    let proposal = propose_best_match(&options, "metre", &grounded_options(), &mut log)
        .expect("an enabled, fitting model picks from the options");
    assert_eq!(
        proposal.candidate_index, 1,
        "the metre (Q11573) option wins for 'metre'"
    );
    assert!(proposal.confidence > 0.0);
    assert!(log.first_of("small_model:proposal").is_some());
    assert!(
        log.first_of("small_model:candidate")
            .map(|e| e.payload.as_str())
            == Some("Q11573"),
        "the chosen Q-id is logged"
    );
}

#[test]
fn the_proposal_is_advisory_and_the_formal_rules_hold_the_pen() {
    let options = SmallModelOptions::enabled_for(1024);
    let mut log = EventLog::new();
    let proposal = propose_best_match(&options, "metre", &grounded_options(), &mut log)
        .expect("proposal exists");
    assert!(
        proposal.is_advisory(),
        "LLMs are never at the steering wheel"
    );
    assert!(
        !confirm_proposal(&proposal, false),
        "rules disagree: the model cannot commit"
    );
    assert!(
        confirm_proposal(&proposal, true),
        "rules agree: the match is confirmed"
    );
}

#[test]
fn no_overlap_means_silence_not_a_guess() {
    let options = SmallModelOptions::enabled_for(1024);
    let mut log = EventLog::new();
    assert!(
        propose_best_match(&options, "xyzzy plugh", &grounded_options(), &mut log).is_none(),
        "the fallback stays silent when it cannot help"
    );
}
