//! Issue #1175 R3, the last five claim rows: `docs_method_explanation`,
//! `kupi_slona`, `source_conflict`, `github_repository_traffic` and
//! `formalization_request` read the operand their answer is about
//! (`rust/src/capability_routing/claim_operands.rs`), so every handler of the
//! precedence table carries a claim row. Held-out probes in en, ru, hi and zh;
//! mirrored by `rust/tests/web/issue-1175-claim-routing.test.mjs`.

use formal_ai::capability_routing::{
    ClaimAdmission, claim_admission, claim_operands, claim_rows, refusal_recorded,
};
use formal_ai::event_log::EventLog;
use formal_ai::rule_interpreter::run_handler;

#[test]
fn last_five_rows_every_precedence_handler_carries_a_claim_row() {
    for handler in formal_ai::seed::handler_precedence() {
        assert!(
            claim_rows().iter().any(|row| row.handler == handler),
            "handler `{handler}` has no claim row"
        );
    }
}

fn operands(kind: &str, prompt: &str) -> Vec<String> {
    claim_operands(kind, prompt).unwrap_or_else(|| panic!("`{kind}` is an operand kind"))
}

#[test]
fn last_five_rows_the_documented_method_is_read_from_the_prompt() {
    for prompt in [
        "how does pandas DataFrame.join work?",
        "как работает pandas DataFrame.join?",
        "pandas DataFrame.join कैसे काम करता है?",
        "pandas DataFrame.join 如何工作？",
    ] {
        assert_eq!(
            operands("documented_method", prompt)
                .first()
                .map(String::as_str),
            Some("pandas.DataFrame.join"),
            "{prompt}"
        );
        assert_eq!(
            claim_admission("docs_method_explanation", prompt, &prompt.to_lowercase()),
            ClaimAdmission::Full,
            "{prompt}"
        );
    }
    // An explanation lead and the project without the documented method.
    for prompt in [
        "Explain pandas DataFrame.merge",
        "объясни pandas DataFrame.merge",
        "pandas DataFrame.merge कैसे काम करता है?",
        "pandas DataFrame.merge 如何工作？",
    ] {
        assert_eq!(
            claim_admission("docs_method_explanation", prompt, &prompt.to_lowercase()),
            ClaimAdmission::Denied,
            "{prompt}"
        );
    }
}

#[test]
fn last_five_rows_the_idiom_claims_only_as_the_whole_utterance() {
    for (prompt, idiom) in [
        ("Hey, buy an elephant!", "buy an elephant"),
        ("Ну купи слона, пожалуйста", "купи слона"),
        ("चलो हाथी खरीदो ना", "हाथी खरीदो"),
        ("快买大象吧", "买大象"),
    ] {
        assert_eq!(
            operands("idiom_utterance", prompt),
            vec![idiom.to_owned()],
            "{prompt}"
        );
        let mut log = EventLog::new();
        assert_eq!(
            run_handler("kupi_slona", prompt, &prompt.to_lowercase(), &mut log)
                .map(|answer| answer.intent),
            Some("kupi_slona".to_owned()),
            "{prompt}"
        );
    }
    for prompt in [
        "Where can I buy an elephant figurine?",
        "Мама сказала: купи слона в магазине игрушек",
        "दुकान से हाथी खरीदो और घर लाओ",
        "我想在动物园买大象玩具",
    ] {
        assert_eq!(
            claim_admission("kupi_slona", prompt, &prompt.to_lowercase()),
            ClaimAdmission::Denied,
            "{prompt}"
        );
    }
}

#[test]
fn last_five_rows_a_source_conflict_names_its_attributed_alternatives() {
    for (prompt, first, second) in [
        (
            "The sources conflict: Wikipedia says Tesla was born in 1856, but an old almanac says 1857.",
            "Wikipedia says Tesla was born in 1856",
            "an old almanac says 1857",
        ),
        (
            "Источники противоречат друг другу: по данным Википедии он родился в 1880 году, а по данным Британники в 1881.",
            "по данным Википедии он родился в 1880 году",
            "по данным Британники в 1881",
        ),
        (
            "स्रोतों में विरोधाभास है: विकिपीडिया के अनुसार वह 1880 में पैदा हुआ, लेकिन ब्रिटानिका के अनुसार 1881 में।",
            "विकिपीडिया के अनुसार वह 1880 में पैदा हुआ",
            "ब्रिटानिका के अनुसार 1881 में",
        ),
        (
            "来源之间有矛盾：维基百科说他生于1880年，大英百科说他生于1881年。",
            "维基百科说他生于1880年",
            "大英百科说他生于1881年",
        ),
    ] {
        assert_eq!(
            claim_admission("source_conflict", prompt, &prompt.to_lowercase()),
            ClaimAdmission::Full,
            "{prompt}"
        );
        let mut log = EventLog::new();
        let answer = run_handler("source_conflict", prompt, &prompt.to_lowercase(), &mut log)
            .unwrap_or_else(|| panic!("an attributed conflict is answered: {prompt}"));
        assert!(
            answer.answer.contains(first) && answer.answer.contains(second),
            "the answer names both alternatives: {}",
            answer.answer
        );
    }
    // The cue without attributed alternatives is admitted to refuse only,
    // and the handler's answer there is its named refusal.
    for prompt in [
        "The sources conflict on this question.",
        "Источники противоречат друг другу.",
        "स्रोतों में विरोधाभास है।",
        "来源之间有矛盾。",
    ] {
        assert_eq!(
            claim_admission("source_conflict", prompt, &prompt.to_lowercase()),
            ClaimAdmission::RefusalOnly,
            "{prompt}"
        );
        let mut log = EventLog::new();
        assert!(
            run_handler("source_conflict", prompt, &prompt.to_lowercase(), &mut log).is_some(),
            "{prompt}"
        );
        assert!(
            refusal_recorded("source_conflict", log.events()),
            "{prompt}"
        );
    }
}

