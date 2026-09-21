//! Issue #1138 B12, plan 12 leaves 12-14: refutation-first 2-4-6 hypothesis
//! search (#802).
//!
//! Each try should reduce the number of live possibilities ideally in half or
//! less, and a probe that attempts to *disprove* the leading hypothesis is
//! preferred over one that would only confirm it. Every elimination is backed
//! by an `Evidence` record (plan 00 section 4.3), so the search is replayable
//! and the verdict is honest.
//!
//! Written before the leaves that make them pass (plan 14 wave T).

use formal_ai::execution_evidence::{Evidence, EvidenceDetail, EvidenceSource, ObservationKind};
use formal_ai::selection_heuristics::{
    Experiment, ExperimentChooser, HypothesisSpace, RefutationSearch, SearchHypothesis,
    SearchVerdict,
};

fn hypothesis(id: &str, rule: &str) -> SearchHypothesis {
    SearchHypothesis {
        hypothesis_id: id.to_owned(),
        rule: rule.to_owned(),
        alive: true,
        refuted_by: None,
    }
}

fn experiment(id: &str, probe: &str, yes: &[&str], no: &[&str]) -> Experiment {
    Experiment {
        experiment_id: id.to_owned(),
        probe: probe.to_owned(),
        predicts_yes: yes.iter().map(|value| (*value).to_owned()).collect(),
        predicts_no: no.iter().map(|value| (*value).to_owned()).collect(),
    }
}

/// The 2-4-6 space: four rules survive the opening triple, and the probes
/// discriminate them by different amounts.
fn space() -> HypothesisSpace {
    HypothesisSpace {
        space_id: "space_246".to_owned(),
        hypotheses: vec![
            hypothesis("h_ascending_by_two", "ascending_by_two"),
            hypothesis("h_ascending", "ascending"),
            hypothesis("h_even", "all_even"),
            hypothesis("h_any", "any_three_numbers"),
        ],
        experiments: vec![
            // Splits 2 / 2: the halving probe.
            experiment(
                "e_1_3_5",
                "1 3 5",
                &["h_ascending", "h_any"],
                &["h_ascending_by_two", "h_even"],
            ),
            // Splits 3 / 1: weaker.
            experiment(
                "e_2_4_6",
                "2 4 6",
                &["h_ascending_by_two", "h_ascending", "h_even"],
                &["h_any"],
            ),
            // Also 2 / 2, and it attempts to refute the leading hypothesis.
            experiment(
                "e_6_4_2",
                "6 4 2",
                &["h_even", "h_any"],
                &["h_ascending_by_two", "h_ascending"],
            ),
        ],
        observations: Vec::new(),
    }
}

fn observation(command: &str, exit: Option<i64>) -> Evidence {
    Evidence {
        evidence_id: format!("evidence_{command}"),
        for_need: "need_246".to_owned(),
        produced_by: "refutation_search".to_owned(),
        command: command.to_owned(),
        argv: command.split(' ').map(str::to_owned).collect(),
        exit_code: exit,
        observed_output_sha256: String::new(),
        observed_byte_length: 0,
        source_ids: Vec::new(),
        kind: ObservationKind::SymbolicCheck,
        source: EvidenceSource::Engine,
        detail: EvidenceDetail::None,
        recorded_at: None,
    }
}

#[test]
fn the_chosen_experiment_minimizes_worst_case_survivors() {
    let space = space();
    let chosen = RefutationSearch
        .next_experiment(&space)
        .expect("a discriminating probe exists");
    assert_eq!(
        chosen.worst_case_survivors(),
        2,
        "the halving criterion (#802): the chosen probe leaves two of four alive in the \
         worse outcome, not three"
    );
    assert_ne!(
        chosen.experiment_id, "e_2_4_6",
        "the 3/1 probe is the confirming one and must not be chosen while a 2/2 probe \
         exists"
    );
}

