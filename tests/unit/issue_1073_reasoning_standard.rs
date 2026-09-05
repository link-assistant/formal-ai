//! Regression coverage for issue #1073's reasoning standard.
//!
//! Issue #1073 pointed at a reference dialog and asked for its depth to become
//! the floor everywhere: evidence before claims, an adversarial self-check before
//! acting, honest reporting of partial failures, and a re-measurement after every
//! change. It then asked for four more things — unconditional depth, formalized
//! internet instructions, documentation as a default step, computed source trust
//! — and, in requirement 6, that all of it be "expressed as formal, checkable
//! procedures rather than stylistic guidance", reachable "with the language model
//! removed".
//!
//! One test per requirement follows, then the whole-task test: the reference
//! dialog itself, plus the mutations that must break it.

use formal_ai::event_log::EventLog;
use formal_ai::intent_formalization::formalize_intent;
use formal_ai::language::{Language, registered_languages};
use formal_ai::reasoning_standard::episode::{ActionOutcome, ReasoningEpisode};
use formal_ai::reasoning_standard::instructions::{
    SourceExcerpt, SourceStep, StepStatus, formalize,
};
use formal_ai::reasoning_standard::refutation::RefutationAxis;
use formal_ai::reasoning_standard::trust::{
    ConflictResolution, DerivationReason, PrimacyChain, PrimacyKind, PrimacyStep, SourceAssessment,
    resolve_conflict,
};
use formal_ai::reasoning_standard::{
    GateStatus, ReasoningStandard, Verdict, audit, open_episode, record_reasoning_standard,
    reference_episode, standard,
};
use formal_ai::relative_meta_logic::SourceTier;
use formal_ai::seed::source_registry;
use formal_ai::translation::formalize_prompt;

fn loaded() -> ReasoningStandard {
    standard().expect("the reasoning standard must load from its data file")
}

fn blockers_of(verdict: &Verdict) -> Vec<String> {
    verdict.blockers().to_vec()
}

/// R1073-1: reference-dialog depth is the floor for *every* request.
///
/// A trivial episode — no observations, no sources, no actions — still gets the
/// complete gate ledger. Depth cannot quietly become conditional on how hard the
/// task looked, because a gate that does not apply must say so by name.
#[test]
fn depth_floor_enumerates_every_gate_even_for_a_trivial_episode() {
    let standard = loaded();
    assert!(
        standard.is_unconditional(),
        "the standard's depth floor must be unconditional"
    );

    let trivial = ReasoningEpisode::default();
    let result = audit(&standard, &trivial);

    assert_eq!(
        result.outcomes.len(),
        standard.gates.len(),
        "every declared gate must appear in the ledger of a trivial episode"
    );
    for (outcome, gate) in result.outcomes.iter().zip(&standard.gates) {
        assert_eq!(outcome.gate, gate.slug, "gates must stay in pipeline order");
        assert_eq!(
            outcome.status,
            GateStatus::NotTriggered,
            "{}: a trivial episode triggers nothing, but the gate must still be reported",
            gate.slug
        );
        assert_eq!(
            outcome.trigger, gate.trigger,
            "{}: a skipped gate must name the trigger that was false",
            gate.slug
        );
    }
    assert_eq!(
        result.verdict.slug(),
        "not_confirmed_not_refuted",
        "an episode that concluded nothing has confirmed nothing"
    );
}

