//! Issue #674 (E55): compile arbitrary natural-language programs beyond the
//! supported skill subset.
//!
//! `docs/USER-JOURNEYS.md` F2 states the journey: a user describes a multi-step
//! procedure in ordinary prose and the system compiles it into a typed, executable,
//! inspectable skill. These tests pin the three acceptance criteria from the issue:
//!
//! 1. a freely phrased four-step procedure compiles, executes end to end, and
//!    re-states its steps on request;
//! 2. the same procedure in Russian (and Hindi, and Chinese) compiles to the same
//!    skill links — the compiled program is language-independent;
//! 3. a procedure carrying one uncompilable step yields the honest named gap plus a
//!    `skill_gap` event, and compiles nothing partially.

use formal_ai::agentic_coding::{plan_chat_step, run_agentic_task, AgenticPlan};
use formal_ai::intent_formalization::impulse_id_for;
use formal_ai::protocol::{ChatMessage, ToolCall};
use formal_ai::{
    compile_procedure, compile_procedure_with_ledger, extract_compiled_procedure_artifact,
    CompiledProcedure, ProcedureCapabilityLedger, ProcedureCapabilityLesson, ProcedureCompileError,
    ProcedureHost, ProcedureLearningApproval, ProcedureLearningGate, ProcedureLearningProposal,
    ProcedureStep, UniversalSolver,
};

/// The English procedure under test: five clauses, four of them steps, phrased as
/// running prose rather than as any template the typed `skill_compiler` accepts.
const ENGLISH_PROCEDURE: &str =
    "When I paste a link, fetch its title, translate it to Russian, save both, \
     and reply with the translation.";

const RUSSIAN_PROCEDURE: &str = "Когда я вставляю ссылку, получи её заголовок, \
                                 переведи его на русский, сохрани оба и ответь переводом.";

const HINDI_PROCEDURE: &str = "जब मैं लिंक भेजूँ, उसका शीर्षक लाओ, उसे रूसी में अनुवाद करो, \
                               दोनों सहेजो और अनुवाद के साथ जवाब दो।";

const CHINESE_PROCEDURE: &str = "当我粘贴链接，获取标题，翻译成俄语，保存两者，然后用译文回复。";

/// A deterministic host: every step reports what it did to what it received, so the
/// final answer is a full execution trace rather than a mocked constant.
struct TracingHost;

impl ProcedureHost for TracingHost {
    fn perform(&mut self, step: &ProcedureStep, input: &str) -> Result<String, String> {
        Ok(format!("{}({input})", step.kind))
    }
}

fn compile(description: &str) -> CompiledProcedure {
    match compile_procedure(description) {
        Ok(procedure) => procedure,
        Err(error) => panic!("expected {description:?} to compile, got {error:?}"),
    }
}

#[test]
fn arbitrary_four_step_procedure_compiles_executes_and_restates_its_steps() {
    let procedure = compile(ENGLISH_PROCEDURE);

    let kinds: Vec<&str> = procedure
        .steps
        .iter()
        .map(|step| step.kind.as_str())
        .collect();
    assert_eq!(
        kinds,
        [
            "skill_procedure_fetch",
            "skill_procedure_translate",
            "skill_procedure_store",
            "skill_procedure_reply",
        ],
        "four freely phrased clauses should map onto the seeded step vocabulary"
    );
    assert_eq!(
        procedure.steps[0].arguments(),
        ["skill_procedure_object_title"],
        "\"fetch its title\" should carry the title object"
    );
    assert_eq!(
        procedure.steps[1].arguments(),
        ["language_russian"],
        "\"translate it to Russian\" should resolve the target language slug"
    );
    assert_eq!(
        procedure.trigger.objects,
        ["skill_procedure_object_link"],
        "the trigger should name what the user supplies"
    );

    let run = procedure
        .execute("https://example.com/article", &mut TracingHost)
        .expect("the tracing host performs every compiled step");
    assert_eq!(run.package_id, procedure.id);
    assert_eq!(run.outcomes.len(), 4, "every compiled step should execute");
    assert_eq!(
        run.answer(),
        "skill_procedure_reply(skill_procedure_store(skill_procedure_translate\
         (skill_procedure_fetch(https://example.com/article))))",
        "each step's output should thread into the next"
    );

    // "Why did you do that?" cites the compiled steps and their source spans.
    let restated = procedure.restate_steps();
    for step in &procedure.steps {
        assert!(
            restated.contains(&step.kind),
            "restatement should name step kind {}, got: {restated}",
            step.kind
        );
        let quoted = &ENGLISH_PROCEDURE[step.source_span.0..step.source_span.1];
        assert_eq!(
            quoted, step.source_text,
            "each step should cite the exact span it was read from"
        );
    }
}

