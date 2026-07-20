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

use formal_ai::{
    compile_procedure, CompiledProcedure, ProcedureCompileError, ProcedureHost, ProcedureStep,
    UniversalSolver,
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
