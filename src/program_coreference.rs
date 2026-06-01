//! Program-artifact coreference markers shared by write-program recovery.

const PROGRAM_FOLLOW_UP_REFERENTS: &[&str] = &[
    "result",
    "results",
    "output",
    "program",
    "programme",
    "script",
    "code",
    // Issue #386: a sort/ordering is itself a program aspect a follow-up can
    // refer to ("undo the sort", "cancel the sorting").
    "sort",
    "sorting",
    "результат",
    "результата",
    "результаты",
    "результатов",
    "вывод",
    "программа",
    "программу",
    "программы",
    "скрипт",
    "код",
    "сортировка",
    "сортировку",
    "сортировки",
    "сортировке",
    "сортировкой",
    "परिणाम",
    "परिणामों",
    "नतीजा",
    "नतीजे",
    "आउटपुट",
    "प्रोग्राम",
    "कोड",
    "सॉर्ट",
    "क्रम",
    "结果",
    "输出",
    "程序",
    "代码",
    "排序",
    "顺序",
];

const PROGRAM_FOLLOW_UP_ACTIONS: &[&str] = &[
    "sort",
    "sorted",
    "reverse",
    "reorder",
    "order",
    "change",
    "modify",
    "update",
    "make",
    // Issue #386: subtractive ("cancel/undo") verbs are first-class program
    // follow-up actions, mirroring the additive verbs above.
    "cancel",
    "undo",
    "remove",
    "revert",
    "сделай",
    "сделайте",
    "сортировка",
    "сортировку",
    "сортировать",
    "отсортируй",
    "отсортируйте",
    "обратном",
    "обратный",
    "измени",
    "изменить",
    "обнови",
    "отмени",
    "отмените",
    "отменить",
    "убери",
    "уберите",
    "убрать",
    "क्रमबद्ध",
    "उल्टे",
    "उल्टा",
    "बनाओ",
    "बदलें",
    "बदलो",
    "अपडेट",
    "रद्द",
    "हटाओ",
    "हटाएं",
    "हटा",
    "排序",
    "反向",
    "相反",
    "倒序",
    "修改",
    "改",
    "更新",
    "取消",
    "撤销",
    "去掉",
    "去除",
];

#[must_use]
pub fn looks_like_bare_program_artifact_follow_up(normalized: &str) -> bool {
    has_any_token(normalized, PROGRAM_FOLLOW_UP_REFERENTS)
        && has_any_token(normalized, PROGRAM_FOLLOW_UP_ACTIONS)
}

fn has_any_token(normalized: &str, tokens: &[&str]) -> bool {
    tokens.iter().any(|token| contains_token(normalized, token))
}

fn contains_token(normalized: &str, expected: &str) -> bool {
    if crate::coding::contains_cjk(expected) {
        return normalized.contains(expected);
    }
    normalized.split_whitespace().any(|token| token == expected)
}

#[cfg(test)]
mod tests {
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
}
