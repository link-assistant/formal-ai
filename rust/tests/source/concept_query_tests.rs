//! Concept query extraction over terminal punctuation (moved from the unit
//! suite, which cannot reach the crate-private `concepts` module).

#[test]
fn concept_query_trims_ascii_and_full_width_terminal_punctuation() {
    for punctuation in ["?", "？", "!", "！", "。", ",", "，", ";", "；", ":", "："] {
        let prompt = format!("Redis是什么{punctuation}");
        let query = super::extract_concept_query(&prompt)
            .unwrap_or_else(|| panic!("definition query: {prompt}"));
        assert_eq!(query.term, "redis", "{prompt}");
    }
    let query = super::extract_concept_query("What is foo.bar?").unwrap();
    assert_eq!(
        query.term, "foo.bar",
        "internal punctuation is part of the subject"
    );
}