/// Requirement 1, at the seam the pipeline actually uses. The episode above is
/// empty in every field; the one the meta core opens is not — it carries the
/// request's task class — so the floor has to hold for that shape too, and it
/// has to hold for the smallest request there is rather than for a hard one.
#[test]
fn the_depth_floor_holds_for_the_smallest_request_the_pipeline_can_formalize() {
    let standard = loaded();
    let candidate = formalize_prompt("hi", "en");
    let formalization = formalize_intent("hi", "en", Some(&candidate));
    let result = audit(&standard, &open_episode(&formalization));

    assert_eq!(
        result.outcomes.len(),
        standard.gates.len(),
        "a greeting must have the whole checklist enumerated, not a short one"
    );

    // The task class alone is enough to owe instructions, so the standard says
    // what it would have needed instead of passing the request for being small.
    let instructions = result
        .outcome("instruction_formalization")
        .expect("the instruction gate must be reported");
    assert_eq!(instructions.status, GateStatus::Violated);
    assert!(
        instructions
            .findings
            .iter()
            .any(|finding| finding.contains("no_instructions_gathered")),
        "the violation must name what was missing: {:?}",
        instructions.findings
    );

    for (outcome, gate) in result.outcomes.iter().zip(&standard.gates) {
        assert_eq!(outcome.gate, gate.slug, "gates must stay in pipeline order");
        assert_eq!(
            outcome.trigger, gate.trigger,
            "{}: every reported gate must name the trigger that decided it",
            gate.slug
        );
        if outcome.gate == "instruction_formalization" {
            continue;
        }
        assert_eq!(
            outcome.status,
            GateStatus::NotTriggered,
            "{}: nothing else was observed for a greeting",
            gate.slug
        );
    }

    assert_eq!(
        result.verdict.slug(),
        "not_confirmed_not_refuted",
        "a greeting concluded nothing, so nothing may be leaned toward"
    );
    assert!(
        result
            .verdict
            .blockers()
            .iter()
            .any(|blocker| blocker.contains("no_conclusion_recorded")),
        "the verdict must name what blocked the check: {:?}",
        result.verdict.blockers()
    );
}

/// The same requirement across the whole registered language matrix.
///
/// The standard is a predicate over an episode's *shape* — did evidence precede
/// the claim, was the action re-measured, do the refutations differ in mechanism
/// — so the language a request arrives in must not move a gate. Driving the
/// matrix from `registered_languages()` rather than a hand-written list makes a
/// newly registered language a failing test instead of a silent gap.
///
/// Two values are folded away before the comparison, and neither is folded away
/// unchecked. The episode id is the request's impulse hash, different per prompt
/// by construction. The task class is what the router assigned, which is a fact
/// about routing rather than about the standard: `hola` currently routes to
/// `statement` where `hello`, `привет`, `नमस्ते` and `你好` route to `courtesy`,
/// because `data/seed/prompt-patterns.lino` carries greeting keywords for en, ru,
/// hi and zh and none for es. That gap predates this branch and is left to a
/// change of its own — `check_language_change_parity` requires every supported
/// language to move together in one pull request, and four of them need no
/// change. What is asserted here is that the standard reports the same seven
/// gates, the same triggers, the same statuses, the same finding shapes and the
/// same verdict in all five, naming whatever class it was handed.
#[test]
fn the_depth_floor_is_the_same_in_every_registered_language() {
    // One greeting per registered language, each short enough that the
    // formalizer resolves it without a surrounding sentence.
    const GREETINGS: &[(&str, &str)] = &[
        ("en", "Hello"),
        ("ru", "привет"),
        ("hi", "नमस्ते"),
        ("zh", "你好"),
        ("es", "hola"),
    ];

    let mut registered = registered_languages()
        .into_iter()
        .map(Language::slug)
        .collect::<Vec<_>>();
    let mut covered = GREETINGS
        .iter()
        .map(|(language, _)| *language)
        .collect::<Vec<_>>();
    registered.sort_unstable();
    covered.sort_unstable();
    assert_eq!(
        covered, registered,
        "every registered language needs a greeting here"
    );

    let standard = loaded();
    let audit_greeting = |language: &str, prompt: &str| {
        let candidate = formalize_prompt(prompt, language);
        let formalization = formalize_intent(prompt, language, Some(&candidate));
        let episode = open_episode(&formalization);
        let task_class = episode.task_class.clone();
        (audit(&standard, &episode), task_class)
    };

    // Fold the two per-request values out of every reported string, so what is
    // compared is the shape the standard produced and not the identity of the
    // request that produced it.
    let generalize = |text: &str, episode_id: &str, task_class: &str| {
        text.replace(episode_id, "<episode>")
            .replace(task_class, "<task-class>")
    };

    let (english, english_class) = audit_greeting(GREETINGS[0].0, GREETINGS[0].1);
    for (language, prompt) in GREETINGS {
        let (result, task_class) = audit_greeting(language, prompt);

        assert_eq!(
            result.outcomes.len(),
            english.outcomes.len(),
            "{language}: the checklist must be the same length in every language"
        );
        for (outcome, expected) in result.outcomes.iter().zip(&english.outcomes) {
            assert_eq!(outcome.gate, expected.gate, "{language}: gate order");
            assert_eq!(
                outcome.order, expected.order,
                "{language}: gate order index"
            );
            assert_eq!(
                outcome.trigger, expected.trigger,
                "{}: the trigger that decided {} must not depend on the language",
                language, expected.gate
            );
            assert_eq!(
                outcome.status, expected.status,
                "{}: the status of {} must not depend on the language",
                language, expected.gate
            );

            let reported = outcome
                .findings
                .iter()
                .map(|finding| generalize(finding, &result.episode_id, &task_class))
                .collect::<Vec<_>>();
            let baseline = expected
                .findings
                .iter()
                .map(|finding| generalize(finding, &english.episode_id, &english_class))
                .collect::<Vec<_>>();
            assert_eq!(
                reported, baseline,
                "{}: {} must report the same findings in every language",
                language, expected.gate
            );
        }

        // The instruction gate names the class it was handed, so folding the
        // class away above hides nothing: it is checked here against the class
        // the router actually assigned to this prompt.
        let instructions = result
            .outcome("instruction_formalization")
            .expect("the instruction gate must be reported");
        assert!(
            instructions
                .findings
                .iter()
                .any(|finding| finding == &format!("{task_class}:no_instructions_gathered")),
            "{language}: the violation must name this request's own task class: {:?}",
            instructions.findings
        );

        assert_eq!(
            result.verdict.slug(),
            english.verdict.slug(),
            "{language}: the verdict must not depend on the language the request arrived in"
        );
        let reported_blockers = result
            .verdict
            .blockers()
            .iter()
            .map(|blocker| generalize(blocker, &result.episode_id, &task_class))
            .collect::<Vec<_>>();
        let baseline_blockers = english
            .verdict
            .blockers()
            .iter()
            .map(|blocker| generalize(blocker, &english.episode_id, &english_class))
            .collect::<Vec<_>>();
        assert_eq!(
            reported_blockers, baseline_blockers,
            "{language}: the same checks must be reported as blocked, named the same way"
        );
    }
}