#[test]
fn last_five_rows_a_traffic_answer_names_the_repository_the_prompt_names() {
    for (prompt, repository) in [
        (
            "Can I see who visited my GitHub repository konard/formal-ai?",
            "konard/formal-ai",
        ),
        (
            "Можно ли узнать, кто заходил в репозиторий facebook/react на GitHub?",
            "facebook/react",
        ),
        (
            "क्या मैं जान सकता हूँ कि GitHub रेपो rust-lang/rust में कौन आया?",
            "rust-lang/rust",
        ),
        (
            "能知道谁访问过 GitHub 仓库 vercel/next.js 吗？",
            "vercel/next.js",
        ),
        // The assistant's own repository, addressed by a second-person possessive.
        (
            "можно ли узнать заходил ли кто либо в твое репо на github?",
            "link-assistant/formal-ai",
        ),
    ] {
        assert_eq!(
            operands("named_repository", prompt),
            vec![repository.to_owned()],
            "{prompt}"
        );
        let mut log = EventLog::new();
        let answer = run_handler(
            "github_repository_traffic",
            prompt,
            &prompt.to_lowercase(),
            &mut log,
        )
        .unwrap_or_else(|| panic!("a traffic question is answered: {prompt}"));
        assert!(answer.answer.contains(repository), "{}", answer.answer);
    }
    // "My repository" names none: the answer stays generic, through the
    // refusal lane, and never names another repository.
    for prompt in [
        "Can I know who visited my GitHub repo?",
        "Можно ли узнать, кто заходил в мой репозиторий на GitHub?",
        "क्या मैं जान सकता हूँ कि मेरे GitHub रेपो में कौन आया?",
        "能知道谁访问过我的 GitHub 仓库吗？",
    ] {
        assert_eq!(
            claim_admission("github_repository_traffic", prompt, &prompt.to_lowercase()),
            ClaimAdmission::RefusalOnly,
            "{prompt}"
        );
        let mut log = EventLog::new();
        let answer = run_handler(
            "github_repository_traffic",
            prompt,
            &prompt.to_lowercase(),
            &mut log,
        )
        .unwrap_or_else(|| panic!("a traffic question is answered: {prompt}"));
        assert!(
            !answer.answer.contains("link-assistant/formal-ai"),
            "no wrong repository: {}",
            answer.answer
        );
        assert!(
            refusal_recorded("github_repository_traffic", log.events()),
            "{prompt}"
        );
    }
}

#[test]
fn last_five_rows_a_formalization_claims_only_with_a_statement() {
    for prompt in [
        "Formalize: all humans are mortal",
        "Формализуй: все люди смертны",
        "औपचारिक बनाओ: हर छात्र जो पढ़ता है, परीक्षा पास करता है",
        "形式化：每个学习的学生都通过考试",
    ] {
        assert_eq!(
            claim_admission("formalization_request", prompt, &prompt.to_lowercase()),
            ClaimAdmission::Full,
            "{prompt}"
        );
    }
    for prompt in [
        "Formalize this in Lean",
        "Формализуй это",
        "इसे औपचारिक बनाओ",
        "请形式化这个",
    ] {
        assert_eq!(
            claim_admission("formalization_request", prompt, &prompt.to_lowercase()),
            ClaimAdmission::RefusalOnly,
            "{prompt}"
        );
        let mut log = EventLog::new();
        let answer = formal_ai::solver_handlers::handle_formalization_request(
            prompt,
            &prompt.to_lowercase(),
            &mut log,
        )
        .unwrap_or_else(|| panic!("the cue is refused by name: {prompt}"));
        assert_eq!(answer.intent, "formalization", "{prompt}");
        assert!(
            refusal_recorded("formalization_request", log.events()),
            "{prompt}"
        );
    }
}
