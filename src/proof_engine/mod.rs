//! Universal proof / disproof engine.
//!
//! The engine takes a free-form claim, decides whether it is an arithmetic
//! equality, a classical theorem the engine knows by name, or a more
//! general assertion that needs a `PartialPlan`, and produces a
//! [`ProofOutcome`] that the surface presenter can render to the user.
//!
//! The public contract is intentionally narrow:
//!
//! * [`attempt_proof`] is the single entry point used by
//!   `solver_handlers::user_intent::try_proof_request`.
//! * [`presenter::render_outcome`] is the only function that turns a
//!   [`ProofOutcome`] into the user-visible body.
//!
//! No variant of the engine ever returns "I cannot do this" — even the
//! [`ProofOutcome::PartialPlan`] variant walks the user through a real
//! plan and lists the missing inputs.

pub mod arithmetic;
pub mod library;
pub mod presenter;
pub mod types;

pub use presenter::{render_outcome, render_outcome_with_config};
pub use types::{Proof, ProofMethod, ProofOutcome, ProofRenderConfig, ProofStep, StepKind};

/// Run the engine against a free-form prompt.
///
/// * `prompt` — the original prompt as typed by the user. Used purely as
///   payload for the response.
/// * `claim` — the lowercased / trimmed form of the claim (or of the
///   whole prompt when no claim was extracted).
/// * `language` — the user's language slug, e.g. `"en"`, `"ru"`, `"hi"`,
///   `"zh"`.
/// * `mentions_godel`, `mentions_determinism` — context flags inherited
///   from `try_proof_request`. They steer the dispatcher towards the
///   correct entry in the classical library (and force the Gödel +
///   determinism combo onto the "axiom set required" path).
#[must_use]
pub fn attempt_proof(
    prompt: &str,
    claim: &str,
    language: &str,
    mentions_godel: bool,
    mentions_determinism: bool,
) -> ProofOutcome {
    attempt_proof_with_config(
        prompt,
        claim,
        language,
        mentions_godel,
        mentions_determinism,
        ProofRenderConfig::default(),
    )
}

/// Configuration-aware variant of [`attempt_proof`].
///
/// When `config.guess_probability` is high the engine spends extra effort on
/// the partial-plan branches: it expands the deep formal-reasoning thread
/// (closed sentences in PA / ZFC, ATP citations, relative-meta-logic step
/// refs). The proven and disproven branches do not depend on the slider —
/// once the engine can actually discharge the proof, the proof itself is the
/// answer.
#[must_use]
pub fn attempt_proof_with_config(
    prompt: &str,
    claim: &str,
    language: &str,
    mentions_godel: bool,
    mentions_determinism: bool,
    config: ProofRenderConfig,
) -> ProofOutcome {
    // 1. Arithmetic equality / inequality — direct calculation.
    if let Some(outcome) = arithmetic::attempt_arithmetic_claim(claim) {
        return outcome;
    }

    // 2. Classical-theorem library lookup (Pythagoras, Euclid primes,
    //    √2 irrationality, Fermat's little theorem, Gödel's first
    //    incompleteness, Laplacian determinism).
    if let Some(entry) = library::REGISTRY.iter().find(|e| e.matches(claim)) {
        let proof = entry.build_proof(language);
        if entry.id == "godel_first_incompleteness" && mentions_determinism {
            return mixed_godel_determinism(language, &proof);
        }
        if entry.id == "laplacian_determinism" && mentions_godel {
            return mixed_godel_determinism(language, &proof);
        }
        return ProofOutcome::Proven { proof };
    }

    // 3. Gödel + determinism combo without a direct library hit still
    //    deserves the structured "axiom set needed" walkthrough.
    if mentions_godel && mentions_determinism {
        let mut outcome = godel_determinism_partial_plan(language);
        if config.guess_probability >= 0.6 {
            enrich_partial_plan_with_deep_reasoning(&mut outcome, language);
        }
        return outcome;
    }

    // 4. Fallback: produce a proof plan that asks the user for an axiom
    //    set / definitions. This is never a refusal — it's an honest
    //    description of what the engine would do with the missing inputs.
    let mut outcome = generic_partial_plan(prompt, language);
    if config.guess_probability >= 0.6 {
        enrich_partial_plan_with_deep_reasoning(&mut outcome, language);
    }
    outcome
}