/// R1073-2: gathered instructions are *formalized*, not paraphrased.
///
/// Two sources describing the same step corroborate it; a step nobody can check
/// is unverifiable and fails the gate rather than passing as advice.
#[test]
fn gathered_instructions_are_compiled_into_checkable_steps() {
    let standard = loaded();
    let minimum = standard.threshold("minimum_instruction_sources");
    assert!(minimum >= 2, "corroboration needs at least two sources");

    let excerpts = vec![
        SourceExcerpt::new(
            "manual",
            vec![
                SourceStep::new("measure the directory", "du reports a size in bytes"),
                SourceStep::new("remove the directory", "the directory no longer exists"),
            ],
        ),
        SourceExcerpt::new(
            "community_answer",
            vec![
                SourceStep::new("measure the directory", "du reports a size in bytes"),
                SourceStep::new("free the space quickly", ""),
            ],
        ),
    ];
    let set = formalize("reclaim_disk_space", &excerpts, minimum);

    assert_eq!(set.source_count(), 2);
    let orders: Vec<usize> = set.steps.iter().map(|step| step.order).collect();
    assert_eq!(
        orders,
        (1..=set.steps.len()).collect::<Vec<_>>(),
        "a formalized set is ordered contiguously"
    );
    let measure = set
        .steps
        .iter()
        .find(|step| step.action == "measure the directory")
        .expect("the shared step must survive the merge");
    assert_eq!(
        measure.status,
        StepStatus::Corroborated,
        "a step both sources give is corroborated, not merely repeated"
    );
    assert_eq!(measure.sources.len(), 2);

    let unverifiable: Vec<&str> = set
        .unverifiable_steps()
        .iter()
        .map(|step| step.action.as_str())
        .collect();
    assert_eq!(
        unverifiable,
        vec!["free the space quickly"],
        "prose with no check is unverifiable"
    );

    let mut episode = reference_episode();
    episode.excerpts = excerpts;
    let result = audit(&standard, &episode);
    let gate = result
        .outcome("instruction_formalization")
        .expect("the gate must be in the ledger");
    assert_eq!(gate.status, GateStatus::Violated);
    assert!(
        gate.findings
            .iter()
            .any(|finding| finding.contains("free the space quickly")),
        "the gate must name the step that cannot be checked: {:?}",
        gate.findings
    );
}