#[test]
fn same_procedure_in_every_supported_language_compiles_to_the_same_skill_links() {
    let english = compile(ENGLISH_PROCEDURE);
    for (language, description) in [
        ("ru", RUSSIAN_PROCEDURE),
        ("hi", HINDI_PROCEDURE),
        ("zh", CHINESE_PROCEDURE),
    ] {
        let other = compile(description);
        assert_eq!(
            other.links_notation(),
            english.links_notation(),
            "{language} should compile to byte-identical skill links"
        );
        assert_eq!(
            other.id, english.id,
            "{language} should content-address to the same compiled program"
        );
        assert_eq!(
            other.link_records().len(),
            english.link_records().len(),
            "{language} should project the same number of link records"
        );
        // The compiled program is shared; only the citations are language-specific.
        assert_ne!(
            other.restate_steps(),
            english.restate_steps(),
            "{language} should still quote its own source sentence spans"
        );
    }
}

#[test]
fn uncompilable_step_reports_a_named_gap_and_compiles_nothing_partially() {
    let with_gap =
        "When I paste a link, fetch its title, print it on my printer, and reply with the title.";
    match compile_procedure(with_gap) {
        Err(ProcedureCompileError::UncompilableStep { step, span, gap }) => {
            assert_eq!(step, "print it on my printer");
            assert_eq!(
                gap, "no compiled capability for \"print it on my printer\"",
                "the gap should name the missing capability, not merely fail"
            );
            assert_eq!(
                &with_gap[span.0..span.1],
                step,
                "the gap should point at the exact clause"
            );
        }
        other => panic!("expected a named gap, got {other:?}"),
    }
}

#[test]
fn solver_answers_an_uncompilable_step_with_the_gap_and_a_skill_gap_event() {
    let solver = UniversalSolver::default();
    let response = solver.solve(
        "When I paste a link, fetch its title, print it on my printer, and reply with the title.",
    );

    assert_eq!(response.intent, "skill_gap");
    assert!(
        response.answer.contains("print it on my printer"),
        "the reply should name the uncompilable step, got: {}",
        response.answer
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("skill_gap:")),
        "a skill_gap event should be recorded, got: {:?}",
        response.evidence_links
    );
    assert!(
        !response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("skill_compile:procedure")),
        "nothing should be compiled when a step has no capability, got: {:?}",
        response.evidence_links
    );
}

#[test]
fn solver_compiles_a_freely_phrased_procedure_and_can_restate_it_later() {
    let solver = UniversalSolver::default();
    let response = solver.solve(ENGLISH_PROCEDURE);

    assert_eq!(response.intent, "compiled_procedure");
    let procedure = compile(ENGLISH_PROCEDURE);
    assert!(
        response.answer.contains(&procedure.id),
        "the reply should expose the compiled program id, got: {}",
        response.answer
    );
    for step in &procedure.steps {
        assert!(
            response.answer.contains(&step.kind),
            "the reply should list step {}, got: {}",
            step.kind,
            response.answer
        );
    }
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == &format!("skill_compile:procedure:{}", procedure.id)),
        "compiling should be evidenced, got: {:?}",
        response.evidence_links
    );

    let history = [
        formal_ai::ConversationTurn::user(ENGLISH_PROCEDURE),
        formal_ai::ConversationTurn::assistant(response.answer),
    ];
    let why = solver.solve_with_history("Why did you do that?", &history);
    for step in &procedure.steps {
        assert!(
            why.answer.contains(&step.kind),
            "the rationale should cite compiled step {}, got: {}",
            step.kind,
            why.answer
        );
        assert!(
            why.answer.contains(&step.source_text),
            "the rationale should quote the source span of {}, got: {}",
            step.kind,
            why.answer
        );
    }
}

