//! Compile freely phrased multi-step procedures into inspectable skills.
//!
//! Issue #674: `skill_compiler` recognizes a typed trigger/response shape. Procedures
//! stated as ordinary prose ("when I paste a link, fetch its title, translate it to
//! Russian, save both, and reply with the translation") fall outside that shape, so
//! this handler runs `skill_procedure` after the typed compiler declines.
//!
//! Two outcomes are user-visible and both are honest:
//!
//! * every clause maps onto the seeded step vocabulary — the compiled program is
//!   echoed back as Links Notation plus a numbered restatement citing the source
//!   sentence spans, so "why did you do that?" can quote the compiled steps;
//! * one clause has no vocabulary entry — nothing is compiled, the named gap is
//!   reported, and a `skill_gap` event records which capability is missing.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::seed;
use crate::skill_procedure::{
    CompiledProcedure, ProcedureCompileError, ProcedureLearningProposal, compile_procedure,
};

use super::finalize_simple;

/// Compile `prompt` as a multi-step procedure, or decline so later handlers can run.
pub fn try_compiled_procedure(
    prompt: &str,
    log: &mut EventLog,
    language: &str,
) -> Option<SymbolicAnswer> {
    match compile_procedure(prompt) {
        Ok(procedure) => {
            log.append("skill_compile:procedure", procedure.id.clone());
            log.append(
                "skill_compile:procedure_artifact",
                procedure.artifact_links_notation(),
            );
            for step in &procedure.steps {
                log.append(
                    "skill_compile:procedure_step",
                    format!("{} {} {}", step.index, step.kind, step.id),
                );
            }
            let body = render_compiled_procedure(&procedure, language);
            Some(finalize_simple(
                prompt,
                log,
                "compiled_procedure",
                "response:compiled_procedure",
                &body,
                1.0,
            ))
        }
        Err(error @ ProcedureCompileError::UncompilableStep { .. }) => {
            let proposal = ProcedureLearningProposal::from_compile_error(&error)
                .expect("an uncompilable procedure step always produces a proposal");
            let ProcedureCompileError::UncompilableStep { step, gap, .. } = error else {
                unreachable!("matched above")
            };
            log.append("skill_gap", gap.clone());
            log.append("skill_learning_proposal", proposal.id.clone());
            log.append(
                "skill_learning_proposal:artifact",
                proposal.links_notation(),
            );
            let body = render_procedure_gap(&step, &gap, &proposal, language);
            Some(finalize_simple(
                prompt,
                log,
                "skill_gap",
                "response:skill_gap",
                &body,
                1.0,
            ))
        }
        Err(ProcedureCompileError::NotAProcedure) => None,
    }
}

/// Look a response template up by intent, falling back to English (R379).
fn template(intent: &str, language: &str) -> String {
    seed::localized_response(intent, language).unwrap_or_default()
}

/// The compiled program, its steps, and how to run it.
///
/// The prose lives in `data/seed/multilingual-responses-procedure.lino` under the
/// `compiled_procedure` intent; this function fills the complete persisted artifact
/// and the inspectable step restatement.
#[allow(clippy::literal_string_with_formatting_args)]
fn render_compiled_procedure(procedure: &CompiledProcedure, language: &str) -> String {
    template("compiled_procedure", language)
        .replace("{program}", &procedure.artifact_links_notation())
        .replace("{steps}", &procedure.restate_steps())
}

/// The honest named gap: which clause has no compiled capability, and what follows.
///
/// The prose lives under the `skill_gap` intent; this function fills the named
/// gap and the non-executable, human-gated learning proposal.
#[allow(clippy::literal_string_with_formatting_args)]
fn render_procedure_gap(
    step: &str,
    gap: &str,
    proposal: &ProcedureLearningProposal,
    language: &str,
) -> String {
    template("skill_gap", language)
        .replace("{step}", step)
        .replace("{gap}", gap)
        .replace("{proposal}", &proposal.links_notation())
}
