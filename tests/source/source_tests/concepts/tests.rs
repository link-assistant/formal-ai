use super::*;

#[test]
fn issue_242_definition_prompt_typo_extracts_dictionary_term() {
    let query = extract_concept_query("what i digress mean?")
        .expect("definition typo should still route to concept lookup");
    assert_eq!(query.term, "digress");
    assert_eq!(query.context, None);
}

#[test]
fn meaning_question_variants_extract_dictionary_terms() {
    for (prompt, expected) in [
        ("what does digress mean?", "digress"),
        ("what do digress mean?", "digress"),
        ("what is digress meaning?", "digress"),
        ("what is the meaning of digress?", "digress"),
        ("what does the word \"digress\" mean?", "digress"),
    ] {
        let query = extract_concept_query(prompt)
            .unwrap_or_else(|| panic!("expected concept query for `{prompt}`"));
        assert_eq!(query.term, expected);
    }
}

#[test]
fn supported_language_meaning_prompts_extract_dictionary_terms() {
    for (language, prompt) in [
        ("en", "what do flibbertigibbet mean?"),
        ("ru", "что означает слово flibbertigibbet?"),
        ("hi", "flibbertigibbet का अर्थ बताओ"),
        ("zh", "flibbertigibbet是什么意思?"),
    ] {
        let query = extract_concept_query(prompt).unwrap_or_else(|| {
            panic!("expected {language} meaning prompt to extract a concept query")
        });
        assert_eq!(query.term, "flibbertigibbet", "{language}");
    }
}
