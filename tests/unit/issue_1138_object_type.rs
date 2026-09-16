//! Issue #1138 B10, plan 10 leaves 4-5 and 7: the object in a request and the
//! locus its effect lands in are derived *structurally*, with no per-language
//! branch and no possessive in the derivation.
//!
//! `Find X on my desktop` and `Find X on desktop` are the same request. They
//! route differently today only because `ROLE_LOCAL_PATH_SCOPE_DESKTOP` is
//! matched with an adjacent possessive; the locus is evidence about *where*,
//! and a possessive is not evidence about where.
//!
//! Written before the leaves that make it pass (plan 14 wave T).

use formal_ai::capability_routing::{Act, Locus, ObjectType, act, locus, object_type};

/// The same request in five languages. If the derivation is structural, the
/// highest-ranked object is the same in every one of them.
const FIVE_LANGUAGE_CASES: [(&str, [&str; 5], ObjectType); 6] = [
    (
        "an absolute URL is an object whatever the verb",
        [
            "open https://example.com and tell me what is there",
            "открой https://example.com и скажи, что там",
            "https://example.com खोलिए और बताइए वहाँ क्या है",
            "打开 https://example.com 看看那里有什么",
            "abre https://example.com y dime qué hay",
        ],
        ObjectType::Url,
    ),
    (
        "a file literal is a path",
        [
            "show me notes.txt",
            "покажи мне notes.txt",
            "मुझे notes.txt दिखाइए",
            "给我看看 notes.txt",
            "muéstrame notes.txt",
        ],
        ObjectType::Path,
    ),
    (
        "a clock time is a time expression",
        [
            "set it for 20:00",
            "поставь на 20:00",
            "इसे 20:00 पर रखिए",
            "定在 20:00",
            "ponlo a las 20:00",
        ],
        ObjectType::TimeExpression,
    ),
    (
        "a registered language name is a language name",
        [
            "answer in Hindi",
            "ответь на хинди",
            "हिंदी में उत्तर दीजिए",
            "用印地语回答",
            "responde en hindi",
        ],
        ObjectType::LanguageName,
    ),
    (
        "an interrogative asking for a magnitude is a quantity question",
        [
            "how deep does an oak root go",
            "насколько глубоко уходит корень дуба",
            "बलूत की जड़ कितनी गहरी जाती है",
            "橡树的根能扎多深",
            "hasta qué profundidad llega la raíz del roble",
        ],
        ObjectType::QuantityQuestion,
    ),
    (
        "the assistant's own surface is a self surface",
        [
            "the panel you drew is impossible to drag",
            "панель, которую ты нарисовал, невозможно перетащить",
            "आपने जो पैनल बनाया है उसे खींचना नामुमकिन है",
            "你画的那个面板根本拖不动",
            "el panel que dibujaste no hay quien lo arrastre",
        ],
        ObjectType::SelfSurface,
    ),
];

#[test]
fn object_type_is_structural_in_every_language() {
    let mut failures: Vec<String> = Vec::new();
    for (claim, prompts, expected) in FIVE_LANGUAGE_CASES {
        for (language, prompt) in ["en", "ru", "hi", "zh", "es"].iter().zip(prompts) {
            let derived = object_type(prompt);
            let highest = derived.first().copied().unwrap_or(ObjectType::None);
            if highest != expected {
                failures.push(format!(
                    "{claim} [{language}]: {prompt:?} derived {highest:?}, expected {expected:?}"
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "the object derivation must carry no natural-language vocabulary, so the same \
         request yields the same object in every language: {failures:?}"
    );
}

#[test]
fn a_possessive_never_changes_the_locus() {
    // The maintainer's three prompts differ only by a possessive and a hyphen;
    // they are structurally one request and must derive one locus.
    let pairs: [(&str, &str); 5] = [
        (
            "Find hive-mind-control center folder on my desktop",
            "Find hive-mind-control center folder on desktop",
        ),
        (
            "Найди папку hive-mind-control center на моём рабочем столе",
            "Найди папку hive-mind-control center на рабочем столе",
        ),
        (
            "मेरे डेस्कटॉप पर hive-mind-control-center फ़ोल्डर ढूँढिए।",
            "डेस्कटॉप पर hive-mind-control-center फ़ोल्डर ढूँढिए।",
        ),
        (
            "在我的桌面上找 hive-mind-control-center 文件夹。",
            "在桌面上找 hive-mind-control-center 文件夹。",
        ),
        (
            "Busca la carpeta hive-mind-control-center en mi escritorio.",
            "Busca la carpeta hive-mind-control-center en el escritorio.",
        ),
    ];
    for (possessive, bare) in pairs {
        assert_eq!(
            locus(possessive),
            locus(bare),
            "a possessive is not evidence about where the effect lands: {possessive:?} \
             and {bare:?} must derive one locus"
        );
        assert_eq!(
            locus(bare),
            Locus::Workspace,
            "a filesystem scope noun puts the effect in the workspace: {bare:?}"
        );
    }
}

#[test]
fn a_verb_synonym_never_changes_the_act() {
    // Verbs select the act; they never select the capability. Every surface of
    // one act in five languages must resolve to that one act.
    let retrieve = [
        "find rust ownership",
        "найди rust ownership",
        "rust ownership खोजें",
        "查找 rust ownership",
        "busca rust ownership",
    ];
    for prompt in retrieve {
        assert_eq!(
            act(prompt),
            Act::Retrieve,
            "`{prompt}` asks to retrieve, whatever verb its language uses"
        );
    }

    let enumerate = [
        "list the files here",
        "перечисли файлы здесь",
        "यहाँ फ़ाइलें सूचीबद्ध करें",
        "列出这里的文件",
        "enumera los archivos de aquí",
    ];
    for prompt in enumerate {
        assert_eq!(act(prompt), Act::Enumerate, "`{prompt}` asks to enumerate");
    }
}