/// R1073-3: consulting primary documentation is a default step.
///
/// An episode that only ever read second-hand write-ups fails the gate even
/// though its sources are perfectly well-formed.
#[test]
fn primary_documentation_is_required_by_default() {
    let standard = loaded();
    let mut episode = reference_episode();
    assert_eq!(
        audit(&standard, &episode)
            .outcome("documentation_default")
            .map(|outcome| outcome.status),
        Some(GateStatus::Satisfied),
        "the reference episode read the tools' own documentation"
    );

    for source in &mut episode.sources {
        source.chain = PrimacyChain::new(vec![PrimacyStep::new(
            PrimacyKind::Citation,
            "an unnamed upstream write-up",
            "the article cites a write-up it links",
        )]);
        source.asserted_tier = None;
    }
    let result = audit(&standard, &episode);
    let gate = result
        .outcome("documentation_default")
        .expect("the gate must be in the ledger");
    assert_eq!(
        gate.status,
        GateStatus::Violated,
        "second-hand sources alone cannot satisfy the documentation default"
    );
    assert!(
        gate.findings
            .iter()
            .any(|finding| finding.starts_with("primary_sources_consulted:0:")),
        "{:?}",
        gate.findings
    );
}

/// R1073-4: trust is computed from primacy, never assumed.
///
/// A tier stated with no chain behind it is an assumption and fails the gate; a
/// stated tier that disagrees with the derived one fails too; and a conflict
/// between two sources resolves toward the one closer to the primary record.
#[test]
fn source_trust_is_derived_from_the_primacy_chain() {
    let standard = loaded();

    let assumed = SourceAssessment {
        id: "assumed".to_owned(),
        label: "a widely trusted site".to_owned(),
        subject: "disk usage".to_owned(),
        chain: PrimacyChain::new(Vec::new()),
        asserted_tier: Some(SourceTier::OriginalFirstParty),
    };
    assert!(
        !assumed.is_derived(),
        "a tier with no chain behind it is asserted, not derived"
    );
    assert_eq!(
        assumed.derive_trust().reason,
        DerivationReason::NoPrimacyChain
    );
    assert_eq!(assumed.derive_trust().tier, SourceTier::Unoriginal);

    let mut episode = reference_episode();
    episode.sources.push(assumed);
    let result = audit(&standard, &episode);
    let gate = result
        .outcome("computed_source_trust")
        .expect("the gate must be in the ledger");
    assert_eq!(gate.status, GateStatus::Violated);
    assert!(
        gate.findings
            .contains(&"assumed:trust_asserted_without_chain".to_owned()),
        "{:?}",
        gate.findings
    );

    let mut overstated = reference_episode();
    overstated.sources[2].asserted_tier = Some(SourceTier::OriginalFirstParty);
    let overstated_gate = audit(&standard, &overstated)
        .outcome("computed_source_trust")
        .cloned()
        .expect("the gate must be in the ledger");
    assert_eq!(overstated_gate.status, GateStatus::Violated);
    assert!(
        overstated_gate
            .findings
            .iter()
            .any(|finding| finding.contains("asserted_tier_disagrees_with_derived")),
        "{:?}",
        overstated_gate.findings
    );

    let episode = reference_episode();
    let manual = &episode.sources[1];
    let answer = &episode.sources[2];
    match resolve_conflict(manual, answer) {
        ConflictResolution::Prefer { winner, loser, .. } => {
            assert_eq!(winner, manual.id, "the tool's own manual stands closer");
            assert_eq!(loser, answer.id);
        }
        ConflictResolution::Unresolved { .. } => {
            panic!("a manual and a citation of it are not equally primary")
        }
    }
    match resolve_conflict(&episode.sources[0], manual) {
        ConflictResolution::Unresolved { tied, .. } => {
            assert_eq!(tied.len(), 2, "two first-party manuals stand equally close");
        }
        ConflictResolution::Prefer { .. } => {
            panic!("equal distance must leave the conflict open rather than pick a side")
        }
    }

    // The live registry is held to the same rule: its tiers used to be declared
    // and read back verbatim (with a silent `independent_corroboration` default
    // for anything that declared nothing). Every entry now carries the primacy
    // chain its tier is derived from, and the derivation must reproduce what the
    // registry asserts — the value is unchanged, but it is now computed.
    let registry = source_registry();
    assert!(!registry.is_empty(), "the registry must load");
    for record in registry {
        assert!(
            !record.primacy.steps.is_empty(),
            "source `{}` declares no primacy chain, so its trust would be assumed",
            record.id
        );
        for step in &record.primacy.steps {
            assert!(
                step.is_well_founded(),
                "source `{}` has a hop with no basis or no upstream",
                record.id
            );
        }
        assert_eq!(
            record.tier,
            record.primacy.derive_tier(),
            "source `{}` must take the tier its chain derives",
            record.id
        );
        if let Some(asserted) = record.asserted_tier {
            assert_eq!(
                record.tier, asserted,
                "source `{}`: the derived tier must reproduce the one the registry asserts",
                record.id
            );
        }
    }
}

