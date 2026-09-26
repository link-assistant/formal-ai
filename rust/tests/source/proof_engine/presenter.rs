//! Render a [`ProofOutcome`] to the localized markdown text that goes back
//! into the chat response.
//!
//! Every variant of the outcome produces a deterministic, fully spelled-out
//! body so the surface presenter (in `solver_handlers::user_intent`) can
//! just hand it through. We never emit `"I cannot do that"` here — the
//! [`ProofOutcome::PartialPlan`] arm explicitly walks the user through the
//! plan and the missing inputs.

use std::fmt::Write as _;

use crate::proof_engine::types::{Proof, ProofOutcome, ProofRenderConfig, ProofStep, StepKind};

/// Render a finished outcome using the default [`ProofRenderConfig`].
///
/// Thin wrapper around [`render_outcome_with_config`] kept for backwards
/// compatibility with handlers that don't carry an explicit config.
#[must_use]
pub fn render_outcome(outcome: &ProofOutcome, language: &str) -> String {
    render_outcome_with_config(outcome, language, ProofRenderConfig::default())
}

/// Render a finished outcome, honoring the two presentation sliders:
///
/// * `config.guess_probability` controls whether the engine prepends an
///   "Interpretation" header that explains how the prompt was translated into
///   the formal system. High values mean "show me how you interpreted it".
/// * `config.follow_up_probability` controls whether the engine appends a
///   "Clarifying questions" footer that lists what the user still has to
///   confirm before final execution. High values mean "ask me before you
///   commit".
#[must_use]
pub fn render_outcome_with_config(
    outcome: &ProofOutcome,
    language: &str,
    config: ProofRenderConfig,
) -> String {
    let mut body = String::new();
    if config.show_interpretation() {
        body.push_str(&render_interpretation(outcome, language));
        body.push_str("\n\n");
    }
    let core = match outcome {
        ProofOutcome::Proven { proof } => render_proven(proof, language),
        ProofOutcome::Disproven {
            counterexample,
            method,
            partial_proof,
        } => render_disproven(counterexample, *method, partial_proof.as_ref(), language),
        ProofOutcome::PartialPlan {
            plan,
            missing_inputs,
            method,
        } => render_partial_plan(plan, missing_inputs, *method, language),
        ProofOutcome::Inconclusive { reason } => render_inconclusive(reason, language),
    };
    body.push_str(&core);
    if config.ask_follow_ups() {
        if let Some(footer) = render_follow_up_questions(outcome, language) {
            body.push_str("\n\n");
            body.push_str(&footer);
        }
    }
    body
}

