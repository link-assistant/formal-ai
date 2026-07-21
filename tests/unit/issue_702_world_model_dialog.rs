//! Issue #702: the symbolic world model, driven by the dialogue.
//!
//! Issue #649 built the substrate (contexts as links networks, difference,
//! prediction, merge/split, relative-meta-logic recalculation). Issue #702 wires
//! it into the conversation. One test per requirement, then a whole-task test
//! that runs a scripted multi-turn dialogue end to end.

use formal_ai::relative_meta_logic::Stance;
use formal_ai::solver::{ConversationRole, ConversationTurn};
use formal_ai::world_model::Action;
use formal_ai::world_model_atoms::{classify, state_atom, UtteranceKind, CONSULTED_CUE_SETS};
use formal_ai::world_model_dialog::{DialogueWorldModel, SyncEventKind, WorldModelMode};

/// The four supported languages saying the same three things: a current-state
/// fact, a wish, and the "what is left?" question.
const DIALOGUE_BY_LANGUAGE: &[(&str, &str, &str, &str)] = &[
    (
        "en",
        "the door is closed",
        "I want the door to be open",
        "what is left to do?",
    ),
    (
        "ru",
        "дверь закрыта",
        "я хочу чтобы дверь была открыта",
        "что осталось сделать?",
    ),
    (
        "hi",
        "दरवाज़ा बंद है",
        "मुझे चाहिए दरवाज़ा खुला",
        "क्या बाकी है?",
    ),
    ("zh", "门是关着的", "我想要门是开着的", "还剩什么?"),
];

// -- Requirement 1: the current-state context is seeded from the dialogue ----

#[test]
fn declarative_turns_seed_the_current_state_context_with_provenance() {
    let mut model = DialogueWorldModel::new();
    assert_eq!(model.observe_user("the door is closed"), UtteranceKind::CurrentState);

    assert!(
        model.model.current.holds("door", "closed"),
        "the fact must land in the current-state context as a links atom:\n{}",
        model.model.current.links_notation()
    );
    let statement = model
        .model
        .current
        .statements()
        .values()
        .find(|statement| statement.text == "the door is closed")
        .expect("the utterance itself is kept as a statement");
    let notation = model.model.current.links_notation();
    assert!(
        notation.contains(&format!("{} -> provenance:turn:1", statement.id)),
        "every statement must trace back to the turn that said it:\n{notation}"
    );
    assert!(
        notation.contains(&format!("{} -> asserts:door -> closed", statement.id)),
        "the statement must point at the atom it produced:\n{notation}"
    );
    assert!(
        model
            .events()
            .iter()
            .any(|event| event.kind == SyncEventKind::CurrentAsserted),
        "seeding the current state is itself a recorded event"
    );
}

// -- Requirement 2: the target-state context is built from intent ------------

#[test]
fn wishes_and_requests_build_the_target_state_context() {
    let mut model = DialogueWorldModel::new();
    model.observe_user("the door is closed");
    assert_eq!(
        model.observe_user("I want the door to be open"),
        UtteranceKind::TargetState
    );

    assert!(
        model.model.target.holds("door", "open"),
        "an \"I want …\" turn edits the target, not the current state:\n{}",
        model.model.target.links_notation()
    );
    assert!(
        !model.model.current.holds("door", "open"),
        "a wish must never be recorded as a fact about the current world"
    );
}

#[test]
fn all_four_languages_route_the_same_three_utterances_the_same_way() {
    for (language, fact, wish, question) in DIALOGUE_BY_LANGUAGE {
        assert_eq!(
            classify(fact),
            UtteranceKind::CurrentState,
            "[{language}] `{fact}` is a current-state assertion"
        );
        assert_eq!(
            classify(wish),
            UtteranceKind::TargetState,
            "[{language}] `{wish}` is a target-state edit"
        );
        assert_eq!(
            classify(question),
            UtteranceKind::RemainingQuery,
            "[{language}] `{question}` asks for the difference"
        );

        let mut model = DialogueWorldModel::new();
        model.observe_user(fact);
        model.observe_user(wish);
        let fact_atom = state_atom(fact).expect("[{language}] the fact yields an atom");
        let wish_atom = state_atom(wish).expect("[{language}] the wish yields an atom");
        assert_eq!(
            fact_atom.from, wish_atom.from,
            "[{language}] both utterances must be about the same subject: {fact_atom:?} vs {wish_atom:?}"
        );
        assert_ne!(
            fact_atom.to, wish_atom.to,
            "[{language}] the wished-for state must differ from the current one"
        );
        assert_eq!(
            model.remaining().len(),
            1,
            "[{language}] exactly one target link is still missing:\n{}",
            model.links_notation()
        );
    }
}