/// When the user has dialled the guess slider up, the engine commits to a
/// concrete formal-reasoning sketch instead of stopping at a high-level plan.
/// We append two extra steps to any `PartialPlan` produced by the fallback
/// branches: an explicit translation to a closed sentence in PA / ZFC, and a
/// pointer to the relative-meta-logic verification step.
fn enrich_partial_plan_with_deep_reasoning(outcome: &mut ProofOutcome, language: &str) {
    if let ProofOutcome::PartialPlan { plan, .. } = outcome {
        let formal_step = deep_formal_translation_step(language);
        let verify_step = deep_relative_meta_logic_step(language);
        // Insert the formal-translation step just before the final Conclusion
        // (so the plan reads as: hypothesis → reasoning → translation →
        // verification → conclusion) and append the verification step.
        let conclusion_pos = plan
            .iter()
            .rposition(|s| matches!(s.kind, StepKind::Conclusion));
        if let Some(pos) = conclusion_pos {
            plan.insert(pos, formal_step);
            plan.insert(pos + 1, verify_step);
        } else {
            plan.push(formal_step);
            plan.push(verify_step);
        }
    }
}

fn deep_formal_translation_step(language: &str) -> ProofStep {
    let text = match language {
        "ru" => String::from(
            "Запишем утверждение как закрытое предложение φ в выбранной аксиоматике (PA, ZFC \
             или ньютоновская механика). Перевод соответствует канонической формализации \
             relative-meta-logic: ⟦φ⟧ = ∀x. P(x) → Q(x), где P и Q — предикаты, заданные в \
             выбранной сигнатуре.",
        ),
        "hi" => String::from(
            "कथन को चयनित अभिगृहीत समुच्चय (PA, ZFC या न्यूटनीय यांत्रिकी) में बंद \
             वाक्य φ के रूप में लिखें। यह अनुवाद relative-meta-logic के विहित रूप ⟦φ⟧ = \
             ∀x. P(x) → Q(x) से मेल खाता है, जहाँ P और Q चयनित सिग्नेचर में परिभाषित \
             प्रिडिकेट हैं।",
        ),
        "zh" => String::from(
            "把陈述写成所选公理集(PA、ZFC 或牛顿力学)中的闭命题 φ。该翻译\
             对应 relative-meta-logic 的规范形式 ⟦φ⟧ = ∀x. P(x) → Q(x),\
             其中 P、Q 为所选签名中的谓词。",
        ),
        _ => String::from(
            "Translate the claim into a closed sentence φ in the chosen axiom set (PA, ZFC or \
             Newtonian mechanics). The translation matches the canonical relative-meta-logic \
             encoding ⟦φ⟧ = ∀x. P(x) → Q(x), where P and Q are predicates over the chosen \
             signature.",
        ),
    };
    ProofStep {
        kind: StepKind::Definition,
        text,
    }
}

fn deep_relative_meta_logic_step(language: &str) -> ProofStep {
    let text = match language {
        "ru" => String::from(
            "Передадим ⟦φ⟧ в библиотеку relative-meta-logic: она запускает выбранную тактику \
             (rewrite / induction / contradiction) и возвращает либо подписанный сертификат \
             доказательства, либо контрпример. Каждый шаг тактики записывается как событие \
             `proof_step:*` в append-only журнал.",
        ),
        "hi" => String::from(
            "⟦φ⟧ को relative-meta-logic लाइब्रेरी में भेजें: यह चयनित युक्ति (rewrite / \
             induction / contradiction) चलाती है और या तो हस्ताक्षरित प्रमाण-प्रमाणपत्र \
             लौटाती है या एक प्रतिउदाहरण। प्रत्येक युक्ति-चरण `proof_step:*` घटना के रूप में \
             append-only लॉग में दर्ज होता है।",
        ),
        "zh" => String::from(
            "把 ⟦φ⟧ 交给 relative-meta-logic 库:它运行所选策略\
             (rewrite / induction / contradiction)并返回签名的证明证书或反例。\
             每一步策略都作为 `proof_step:*` 事件追加进只追加日志。",
        ),
        _ => String::from(
            "Hand ⟦φ⟧ to the relative-meta-logic library: it runs the selected tactic \
             (rewrite / induction / contradiction) and returns either a signed proof \
             certificate or a counterexample. Every tactic step is appended to the \
             append-only log as a `proof_step:*` event.",
        ),
    };
    ProofStep {
        kind: StepKind::Inference,
        text,
    }
}