/// Localized "Interpretation:" header that explains, in plain language, how
/// the engine translated the prompt into the formal system. Surfaces the
/// pipeline step the engine actually took (arithmetic, library lookup, axiom
/// reduction) so the user can see *why* the proof reads the way it does.
fn render_interpretation(outcome: &ProofOutcome, language: &str) -> String {
    let label = match language {
        "ru" => "Как я понял запрос",
        "hi" => "मैंने प्रश्न को कैसे समझा",
        "zh" => "对问题的理解",
        _ => "How I interpreted the request",
    };
    let detail = match (outcome, language) {
        (ProofOutcome::Proven { proof }, "ru") => format!(
            "трактуем запрос как формальное утверждение «{}» и доказываем методом «{}» в \
             relative-meta-logic.",
            proof.statement,
            proof.method.label("ru"),
        ),
        (ProofOutcome::Proven { proof }, "hi") => format!(
            "प्रश्न को औपचारिक कथन \"{}\" मानकर relative-meta-logic में \"{}\" विधि से \
             प्रमाणित कर रहे हैं।",
            proof.statement,
            proof.method.label("hi"),
        ),
        (ProofOutcome::Proven { proof }, "zh") => format!(
            "把问题视为形式命题“{}”,在 relative-meta-logic 中用“{}”方法证明。",
            proof.statement,
            proof.method.label("zh"),
        ),
        (ProofOutcome::Proven { proof }, _) => format!(
            "treating the request as the formal claim \"{}\" and discharging it by {} inside \
             relative-meta-logic.",
            proof.statement,
            proof.method.label("en"),
        ),
        (ProofOutcome::Disproven { method, .. }, "ru") => format!(
            "трактуем запрос как утверждение, которое нужно опровергнуть; используем {} и \
             приводим контрпример.",
            method.label("ru"),
        ),
        (ProofOutcome::Disproven { method, .. }, "hi") => format!(
            "प्रश्न को खंडन योग्य कथन मानकर {} का उपयोग कर रहे हैं और प्रतिउदाहरण देते हैं।",
            method.label("hi"),
        ),
        (ProofOutcome::Disproven { method, .. }, "zh") => format!(
            "把问题视为应予反驳的断言,用{}并给出反例。",
            method.label("zh"),
        ),
        (ProofOutcome::Disproven { method, .. }, _) => format!(
            "treating the request as a claim to be refuted; applying {} and producing a \
             counterexample.",
            method.label("en"),
        ),
        (ProofOutcome::PartialPlan { method, .. }, "ru") => format!(
            "пока что не хватает входных данных для финального исполнения; {} — выбранный \
             метод. Ниже план: переводим прошедшую информацию в формальную сигнатуру, \
             идентифицируем недостающее, и предлагаем вопросы для уточнения.",
            method.label("ru"),
        ),
        (ProofOutcome::PartialPlan { method, .. }, "hi") => format!(
            "अंतिम निष्पादन के लिए कुछ इनपुट कम हैं; चयनित विधि: {}. नीचे योजना है — उपलब्ध \
             जानकारी को औपचारिक सिग्नेचर में अनुवादित करें, अनुपस्थित इनपुट चिन्हित करें, और \
             स्पष्टीकरण के प्रश्न पूछें।",
            method.label("hi"),
        ),
        (ProofOutcome::PartialPlan { method, .. }, "zh") => format!(
            "尚缺少最终执行所需的输入;选用方法:{}。下方计划:把已有信息翻译\
             为形式签名,标出缺失输入,并给出澄清问题。",
            method.label("zh"),
        ),
        (ProofOutcome::PartialPlan { method, .. }, _) => format!(
            "the prompt is not yet a closed sentence in a fixed axiom set; selected method: \
             {}. The plan below translates what we have into a formal signature, names the \
             missing inputs, and lists clarifying questions for the final execution.",
            method.label("en"),
        ),
        (ProofOutcome::Inconclusive { .. }, "ru") => String::from(
            "запрос синтаксически проходит как претензия, но не редуцируется ни к одному \
             известному инварианту; ниже приведена причина.",
        ),
        (ProofOutcome::Inconclusive { .. }, "hi") => String::from(
            "प्रश्न सिंटैक्टिक रूप से मान्य है पर किसी ज्ञात अपरिवर्ती तक नहीं घटाया जा सका; \
             नीचे कारण दिया है।",
        ),
        (ProofOutcome::Inconclusive { .. }, "zh") => {
            String::from("请求在语法上可作为断言,但无法化归为已知不变量;下面给出原因。")
        }
        (ProofOutcome::Inconclusive { .. }, _) => String::from(
            "the prompt parses as a claim but did not reduce to any known invariant; the \
             reason is given below.",
        ),
    };
    format!("{label}: {detail}")
}

/// Localized "Clarifying questions:" footer. Emitted only when the engine has
/// something genuine to ask — currently for [`ProofOutcome::PartialPlan`] (where
/// `missing_inputs` are the questions) and [`ProofOutcome::Disproven`] (so the
/// user can decide whether to weaken the claim).
fn render_follow_up_questions(outcome: &ProofOutcome, language: &str) -> Option<String> {
    match outcome {
        ProofOutcome::PartialPlan { missing_inputs, .. } if !missing_inputs.is_empty() => {
            Some(format_follow_up_questions(missing_inputs, language))
        }
        ProofOutcome::Disproven { .. } => {
            let label = follow_up_label(language);
            let questions = disproven_follow_up_questions(language);
            Some(format_follow_up_list(label, &questions))
        }
        ProofOutcome::Inconclusive { .. } => {
            let label = follow_up_label(language);
            let questions = inconclusive_follow_up_questions(language);
            Some(format_follow_up_list(label, &questions))
        }
        _ => None,
    }
}