#[test]
fn a_refuting_probe_is_preferred_over_a_confirming_one_at_equal_power() {
    let space = space();
    let leading = "h_ascending_by_two";
    let chosen = RefutationSearch
        .next_experiment(&space)
        .expect("a discriminating probe exists");
    assert!(
        chosen.attempts_refutation_of(leading),
        "#802: first try to disprove the hypothesis, and only if that is not possible \
         find a proof of it. Chosen probe was `{}`",
        chosen.experiment_id
    );
}

#[test]
fn observing_a_result_kills_every_contradicted_hypothesis_and_names_the_experiment() {
    let mut space = space();
    // The probe `1 3 5` succeeds: every hypothesis that predicted failure dies.
    let killed = space.observe("e_1_3_5", observation("probe 1 3 5", Some(0)));
    assert_eq!(killed, 2, "two hypotheses predicted this probe would fail");
    for dead in ["h_ascending_by_two", "h_even"] {
        let record = space
            .hypotheses
            .iter()
            .find(|hypothesis| hypothesis.hypothesis_id == dead)
            .expect("the hypothesis is still in the space, marked dead");
        assert!(!record.alive, "`{dead}` must be refuted");
        assert_eq!(
            record.refuted_by.as_deref(),
            Some("e_1_3_5"),
            "`{dead}` must name the experiment that killed it"
        );
    }
    assert_eq!(space.alive().len(), 2);
}

#[test]
fn two_survivors_with_no_discriminating_probe_report_not_confirmed_not_refuted() {
    let mut space = HypothesisSpace {
        space_id: "space_tied".to_owned(),
        hypotheses: vec![hypothesis("h_a", "rule_a"), hypothesis("h_b", "rule_b")],
        experiments: vec![experiment("e_both", "probe", &["h_a", "h_b"], &[])],
        observations: Vec::new(),
    };
    space.observe("e_both", observation("probe", Some(0)));
    match space.verdict() {
        SearchVerdict::NotConfirmedNotRefuted { survivors, blocker } => {
            assert_eq!(survivors, vec!["h_a".to_owned(), "h_b".to_owned()]);
            assert!(
                !blocker.trim().is_empty(),
                "the verdict must name what stopped the search, matching the default \
                 data/meta/recursive-core-recipe.lino:105 already requires"
            );
        }
        other => panic!("two survivors and no discriminating probe left, got {other:?}"),
    }
}

#[test]
fn every_elimination_is_backed_by_an_evidence_record() {
    let mut space = space();
    space.observe("e_1_3_5", observation("probe 1 3 5", Some(0)));
    assert_eq!(
        space.observations.len(),
        1,
        "the observation that eliminated two hypotheses must be retained, so the \
         elimination is evidence rather than assertion (plan 00 section 4.3)"
    );
    let dead = space
        .hypotheses
        .iter()
        .filter(|hypothesis| !hypothesis.alive)
        .count();
    assert!(
        dead > 0 && !space.observations.is_empty(),
        "a refuted hypothesis with no observation behind it is an assertion"
    );
}

#[test]
fn the_same_space_and_seed_produce_the_same_experiment_sequence() {
    let first: Vec<String> = {
        let mut space = space();
        let mut order = Vec::new();
        while let Some(next) = RefutationSearch.next_experiment(&space) {
            let id = next.experiment_id.clone();
            order.push(id.clone());
            let killed = space.observe(&id, observation(&id, Some(0)));
            if killed == 0 {
                break;
            }
        }
        order
    };
    let second: Vec<String> = {
        let mut space = space();
        let mut order = Vec::new();
        while let Some(next) = RefutationSearch.next_experiment(&space) {
            let id = next.experiment_id.clone();
            order.push(id.clone());
            let killed = space.observe(&id, observation(&id, Some(0)));
            if killed == 0 {
                break;
            }
        }
        order
    };
    assert_eq!(
        first, second,
        "the search must be deterministic: the same space produces the same probe \
         sequence, so the elimination replays"
    );
    assert!(!first.is_empty(), "the search must run at least one probe");
}