#[test]
fn every_consulted_cue_set_exists_in_the_lexicon_data() {
    for set in CONSULTED_CUE_SETS {
        assert!(
            !formal_ai::cue_lexicon::cues(set).is_empty(),
            "cue set `{set}` must be declared in data/meta/cue-lexicon.lino"
        );
    }
}

// -- Requirement 3: the difference is queryable from the chat ----------------

#[test]
fn the_difference_is_a_links_network_that_answers_what_is_left() {
    let mut model = DialogueWorldModel::new();
    model.observe_user("the door is closed");
    model.observe_user("I want the door to be open");
    assert_eq!(
        model.observe_user("what is left to do?"),
        UtteranceKind::RemainingQuery
    );

    let difference = model.difference();
    assert!(!difference.is_empty(), "the goal is not reached yet");
    let notation = difference.links_notation();
    assert!(
        notation.contains("to_add \"door -> open\""),
        "the missing target atom must be inspectable:\n{notation}"
    );
    assert!(
        notation.contains("conflict \"door -> closed vs door -> open\""),
        "a same-subject disagreement is a conflict the sync must resolve:\n{notation}"
    );
    assert!(
        model
            .events()
            .iter()
            .any(|event| event.kind == SyncEventKind::StateQueried),
        "asking the question is itself an append-only event"
    );

    model.observe_user("the door is open");
    assert!(
        model.target_reached(),
        "once the world catches up the difference is empty:\n{}",
        model.links_notation()
    );
}

// -- Requirement 4: the synchronization loop is append-only ------------------

#[test]
fn the_agent_proposes_the_user_confirms_and_both_converge() {
    let mut model = DialogueWorldModel::new();
    model.observe_user("the report is empty");
    model.propose_target("report", "published");
    assert_eq!(
        model.pending_proposal().map(|link| link.pattern_text()),
        Some(String::from("report -> published")),
        "a proposal waits for the user instead of overwriting the goal"
    );
    assert!(
        !model.model.target.holds("report", "published"),
        "an unconfirmed proposal must not enter the target context"
    );

    model.observe_user("yes, exactly");
    assert!(
        model.model.target.holds("report", "published"),
        "confirmation commits the proposal"
    );
    let kinds: Vec<&str> = model.events().iter().map(|e| e.kind.slug()).collect();
    assert_eq!(
        kinds,
        vec!["current_asserted", "target_proposed", "target_confirmed"],
        "every step of the loop is recorded, in order"
    );
}

#[test]
fn a_correction_replaces_the_proposal_and_supersedes_the_old_target() {
    let mut model = DialogueWorldModel::new();
    model.propose_target("report", "published");
    assert_eq!(
        model.observe_user("no, the report is archived"),
        UtteranceKind::Correction
    );
    assert!(
        model.model.target.holds("report", "archived"),
        "the correction becomes the target:\n{}",
        model.model.target.links_notation()
    );
    assert!(
        !model.model.target.holds("report", "published"),
        "the superseded goal for the same subject is retracted"
    );
    assert!(model.pending_proposal().is_none(), "the proposal is resolved");
}

#[test]
fn the_synchronization_log_is_append_only_and_hash_chained() {
    let mut model = DialogueWorldModel::new();
    model.observe_user("the door is closed");
    model.propose_target("door", "open");
    model.observe_user("yes");
    model.observe_user("what is left?");

    assert!(model.chain_is_intact(), "the freshly built chain re-derives");
    let events = model.events().to_vec();
    for (index, event) in events.iter().enumerate() {
        assert_eq!(event.sequence, index, "sequences are dense and ordered");
        assert_eq!(
            event.id,
            event.derived_id(),
            "each id is content-addressed over the event and its parent"
        );
    }
    for window in events.windows(2) {
        assert_eq!(
            window[1].parent, window[0].id,
            "each event commits to its predecessor, so nothing can be dropped"
        );
    }
    // Replaying the same dialogue rebuilds the identical chain — the log is a
    // deterministic function of the conversation, not of when it ran.
    let replay = DialogueWorldModel::from_turns(&[
        ConversationTurn::user("the door is closed"),
        ConversationTurn::user("yes"),
    ]);
    let direct = {
        let mut model = DialogueWorldModel::new();
        model.observe_user("the door is closed");
        model.observe_user("yes");
        model
    };
    assert_eq!(
        replay.events(),
        direct.events(),
        "replay is byte-for-byte reproducible"
    );
}

