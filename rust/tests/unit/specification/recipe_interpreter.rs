//! Issue #559 (R343): executing the recursive-core recipe as data.
//!
//! These tests pin the algorithm-as-data execution to the algorithm-as-code. The
//! recipe (`data/meta/recursive-core-recipe.lino`) is parsed into an ordered
//! program and *run*, driving the live recorder primitives in the order the data
//! declares. The headline guarantee is parity: the event log produced by executing
//! the recipe is identical to the one the hand-written
//! `meta_core::record_meta_core` pipeline produces for the same input and modes.
//! That equality is the concrete proof that the recipe is a faithful, executable
//! description of the pipeline — not just prose pinned alongside it.

use formal_ai::intent_formalization::formalize_intent;
use formal_ai::meta_construction::RecursionMode;
use formal_ai::meta_self_improvement::MetaSelfImprovement;
use formal_ai::recipe_interpreter::RecipeProgram;
use formal_ai::selection::SelectionMode;
use formal_ai::skill_ledger::SkillMode;
use formal_ai::translation::formalize_prompt;

/// Build a formalization through the public path, exactly as the solver does.
fn formalize(prompt: &str) -> formal_ai::intent_formalization::IntentFormalization {
    let candidate = formalize_prompt(prompt, "en");
    formalize_intent(prompt, "en", Some(&candidate))
}

/// The three prompt shapes the meta core must account for: a routed single need, a
/// conjunction of two needs, and an unroutable need.
const PROMPTS: &[&str] = &[
    "translate apple to Russian",
    "translate apple to Russian and write a hello world program in Python",
    "zzqqx unfathomable gibberish token",
];

const RECURSION_MODES: &[RecursionMode] =
    &[RecursionMode::Down, RecursionMode::Up, RecursionMode::Both];
const SELECTION_MODES: &[SelectionMode] = &[SelectionMode::Off, SelectionMode::Record];
const SKILL_MODES: &[SkillMode] = &[SkillMode::Off, SkillMode::Accumulate];

#[test]
fn the_program_parses_fourteen_contiguously_ordered_steps() {
    let program = RecipeProgram::from_repo();
    assert_eq!(
        program.step_count(),
        14,
        "the recipe declares the fourteen-step recursive meta core"
    );
    let mut orders: Vec<u8> = program.steps.iter().map(|step| step.order).collect();
    orders.sort_unstable();
    assert_eq!(
        orders,
        (1..=14).collect::<Vec<_>>(),
        "the parsed program preserves the fourteen contiguous step orders"
    );
    // Steps must come back sorted by order so execution follows the declared plan.
    let as_parsed: Vec<u8> = program.steps.iter().map(|step| step.order).collect();
    assert_eq!(
        as_parsed, orders,
        "steps must be sorted by their declared order"
    );
}

#[test]
fn the_recorder_sequence_matches_the_live_pipeline_order() {
    let program = RecipeProgram::from_repo();
    let data_order = program.recorder_sequence();
    let pipeline = MetaSelfImprovement::from_repo();
    let code_order: Vec<&str> = pipeline
        .pipeline_stages()
        .iter()
        .map(|stage| stage.function.as_str())
        .collect();
    assert_eq!(
        data_order, code_order,
        "the recorder primitives the recipe drives must match, in order, the \
         stages the live pipeline actually invokes"
    );
    // The eleven trace-recorded stages; the other three steps are external.
    assert_eq!(
        data_order.len(),
        11,
        "eleven of the fourteen steps record trace events"
    );
}

#[test]
fn executing_the_recipe_reproduces_the_pipeline_trace_in_the_quiet_modes() {
    let program = RecipeProgram::from_repo();
    for prompt in PROMPTS {
        let formalization = formalize(prompt);
        assert!(
            program.reproduces_pipeline(
                &formalization,
                4,
                RecursionMode::Down,
                SelectionMode::Off,
                SkillMode::Off,
            ),
            "data-driven execution must reproduce the pipeline trace for {prompt:?}"
        );
    }
}