fn format_follow_up_questions(missing_inputs: &[String], language: &str) -> String {
    let label = follow_up_label(language);
    let intro = match language {
        "ru" => "Чтобы перейти к финальному исполнению, уточните, пожалуйста:",
        "hi" => "अंतिम निष्पादन के लिए कृपया स्पष्ट करें:",
        "zh" => "为进入最终执行,请回答:",
        _ => "To move to the final execution, please clarify:",
    };
    let mut body = format!("{label}\n{intro}\n");
    for (index, q) in missing_inputs.iter().enumerate() {
        let _ = writeln!(body, "{n}. {q}", n = index + 1);
    }
    body.trim_end().to_owned()
}

fn format_follow_up_list(label: &str, questions: &[String]) -> String {
    let mut body = format!("{label}\n");
    for (index, q) in questions.iter().enumerate() {
        let _ = writeln!(body, "{n}. {q}", n = index + 1);
    }
    body.trim_end().to_owned()
}

fn follow_up_label(language: &str) -> &'static str {
    match language {
        "ru" => "Уточняющие вопросы:",
        "hi" => "स्पष्टीकरण के प्रश्न:",
        "zh" => "澄清问题:",
        _ => "Clarifying questions:",
    }
}

fn disproven_follow_up_questions(language: &str) -> Vec<String> {
    match language {
        "ru" => vec![
            String::from(
                "хотите ли вы ослабить утверждение до проверяемой формы (например, заменить \
                 равенство неравенством или ограничить область)?",
            ),
            String::from(
                "если требуется ровно это утверждение, нужно ли добавить аксиому, при которой \
                 контрпример исключается?",
            ),
        ],
        "hi" => vec![
            String::from(
                "क्या आप कथन को जाँचने योग्य रूप तक शिथिल करना चाहते हैं (जैसे समता को \
                 असमिका से बदलना या क्षेत्र सीमित करना)?",
            ),
            String::from(
                "यदि वही कथन ज़रूरी है, क्या आप कोई अभिगृहीत जोड़ना चाहते हैं जिससे \
                 प्रतिउदाहरण बाहर रहे?",
            ),
        ],
        "zh" => vec![
            String::from("是否希望把命题弱化为可证形式(例如把等式改为不等式,或限制定义域)?"),
            String::from("若需保留原命题,是否要新增一条公理以排除该反例?"),
        ],
        _ => vec![
            String::from(
                "do you want to weaken the claim into a checkable form (e.g. replace equality \
                 with an inequality, or restrict the domain)?",
            ),
            String::from(
                "if the exact claim is required, should we add an axiom under which the \
                 counterexample is excluded?",
            ),
        ],
    }
}

fn inconclusive_follow_up_questions(language: &str) -> Vec<String> {
    match language {
        "ru" => vec![
            String::from("какой именно факт нужно проверить (одно предложение)?"),
            String::from("в какой системе аксиом (PA, ZFC, ньютоновская механика, …)?"),
            String::from("есть ли предпочитаемая техника доказательства?"),
        ],
        "hi" => vec![
            String::from("कौन-सा कथन परीक्षणीय है (एक वाक्य में)?"),
            String::from("किस अभिगृहीत प्रणाली में (PA, ZFC, न्यूटनीय यांत्रिकी, …)?"),
            String::from("क्या कोई वांछित प्रमाण-तकनीक है?"),
        ],
        "zh" => vec![
            String::from("准确而言要检验哪一个命题(用一句话表达)?"),
            String::from("在哪一个公理系统中(PA、ZFC、牛顿力学……)?"),
            String::from("是否有偏好的证明技术?"),
        ],
        _ => vec![
            String::from("which exact statement do you want checked (one sentence)?"),
            String::from("in which axiom system (PA, ZFC, Newtonian mechanics, …)?"),
            String::from("do you have a preferred proof technique?"),
        ],
    }
}

