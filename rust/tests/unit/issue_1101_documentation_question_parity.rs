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

#[allow(clippy::duplicate_mod)]
#[path = "../fixtures/documentation-method-fixture.rs"]
mod docs_fixture;

/// The table from the issue: one source-backed answer in four languages.
#[test]
fn a_bare_documentation_question_answers_the_same_in_every_language() {
    for (language, prompt) in [
        ("en", "how does pandas DataFrame.join work?"),
        ("ru", "как работает pandas DataFrame.join?"),
        ("hi", "pandas DataFrame.join कैसे काम करता है?"),
        ("zh", "pandas DataFrame.join 如何工作？"),
    ] {
        let response = docs_fixture::documented_method_answer(prompt);
        assert_eq!(response.intent, "docs_method_explanation", "{language}");
        assert_eq!(response.answer, docs_fixture::expected_join(language));
        assert!(
            response.answer.contains(docs_fixture::JOIN_SOURCE),
            "{language}"
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| { link == &format!("source:{}", docs_fixture::JOIN_SOURCE) }),
            "{language}: {:?}",
            response.evidence_links
        );
    }
}

#[test]
fn documentation_capture_resolves_another_method_and_another_publisher() {
    for (prompt, signature, excerpt, symbol, source) in [
        (
            "How does pandas Series.diff work?",
            "Series.diff(periods=1)",
            "First discrete difference of Series elements.",
            "pandas.Series.diff",
            docs_fixture::DIFF_SOURCE,
        ),
        (
            "Explain numpy.ndarray.reshape",
            "ndarray.reshape(shape, /, *, order='C', copy=None)",
            "Returns an array containing the same data with a new shape.",
            "numpy.ndarray.reshape",
            docs_fixture::RESHAPE_SOURCE,
        ),
        (
            "Explain numpy.empty",
            "numpy.empty(shape, dtype=float, order='C')",
            "Return a new array of given shape and type, without initializing entries.",
            "numpy.empty",
            docs_fixture::EMPTY_SOURCE,
        ),
    ] {
        let response = docs_fixture::documented_method_answer(prompt);
        assert_eq!(
            response.answer,
            format!(
                "`{signature}`\n\n{excerpt}\n\n[Official documentation for {symbol}]({source})"
            )
        );
    }
}

#[test]
fn ambiguous_method_name_uses_the_publisher_index_to_ask_for_the_type() {
    let response = docs_fixture::documented_method_answer("how the join method works in pandas");
    assert_eq!(response.confidence, 0.0);
    assert_eq!(
        response.answer,
        "Which `join` method do you mean? The documentation lists pandas.DataFrame.join, pandas.Index.join, pandas.Series.str.join."
    );
    assert!(
        response
            .evidence_links
            .contains(&format!("source:{}", docs_fixture::INDEX_SOURCE))
    );
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
