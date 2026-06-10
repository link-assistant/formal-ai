use crate::event_log::EventLog;

use super::try_program_synthesis;

#[test]
fn program_synthesis_accepts_hindi_count_vowels_operation_verbs() {
    let prompt = "Python फ़ंक्शन count_vowels(text: str) -> int लागू करें। पाठ में स्वरों की संख्या लौटाएँ।";
    let mut log = EventLog::new();
    let response = try_program_synthesis(prompt, &prompt.to_lowercase(), &mut log)
        .expect("Hindi count_vowels prompt should synthesize a Python function");

    assert_eq!(response.intent, "write_program");
    assert!(response.answer.contains("def count_vowels"));
    assert!(response
        .links_notation
        .contains("synthesis:verification tests_passed"));
    assert!(response
        .links_notation
        .contains("synthesis:syntax_tree python_function_syntax_tree"));
    assert!(response
        .links_notation
        .contains("synthesis:cst_tree cst_tree"));
    assert!(response
        .links_notation
        .contains("synthesis:cst_engine meta_language"));
    assert!(response
        .links_notation
        .contains("component meta-language"));
    assert!(response
        .links_notation
        .contains("semantic_node matching_character_count_return"));
}

#[test]
fn declared_signature_stops_at_supported_language_sentence_marks() {
    let hindi = "Python फ़ंक्शन count_vowels(text: str) -> int लागू करें। पाठ में स्वरों की संख्या लौटाएँ।";
    let chinese = "实现 Python 函数 count_vowels(text: str) -> int。返回文本中的元音数量。";

    assert_eq!(
        super::declared_signature(hindi, "count_vowels").as_deref(),
        Some("count_vowels(text: str) -> int")
    );
    assert_eq!(
        super::declared_signature(chinese, "count_vowels").as_deref(),
        Some("count_vowels(text: str) -> int")
    );
}