/// R1073-5: refutation-first, with variety, or the honest default.
///
/// A conclusion whose refutations were never attempted is not confirmed; one
/// probed three ways along one axis is still not confirmed; and a refutation that
/// survives on evidence refutes rather than confirms.
#[test]
fn conclusions_need_varied_refutations_before_they_may_be_leaned_toward() {
    let standard = loaded();
    assert_eq!(standard.threshold("minimum_refutation_attempts"), 3);
    assert_eq!(standard.threshold("minimum_refutation_axis_kinds"), 2);

    let mut unprobed = reference_episode();
    unprobed.probes.clear();
    let result = audit(&standard, &unprobed);
    assert_eq!(
        result.verdict.slug(),
        "not_confirmed_not_refuted",
        "a conclusion nobody tried to refute is not confirmed"
    );
    assert!(
        blockers_of(&result.verdict)
            .iter()
            .any(|blocker| blocker.contains("no_refutation_attempted")
                || blocker.contains("refutation_variety")),
        "the blockers must say what stopped the check: {:?}",
        blockers_of(&result.verdict)
    );

    let mut single_axis = reference_episode();
    for probe in &mut single_axis.probes {
        probe.axis = RefutationAxis::Mechanism;
    }
    let single_axis_result = audit(&standard, &single_axis);
    let gate = single_axis_result
        .outcome("refutation_variety")
        .expect("the gate must be in the ledger");
    assert_eq!(
        gate.status,
        GateStatus::Violated,
        "three probes along one axis are one refutation wearing three hats"
    );

    let mut survived = reference_episode();
    survived.probes[0].outcome = formal_ai::reasoning_standard::refutation::ProbeOutcome::Survived;
    assert_eq!(
        audit(&standard, &survived).verdict,
        Verdict::Refuted,
        "a refutation that survives on evidence proves the alternative"
    );

    let mut unchecked = reference_episode();
    unchecked.probes[0].outcome =
        formal_ai::reasoning_standard::refutation::ProbeOutcome::Unchecked;
    unchecked.probes[0].blocker = "the machine was offline, so df could not be re-run".to_owned();
    let unchecked_result = audit(&standard, &unchecked);
    assert_eq!(unchecked_result.verdict.slug(), "not_confirmed_not_refuted");
    assert!(
        blockers_of(&unchecked_result.verdict)
            .iter()
            .any(|blocker| blocker.contains("the machine was offline")),
        "an unchecked refutation must report what blocked it: {:?}",
        blockers_of(&unchecked_result.verdict)
    );
}