// -- Requirement 5: merge and split are first-class --------------------------

#[test]
fn contexts_merge_with_conflict_detection_and_split_round_trips() {
    let mut left = DialogueWorldModel::new();
    left.observe_user("the door is closed");
    left.observe_user("I want the door to be open");

    let mut right = DialogueWorldModel::new();
    right.observe_user("the door is open");
    right.observe_user("I want the window to be open");

    let conflicts = left.merge_from(&right);
    assert!(
        !conflicts.is_empty(),
        "merging disagreeing current states must report the conflict"
    );
    assert!(
        left.model.target.holds("window", "open") && left.model.target.holds("door", "open"),
        "the union of the two targets survives the merge"
    );
    assert!(
        left.events()
            .iter()
            .any(|event| event.kind == SyncEventKind::ContextMerged),
        "the merge is recorded"
    );

    let ids: Vec<String> = left
        .model
        .current
        .statements()
        .values()
        .filter(|statement| statement.text.contains("door"))
        .map(|statement| statement.id.clone())
        .collect();
    assert!(!ids.is_empty(), "there are door statements to split off");
    let child = left.split_current("door_topic", &ids);
    assert_eq!(
        child.statements().len(),
        ids.len(),
        "the split child carries exactly the named statements"
    );
    for id in &ids {
        assert!(
            child.statement(id).is_some(),
            "statement {id} must survive the split"
        );
        assert!(
            left.model.current.statement(id).is_some(),
            "splitting must not damage the parent (round-trip)"
        );
    }
    assert!(
        left.events()
            .iter()
            .any(|event| event.kind == SyncEventKind::ContextSplit),
        "the split is recorded"
    );
}

// -- Requirement 6: dependent statements recalculate -------------------------

#[test]
fn revising_a_premise_recalculates_every_dependent_statement() {
    let mut model = DialogueWorldModel::new();
    model.observe_user("the build is broken because the test is failing");
    let dependent = model
        .model
        .current
        .statements()
        .values()
        .find(|statement| statement.text.contains("build"))
        .expect("the consequent is a statement")
        .clone();
    assert!(
        !dependent.dependencies.is_empty(),
        "a causal utterance records a dependency: {dependent:?}"
    );

    let before = dependent.truth;
    let report = model.revise_statement("the test is failing", Stance::Contradicts, 1.0);
    assert!(
        report.converged,
        "the recalculation cascade must reach a fixpoint"
    );
    assert!(
        report
            .updated
            .iter()
            .any(|change| change.statement_id == dependent.id),
        "changing the premise must move the dependent statement: {:?}",
        report.updated
    );
    let after = model
        .model
        .current
        .statement(&dependent.id)
        .expect("the dependent statement is still there")
        .truth;
    assert!(
        after.get() < before.get(),
        "contradicting the premise must lower the dependent posterior: {before} -> {after}"
    );

    let event = model
        .events()
        .iter()
        .find(|event| event.kind == SyncEventKind::StatementRevised)
        .expect("the revision is recorded");
    assert!(
        event.detail.contains(&dependent.id),
        "the trace must name every recalculated link: {}",
        event.detail
    );
}

// -- Requirement 7: action-consequence prediction ----------------------------

#[test]
fn a_helpful_action_shrinks_the_gap_and_a_destructive_one_is_flagged() {
    let mut model = DialogueWorldModel::new();
    model.observe_user("the report is empty");
    model.observe_user("I want the report to be published");
    model.observe_user("I want the backup to be kept");
    model.observe_user("the backup is kept");

    let publish = Action::new("write report file")
        .removing("report", "empty")
        .adding("report", "published");
    let forecast = model.forecast(&publish);
    assert!(
        forecast.shrinks_gap(),
        "publishing must move the world closer to the goal:\n{}",
        forecast.links_notation()
    );
    assert!(
        !forecast.violates_target(),
        "publishing violates nothing the user asked for"
    );
    assert_eq!(
        forecast.satisfied.iter().map(|l| l.pattern_text()).collect::<Vec<_>>(),
        vec![String::from("report -> published")],
        "the satisfied need is named"
    );
    assert!(
        model.model.current.holds("report", "empty"),
        "prediction must never mutate the real world model"
    );

    let wipe = Action::new("delete the backup directory").removing("backup", "kept");
    let destructive = model.forecast(&wipe);
    assert!(
        destructive.violates_target(),
        "deleting a backup the user wants kept must be flagged before execution:\n{}",
        destructive.links_notation()
    );
    assert_eq!(
        destructive
            .violated
            .iter()
            .map(|l| l.pattern_text())
            .collect::<Vec<_>>(),
        vec![String::from("backup -> kept")],
        "the violated need is named"
    );
    assert!(
        !destructive.shrinks_gap(),
        "a destructive action does not bring the goal closer"
    );
    assert!(
        destructive.links_notation().contains("verdict \"violates_target\""),
        "the verdict is inspectable:\n{}",
        destructive.links_notation()
    );
}