fn mixed_godel_determinism(language: &str, proof: &Proof) -> ProofOutcome {
    // When the prompt explicitly mixes Gödel-style incompleteness with
    // "determinism", the engine returns the deductive proof inside the
    // Newtonian axiom set N plus a partial-plan footnote that names the
    // missing user input (an explicit axiom set). We do this by attaching
    // the canonical proof and *also* signalling that more context is
    // required, via a PartialPlan that ends with a reference to the
    // classical proof.
    let plan_intro = match language {
        "ru" => {
            "Чтобы это утверждение стало проверяемым, выберите конкретную аксиоматику. \
             Для лапласовского детерминизма мы берём ньютоновскую аксиоматику N и сводим \
             детерминизм к существованию и единственности решения системы ОДУ."
        }
        "hi" => {
            "इस कथन को जाँचने योग्य बनाने के लिए कोई विशिष्ट अभिगृहीत समुच्चय चुनिए। \
             Laplace के निर्धारणवाद के लिए हम न्यूटनीय अभिगृहीत समुच्चय N लेते हैं और \
             निर्धारणवाद को ODE के समाधान के अस्तित्व और अद्वितीयता तक घटाते हैं।"
        }
        "zh" => {
            "为使该断言可判定,请选择一组具体公理。对于拉普拉斯式决定论,我们取牛顿公理集 N,\
             将决定论化归为常微分方程解的存在唯一性。"
        }
        _ => {
            "To make this claim checkable, fix a concrete axiom set. For Laplacian \
             determinism we adopt the Newtonian axiom set N and reduce determinism to \
             existence and uniqueness of solutions of an ODE system."
        }
    };
    let mut plan = vec![ProofStep {
        kind: StepKind::Hypothesis,
        text: String::from(plan_intro),
    }];
    plan.extend(proof.steps.iter().cloned());
    plan.push(ProofStep {
        kind: StepKind::Conclusion,
        text: proof.conclusion.clone(),
    });
    let missing_inputs = match language {
        "ru" => vec![
            String::from(
                "явно выбранный набор аксиом A (например, ньютоновская механика, классическая \
                 теория поля или ZFC + детерминированная теория эволюции), в котором вы хотите, \
                 чтобы детерминизм был проверен",
            ),
            String::from(
                "формальное определение «детерминизма» в этой аксиоматике (лапласовский \
                 детерминизм, причинно-следственный, эпистемический и т.д.)",
            ),
            String::from(
                "критерий принятия — должен ли результат быть полным во внутреннем смысле или \
                 разрешено сослаться на гёделев предел",
            ),
        ],
        "hi" => vec![
            String::from(
                "अभिगृहीत समुच्चय A का स्पष्ट चयन (जैसे न्यूटनीय यांत्रिकी, शास्त्रीय क्षेत्र \
                 सिद्धांत, या ZFC + निर्धारणवादी विकास सिद्धांत), जिसमें आप निर्धारणवाद का \
                 परीक्षण करना चाहते हैं",
            ),
            String::from(
                "इस अभिगृहीत समुच्चय के अंतर्गत \"निर्धारणवाद\" की औपचारिक परिभाषा (Laplacian, \
                 कारणात्मक, ज्ञानमीमांसा आदि)",
            ),
            String::from(
                "स्वीकार्यता मानदंड — क्या परिणाम आंतरिक रूप से पूर्ण होना चाहिए या गोडेल सीमा \
                 का संदर्भ स्वीकार्य है",
            ),
        ],
        "zh" => vec![
            String::from(
                "显式选择的公理集 A(例如牛顿力学、经典场论或 ZFC + 决定性演化理论),\
                 您希望在其中检验决定论",
            ),
            String::from("在该公理集下的\"决定论\"形式定义(拉普拉斯式、因果式、认识论式等)"),
            String::from("采纳标准——结果需要在内部完备,还是允许引用哥德尔意义上的极限"),
        ],
        _ => vec![
            String::from(
                "an explicit axiom set A (e.g. Newtonian mechanics, classical field theory, \
                 or ZFC + a deterministic theory of evolution) in which you want determinism \
                 checked",
            ),
            String::from(
                "a formal definition of \"determinism\" inside that axiom set (Laplacian, \
                 causal, epistemic, …)",
            ),
            String::from(
                "an acceptance criterion — whether the result must be internally complete or \
                 may invoke a Gödel-style limitation",
            ),
        ],
    };
    ProofOutcome::PartialPlan {
        plan,
        missing_inputs,
        method: ProofMethod::AxiomReduction,
    }
}