fn render_proven(proof: &Proof, language: &str) -> String {
    let heading = match language {
        "ru" => "Доказательство",
        "hi" => "प्रमाण",
        "zh" => "证明",
        _ => "Proof",
    };
    let method_label = proof.method.label(language);
    let statement_label = match language {
        "ru" => "Утверждение",
        "hi" => "कथन",
        "zh" => "命题",
        _ => "Statement",
    };
    let method_intro = match language {
        "ru" => "метод",
        "hi" => "विधि",
        "zh" => "方法",
        _ => "method",
    };
    let mut body = format!(
        "{heading} ({method_intro}: {method_label}).\n\n{statement_label}: {statement}\n",
        statement = proof.statement
    );
    body.push_str(&render_steps(&proof.steps, language));
    body.push('\n');
    body.push_str(&proof.conclusion);
    body
}

fn render_disproven(
    counterexample: &str,
    method: crate::proof_engine::types::ProofMethod,
    partial_proof: Option<&Proof>,
    language: &str,
) -> String {
    let heading = match language {
        "ru" => "Опровержение",
        "hi" => "खंडन",
        "zh" => "反驳",
        _ => "Disproof",
    };
    let counter_label = match language {
        "ru" => "Контрпример",
        "hi" => "प्रतिउदाहरण",
        "zh" => "反例",
        _ => "Counterexample",
    };
    let method_intro = match language {
        "ru" => "метод",
        "hi" => "विधि",
        "zh" => "方法",
        _ => "method",
    };
    let method_label = method.label(language);
    let mut body =
        format!("{heading} ({method_intro}: {method_label}).\n\n{counter_label}: {counterexample}");
    if let Some(proof) = partial_proof {
        body.push_str("\n\n");
        body.push_str(&render_steps(&proof.steps, language));
        body.push('\n');
        body.push_str(&proof.conclusion);
    }
    body
}

fn render_partial_plan(
    plan: &[ProofStep],
    missing_inputs: &[String],
    method: crate::proof_engine::types::ProofMethod,
    language: &str,
) -> String {
    let heading = match language {
        "ru" => "План доказательства",
        "hi" => "प्रमाण योजना",
        "zh" => "证明计划",
        _ => "Proof plan",
    };
    let missing_label = match language {
        "ru" => "Нужно от вас",
        "hi" => "आपसे चाहिए",
        "zh" => "需要您提供",
        _ => "Still needed from you",
    };
    let method_intro = match language {
        "ru" => "метод",
        "hi" => "विधि",
        "zh" => "方法",
        _ => "method",
    };
    let method_label = method.label(language);
    let mut body = format!("{heading} ({method_intro}: {method_label}).\n\n");
    body.push_str(&render_steps(plan, language));
    if !missing_inputs.is_empty() {
        body.push_str("\n\n");
        body.push_str(missing_label);
        body.push_str(":\n");
        for input in missing_inputs {
            body.push_str("- ");
            body.push_str(input);
            body.push('\n');
        }
    }
    body
}

fn render_inconclusive(reason: &str, language: &str) -> String {
    let heading = match language {
        "ru" => "Неокончательный результат",
        "hi" => "अनिर्णायक परिणाम",
        "zh" => "结论待定",
        _ => "Inconclusive result",
    };
    format!("{heading}.\n\n{reason}")
}

fn render_steps(steps: &[ProofStep], language: &str) -> String {
    let mut body = String::new();
    for (index, step) in steps.iter().enumerate() {
        let label = step.kind.label(language);
        let _ = write!(
            body,
            "\n{number}. {label}: {text}",
            number = index + 1,
            text = step.text
        );
        // Add a trailing blank line between top-level kinds for readability,
        // but not between two inferences in a row (they read as one chain).
        if matches!(step.kind, StepKind::Conclusion) {
            body.push('\n');
        }
    }
    body
}

#[path = "../source_tests/proof_engine/presenter/tests.rs"]
mod tests;