// -- Requirement 8: everything is links, off until opted in ------------------

#[test]
fn the_whole_dialogue_model_renders_as_links_notation() {
    let mut model = DialogueWorldModel::new();
    model.observe_user("the door is closed");
    model.observe_user("I want the door to be open");
    let notation = model.links_notation();
    for needle in [
        "record_type \"dialogue_world_model\"",
        "current_state",
        "target_state",
        "state_diff",
        "sync_event",
    ] {
        assert!(
            notation.contains(needle),
            "the model must render `{needle}`:\n{notation}"
        );
    }
    for forbidden in ["embedding", "vector", "vertex", " edge ", "graph"] {
        assert!(
            !notation.contains(forbidden),
            "the rendering must stay links-only, found `{forbidden}`:\n{notation}"
        );
    }
}

#[test]
fn the_world_model_mode_is_off_until_it_is_opted_in() {
    assert_eq!(
        WorldModelMode::default(),
        WorldModelMode::Off,
        "the default must preserve the previous behaviour"
    );
    assert!(!WorldModelMode::Off.emits_artifact());
    assert!(WorldModelMode::Track.emits_artifact());
    assert_eq!(WorldModelMode::from_slug("track"), Some(WorldModelMode::Track));
    assert_eq!(WorldModelMode::from_slug("nonsense"), None);

    let mut log = formal_ai::event_log::EventLog::new();
    let model = DialogueWorldModel::new();
    assert!(
        formal_ai::world_model_dialog::record_world_model(&mut log, &model, WorldModelMode::Off)
            .is_none(),
        "the default mode records nothing"
    );
    assert!(
        formal_ai::world_model_dialog::record_world_model(&mut log, &model, WorldModelMode::Track)
            .is_some(),
        "opting in records the artifact"
    );
}

// -- The whole task: a scripted multi-turn dialogue --------------------------

#[test]
fn a_scripted_dialogue_tracks_state_answers_the_goal_question_and_forecasts() {
    let turns = vec![
        ConversationTurn::user("the door is closed"),
        ConversationTurn::assistant("Noted."),
        ConversationTurn::user("the light is off"),
        ConversationTurn::user("I want the door to be open"),
        ConversationTurn::user("I want the light to be on"),
    ];
    let mut model = DialogueWorldModel::from_turns(&turns);

    assert!(model.model.current.holds("door", "closed"));
    assert!(model.model.current.holds("light", "off"));
    assert!(model.model.target.holds("door", "open"));
    assert!(model.model.target.holds("light", "on"));
    assert_eq!(
        model.remaining().len(),
        2,
        "two goals are still open:\n{}",
        model.links_notation()
    );

    // "what remains to reach my goal?"
    assert_eq!(
        model.observe(ConversationRole::User, "what remains?"),
        UtteranceKind::RemainingQuery
    );

    // A bounded agent action closes one of them, and the forecast says so before
    // anything is executed.
    let open_door = Action::new("open the door")
        .removing("door", "closed")
        .adding("door", "open");
    let forecast = model.forecast(&open_door);
    assert_eq!(forecast.remaining_before, 2);
    assert_eq!(forecast.remaining_after, 1);
    assert!(forecast.shrinks_gap() && !forecast.violates_target());

    // Executing it for real leaves exactly the other goal open.
    model.observe_user("the door is open");
    let remaining: Vec<String> = model
        .remaining()
        .iter()
        .map(|link| link.pattern_text())
        .collect();
    assert_eq!(remaining, vec![String::from("light -> on")]);
    assert!(model.chain_is_intact(), "the whole dialogue stays append-only");
}
