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

/// The table from the issue: one question, four languages, one intent.
#[test]
fn a_bare_documentation_question_answers_the_same_in_every_language() {
    let english = answer("how does pandas DataFrame.join work?");
    assert_eq!(english.intent, "docs_method_explanation");
    assert!(english.answer.contains("official pandas docs"));

    for (language, prompt) in [
        ("ru", "как работает pandas DataFrame.join?"),
        ("hi", "pandas DataFrame.join कैसे काम करता है?"),
        ("zh", "pandas DataFrame.join 如何工作？"),
    ] {
        let response = answer(prompt);
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