/// R1073-6: the standard is data and pure predicates, not prose in a prompt.
///
/// The gates come from a Links Notation file, the episode round-trips through
/// Links Notation, and replaying the record reaches the same verdict — which is
/// what "reachable with the language model removed" has to mean concretely.
#[test]
fn the_standard_is_a_formal_procedure_that_replays_without_a_model() {
    let standard = loaded();
    assert!(!standard.gates.is_empty());
    for gate in &standard.gates {
        assert!(!gate.requirement.trim().is_empty(), "{}", gate.slug);
        assert!(!gate.failure_slug.trim().is_empty(), "{}", gate.slug);
    }

    let episode = reference_episode();
    let replayed = ReasoningEpisode::from_lino(&episode.to_links_notation());
    assert_eq!(
        replayed, episode,
        "an episode must survive a round trip through its own notation"
    );
    let first = audit(&standard, &episode);
    let second = audit(&standard, &replayed);
    assert_eq!(first, second, "the audit is a pure function of the record");

    let mut log = EventLog::default();
    let recorded = record_reasoning_standard(&mut log, &episode).expect("the audit must record");
    assert_eq!(recorded.verdict, first.verdict);
    let kinds: Vec<&str> = log.events().iter().map(|event| event.kind).collect();
    assert_eq!(
        kinds,
        vec![
            "reasoning_standard",
            "reasoning_standard:gates",
            "reasoning_standard:verdict"
        ],
        "the audit records the ledger, the gate count and the verdict"
    );
    assert_eq!(
        log.last_of("reasoning_standard:gates")
            .expect("gate count event")
            .payload,
        first.outcomes.len().to_string(),
        "every declared gate is counted in the trace"
    );
    assert_eq!(
        log.last_of("reasoning_standard:verdict")
            .expect("verdict event")
            .payload,
        first.verdict.slug()
    );
}

/// The whole task: the reference dialog passes, and every behaviour issue #1073
/// asked us to adopt from it is load-bearing.
///
/// Each mutation removes exactly one adopted behaviour — the evidence behind a
/// claim, the re-measurement after a change, the honest report of a partial
/// failure — and each must flip its own gate. A standard whose gates cannot fail
/// is decoration.
#[test]
fn the_reference_dialog_passes_and_each_adopted_behaviour_is_load_bearing() {
    let standard = loaded();
    let episode = reference_episode();
    let result = audit(&standard, &episode);
    assert!(
        result.all_triggered_gates_satisfied(),
        "the reference dialog must clear its own standard: {:?}",
        result.outcomes
    );
    assert_eq!(result.verdict, Verdict::Confirmed);

    let mut unevidenced = reference_episode();
    unevidenced.claims[0].support.clear();
    assert_eq!(
        audit(&standard, &unevidenced)
            .outcome("evidence_before_claim")
            .map(|outcome| outcome.status),
        Some(GateStatus::Violated),
        "a claim about the world with nothing behind it must fail"
    );

    let mut claimed_early = reference_episode();
    claimed_early.claims[0].ordinal = 0;
    assert_eq!(
        audit(&standard, &claimed_early)
            .outcome("evidence_before_claim")
            .map(|outcome| outcome.status),
        Some(GateStatus::Violated),
        "evidence produced after the claim is not evidence for it"
    );

    let mut unverified = reference_episode();
    unverified.actions[0].after = None;
    let unverified_gate = audit(&standard, &unverified)
        .outcome("verify_after_act")
        .cloned()
        .expect("the gate must be in the ledger");
    assert_eq!(unverified_gate.status, GateStatus::Violated);
    assert!(
        unverified_gate
            .findings
            .iter()
            .any(|finding| finding.ends_with("no_measurement_after_action")),
        "{:?}",
        unverified_gate.findings
    );

    let mut smoothed = reference_episode();
    let partial = smoothed
        .actions
        .iter_mut()
        .find(|action| action.outcome == ActionOutcome::PartiallySucceeded)
        .expect("the reference dialog reported a partial failure");
    partial.reported_as = ActionOutcome::Succeeded;
    let honest_gate = audit(&standard, &smoothed)
        .outcome("honest_failure_report")
        .cloned()
        .expect("the gate must be in the ledger");
    assert_eq!(
        honest_gate.status,
        GateStatus::Violated,
        "rounding a partial result up to a success must fail"
    );

    let mut unexplained = reference_episode();
    unexplained
        .actions
        .iter_mut()
        .find(|action| action.outcome == ActionOutcome::PartiallySucceeded)
        .expect("the reference dialog reported a partial failure")
        .reason
        .clear();
    assert_eq!(
        audit(&standard, &unexplained)
            .outcome("honest_failure_report")
            .map(|outcome| outcome.status),
        Some(GateStatus::Violated),
        "a partial result reported without its reason is not an honest report"
    );
}