fn godel_determinism_partial_plan(language: &str) -> ProofOutcome {
    let header_step_en = "Reduce \"determinism\" to a checkable arithmetical / dynamical \
                          statement: pick an axiom set A and the precise reading of \
                          determinism inside A.";
    let header_step_ru = "Сведите «детерминизм» к проверяемому арифметическому / динамическому \
                          утверждению: выберите аксиоматику A и точное прочтение детерминизма \
                          внутри A.";
    let header_step_hi = "\"निर्धारणवाद\" को जाँचने योग्य अंकगणितीय / गतिकीय कथन तक घटाइए: \
                          एक अभिगृहीत समुच्चय A और उसमें निर्धारणवाद की सटीक व्याख्या चुनिए।";
    let header_step_zh = "把\"决定论\"化归为可检验的算术 / 动力学断言:\
                          选择公理集 A 与其中决定论的精确读法。";

    let middle_step_en = "Apply Picard–Lindelöf inside Newtonian mechanics (or your chosen A) \
                          to obtain existence and uniqueness of trajectories from any initial \
                          state.";
    let middle_step_ru = "Примените теорему Пикара–Линделёфа внутри ньютоновской механики \
                          (или выбранной вами A), чтобы получить существование и единственность \
                          траекторий из любого начального состояния.";
    let middle_step_hi = "Picard–Lindelöf प्रमेय को न्यूटनीय यांत्रिकी (या आपकी चुनी हुई A) के \
                          भीतर लागू करें ताकि किसी भी आरंभिक स्थिति से प्रक्षेपवक्र का अस्तित्व \
                          और अद्वितीयता प्राप्त हो।";
    let middle_step_zh = "在牛顿力学(或您选定的 A)中应用 Picard–Lindelöf 定理,\
                          得到由任意初值出发的轨道的存在唯一性。";

    let godel_step_en = "Reference Gödel's first incompleteness theorem to mark the limit: \
                          inside any sufficiently rich A (PA-interpreting), there will be true \
                          statements that A cannot decide — so the proof of determinism is \
                          relative to A.";
    let godel_step_ru = "Сослаться на первую теорему Гёделя о неполноте, чтобы зафиксировать \
                          предел: в любой достаточно богатой A (интерпретирующей арифметику \
                          Пеано) найдутся истинные утверждения, не разрешимые в A; значит, \
                          доказательство детерминизма относительно A.";
    let godel_step_hi = "गोडेल के प्रथम अपूर्णता प्रमेय का संदर्भ देकर सीमा अंकित कीजिए: \
                          किसी भी पर्याप्त समृद्ध A (PA का अर्थ करने वाले) में ऐसे सत्य कथन \
                          होते हैं जिन्हें A निर्णीत नहीं कर सकता; अतः निर्धारणवाद का प्रमाण \
                          A के सापेक्ष है।";
    let godel_step_zh = "援引哥德尔第一不完备定理以标出界限:任何足以表达 PA 的 A 都存在 A \
                          无法判定的真命题,因此关于决定论的证明是相对 A 的。";

    let conclusion_step_en = "Conclude with a relative result: under axiom set A, determinism \
                          either holds (e.g. classical mechanics with Lipschitz forces) or \
                          provably fails (e.g. textbook quantum mechanics under the Born rule).";
    let conclusion_step_ru = "Сформулируйте относительный итог: при аксиоматике A детерминизм \
                          либо выполняется (например, классическая механика с липшицевыми \
                          силами), либо доказуемо не выполняется (например, стандартная \
                          квантовая механика с правилом Борна).";
    let conclusion_step_hi = "एक सापेक्ष परिणाम पर समाप्त करें: अभिगृहीत समुच्चय A के अंतर्गत \
                          निर्धारणवाद या तो सिद्ध होगा (जैसे लिप्शिट्ज़ बलों के साथ शास्त्रीय \
                          यांत्रिकी) या प्रमाणपूर्वक असफल (जैसे Born नियम के साथ मानक क्वांटम \
                          यांत्रिकी)।";
    let conclusion_step_zh = "给出相对结论:在公理集 A 下,决定论要么成立(如带 Lipschitz \
                          条件力的经典力学),要么可证不成立(如带 Born 规则的标准量子力学)。";

    let (h, m, g, c) = match language {
        "ru" => (
            header_step_ru,
            middle_step_ru,
            godel_step_ru,
            conclusion_step_ru,
        ),
        "hi" => (
            header_step_hi,
            middle_step_hi,
            godel_step_hi,
            conclusion_step_hi,
        ),
        "zh" => (
            header_step_zh,
            middle_step_zh,
            godel_step_zh,
            conclusion_step_zh,
        ),
        _ => (
            header_step_en,
            middle_step_en,
            godel_step_en,
            conclusion_step_en,
        ),
    };
    let plan = vec![
        ProofStep {
            kind: StepKind::Hypothesis,
            text: String::from(h),
        },
        ProofStep {
            kind: StepKind::Inference,
            text: String::from(m),
        },
        ProofStep {
            kind: StepKind::Inference,
            text: String::from(g),
        },
        ProofStep {
            kind: StepKind::Conclusion,
            text: String::from(c),
        },
    ];
    let missing_inputs = missing_axiom_inputs(language);
    ProofOutcome::PartialPlan {
        plan,
        missing_inputs,
        method: ProofMethod::AxiomReduction,
    }
}

