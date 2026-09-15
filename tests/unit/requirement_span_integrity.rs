//! Preserve literal operands and unknown requirements before semantic discovery.
use formal_ai::intent_formalization::{formalize_intent, ordered_requirement_spans};
use formal_ai::meta_frame::ProblemFrame;

fn clauses(text: &str) -> Vec<String> {
    ordered_requirement_spans(text, &["and", "then", "然后"])
        .into_iter()
        .map(|span| {
            assert_eq!(
                &text[span.source_span.0..span.source_span.1],
                span.source_text
            );
            span.source_text
        })
        .collect()
}

#[test]
fn literal_punctuation_and_conjunctions_remain_inside_their_operand() {
    for literal in [
        "`North, and South!`",
        "\"value: 17; then 29?\"",
        "「你好，然后再见。」",
    ] {
        let text = format!("Print {literal}; preserve the observation.");
        assert_eq!(
            clauses(&text),
            [
                format!("Print {literal}"),
                "preserve the observation".to_owned()
            ]
        );
    }
}

#[test]
fn unnumbered_requirements_are_distinct_and_unknown_text_is_retained() {
    let text = "- Create src/probe.rs\n- Preserve the zorbulation constraint\n- Verify the result";
    assert_eq!(
        clauses(text),
        [
            "Create src/probe.rs",
            "Preserve the zorbulation constraint",
            "Verify the result",
        ]
    );
}

#[test]
fn bare_source_addresses_and_paths_are_not_decomposed_into_words() {
    let text = "Read https://docs.python.org/3/library/functions.html#print; inspect src/tool.rs.";
    assert_eq!(
        clauses(text),
        [
            "Read https://docs.python.org/3/library/functions.html#print",
            "inspect src/tool.rs",
        ]
    );
}

#[test]
fn unicode_case_expansion_does_not_move_requirement_source_offsets() {
    let text = "İnceleme `a,b!`；保留证据。";
    assert_eq!(clauses(text), ["İnceleme `a,b!`", "保留证据"]);
}

#[test]
fn problem_frame_does_not_split_a_literal_before_the_shared_clause_reader() {
    let request = "Print `North, and South!`; preserve the zorbulation constraint.";
    let frame = ProblemFrame::from_formalization(&formalize_intent(request, "en", None));
    assert_eq!(
        frame
            .needs
            .iter()
            .map(|need| need.source_span.as_str())
            .collect::<Vec<_>>(),
        [
            "Print `North, and South!`",
            "preserve the zorbulation constraint"
        ],
    );
}

#[test]
fn list_enumerators_do_not_become_independent_needs() {
    let request =
        "1. Create src/probe.rs\n2. Preserve the zorbulation constraint\n3. Verify the result";
    let expected = [
        "Create src/probe.rs",
        "Preserve the zorbulation constraint",
        "Verify the result",
    ];
    assert_eq!(clauses(request), expected);
    let frame = ProblemFrame::from_formalization(&formalize_intent(request, "en", None));
    assert_eq!(
        frame
            .needs
            .iter()
            .map(|need| need.source_span.as_str())
            .collect::<Vec<_>>(),
        expected,
    );
}

#[test]
fn fenced_multiline_payload_keeps_list_markers_and_sentence_punctuation() {
    let request =
        "Retain ```text\n- North, and South!\n1. URL: https://example.org/a\n```; verify bytes.";
    assert_eq!(
        clauses(request),
        [
            "Retain ```text\n- North, and South!\n1. URL: https://example.org/a\n```",
            "verify bytes",
        ]
    );
}

#[test]
fn coordination_match_cannot_cut_inside_one_expanded_source_character() {
    let request = "Check İ value; preserve Ω.";
    let spans = ordered_requirement_spans(request, &["i", ""]);
    assert_eq!(
        spans
            .iter()
            .map(|span| span.source_text.as_str())
            .collect::<Vec<_>>(),
        ["Check İ value", "preserve Ω"],
    );
}
