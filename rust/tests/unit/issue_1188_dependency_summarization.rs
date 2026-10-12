//! R1188-U21 (extends R197): summarization by dependency, with no language model.
//!
//! The kept statements are the ones the other statements depend on, and a
//! statement whose formal content an earlier one already holds is dropped. The
//! JavaScript twin is `rust/tests/web/dependency-summarization.test.mjs`; both
//! pin the same outputs.

use formal_ai::summarization::dependency::{
    closing_of, keep_budget, key_facts, restates, retained_facts, sentence_statements,
    summarize_by_dependency, without_duplicates,
};

const TEXT: &str = "The Moon orbits Earth. Earth is orbited by the Moon. The Moon has craters. \
                    Craters cover the Moon. Paris is a city.";

#[test]
fn a_restatement_in_other_words_is_dropped_by_its_formal_identity() {
    let entries = sentence_statements(TEXT, "en");
    assert_eq!(entries.len(), 5);
    assert!(restates(&entries[1].statement, &entries[0].statement));
    assert!(!restates(&entries[2].statement, &entries[0].statement));
    let (unique, duplicates) = without_duplicates(entries);
    assert_eq!((unique.len(), duplicates), (4, 1));
}

#[test]
fn the_graph_links_a_statement_to_the_earlier_ones_whose_subject_it_mentions() {
    let summary = summarize_by_dependency(TEXT, "en");
    let graph: Vec<(Vec<usize>, Vec<usize>)> = summary
        .statements
        .iter()
        .map(|node| (node.depends_on.clone(), node.dependents.clone()))
        .collect();
    assert_eq!(
        graph,
        vec![
            (vec![], vec![1, 2]),
            (vec![0], vec![2]),
            (vec![0, 1], vec![]),
            (vec![], vec![]),
        ]
    );
}

#[test]
fn the_kept_core_is_the_depended_on_root_and_its_most_depended_on_statement() {
    let summary = summarize_by_dependency(TEXT, "en");
    assert_eq!(summary.kept, vec![0, 1]);
    assert_eq!(summary.text, "The Moon orbits Earth. The Moon has craters.");
    let facts = key_facts(TEXT, "en");
    assert_eq!(facts, vec!["moon", "name:earth", "unknown:orbit"]);
    assert_eq!(retained_facts(&summary, &facts), 3);
}

#[test]
fn a_fragment_that_ends_its_sentence_in_the_summary_is_closed_with_its_punctuation() {
    let summary = summarize_by_dependency(
        "The Moon orbits Earth, and it has craters. Craters are old. The Moon is bright.",
        "en",
    );
    assert!(summary.text.ends_with('.'), "{}", summary.text);
    assert_eq!(closing_of("月球是卫星。"), "。");
    assert_eq!(closing_of("no closing"), "");
}

#[test]
fn the_summary_keeps_one_statement_in_three() {
    let budgets: Vec<usize> = [0, 1, 2, 3, 4, 7].into_iter().map(keep_budget).collect();
    assert_eq!(budgets, vec![0, 1, 1, 1, 2, 3]);
}
