//! Program-artifact coreference markers shared by write-program recovery.

const PROGRAM_FOLLOW_UP_REFERENTS: &[&str] = &[
    "result",
    "results",
    "output",
    "program",
    "programme",
    "script",
    "code",
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
    "परिणाम",
    "परिणामों",
    "नतीजा",
    "नतीजे",
    "आउटपुट",
    "प्रोग्राम",
    "कोड",
    "结果",
    "输出",
    "程序",
    "代码",
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
    "क्रमबद्ध",
    "उल्टे",
    "उल्टा",
    "बनाओ",
    "बदलें",
    "बदलो",
    "अपडेट",
    "排序",
    "反向",
    "相反",
    "倒序",
    "修改",
    "改",
    "更新",
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