#[test]
fn compiler_reuses_intent_formalization_and_records_every_ordered_requirement() {
    let procedure = compile(ENGLISH_PROCEDURE);

    assert_eq!(
        procedure.impulse_id,
        impulse_id_for(ENGLISH_PROCEDURE),
        "the procedure must reuse the solver's public intent identity"
    );
    assert_eq!(
        procedure.requirements.len(),
        5,
        "trigger plus all four ordered steps must be formalized"
    );
    for (index, requirement) in procedure.requirements.iter().enumerate() {
        assert_eq!(requirement.index, index + 1);
        assert_eq!(
            requirement.source_text,
            ENGLISH_PROCEDURE[requirement.source_span.0..requirement.source_span.1],
            "requirement provenance must point into the original impulse"
        );
    }
    assert_eq!(
        procedure.trigger.requirement_id,
        procedure.requirements[0].id
    );
    for (step, requirement) in procedure.steps.iter().zip(&procedure.requirements[1..]) {
        assert_eq!(
            step.requirement_id, requirement.id,
            "each executable leaf must link to the requirement it realizes"
        );
    }
}

#[test]
fn compiled_artifact_round_trips_executes_and_explains_without_recompiling_user_prose() {
    let procedure = compile(ENGLISH_PROCEDURE);
    let artifact = procedure.artifact_links_notation();
    let restored = CompiledProcedure::from_artifact_links_notation(&artifact)
        .expect("the persisted artifact should parse and validate");

    assert_eq!(restored, procedure);
    let tampered = artifact.replacen(&procedure.id, "procedure_tampered", 1);
    assert!(
        CompiledProcedure::from_artifact_links_notation(&tampered).is_err(),
        "content-addressed artifact integrity must reject a changed id"
    );
    let tampered_source = artifact.replacen("fetch its title", "fetch its titles", 1);
    assert!(
        CompiledProcedure::from_artifact_links_notation(&tampered_source).is_err(),
        "source provenance must remain bound to the formalized impulse"
    );
    assert_eq!(
        restored
            .execute("https://example.com/article", &mut TracingHost)
            .expect("the restored artifact should walk the generic interpreter")
            .answer(),
        "skill_procedure_reply(skill_procedure_store(skill_procedure_translate\
         (skill_procedure_fetch(https://example.com/article))))"
    );

    let solver = UniversalSolver::default();
    let compiled = solver.solve(ENGLISH_PROCEDURE);
    let embedded = extract_compiled_procedure_artifact(&compiled.answer)
        .expect("the solver reply must carry the full executable artifact");
    assert_eq!(embedded, procedure);

    // Deliberately omit the original user turn. This can only pass if the why
    // handler reads the persisted assistant artifact instead of recompiling prose.
    let history = [formal_ai::ConversationTurn::assistant(compiled.answer)];
    let why = solver.solve_with_history("Why did you do that?", &history);
    for step in &procedure.steps {
        assert!(why.answer.contains(&step.kind), "{}", why.answer);
        assert!(why.answer.contains(&step.source_text), "{}", why.answer);
    }
}

