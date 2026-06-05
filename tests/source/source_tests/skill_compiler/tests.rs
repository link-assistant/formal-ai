use super::{compile_natural_language_skill, looks_like_skill_description, SkillCompileError};

#[test]
fn unsupported_shape_is_rejected() {
    let err = compile_natural_language_skill("This is only a note.")
        .expect_err("free text should not compile");
    assert_eq!(err, SkillCompileError::UnsupportedShape);
}

// This truth table is shared, case for case, with the browser-worker parity
// harness experiments/issue-386-worker-skill-trigger-parity.mjs. Both runtimes
// now read every trigger lead, response verb, edit directive, and when-then
// frame from the same embedded meaning lexicon
// (data/seed/meanings-skill-compiler.lino) by semantic role, so locking the
// Rust side here and the JS side there proves the conversion preserved the
// recogniser exactly — including the surfaces the worker used to miss
// ("when the user says", "when the user asks", "respond").
#[test]
fn skill_description_recogniser_reads_every_language_from_the_lexicon() {
    // Explicit teaching form (trigger lead AND response verb) OR edit
    // directive, plus when-then circumfix frames with backticks on each side.
    let recognised = [
        "When I say `checksum status`, answer `checksum cache is valid.`",
        "When the user says `ping`, respond `pong`",
        "When the user asks `status`, reply `all good`",
        "If I ask `time`, answer `noon`",
        "Add behavior rule: greet politely",
        "Please update behavior rule for greetings",
        "Когда я скажу `привет`, ответь `здравствуй`",
        "Если я спрошу `время`, ответ `полдень`",
        "Добавь правило поведения: будь вежлив",
        "Обнови правило поведения для приветствий",
        "जब मैं कहूँ `नमस्ते` तो उत्तर `नमस्कार`",
        "व्यवहार नियम जोड़ो: विनम्र रहो",
        "当我说`你好`，回答`您好`",
        "添加行为规则：保持礼貌",
        "When `status` then `ok`",
        "When `status` do `report ok`",
        "Когда `привет` тогда `здравствуй`",
        "Если `привет` то `здравствуй`",
        "जब `नमस्ते` तब `नमस्कार`",
        "当 `状态` 时 `一切正常。`",
        "当 `状态`时回答 `一切正常。`",
    ];
    for description in recognised {
        assert!(
            looks_like_skill_description(description),
            "should recognise as a skill description: {description:?}"
        );
    }

    let rejected = [
        "This is only a note.",
        "what is the capital of France",
        // A response verb with no trigger lead and no when-then backticks.
        "Please answer the question",
        "reply to this email",
        // A trigger lead with no response verb and no backticks.
        "When I say hello to people",
        // A when-then frame with NO backticks — structure present, quotes absent.
        "when it rains then it pours",
        // Chinese trigger lead "当我说" (no spaces) with no response verb and
        // no when-then frame: the head "当 " needs a space after 当.
        "当我说你好",
    ];
    for description in rejected {
        assert!(
            !looks_like_skill_description(description),
            "should NOT recognise as a skill description: {description:?}"
        );
    }
}
