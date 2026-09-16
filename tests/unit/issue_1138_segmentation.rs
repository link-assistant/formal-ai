//! Issue #1138, plan 04 L1–L2 — script-aware segmentation with exact spans.
//!
//! The formalizer recognises one sentence terminator today, so a Chinese or
//! Hindi requirement is one undivided blob and a Spanish one splits in the wrong
//! place. These four cases pin the terminators the seed declares per script and
//! the span defect recorded at `docs/case-studies/issue-710/plans/07:371-377`
//! (a span that swallowed the separator whitespace before the text it names).

use formal_ai::formalization::segment::{Script, Segment, sentences};

fn texts(segments: &[Segment]) -> Vec<&str> {
    segments.iter().map(|segment| segment.text.as_str()).collect()
}

#[test]
fn chinese_text_segments_at_the_ideographic_full_stop() {
    let text = "检查必须拒绝重复字母的单词。返回布尔值。";
    let segments = sentences(text);
    assert_eq!(
        texts(&segments),
        vec!["检查必须拒绝重复字母的单词。", "返回布尔值。"],
        "the ideographic full stop ends a sentence"
    );
    assert!(segments.iter().all(|segment| segment.script == Script::Han));
}

#[test]
fn hindi_text_segments_at_the_danda() {
    let text = "जाँच दोहराए गए अक्षर वाले शब्द को अस्वीकार करे। बूलियन लौटाएँ।";
    let segments = sentences(text);
    assert_eq!(
        texts(&segments),
        vec![
            "जाँच दोहराए गए अक्षर वाले शब्द को अस्वीकार करे।",
            "बूलियन लौटाएँ।",
        ],
        "the danda ends a sentence"
    );
    assert!(
        segments
            .iter()
            .all(|segment| segment.script == Script::Devanagari)
    );
}

#[test]
fn spanish_inverted_punctuation_does_not_split_a_sentence() {
    let text = "¿La palabra repite una letra? Sí.";
    let segments = sentences(text);
    assert_eq!(
        texts(&segments),
        vec!["¿La palabra repite una letra?", "Sí."],
        "an opening inverted mark belongs to the sentence it opens"
    );
    assert!(segments.iter().all(|segment| segment.script == Script::Latin));
}

#[test]
fn every_segment_span_selects_exactly_its_own_text() {
    for text in [
        "An isogram repeats no letter. Reject the word.",
        "Проверка отклоняет слово. Возвращает булево.",
        "检查必须拒绝重复字母的单词。返回布尔值。",
        "¿La palabra repite una letra? Sí.",
    ] {
        for segment in sentences(text) {
            assert_eq!(
                &text[segment.start..segment.end],
                segment.text,
                "a span must select exactly its own text, with no leading separator"
            );
            assert!(
                !segment.text.starts_with(char::is_whitespace),
                "a segment never begins with the whitespace that preceded it: {:?}",
                segment.text
            );
        }
    }
}
