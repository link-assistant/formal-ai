//! The software-project phrase vocabulary is seed data (issue #1175).
//!
//! `rust/src/solver_handlers/software_project_phrases.rs` holds only the
//! reading algorithm; every surface it consults lives in
//! `data/seed/software-project-phrases.lino`. These tests pin the seeded word
//! sets to the exact contents the object-phrase discipline was designed
//! around, so extending a language is a data edit that this file turns into
//! a reviewed change, and a lost or mangled seed row fails loudly here
//! instead of silently loosening the routing guards.

use formal_ai::seed;
use formal_ai::seed::{
    ROLE_SOFTWARE_OBJECT_BOUNDARY_CHARACTER, ROLE_SOFTWARE_OBJECT_BOUNDARY_WORD,
    ROLE_SOFTWARE_OBJECT_LEAD_BIGRAM, ROLE_SOFTWARE_OBJECT_LEAD_WORD,
    ROLE_SOFTWARE_OBJECT_WORD_INTERNAL_CHARACTER, ROLE_SOFTWARE_SENTENCE_END_CHARACTER,
};

fn sorted_words(role: &str) -> Vec<String> {
    let mut words = seed::lexicon().words_for_role(role);
    words.sort();
    words
}

#[test]
fn boundary_words_close_the_object_phrase_in_every_seeded_language() {
    assert_eq!(
        sorted_words(ROLE_SOFTWARE_OBJECT_BOUNDARY_WORD),
        [
            // English
            "about", "and", "but", "by", "for", "from", "i", "in", "into", "it", "of", "or",
            "please", "so", "that", "they", "to", "using", "via", "we", "when", "where",
            "which", "who", "with", "without",
            // Russian
            "в", "для", "и", "из", "или", "которая", "которое", "который", "которые", "когда",
            "на", "от", "с", "чтобы",
            // Hindi
            "और", "जो", "के", "को", "से", "में", "या",
        ]
        .map(String::from)
        .to_vec(),
        "the object-phrase boundary vocabulary must stay exactly this set; a data edit that \
         adds or drops a word is a routing-behaviour change this test exists to review",
    );
}

#[test]
fn lead_words_skip_determiners_and_benefactives() {
    assert_eq!(
        sorted_words(ROLE_SOFTWARE_OBJECT_LEAD_WORD),
        [
            "a", "an", "the", "this", "that", "these", "those", "my", "your", "our", "me", "us",
            "please",
        ]
        .map(String::from)
        .to_vec(),
    );
}

#[test]
fn lead_bigrams_cover_the_benefactive_forms_whose_first_word_is_a_boundary() {
    assert_eq!(
        sorted_words(ROLE_SOFTWARE_OBJECT_LEAD_BIGRAM),
        ["for me", "for us", "to me", "to us"].map(String::from).to_vec(),
    );
}

#[test]
fn boundary_characters_end_a_phrase_at_the_edge_of_unspaced_scripts() {
    let mut characters: Vec<char> = seed::lexicon()
        .words_for_role(ROLE_SOFTWARE_OBJECT_BOUNDARY_CHARACTER)
        .iter()
        .flat_map(|word| word.chars())
        .collect();
    characters.sort_unstable();
    let mut expected: Vec<char> = [
        ',', ';', '.', '?', '!', ':', '(', ')', '[', ']', '，', '。', '？', '！', '；', '：', '、',
        '।',
    ]
    .to_vec();
    expected.sort_unstable();
    assert_eq!(characters, expected);
}

#[test]
fn word_internal_characters_cut_a_token_anywhere_only_in_unspaced_scripts() {
    let mut characters: Vec<char> = seed::lexicon()
        .words_for_role(ROLE_SOFTWARE_OBJECT_WORD_INTERNAL_CHARACTER)
        .iter()
        .flat_map(|word| word.chars())
        .collect();
    characters.sort_unstable();
    let mut expected: Vec<char> = ['，', '。', '？', '！', '；', '：', '、', '।'].to_vec();
    expected.sort_unstable();
    assert_eq!(characters, expected);
}

#[test]
fn sentence_end_characters_bound_the_pre_verbal_object_from_the_left() {
    let mut characters: Vec<char> = seed::lexicon()
        .words_for_role(ROLE_SOFTWARE_SENTENCE_END_CHARACTER)
        .iter()
        .flat_map(|word| word.chars())
        .collect();
    characters.sort_unstable();
    let mut expected: Vec<char> = ['.', '?', '!', ';', ':', '。', '？', '！', '；', '।'].to_vec();
    expected.sort_unstable();
    assert_eq!(characters, expected);
}