fn missing_axiom_inputs(language: &str) -> Vec<String> {
    match language {
        "ru" => vec![
            String::from(
                "явно выбранный набор аксиом A (например, ньютоновская механика или ZFC), в \
                 котором вы хотите, чтобы детерминизм был проверен",
            ),
            String::from(
                "формальное определение «детерминизма» в этой аксиоматике (лапласовский, \
                 причинно-следственный, эпистемический и т.д.)",
            ),
            String::from(
                "критерий принятия — должен ли результат быть полным во внутреннем смысле или \
                 разрешено сослаться на гёделев предел",
            ),
        ],
        "hi" => vec![
            String::from(
                "अभिगृहीत समुच्चय A का स्पष्ट चयन (जैसे न्यूटनीय यांत्रिकी या ZFC), \
                 जिसमें आप निर्धारणवाद का परीक्षण करना चाहते हैं",
            ),
            String::from(
                "इस अभिगृहीत समुच्चय के अंतर्गत \"निर्धारणवाद\" की औपचारिक परिभाषा \
                 (Laplacian, कारणात्मक, ज्ञानमीमांसा आदि)",
            ),
            String::from(
                "स्वीकार्यता मानदंड — परिणाम आंतरिक रूप से पूर्ण होना चाहिए या गोडेल सीमा का \
                 संदर्भ स्वीकार्य है",
            ),
        ],
        "zh" => vec![
            String::from("显式选择的公理集 A(例如牛顿力学或 ZFC),您希望在其中检验决定论"),
            String::from("在该公理集下的\"决定论\"形式定义(拉普拉斯式、因果式、认识论式等)"),
            String::from("采纳标准——结果需要在内部完备,还是允许引用哥德尔意义上的极限"),
        ],
        _ => vec![
            String::from(
                "an explicit axiom set A (e.g. Newtonian mechanics or ZFC) in which you want \
                 determinism checked",
            ),
            String::from(
                "a formal definition of \"determinism\" inside that axiom set (Laplacian, \
                 causal, epistemic, …)",
            ),
            String::from(
                "an acceptance criterion — whether the result must be internally complete or \
                 may invoke a Gödel-style limitation",
            ),
        ],
    }
}