#[test]
fn named_gap_proposes_human_gated_learning_and_an_approved_lesson_generalizes() {
    let english =
        "When I paste a link, fetch its title, archive it, and reply with the translation.";
    let error = compile_procedure(english).expect_err("archive is not in the seed vocabulary");
    let proposal = ProcedureLearningProposal::from_compile_error(&error)
        .expect("a named procedure gap should become a reviewable proposal");
    assert_eq!(proposal.missing_step, "archive it");
    assert!(proposal.links_notation().contains("human_review_required"));

    let lesson = ProcedureCapabilityLesson::new(
        "skill_procedure_store",
        [
            ("en", "archive"),
            ("ru", "архивируй"),
            ("hi", "आर्काइव करो"),
            ("zh", "归档"),
        ],
    )
    .expect("the lesson covers every supported language");

    let mut rejected = ProcedureCapabilityLedger::new();
    assert!(
        rejected
            .promote(
                &proposal,
                lesson.clone(),
                ProcedureLearningGate::failed("arbitrary_skill_compilation", 7, 1),
                ProcedureLearningApproval::granted("maintainer"),
            )
            .is_err(),
        "human approval must not bypass a red regression gate"
    );
    assert!(
        rejected
            .promote(
                &proposal,
                lesson.clone(),
                ProcedureLearningGate::passed("arbitrary_skill_compilation", 8),
                ProcedureLearningApproval::declined("maintainer"),
            )
            .is_err(),
        "green tests must not bypass human review"
    );

    let mut ledger = ProcedureCapabilityLedger::new();
    ledger
        .promote(
            &proposal,
            lesson,
            ProcedureLearningGate::passed("arbitrary_skill_compilation", 8),
            ProcedureLearningApproval::granted("maintainer"),
        )
        .expect("a reviewed green lesson should be promotable");
    let durable = ledger.links_notation();
    let tampered_ledger = durable.replacen(&ledger.lessons[0].id, "approved_lesson_tampered", 1);
    assert!(
        ProcedureCapabilityLedger::from_links_notation(&tampered_ledger).is_err(),
        "a durable lesson must retain its review-derived identity"
    );
    let restored = ProcedureCapabilityLedger::from_links_notation(&durable)
        .expect("approved vocabulary growth must survive a process restart");

    let learned = [
        ("en", english),
        (
            "ru",
            "Когда я вставляю ссылку, получи её заголовок, архивируй и ответь переводом.",
        ),
        (
            "hi",
            "जब मैं लिंक भेजूँ, उसका शीर्षक लाओ, आर्काइव करो और अनुवाद के साथ जवाब दो।",
        ),
        ("zh", "当我粘贴链接，获取标题，归档，然后用译文回复。"),
    ];
    let reference = compile_procedure_with_ledger(learned[0].1, &restored)
        .expect("approved English surface should compile");
    for (language, description) in learned {
        let compiled = compile_procedure_with_ledger(description, &restored)
            .unwrap_or_else(|error| panic!("{language} learned surface: {error:?}"));
        assert_eq!(
            compiled.links_notation(),
            reference.links_notation(),
            "{language} must lower to the same canonical program after learning"
        );
    }
}

#[test]
fn solver_records_a_learning_proposal_but_still_compiles_nothing_on_a_gap() {
    let solver = UniversalSolver::default();
    let response = solver
        .solve("When I paste a link, fetch its title, archive it, and reply with the translation.");

    assert_eq!(response.intent, "skill_gap");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("skill_learning_proposal:")),
        "the gap should feed the reviewable learning back-edge: {:?}",
        response.evidence_links
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("skill_learning_proposal:artifact:")),
        "the evidence graph should retain the inspectable proposal artifact: {:?}",
        response.evidence_links
    );
    assert!(
        response.answer.contains("human_review_required"),
        "the reply should expose the proposal without implying it is executable: {}",
        response.answer
    );
    assert!(
        !response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("skill_compile:procedure")),
        "a learning proposal is not permission to emit a partial program"
    );
}

fn answer_tool_call(messages: &mut Vec<ChatMessage>, tool: &str, arguments: &str, result: &str) {
    let id = format!("call_{}", messages.len());
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        id.clone(),
        tool,
        arguments,
    )]));
    messages.push(ChatMessage::tool_result(id, tool, result));
}

