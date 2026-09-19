//! Issue #1101: the same documentation question routed to web search in ru, hi
//! and zh but not en.
//!
//! `how does pandas DataFrame.join work?` was answered from the documentation
//! rule; its Russian, Hindi and Chinese translations were answered with the
//! web-search handler's offline-fetch notice. Four languages are meant to be
//! one behaviour.
//!
//! The cause was not the precedence order. `extract_externally_verifiable_question`
//! -- the last branch of the web-search cascade, for an interrogative that names
//! an engineered brand and carries no search imperative -- claimed all four.
//! English alone escaped, and by accident: once the `how` opener is stripped the
//! residual reads `does pandas dataframe join work`, and `does` is seeded as a
//! `non_referential_subject` (the guard that rejects "does it ..."), so the
//! local-context check returned true. Russian, Hindi and Chinese build the same
//! question without do-support, so nothing rescued them.
//!
//! The fallback now asks the rule set whether a documentation rule already
//! answers the prompt, which is language-neutral by construction.

use formal_ai::{FormalAiEngine, SymbolicAnswer};

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

const PANDAS_JOIN_EN: &str = "pandas `DataFrame.join` joins columns from the `other` DataFrame or named Series into the caller and returns a new DataFrame.\n\nScoped to this method: by default, it performs a left join using the caller's index. If `on` is set, pandas matches that caller column or index level against the `other` object's index. The `how` parameter controls key handling (`left`, `right`, `outer`, `inner`, `cross`, `left_anti`, or `right_anti`). Use `lsuffix` and `rsuffix` when column names overlap, `sort` to order join keys, and `validate` to check one-to-one, one-to-many, many-to-one, or many-to-many relationships. For column-on-column joins, the pandas docs point to `DataFrame.merge`.\n\nSource: [pandas.DataFrame.join](https://pandas.pydata.org/docs/reference/api/pandas.DataFrame.join.html) (official pandas docs).";
const PANDAS_JOIN_RU: &str = "pandas `DataFrame.join` добавляет столбцы из `other` DataFrame или именованной Series к вызывающему DataFrame и возвращает новый DataFrame.\n\nВ рамках этого метода: по умолчанию это left join по индексу вызывающего DataFrame. Если задан `on`, pandas сопоставляет этот столбец или уровень индекса с индексом объекта `other`. Параметр `how` управляет объединением ключей (`left`, `right`, `outer`, `inner`, `cross`, `left_anti` или `right_anti`). `lsuffix` и `rsuffix` нужны при совпадающих именах столбцов, `sort` сортирует ключи join, а `validate` проверяет связи one-to-one, one-to-many, many-to-one или many-to-many. Для join столбец-к-столбцу документация pandas указывает на `DataFrame.merge`.\n\nИсточник: [pandas.DataFrame.join](https://pandas.pydata.org/docs/reference/api/pandas.DataFrame.join.html) (официальная документация pandas).";
const PANDAS_JOIN_HI: &str = "pandas `DataFrame.join` कॉल करने वाले DataFrame में `other` DataFrame या named Series के columns जोड़ता है और नया DataFrame लौटाता है.\n\nइस method के दायरे में: default रूप से यह caller के index पर left join करता है. `on` देने पर pandas caller के उस column या index level को `other` object के index से मिलाता है. `how` parameter keys को मिलाने का तरीका चुनता है (`left`, `right`, `outer`, `inner`, `cross`, `left_anti`, या `right_anti`). Column नाम टकराने पर `lsuffix` और `rsuffix`, join keys को sort करने के लिए `sort`, और one-to-one, one-to-many, many-to-one, या many-to-many संबंध जांचने के लिए `validate` इस्तेमाल करें. Column-on-column joins के लिए pandas docs `DataFrame.merge` की ओर भेजते हैं.\n\nSource: [pandas.DataFrame.join](https://pandas.pydata.org/docs/reference/api/pandas.DataFrame.join.html) (official pandas docs).";

/// The table from the issue: one question, four languages, one intent.
#[test]
fn a_bare_documentation_question_answers_the_same_in_every_language() {
    let english = answer("how does pandas DataFrame.join work?");
    assert_eq!(english.answer, PANDAS_JOIN_EN);
    assert_eq!(english.intent, "docs_method_explanation");
    assert!(english.answer.contains("official pandas docs"));

    for (language, prompt, expected) in [
        ("ru", "как работает pandas DataFrame.join?", PANDAS_JOIN_RU),
        ("hi", "pandas DataFrame.join कैसे काम करता है?", PANDAS_JOIN_HI),
        ("zh", "pandas DataFrame.join 如何工作？", PANDAS_JOIN_EN),
    ] {
        let response = answer(prompt);
        assert_eq!(response.answer, expected, "{language}: {prompt:?}");
        assert_eq!(
            response.intent, "docs_method_explanation",
            "{language}: {prompt:?} answered {:?}",
            response.answer
        );
        // The issue asks for the exact answer, not merely the intent: routing
        // to the right handler that then says something different would be the
        // same defect wearing a passing test. The answer is localised, so the
        // invariant is that it is the *documentation* answer -- it cites the
        // official page and never the offline-fetch notice the web-search
        // handler produces.
        assert!(
            response.answer.contains(
                "https://pandas.pydata.org/docs/reference/api/pandas.DataFrame.join.html"
            ),
            "{language}: the answer must cite the official page: {:?}",
            response.answer
        );
        assert!(
            !response.answer.contains("No captured provider response"),
            "{language}: the offline-fetch notice is the defect: {:?}",
            response.answer
        );
    }
}

/// The fix must not swallow a real search request.
#[test]
fn an_explicit_search_imperative_still_reaches_web_search_in_every_language() {
    for (language, prompt) in [
        ("en", "search for pandas DataFrame.join"),
        ("ru", "поищи pandas DataFrame.join"),
        ("hi", "pandas DataFrame.join खोजो"),
        ("zh", "搜索 pandas DataFrame.join"),
    ] {
        let response = answer(prompt);
        assert_eq!(
            response.intent, "web_search",
            "{language}: an explicit imperative must still search: {:?}",
            response.answer
        );
    }
}

/// The guard is the documentation rule, not the English auxiliary that used to
/// stand in for it. A question about something no documentation rule covers is
/// still externally verifiable, so the fallback must keep claiming it.
#[test]
fn the_fallback_still_claims_a_question_no_documentation_rule_answers() {
    let response = answer("как работает ChatGPT?");
    assert_eq!(
        response.intent, "web_search",
        "a brand question with no seeded documentation rule stays a search: {:?}",
        response.answer
    );
}