#[test]
fn executing_the_recipe_reproduces_the_pipeline_under_every_mode_combination() {
    let program = RecipeProgram::from_repo();
    for prompt in PROMPTS {
        let formalization = formalize(prompt);
        for &recursion in RECURSION_MODES {
            for &selection in SELECTION_MODES {
                for &skill in SKILL_MODES {
                    assert!(
                        program
                            .reproduces_pipeline(&formalization, 4, recursion, selection, skill,),
                        "parity must hold for {prompt:?} under \
                         recursion={recursion:?} selection={selection:?} skill={skill:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn external_stages_are_skipped_and_recorder_stages_run() {
    let program = RecipeProgram::from_repo();
    let formalization = formalize("translate apple to Russian");
    let trace = program
        .execute(
            &formalization,
            4,
            RecursionMode::Down,
            SelectionMode::Off,
            SkillMode::Off,
        )
        .expect("the checked-in recipe executes cleanly");

    // The three external stages never record an event.
    for external in ["formalize_impulse", "resolve_leaves", "project_answer"] {
        assert!(
            trace.skipped.contains(&external.to_owned()),
            "external stage `{external}` must be skipped, not executed"
        );
        assert!(
            !trace.executed.contains(&external.to_owned()),
            "external stage `{external}` must not appear as executed"
        );
    }

    // In the quiet modes eight recorder stages emit events; upward construction,
    // selection, and the skill ledger are gated off and skipped. The reasoning
    // standard audit takes no mode, so it runs here too (issue #1073), and
    // neither does the obligation-discharge pass, which reports honestly that
    // nothing has been observed yet (issue #1138 B5, plan 05 leaf 12).
    assert_eq!(
        trace.executed,
        vec![
            "build_problem_frame",
            "decompose_recursively",
            "account_for_needs",
            "catalogue_methods",
            "reason_white_box",
            "record_evidence",
            "audit_reasoning_standard",
            "verify_obligations",
        ],
        "the executed recorder stages, in order, for the quiet modes"
    );
    for gated in ["construct_upward", "select_methods", "accumulate_skills"] {
        assert!(
            trace.skipped.contains(&gated.to_owned()),
            "mode-gated stage `{gated}` must be skipped once its mode is quietened"
        );
    }
    assert_eq!(
        trace.executed.len() + trace.skipped.len(),
        14,
        "every step is accounted for as executed or skipped"
    );
}

#[test]
fn enabling_modes_executes_the_gated_stages() {
    let program = RecipeProgram::from_repo();
    let formalization = formalize("translate apple to Russian");
    let trace = program
        .execute(
            &formalization,
            4,
            RecursionMode::Both,
            SelectionMode::Record,
            SkillMode::Accumulate,
        )
        .expect("the recipe executes under every mode enabled");
    for now_running in ["construct_upward", "select_methods", "accumulate_skills"] {
        assert!(
            trace.executed.contains(&now_running.to_owned()),
            "`{now_running}` must run once its mode is enabled"
        );
    }
    assert!(
        trace.skipped.is_empty()
            || trace.skipped.iter().all(|id| [
                "formalize_impulse",
                "resolve_leaves",
                "project_answer"
            ]
            .contains(&id.as_str())),
        "with every mode enabled, only the three external stages are skipped"
    );
}

#[test]
fn native_and_data_driven_planning_do_not_fabricate_execution_evidence() {
    let formalization = formalize("translate apple to Russian");
    let program = RecipeProgram::from_repo();
    let trace = program
        .execute(
            &formalization,
            4,
            RecursionMode::Both,
            SelectionMode::Record,
            SkillMode::Accumulate,
        )
        .unwrap();
    assert!(program.reproduces_pipeline(
        &formalization,
        4,
        RecursionMode::Both,
        SelectionMode::Record,
        SkillMode::Accumulate,
    ));
    let need_ledger = &trace.log.first_of("need_ledger").unwrap().payload;
    assert!(need_ledger.contains("planned \"1\""), "{need_ledger}");
    assert!(need_ledger.contains("satisfied \"0\""), "{need_ledger}");
    let evidence = &trace.log.first_of("solution_evidence").unwrap().payload;
    assert!(evidence.contains("fully_resolved \"false\""), "{evidence}");
    let skills = &trace.log.first_of("skill_ledger").unwrap().payload;
    assert!(
        !skills.contains("record_type \"candidate_skill\""),
        "{skills}"
    );
    assert!(
        skills.contains("record_type \"curriculum_item\""),
        "{skills}"
    );
}

#[test]
fn a_recipe_that_runs_a_stage_before_its_dependency_is_an_error() {
    // need_ledger depends on the problem frame; placing it first must be rejected.
    let misordered = "\
step_account_for_needs
  record_type \"meta_step\"
  order \"1\"
  id \"account_for_needs\"
  title \"Account for needs too early\"
  source_file \"rust/src/meta_frame.rs\"
  records \"record_need_ledger\"
step_build_problem_frame
  record_type \"meta_step\"
  order \"2\"
  id \"build_problem_frame\"
  title \"Build the frame too late\"
  source_file \"rust/src/meta_frame.rs\"
  records \"record_problem_frame\"
";
    let program = RecipeProgram::from_lino(misordered);
    let formalization = formalize("translate apple to Russian");
    let error = program
        .execute(
            &formalization,
            4,
            RecursionMode::Down,
            SelectionMode::Off,
            SkillMode::Off,
        )
        .expect_err("a misordered recipe must fail rather than silently misbehave");
    assert!(
        error.contains("record_need_ledger") && error.contains("problem frame"),
        "the error must name the offending stage and its missing dependency: {error}"
    );
}

#[test]
fn an_unknown_recorder_binding_is_an_error() {
    let bogus = "\
step_unknown
  record_type \"meta_step\"
  order \"1\"
  id \"unknown\"
  title \"Bind a recorder that does not exist\"
  source_file \"rust/src/meta_frame.rs\"
  records \"record_nonexistent\"
";
    let program = RecipeProgram::from_lino(bogus);
    let formalization = formalize("translate apple to Russian");
    let error = program
        .execute(
            &formalization,
            4,
            RecursionMode::Down,
            SelectionMode::Off,
            SkillMode::Off,
        )
        .expect_err("an unknown recorder binding must be rejected");
    assert!(
        error.contains("record_nonexistent"),
        "the error must name the unknown recorder: {error}"
    );
}

#[test]
fn the_program_serializes_as_links_notation() {
    let program = RecipeProgram::from_repo();
    let lino = program.to_links_notation();
    assert!(lino.contains("recipe_program"), "header record present");
    assert!(lino.contains("step_count \"14\""), "step count serialized");
    assert!(
        lino.contains("recorder_count \"11\""),
        "recorder count serialized"
    );
    assert!(
        lino.contains("executes \"record_problem_frame\""),
        "each plan step names the recorder it drives"
    );
    assert!(
        lino.contains("executes \"external\""),
        "external stages are marked as non-recording"
    );
}

/// Issue #1138 B5 (plan 05, leaf 11): the execution pass is a bound recorder.
///
/// Recipe step 14 names `record_obligation_ledger`. Without the binding in
/// `run_recorder` the data-driven path errors with "recipe binds unknown
/// recorder"; with it the program resolves fourteen contiguous steps and names
/// the recorder it drives.
#[test]
fn the_execution_pass_is_bound_to_a_known_recorder() {
    let program = RecipeProgram::from_repo();
    let lino = program.to_links_notation();
    assert!(
        lino.contains("step_count \"14\""),
        "the recipe gains step 14, the obligation-discharge pass:\n{lino}"
    );
    assert!(
        lino.contains("executes \"record_obligation_ledger\""),
        "step 14 must bind the obligation-ledger recorder:\n{lino}"
    );
}

/// Issue #1138 B5 (plan 05, leaf 11): R343 parity survives the new stage.
///
/// Executing the recipe must still reproduce the native log event for event
/// across every mode combination, and the executed trace must carry the
/// obligation ledger the new stage records.
#[test]
fn native_and_data_driven_execution_produce_the_same_events() {
    for prompt in PROMPTS {
        let formalization = formalize(prompt);
        for recursion in RECURSION_MODES {
            for selection in SELECTION_MODES {
                for skill in SKILL_MODES {
                    let program = RecipeProgram::from_repo();
                    assert!(
                        program.reproduces_pipeline(
                            &formalization,
                            4,
                            *recursion,
                            *selection,
                            *skill,
                        ),
                        "parity must hold for {prompt:?} at {recursion:?}/{selection:?}/{skill:?}"
                    );
                    let trace = program
                        .execute(&formalization, 4, *recursion, *selection, *skill)
                        .expect("the recipe must execute");
                    let obligations = trace
                        .log
                        .first_of("obligation_ledger")
                        .expect("the execution pass records an obligation ledger");
                    assert!(
                        obligations.payload.contains("unattempted")
                            || obligations.payload.contains("satisfied"),
                        "the obligation ledger reports its counts: {}",
                        obligations.payload
                    );
                }
            }
        }
    }
}