#[test]
fn agentic_planner_writes_verifies_and_returns_the_same_compiled_artifact() {
    let tools = ["write_file", "run_command"];
    let mut messages = vec![ChatMessage::user(ENGLISH_PROCEDURE)];
    let procedure = compile(ENGLISH_PROCEDURE);

    let Some(AgenticPlan::ToolCalls(write)) = plan_chat_step(&messages, &tools) else {
        panic!("the Formal AI agent path should claim an arbitrary procedure");
    };
    assert_eq!(write.len(), 1);
    assert_eq!(write[0].tool, "write_file");
    let arguments: serde_json::Value =
        serde_json::from_str(&write[0].arguments).expect("write arguments");
    assert_eq!(arguments["path"], "compiled-procedure.lino");
    assert_eq!(arguments["content"], procedure.artifact_links_notation());
    answer_tool_call(
        &mut messages,
        &write[0].tool,
        &write[0].arguments,
        "wrote compiled-procedure.lino",
    );

    let Some(AgenticPlan::ToolCalls(verify)) = plan_chat_step(&messages, &tools) else {
        panic!("the second Agent CLI turn should verify the authored artifact");
    };
    assert_eq!(verify.len(), 1);
    assert_eq!(verify[0].tool, "run_command");
    assert!(verify[0].arguments.contains("compiled-procedure.lino"));
    answer_tool_call(
        &mut messages,
        &verify[0].tool,
        &verify[0].arguments,
        &procedure.artifact_links_notation(),
    );

    let Some(AgenticPlan::Final(answer)) = plan_chat_step(&messages, &tools) else {
        panic!("the verified Agent CLI task should finish");
    };
    assert!(answer.contains(&procedure.id));
    assert!(answer.contains(&procedure.restate_steps()));

    let run = run_agentic_task(ENGLISH_PROCEDURE).expect("in-repo Agent CLI replay");
    assert_eq!(
        run.steps
            .iter()
            .map(|step| step.tool.as_str())
            .collect::<Vec<_>>(),
        ["write_file", "run_command"]
    );
    assert!(!run.hit_turn_cap);
    assert!(run.final_answer.contains(&procedure.id));
}

#[test]
fn agentic_planner_rejects_a_corrupted_artifact_readback() {
    let tools = ["write_file", "run_command"];
    let mut messages = vec![ChatMessage::user(ENGLISH_PROCEDURE)];

    let Some(AgenticPlan::ToolCalls(write)) = plan_chat_step(&messages, &tools) else {
        panic!("the Formal AI agent path should author the procedure artifact");
    };
    answer_tool_call(
        &mut messages,
        &write[0].tool,
        &write[0].arguments,
        "wrote compiled-procedure.lino",
    );

    let Some(AgenticPlan::ToolCalls(verify)) = plan_chat_step(&messages, &tools) else {
        panic!("the Formal AI agent path should read the artifact back");
    };
    answer_tool_call(
        &mut messages,
        &verify[0].tool,
        &verify[0].arguments,
        "compiled_procedure_artifact \"corrupted\"",
    );

    let Some(AgenticPlan::Final(answer)) = plan_chat_step(&messages, &tools) else {
        panic!("a failed integrity check should stop with an honest final answer");
    };
    assert!(
        answer.contains("verification failed"),
        "the planner must report the integrity failure: {answer}"
    );
    assert!(
        !answer.contains("was written and verified"),
        "a corrupted readback must never be called verified: {answer}"
    );
}

#[test]
fn agentic_planner_localizes_an_honest_unpersisted_artifact() {
    let messages = [ChatMessage::user(RUSSIAN_PROCEDURE)];
    let Some(AgenticPlan::Final(answer)) = plan_chat_step(&messages, &[]) else {
        panic!("a procedure without tools should return its honest seeded response");
    };
    assert!(
        answer.contains("Процедура скомпилирована"),
        "the Agent response should follow the procedure language: {answer}"
    );
    assert!(
        answer.contains("не сохранён и не проверен"),
        "the Agent must not claim persistence without a write tool: {answer}"
    );
    assert!(
        answer.contains(&compile(RUSSIAN_PROCEDURE).id),
        "the unpersisted artifact should remain inspectable: {answer}"
    );
}

#[test]
fn whole_task_uses_one_artifact_across_solver_interpreter_explanation_and_agent() {
    let solver = UniversalSolver::default();
    let response = solver.solve(ENGLISH_PROCEDURE);
    let artifact = extract_compiled_procedure_artifact(&response.answer)
        .expect("symbolic solver should publish an executable artifact");
    let run = artifact
        .execute("https://example.com/article", &mut TracingHost)
        .expect("published artifact should execute");
    assert_eq!(run.outcomes.len(), 4);

    let agent = run_agentic_task(ENGLISH_PROCEDURE).expect("agent execution");
    let write: serde_json::Value =
        serde_json::from_str(&agent.steps[0].arguments).expect("agent write arguments");
    assert_eq!(write["content"], artifact.artifact_links_notation());

    let history = [formal_ai::ConversationTurn::assistant(response.answer)];
    let why = solver.solve_with_history("Why did you do that?", &history);
    assert!(why.answer.contains(&artifact.steps[0].source_text));
    assert!(why.answer.contains(&artifact.steps[3].source_text));
}