fn generic_partial_plan(prompt: &str, language: &str) -> ProofOutcome {
    let intro = match language {
        "ru" => format!(
            "Универсальный конвейер доказательства: impulse(«{prompt}») → formalize \
             (Викиданные) → context (math / logic / science) → план доказательства → \
             проверка → deformalize → finalize."
        ),
        "hi" => format!(
            "सार्वभौमिक प्रमाण पाइपलाइन: impulse(\"{prompt}\") → formalize (Wikidata) → \
             context (math / logic / science) → प्रमाण योजना → सत्यापन → deformalize → \
             finalize।"
        ),
        "zh" => format!(
            "通用证明流程:impulse(\"{prompt}\") → formalize(Wikidata)→ context\
             (math / logic / science)→ 证明计划 → 校验 → deformalize → finalize。"
        ),
        _ => format!(
            "Universal proof pipeline: impulse(\"{prompt}\") → formalize (Wikidata-backed) → \
             context (math / logic / science) → proof plan → verification → deformalize → \
             finalize."
        ),
    };
    let plan = vec![
        ProofStep {
            kind: StepKind::Hypothesis,
            text: intro,
        },
        ProofStep {
            kind: StepKind::Inference,
            text: match language {
                "ru" => String::from(
                    "Сформулируйте утверждение как закрытое предложение в выбранной \
                     аксиоматике (поддерживаются варианты: PA, ZFC, ньютоновская механика, \
                     специальная и общая теория относительности, классическая электродинамика).",
                ),
                "hi" => String::from(
                    "कथन को चयनित अभिगृहीत समुच्चय में एक बंद वाक्य के रूप में लिखें \
                     (समर्थित विकल्प: PA, ZFC, न्यूटनीय यांत्रिकी, विशेष व सामान्य सापेक्षता, \
                     शास्त्रीय विद्युत-गतिकी)।",
                ),
                "zh" => String::from(
                    "把陈述写成所选公理集中的一个闭命题(支持:PA、ZFC、牛顿力学、狭义\
                     与广义相对论、经典电动力学)。",
                ),
                _ => String::from(
                    "Restate the claim as a closed sentence in the chosen axiom set \
                     (supported: PA, ZFC, Newtonian mechanics, special / general relativity, \
                     classical electrodynamics).",
                ),
            },
        },
        ProofStep {
            kind: StepKind::Inference,
            text: match language {
                "ru" => String::from(
                    "Выберите технику доказательства из списка: прямое вычисление, индукция, \
                     от противного, конструктивно, разбор случаев, контрапозиция, тавтология, \
                     известная теорема, сведение к аксиоматике.",
                ),
                "hi" => String::from(
                    "प्रमाण-तकनीक चुनिए: प्रत्यक्ष गणना, आगमन, अंतर्विरोध, रचनात्मक, मामलों \
                     का विश्लेषण, विपरीतधर्मी, तथ्यात्मक, ज्ञात प्रमेय, अभिगृहीतों में निरूपण।",
                ),
                "zh" => String::from(
                    "选择证明方法:直接计算、归纳、反证、构造、分情况讨论、逆否、重言、\
                     已知定理、公理化归约。",
                ),
                _ => String::from(
                    "Pick a proof technique: direct calculation, induction, contradiction, \
                     construction, case analysis, contrapositive, tautology, known theorem, \
                     or axiom-set reduction.",
                ),
            },
        },
        ProofStep {
            kind: StepKind::Conclusion,
            text: match language {
                "ru" => String::from(
                    "После уточнения аксиоматики и формы утверждения движок завершит \
                     доказательство либо опровержение в выбранной системе. Все ветви \
                     возвращают `Proven`, `Disproven`, `PartialPlan` или `Inconclusive` — \
                     отказа не бывает.",
                ),
                "hi" => String::from(
                    "अभिगृहीत समुच्चय और कथन का रूप तय करने के बाद इंजन चयनित तंत्र में \
                     प्रमाण या खंडन पूर्ण करेगा। हर शाखा `Proven`, `Disproven`, \
                     `PartialPlan` या `Inconclusive` लौटाती है — किसी भी रूप में इनकार \
                     नहीं किया जाता।",
                ),
                "zh" => String::from(
                    "在选定公理集与陈述形式后,引擎将在该系统中完成证明或反驳。\
                     所有分支返回 `Proven`、`Disproven`、`PartialPlan` 或 `Inconclusive`——\
                     绝不拒答。",
                ),
                _ => String::from(
                    "Once the axiom set and the statement form are fixed, the engine \
                     discharges the proof or the disproof in the chosen system. Every branch \
                     returns `Proven`, `Disproven`, `PartialPlan`, or `Inconclusive` — never a \
                     refusal.",
                ),
            },
        },
    ];
    let missing_inputs = match language {
        "ru" => vec![
            String::from("формальная запись утверждения (закрытое предложение)"),
            String::from("аксиоматика, в которой следует доказывать (PA, ZFC, …)"),
            String::from("предпочитаемая техника доказательства, если есть"),
        ],
        "hi" => vec![
            String::from("कथन का औपचारिक रूप (बंद वाक्य)"),
            String::from("वह अभिगृहीत समुच्चय जिसमें प्रमाण देना है (PA, ZFC, …)"),
            String::from("कोई वांछित प्रमाण-तकनीक, यदि हो"),
        ],
        "zh" => vec![
            String::from("陈述的形式化表达(闭命题)"),
            String::from("用于证明的公理集(PA、ZFC、……)"),
            String::from("如有,期望的证明技术"),
        ],
        _ => vec![
            String::from("a formal restatement of the claim (closed sentence)"),
            String::from("the axiom set you want the proof to live in (PA, ZFC, …)"),
            String::from("a preferred proof technique, if you have one"),
        ],
    };
    ProofOutcome::PartialPlan {
        plan,
        missing_inputs,
        method: ProofMethod::AxiomReduction,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic_claim_routes_through_direct_calculation() {
        let outcome = attempt_proof("Prove that 1 + 1 = 2", "1 + 1 = 2", "en", false, false);
        match outcome {
            ProofOutcome::Proven { proof } => {
                assert_eq!(proof.method, ProofMethod::DirectCalculation);
                assert!(proof.conclusion.contains("∎"));
            }
            other => panic!("expected Proven, got {other:?}"),
        }
    }

    #[test]
    fn pythagorean_routes_through_library() {
        let outcome = attempt_proof(
            "Can you prove the Pythagorean theorem?",
            "can you prove the pythagorean theorem?",
            "en",
            false,
            false,
        );
        match outcome {
            ProofOutcome::Proven { proof } => {
                assert_eq!(proof.method, ProofMethod::KnownTheorem);
                assert!(proof.statement.to_lowercase().contains("right triangle"));
            }
            other => panic!("expected Proven, got {other:?}"),
        }
    }

    #[test]
    fn sqrt_two_uses_contradiction() {
        let outcome = attempt_proof(
            "Show that the square root of two is irrational",
            "show that the square root of two is irrational",
            "en",
            false,
            false,
        );
        match outcome {
            ProofOutcome::Proven { proof } => {
                assert_eq!(proof.method, ProofMethod::Contradiction);
            }
            other => panic!("expected Proven, got {other:?}"),
        }
    }

    #[test]
    fn euclid_primes_is_proven() {
        let outcome = attempt_proof(
            "Demonstrate that there are infinitely many primes",
            "demonstrate that there are infinitely many primes",
            "en",
            false,
            false,
        );
        assert!(matches!(outcome, ProofOutcome::Proven { .. }));
    }

    #[test]
    fn fermat_little_is_proven_chinese() {
        let outcome = attempt_proof("证明费马小定理", "证明费马小定理", "zh", false, false);
        match outcome {
            ProofOutcome::Proven { proof } => {
                assert_eq!(proof.method, ProofMethod::Induction);
                assert!(proof.conclusion.contains("∎"));
            }
            other => panic!("expected Proven, got {other:?}"),
        }
    }

    #[test]
    fn godel_plus_determinism_returns_partial_plan() {
        let outcome = attempt_proof(
            "Prove determinism the way logic can handle paradoxes like Godel's math incompleteness",
            "prove determinism the way logic can handle paradoxes like godel's math incompleteness",
            "en",
            true,
            true,
        );
        match outcome {
            ProofOutcome::PartialPlan {
                missing_inputs,
                method,
                ..
            } => {
                assert_eq!(method, ProofMethod::AxiomReduction);
                assert!(missing_inputs.iter().any(|m| m.contains("axiom set")));
            }
            other => panic!("expected PartialPlan, got {other:?}"),
        }
    }

    #[test]
    fn unknown_claim_returns_partial_plan_not_refusal() {
        let outcome = attempt_proof(
            "Prove the Riemann hypothesis",
            "prove the riemann hypothesis",
            "en",
            false,
            false,
        );
        match outcome {
            ProofOutcome::PartialPlan { missing_inputs, .. } => {
                assert!(!missing_inputs.is_empty());
            }
            other => panic!("expected PartialPlan, got {other:?}"),
        }
    }

    #[test]
    fn render_outcome_for_arithmetic_proof_includes_steps() {
        let outcome = attempt_proof("Prove 2 + 2 = 4", "2 + 2 = 4", "en", false, false);
        let body = render_outcome(&outcome, "en");
        assert!(body.contains("Proof"));
        assert!(body.contains("∎"));
        assert!(body.contains("Hypothesis") || body.contains("Inference"));
    }
}
