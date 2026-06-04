use super::looks_like_bare_program_artifact_follow_up;
use crate::engine::normalize_prompt;

#[test]
fn additive_sort_follow_ups_are_recognized_in_every_language() {
    for prompt in [
        "sort the results in reverse order",
        "сделай сортировку результатов в обратном порядке",
        "परिणामों को उल्टे क्रम में क्रमबद्ध करो",
        "对结果倒序排序",
    ] {
        assert!(
            looks_like_bare_program_artifact_follow_up(&normalize_prompt(prompt)),
            "additive follow-up must be recognized: {prompt:?}"
        );
    }
}

#[test]
fn cancel_sort_follow_ups_are_recognized_in_every_language() {
    // Issue #386: the subtractive follow-up must clear the same coreference
    // gate as the additive one it undoes, in every supported language.
    for prompt in [
        "cancel the sorting",
        "undo the sort",
        "Отмени сортировку",
        "убери сортировку",
        "सॉर्ट हटाओ",
        "取消排序",
    ] {
        assert!(
            looks_like_bare_program_artifact_follow_up(&normalize_prompt(prompt)),
            "cancel follow-up must be recognized: {prompt:?}"
        );
    }
}

#[test]
fn unrelated_prompts_do_not_trip_the_gate() {
    for prompt in [
        "what is the capital of France",
        "напиши стихотворение про осень",
    ] {
        assert!(
            !looks_like_bare_program_artifact_follow_up(&normalize_prompt(prompt)),
            "unrelated prompt must not look like a program follow-up: {prompt:?}"
        );
    }
}
