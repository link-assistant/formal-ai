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
use crate::skill_procedure::{compile_procedure, CompiledProcedure, ProcedureCompileError};

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
        Err(ProcedureCompileError::UncompilableStep { step, gap, .. }) => {
            log.append("skill_gap", gap.clone());
            let body = render_procedure_gap(&step, &gap, language);
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

/// The compiled program, its steps, and how to run it.
fn render_compiled_procedure(procedure: &CompiledProcedure, language: &str) -> String {
    let (title, steps_title, hint) = match language {
        "ru" => (
            "Процедура скомпилирована в навык.",
            "Скомпилированные шаги:",
            "Отправьте значение триггера, и я выполню эти шаги по порядку. Спросите «почему ты так сделал?», и я процитирую эти шаги вместе с исходными фрагментами.",
        ),
        "hi" => (
            "प्रक्रिया एक skill में compile की गई.",
            "Compile किए गए steps:",
            "Trigger मान भेजें और मैं इन steps को क्रम से चलाऊँगा. «तुमने ऐसा क्यों किया?» पूछें और मैं ये steps उनके source अंशों सहित उद्धृत करूँगा.",
        ),
        "zh" => (
            "已将该流程编译为技能。",
            "已编译的步骤：",
            "发送触发值，我会按顺序执行这些步骤。问「你为什么这样做？」，我会引用这些步骤及其源文片段。",
        ),
        _ => (
            "Procedure compiled into a skill.",
            "Compiled steps:",
            "Send the trigger value and I will run these steps in order. Ask \"why did you do that?\" and I will cite these steps with their source spans.",
        ),
    };
    format!(
        "{title}\n\n```links\n{}```\n\n{steps_title}\n\n{}\n{hint}",
        procedure.links_notation(),
        procedure.restate_steps()
    )
}

/// The honest named gap: which clause has no compiled capability, and what follows.
fn render_procedure_gap(step: &str, gap: &str, language: &str) -> String {
    let (title, consequence) = match language {
        "ru" => (
            format!("Я не могу скомпилировать шаг «{step}»: {gap}."),
            "Ничего не скомпилировано — я не выполняю процедуру частично. Уберите этот шаг или опишите его через уже поддерживаемые действия.",
        ),
        "hi" => (
            format!("मैं step «{step}» compile नहीं कर सकता: {gap}."),
            "कुछ भी compile नहीं हुआ — मैं procedure को आंशिक रूप से नहीं चलाता. इस step को हटाएँ या पहले से समर्थित actions में बताएँ.",
        ),
        "zh" => (
            format!("我无法编译步骤「{step}」：{gap}。"),
            "没有编译任何内容——我不会部分执行流程。请去掉该步骤，或用已支持的动作重述它。",
        ),
        _ => (
            format!("I cannot compile the step \"{step}\": {gap}."),
            "Nothing was compiled — I do not run a procedure partially. Drop that step or restate it with actions I already support.",
        ),
    };
    format!("{title}\n\n{consequence}")
}
