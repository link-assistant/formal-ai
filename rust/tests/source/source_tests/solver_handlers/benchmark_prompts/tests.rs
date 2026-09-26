use super::detect_relation;

/// Issue #386: `detect_relation` no longer reads a hardcoded keyword table —
/// it queries the `fact_relation` meanings in `data/seed/meanings-facts.lino`.
/// This pins that every relation is still recognised from a representative
/// prompt in each supported language (the words now live in the lexicon), so
/// the data-driven rewrite cannot silently drop a relation or a language.
#[test]
fn every_relation_is_detected_in_every_language() {
    for (slug, prompts) in [
        (
            "capital",
            [
                "what is the capital of france",
                "столица россии",
                "भारत की राजधानी क्या है",
                "法国的首都是什么",
            ],
        ),
        (
            "population",
            [
                "what is the population of india",
                "какое население москвы",
                "जापान की जनसंख्या कितनी है",
                "中国的人口是多少",
            ],
        ),
        (
            "currency",
            [
                "what is the currency of japan",
                "какая валюта в индии",
                "ब्राज़ील की मुद्रा क्या है",
                "美国的货币是什么",
            ],
        ),
        (
            "official_language",
            [
                "what is the official language of brazil",
                "государственный язык швейцарии",
                "स्विट्ज़रलैंड की राजभाषा क्या है",
                "瑞士的官方语言是什么",
            ],
        ),
        (
            "continent",
            [
                "which continent is egypt in",
                "на каком континенте египет",
                "मिस्र किस महाद्वीप में है",
                "埃及在哪个大洲",
            ],
        ),
        (
            "author_of_book",
            [
                "who wrote war and peace",
                "кто автор войны и мира",
                "महाभारत के लेखक कौन हैं",
                "战争与和平的作者是谁",
            ],
        ),
        (
            "painter_of_painting",
            [
                "who painted the mona lisa",
                "кто художник этой картины",
                "इस चित्र का चित्रकार कौन है",
                "蒙娜丽莎是谁画的",
            ],
        ),
        (
            "built_year",
            [
                "when was the eiffel tower built",
                "когда построена эйфелева башня",
                "ताज महल कब बनी थी",
                "长城建于何时",
            ],
        ),
        (
            "physical_constant",
            [
                "what is the speed of light",
                "чему равна скорость света",
                "प्रकाश की गति कितनी है",
                "光速是多少",
            ],
        ),
    ] {
        for prompt in prompts {
            assert_eq!(
                detect_relation(prompt),
                Some(slug),
                "prompt `{prompt}` should resolve to relation `{slug}`",
            );
        }
    }
}

/// The bare Russian verb "написал" (and "кто написал") lexicalises *both*
/// `author_of_book` and `painter_of_painting`. Declaration order in
/// `meanings-facts.lino` is load-bearing: `author_of_book` is declared first,
/// so it must win — exactly as the former first-match-wins pattern table did.
#[test]
fn ambiguous_napisal_prefers_author_over_painter() {
    assert_eq!(
        detect_relation("кто написал войну и мир"),
        Some("author_of_book")
    );
    assert_eq!(
        detect_relation("кто написал эту картину"),
        Some("author_of_book")
    );
}

/// A prompt that mentions no relation surface word resolves to nothing, so
/// the caller falls back to the matched record's declared relation.
#[test]
fn unrelated_prompt_detects_no_relation() {
    assert_eq!(detect_relation("hello there how are you"), None);
    assert_eq!(detect_relation("привет как дела"), None);
}
