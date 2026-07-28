//! Agent-CLI execution of arbitrary natural-language procedures (issue #674).
//!
//! The symbolic solver and the agent surface call the same compiler. Agent mode
//! materializes the complete artifact in its workspace, reads it back through a
//! shell tool, and only then returns the inspectable restatement.

use serde_json::json;

use super::capability_router::tool_for;
use super::planner::{plan_one, write_arguments, AgenticPlan, Capability};
use super::progress::Progress;
use crate::language::detect as detect_language;
use crate::protocol::ChatMessage;
use crate::seed;
use crate::skill_procedure::{
    compile_procedure, extract_compiled_procedure_artifact, CompiledProcedure,
    PROCEDURE_CONFORMANCE_TRIGGER,
};

pub const COMPILED_PROCEDURE_PATH: &str = "compiled-procedure.lino";
const EXECUTION_PLACEHOLDER: &str = concat!("{", "execution", "}");

#[must_use]
pub fn compile_task(task: &str) -> Option<CompiledProcedure> {
    compile_procedure(task).ok()
}

pub(super) fn plan_step(
    messages: &[ChatMessage],
    tool_names: &[&str],
    procedure: &CompiledProcedure,
) -> AgenticPlan {
    let document = procedure.artifact_links_notation();
    let progress = Progress::scan(messages);
    let write_tool = tool_for(tool_names, Capability::Write);
    if let Some(tool) = write_tool.filter(|_| !progress.done(Capability::Write)) {
        return plan_one(tool, write_arguments(COMPILED_PROCEDURE_PATH, &document));
    }
    if write_tool.is_none() {
        return AgenticPlan::Final(render_response(
            "agent_procedure_write_unavailable",
            procedure,
            &document,
            "",
        ));
    }

    let run_tool = tool_for(tool_names, Capability::Run);
    if let Some(tool) = run_tool.filter(|_| progress.run_outputs.is_empty()) {
        let mut command = String::from("cat");
        command.push(' ');
        command.push_str(COMPILED_PROCEDURE_PATH);
        return plan_one(tool, json!({ "command": command }).to_string());
    }
    if run_tool.is_none() {
        return AgenticPlan::Final(render_response(
            "agent_procedure_readback_unavailable",
            procedure,
            &document,
            "",
        ));
    }

    let artifact_verified = progress
        .run_outputs
        .first()
        .and_then(|output| extract_compiled_procedure_artifact(output).ok())
        .is_some_and(|restored| restored == *procedure);
    if !artifact_verified {
        return AgenticPlan::Final(render_response(
            "agent_procedure_verification_failed",
            procedure,
            &document,
            "",
        ));
    }

    let expected_execution = procedure.conformance_links_notation(PROCEDURE_CONFORMANCE_TRIGGER);
    if progress.run_outputs.len() == 1 {
        let command = [
            "formal-ai",
            "procedure",
            "conformance",
            "--artifact",
            COMPILED_PROCEDURE_PATH,
            "--trigger",
            PROCEDURE_CONFORMANCE_TRIGGER,
        ]
        .join(" ");
        return plan_one(
            run_tool.expect("run tool was checked above"),
            json!({ "command": command }).to_string(),
        );
    }
    let execution_verified = progress
        .run_outputs
        .get(1)
        .is_some_and(|output| output.trim() == expected_execution.trim());
    if !execution_verified {
        return AgenticPlan::Final(render_response(
            "agent_procedure_execution_failed",
            procedure,
            &document,
            "",
        ));
    }

    AgenticPlan::Final(render_response(
        "agent_procedure_executed",
        procedure,
        &document,
        &expected_execution,
    ))
}

fn render_response(
    intent: &str,
    procedure: &CompiledProcedure,
    document: &str,
    execution: &str,
) -> String {
    let language = detect_language(&procedure.source_description);
    seed::response_for(intent, language.slug())
        .or_else(|| seed::response_for(intent, "en"))
        .unwrap_or_default()
        .replace("{path}", COMPILED_PROCEDURE_PATH)
        .replace("{procedure_id}", &procedure.id)
        .replace("{artifact}", document)
        .replace(EXECUTION_PLACEHOLDER, execution)
        .replace("{steps}", &procedure.restate_steps())
}
