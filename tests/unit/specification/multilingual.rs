//! Multilingual chat behavior required by `VISION.md` across current and future languages.

use formal_ai::{ConversationTurn, SolverConfig, SymbolicAnswer, UniversalSolver, humanize_url};

mod reported_prompts;

fn answer(prompt: &str) -> SymbolicAnswer {
    UniversalSolver::new(SolverConfig {
        offline: true,
        ..SolverConfig::default()
    })
    .solve(prompt)
}

const RUSSIAN_IIR_IN_ML_ANSWER: &str = "В контексте «ml» (Машинное обучение) Фильтр с бесконечной импульсной характеристикой (signal-processing) означает: Фильтр с бесконечной импульсной характеристикой (рекурсивный фильтр, БИХ-фильтр) или IIR-фильтр (IIR сокр. от англ. infinite impulse response — бесконечная импульсная характеристика) — линейный электронный фильтр, использующий один или более своих выходов в качестве входа, то есть образующий обратную связь. Основным свойством таких фильтров является то, что их импульсная переходная характеристика имеет бесконечную длину во временной области, а передаточная функция имеет дробно-рациональный вид. Такие фильтры могут быть как аналоговыми, так и цифровыми.\n\nИсточник: https://ru.wikipedia.org/wiki/Фильтр_с_бесконечной_импульсной_характеристикой (wikipedia).";

const ENGLISH_IIR_IN_ML_ANSWER: &str = "In the context of «ml» (machine learning), infinite impulse response (IIR) (signal-processing) means: An infinite impulse response (IIR) filter is a type of recursive digital filter whose impulse response is non-zero over an infinite length of time. Such filters can be either analog or digital, and the impulse-response feedback means the same shape can be approximated with far fewer coefficients than a finite impulse response (FIR) filter, at the cost of nonlinear phase and the need to verify stability.\n\nSource: https://en.wikipedia.org/wiki/Infinite_impulse_response (wikipedia).";

const CHINESE_IIR_IN_ML_ANSWER: &str = "在「ml」(机器学习)的语境下,无限脉冲响应(IIR)滤波器(signal-processing)指的是:无限脉冲响应(IIR)滤波器是一种递归型数字滤波器,其冲激响应在时间上具有无限长度,因为当前输出不仅取决于过去的输入,还取决于过去的输出。在信号处理与机器学习的音频/时间序列管线中,IIR 滤波器以远少于等价 FIR 滤波器的系数实现低通、高通、带通和带阻响应,代价是非线性相位以及需要验证稳定性。\n\n来源:[https://zh.wikipedia.org/wiki/无限脉冲响应](https://zh.wikipedia.org/wiki/%E6%97%A0%E9%99%90%E8%84%89%E5%86%B2%E5%93%8D%E5%BA%94)(wikipedia)。";

const HINDI_IIR_IN_ML_ANSWER: &str = "«ml» (मशीन लर्निंग) के संदर्भ में, अनंत आवेग प्रतिक्रिया (IIR) फ़िल्टर (signal-processing) का अर्थ है: अनंत आवेग प्रतिक्रिया (IIR) फ़िल्टर एक पुनरावर्ती डिजिटल फ़िल्टर है जिसकी आवेग प्रतिक्रिया अनंत अवधि तक शून्येतर बनी रहती है क्योंकि वर्तमान आउटपुट पिछले इनपुट के साथ-साथ पिछले आउटपुट पर भी निर्भर करता है। संकेत प्रसंस्करण और मशीन-लर्निंग ऑडियो/समय-शृंखला पाइपलाइनों में IIR फ़िल्टर बराबर FIR फ़िल्टर की तुलना में बहुत कम गुणांकों के साथ लो-पास, हाई-पास, बैंड-पास और बैंड-स्टॉप प्रतिक्रियाएँ प्राप्त करते हैं, अरैखिक फ़ेज़ और स्थिरता-सत्यापन की कीमत पर।\n\nस्रोत: https://hi.wikipedia.org/wiki/अनंत_आवेग_प्रतिक्रिया (wikipedia).";

// ---------------------------------------------------------------------------
// Active expectation: implementation English greeting.
// ---------------------------------------------------------------------------

#[test]
fn english_greeting_is_handled_today() {
    assert_eq!(answer("Hi").intent, "greeting");
    assert_eq!(answer("Hello").intent, "greeting");
    assert_eq!(answer("Hey").intent, "greeting");
}

// ---------------------------------------------------------------------------
// full-scope expectations: Russian, Hindi, Chinese baseline greetings and identity.
// ---------------------------------------------------------------------------

#[test]
fn russian_greeting_returns_greeting_intent() {
    let response = answer("Привет");
    assert_eq!(response.intent, "greeting");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "language:ru"),
        "Russian answers should tag the detected language"
    );
}

#[test]
fn russian_greeting_reply_is_in_russian() {
    let response = answer("Привет");
    assert_eq!(response.answer, "Здравствуйте! Чем могу помочь?");
    assert!(
        response.answer.contains("Здравствуйте") || response.answer.contains("Привет"),
        "Russian greeting should be answered in Russian, got: {}",
        response.answer
    );
}

#[test]
fn hindi_greeting_returns_greeting_intent() {
    let response = answer("नमस्ते");
    assert_eq!(response.intent, "greeting");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "language:hi")
    );
}

#[test]
fn chinese_greeting_returns_greeting_intent() {
    let response = answer("你好");
    assert_eq!(response.intent, "greeting");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "language:zh")
    );
}

#[test]
fn russian_identity_question_returns_identity_intent() {
    let response = answer("Кто ты?");
    assert_eq!(response.intent, "identity");
}

#[test]
fn russian_combined_greeting_and_identity_question_returns_identity_intent() {
    let response = answer("Привет. ты кто?");
    assert_eq!(response.intent, "identity");
    assert_eq!(
        response.answer,
        "Я formal-ai — детерминированный символьный ИИ, который отвечает на основе локальных правил Links Notation и совместимых OpenAI-форматов. В этой демонстрации я не выполняю нейросетевой инференс."
    );
    assert!(
        response.answer.contains("formal-ai"),
        "combined greeting and identity prompt should answer identity, got: {}",
        response.answer
    );
}

#[test]
fn hindi_identity_question_returns_identity_intent() {
    let response = answer("तुम कौन हो?");
    assert_eq!(response.intent, "identity");
}

#[test]
fn chinese_identity_question_returns_identity_intent() {
    let response = answer("你是谁?");
    assert_eq!(response.intent, "identity");
}

#[test]
fn every_multilingual_answer_declares_detected_language_link() {
    for prompt in ["Hi", "Привет", "你好", "नमस्ते"] {
        let response = answer(prompt);
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link.starts_with("language:")),
            "missing language tag for prompt {prompt:?}"
        );
    }
}

#[test]
fn unknown_language_prompts_fall_back_to_english_with_unknown_language_link() {
    let response = answer("لطفاً سلام بگو");
    assert_eq!(
        response.answer,
        "I detected an unsupported language and am falling back to English. I could not determine `لطفاً سلام بگو` from local Links Notation memory, cached public knowledge, or the source cache, and cannot infer a verified answer. I recorded the failed gather attempts in the trace.\n\nIf reasoning still cannot resolve this and a shared Links Notation seed fact or links rule is needed, use Report issue with the trace. To keep a dialog-local rule durable, export memory or teach it with `When I say ... answer ...`; inspect routes with `List behavior rules` and `Show behavior rule unknown`.\n\nI detected an unsupported language and am falling back to English. I detected a failure while working on this request. Would you like me to prepare an issue report with the diagnostic context? Reply `Report issue`."
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "language:unknown"),
        "answers in unsupported languages should record an unknown-language link"
    );
    assert!(response.answer.contains("English"));
}

// ---------------------------------------------------------------------------
// Issue #16: "What is X?" style prompts must work in Russian, Hindi, Chinese.
// ---------------------------------------------------------------------------

#[test]
fn russian_concept_question_returns_concept_lookup_intent() {
    let response = answer("Что такое Википедия?");
    assert!(
        response.intent.starts_with("concept_lookup"),
        "Russian concept lookup should map to concept_lookup intent, got: {}",
        response.intent
    );
    assert_eq!(
        response.answer,
        "Wikipedia (encyclopedia): Wikipedia is a free, multilingual online encyclopedia written and maintained by a community of volunteer contributors through a model of open collaboration.\n\nSource: https://en.wikipedia.org/wiki/Wikipedia (wikipedia)."
    );
    assert!(
        response.answer.to_lowercase().contains("wikipedia")
            || response.answer.to_lowercase().contains("encyclopedia")
            || response.answer.to_lowercase().contains("википед"),
        "Russian Wikipedia answer should reference the concept, got: {}",
        response.answer
    );
}

#[test]
fn russian_antiregime_question_returns_seeded_concept_lookup() {
    let response = answer("Что такое антирежим?");
    assert_eq!(
        response.intent, "concept_lookup",
        "reported prompt should resolve from the seed, got {} -> {}",
        response.intent, response.answer
    );
    assert_eq!(
        response.answer,
        "Антирежим (political-adjective): Антирежим — это позиция, действие или характеристика, направленная против политического режима. Слово соответствует английскому antiregime: «противостоящий режиму».\n\nSource: https://en.wiktionary.org/wiki/antiregime (wiktionary)."
    );
    assert!(
        response.answer.contains("режим"),
        "Russian answer should define the reported term, got: {}",
        response.answer
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "concept_lookup:hit:concept_antiregime"),
        "answer should cite the antiregime seed record, got {:?}",
        response.evidence_links
    );
}

#[test]
fn false_totality_questions_resolve_across_supported_languages() {
    let cases = [
        (
            "english",
            "What is false totality?",
            "false totality",
            "language:en",
            "false totality (philosophy): False totality is a critical-theory and dialectical-materialist term for a mistaken whole: an explanatory totality that is treated as closed, independent, or higher than its concrete facts and contradictions. Karel Kosik contrasts it with concrete totality and describes empty, abstract, and bad totality as forms that flatten or mystify reality.\n\nSource: https://www.lust-for-life.org/Lust-For-Life/DialecticOfTheConcrete/DialecticOfTheConcrete.htm (philosophy-text).",
        ),
        (
            "russian",
            "Что такое ложная тотальность?",
            "Ложная тотальность",
            "language:ru",
            "Ложная тотальность (philosophy): Ложная тотальность — это понятие критической теории и диалектического материализма о неверно понятом целом: объясняющей целостности, которую принимают за замкнутую, самостоятельную или более реальную, чем конкретные факты и противоречия. Карел Косик противопоставляет ее конкретной тотальности и выделяет пустую, абстрактную и плохую тотальность как формы, которые уплощают или мистифицируют реальность.\n\nSource: https://www.lust-for-life.org/Lust-For-Life/DialecticOfTheConcrete/DialecticOfTheConcrete.htm (philosophy-text).",
        ),
        (
            "hindi",
            "झूठी समग्रता क्या है",
            "झूठी समग्रता",
            "language:hi",
            "झूठी समग्रता (philosophy): झूठी समग्रता आलोचनात्मक सिद्धांत और द्वंद्वात्मक भौतिकवाद का शब्द है: ऐसा गलत ढंग से समझा गया समग्र, जिसे बंद, स्वतंत्र या ठोस तथ्यों और अंतर्विरोधों से अधिक वास्तविक मान लिया जाता है. कारेल कोसिक इसे concrete totality के विरुद्ध रखते हैं और empty, abstract, तथा bad totality को इसके रूप बताते हैं.\n\nSource: https://www.lust-for-life.org/Lust-For-Life/DialecticOfTheConcrete/DialecticOfTheConcrete.htm (philosophy-text).",
        ),
        (
            "chinese",
            "虚假总体性是什么",
            "虚假总体性",
            "language:zh",
            "虚假总体性 (philosophy): 虚假总体性是批判理论和辩证唯物主义中的术语, 指一种被误解的整体: 它被当作封闭、独立、或高于具体事实和矛盾的解释性总体。卡雷尔·科西克把它同具体总体性相区分, 并把空洞、抽象和坏的总体性视为会压平或神秘化现实的形式。\n\nSource: https://www.lust-for-life.org/Lust-For-Life/DialecticOfTheConcrete/DialecticOfTheConcrete.htm (philosophy-text).",
        ),
    ];

    for (language_name, prompt, expected_term, language_link, expected_answer) in cases {
        let response = answer(prompt);
        assert_eq!(
            response.intent, "concept_lookup",
            "{language_name} prompt should resolve from the false totality seed, got {} -> {}",
            response.intent, response.answer
        );
        assert_eq!(response.answer, expected_answer);
        assert!(
            response.answer.contains(expected_term),
            "{language_name} answer should use the localized term {expected_term:?}, got: {}",
            response.answer
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == "concept_lookup:hit:concept_false_totality"),
            "{language_name} answer should cite the false totality seed record, got {:?}",
            response.evidence_links
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == language_link),
            "{language_name} answer should declare {language_link}, got {:?}",
            response.evidence_links
        );
    }
}

// Issue #161: graph prompts should be answered from the local concept seed in
// every supported language. With associative project promotion enabled by
// default, each localized answer explains graphs through the Link Foundation
// meta-theory / Links Notation lens.
#[test]
fn graph_questions_promote_links_notation_context_across_supported_languages() {
    let cases: &[(&str, &str, &[&str], &str)] = &[
        (
            "what is graph",
            "language:en",
            &["Graph", "vertices", "edges", "Links Notation"],
            "Graph (knowledge-representation): A graph is a mathematical structure made of vertices and edges, where edges relate pairs of vertices. Through Link Foundation meta-theory, a Links Notation links network can represent any graph while also allowing links to link to links. That avoids treating knowledge as two artificial classes of vertices and edges; in many common graph definitions, edges between edges are not allowed.\n\nSource: https://github.com/link-foundation/meta-theory (official-repository).",
        ),
        (
            "что такое граф",
            "language:ru",
            &["Граф", "вершин", "ребер", "Links Notation", "сеть связей"],
            "Граф (knowledge-representation): Граф — математическая структура из вершин и ребер, где ребра связывают пары вершин. В контексте Link Foundation meta-theory и Links Notation сеть связей может представить любой граф, а также позволяет ссылкам ссылаться на ссылки. Поэтому Links Notation не ограничивает знание искусственным разделением на вершины и ребра: в популярных определениях графов ребра между ребрами обычно не допускаются.\n\nSource: https://github.com/link-foundation/meta-theory (official-repository).",
        ),
        (
            "ग्राफ क्या है",
            "language:hi",
            &["ग्राफ", "शीर्ष", "किनार", "Links Notation", "links network"],
            "ग्राफ (knowledge-representation): ग्राफ शीर्षों और किनारों से बनी गणितीय संरचना है, जहाँ किनारे शीर्षों के जोड़ों को जोड़ते हैं. Link Foundation meta-theory और Links Notation के संदर्भ में links network किसी भी graph को व्यक्त कर सकता है और links को links से जोड़ने देता है. इससे ज्ञान को vertices और edges की दो कृत्रिम श्रेणियों में बाँधने की जरूरत नहीं रहती; कई प्रचलित graph परिभाषाओं में edges between edges की अनुमति नहीं होती.\n\nSource: https://github.com/link-foundation/meta-theory (official-repository).",
        ),
        (
            "图是什么",
            "language:zh",
            &["图", "顶点", "边", "Links Notation", "链接网络"],
            "图 (knowledge-representation): 图是由顶点和边组成的数学结构, 边连接成对的顶点. 在 Link Foundation meta-theory 和 Links Notation 语境中, 链接网络可以表示任何图, 也允许链接指向链接. 因此 Links Notation 不必把知识人为拆成顶点和边两类; 许多常见图定义通常不允许边连接边.\n\nSource: https://github.com/link-foundation/meta-theory (official-repository).",
        ),
    ];

    for (prompt, language_link, fragments, expected_answer) in cases {
        let response = answer(prompt);
        assert_eq!(
            response.intent, "concept_lookup",
            "graph question {prompt:?} should resolve as concept_lookup, got {} -> {}",
            response.intent, response.answer
        );
        assert_eq!(response.answer, *expected_answer);
        assert_ne!(
            response.intent, "unknown",
            "graph question {prompt:?} must not fall through to unknown"
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == language_link),
            "graph question {prompt:?} should record {language_link}, got {:?}",
            response.evidence_links
        );
        for fragment in *fragments {
            assert!(
                response.answer.contains(fragment),
                "graph answer for {prompt:?} should contain {fragment:?}, got: {}",
                response.answer
            );
        }
        assert!(
            response
                .answer
                .contains("https://github.com/link-foundation/meta-theory"),
            "graph answer for {prompt:?} should cite the Link Foundation meta-theory repository, got: {}",
            response.answer
        );
    }
}

#[test]
fn hindi_concept_question_returns_concept_lookup_intent() {
    let response = answer("विकिपीडिया क्या है?");
    assert!(
        response.intent.starts_with("concept_lookup"),
        "Hindi concept lookup should map to concept_lookup intent, got: {}",
        response.intent
    );
}

#[test]
fn chinese_concept_question_returns_concept_lookup_intent() {
    let response = answer("维基百科是什么?");
    assert!(
        response.intent.starts_with("concept_lookup"),
        "Chinese concept lookup should map to concept_lookup intent, got: {}",
        response.intent
    );
}

// Issue #20: "что такое X в Y" — (concept, context) disambiguation across
// English, Russian, Hindi, and Chinese, matching every typical phrasing.
// The reporter's exact prompt is in the Russian test.
// ---------------------------------------------------------------------------

#[test]
fn russian_iir_in_ml_returns_context_aware_concept_lookup() {
    let response = answer("что такое iir в ml");
    assert_eq!(
        response.intent, "concept_lookup_in_context",
        "Russian (concept,context) prompt should map to concept_lookup_in_context, got: {}",
        response.intent
    );
    assert_eq!(response.answer, RUSSIAN_IIR_IN_ML_ANSWER);
    let lower = response.answer.to_lowercase();
    assert!(
        lower.contains("iir") && lower.contains("ml"),
        "Russian (concept,context) answer should reference both halves, got: {}",
        response.answer
    );
}

#[test]
fn russian_bsd_ports_question_returns_ports_not_openbsd() {
    let response = answer("что такое порты в bsd");
    assert_eq!(
        response.intent, "concept_lookup_in_context",
        "Russian BSD ports prompt should resolve as a context-aware local concept, got: {}",
        response.intent
    );
    assert_eq!(
        response.answer,
        "В контексте BSD Порты BSD (package-management) означает: Порты BSD — это не сетевые порты, а система рецептов для сборки и установки сторонних приложений из исходного кода. Обычно порт представляет собой каталог с метаданными и Makefile: он описывает, откуда взять исходники, какие зависимости и патчи нужны, как собрать пакет и как установить приложение. Для обычной установки чаще используют готовые бинарные пакеты, а порты полезны, когда нужны свои параметры сборки или сопровождение пакета.\n\nИсточник: https://docs.freebsd.org/en/books/handbook/ports/ (official-docs)."
    );
    let lower = response.answer.to_lowercase();
    assert!(
        lower.contains("порты") && lower.contains("bsd"),
        "answer should explain BSD ports, got: {}",
        response.answer
    );
    assert!(
        !lower.contains("openbsd") || lower.contains("freebsd") || lower.contains("netbsd"),
        "answer should not collapse the question to a generic OpenBSD article, got: {}",
        response.answer
    );
}

#[test]
fn hindi_bsd_ports_question_returns_localized_ports_answer() {
    let response = answer("BSD में पोर्ट्स क्या है?");
    assert_eq!(
        response.intent, "concept_lookup_in_context",
        "Hindi BSD ports prompt should resolve as a context-aware local concept, got: {}",
        response.intent
    );
    assert_eq!(
        response.answer,
        "BSD के संदर्भ में, BSD पोर्ट्स (package-management) का अर्थ है: BSD पोर्ट्स नेटवर्क पोर्ट नहीं हैं; वे स्रोत कोड से तृतीय-पक्ष सॉफ्टवेयर बनाने और इंस्टॉल करने की पैकेज recipes हैं। आम तौर पर एक port metadata और Makefile वाली directory होता है: वह source कहां से लाना है, dependencies और patches क्या हैं, package कैसे बनाना है, और application कैसे install करना है, यह बताता है। सामान्य installation के लिए binary packages तेज होते हैं; ports custom build options और maintainers के लिए उपयोगी हैं।\n\nस्रोत: https://docs.freebsd.org/en/books/handbook/ports/ (official-docs)."
    );
    assert!(
        response.answer.contains("BSD")
            && response.answer.contains("पोर्ट्स")
            && response.answer.contains("पैकेज"),
        "Hindi answer should explain BSD ports as package recipes, got: {}",
        response.answer
    );
}

#[test]
fn chinese_bsd_ports_question_returns_localized_ports_answer() {
    let response = answer("BSD中的端口集合是什么?");
    assert_eq!(
        response.intent, "concept_lookup_in_context",
        "Chinese BSD ports prompt should resolve as a context-aware local concept, got: {}",
        response.intent
    );
    assert_eq!(
        response.answer,
        "在BSD的语境下,BSD Ports(package-management)指的是:BSD Ports 不是网络端口，而是 BSD 操作系统用来从源代码构建和安装第三方软件的包管理配方。一个 port 通常是包含 metadata 和 Makefile 的目录：它说明源代码从哪里获取、需要哪些依赖和补丁、如何构建 package，以及如何安装 application。日常安装通常使用预构建的二进制包更快；ports 更适合需要自定义构建选项或维护软件包的场景。\n\n来源:https://docs.freebsd.org/en/books/handbook/ports/(official-docs)。"
    );
    assert!(
        response.answer.contains("BSD")
            && response.answer.contains("Ports")
            && response.answer.contains("源代码"),
        "Chinese answer should explain BSD ports as source-based package recipes, got: {}",
        response.answer
    );
}

#[test]
fn english_what_is_iir_in_ml_returns_context_aware_concept_lookup() {
    let response = answer("what is IIR in ML?");
    assert_eq!(response.intent, "concept_lookup_in_context");
    assert_eq!(response.answer, ENGLISH_IIR_IN_ML_ANSWER);
    let lower = response.answer.to_lowercase();
    assert!(lower.contains("iir"));
    assert!(lower.contains("ml") || lower.contains("machine learning"));
}

#[test]
fn hindi_iir_in_ml_returns_context_aware_concept_lookup() {
    // Hindi places the context before the concept ("ML में IIR क्या है?").
    let response = answer("ML में IIR क्या है?");
    assert_eq!(
        response.intent, "concept_lookup_in_context",
        "Hindi context-first prompt should map to concept_lookup_in_context, got: {}",
        response.intent
    );
}

#[test]
fn chinese_iir_in_ml_returns_context_aware_concept_lookup() {
    // Chinese also places the context before the concept ("ML 中的 IIR 是什么?").
    let response = answer("ML中的IIR是什么?");
    assert_eq!(
        response.intent, "concept_lookup_in_context",
        "Chinese context-first prompt should map to concept_lookup_in_context, got: {}",
        response.intent
    );
}

#[test]
fn bare_iir_without_context_still_resolves() {
    // Without a context clause the solver should still find the term and
    // return the plain concept_lookup intent (not the in-context variant).
    let response = answer("what is IIR?");
    assert_eq!(
        response.intent, "concept_lookup",
        "Bare term should map to plain concept_lookup, got: {}",
        response.intent
    );
}

#[test]
fn russian_colloquial_kubatorit_definition_uses_seeded_dictionary_source() {
    let response = answer("Что такое кубаторит?");

    assert_eq!(
        response.intent, "concept_lookup",
        "reported Russian dictionary prompt should not fall through to unknown: {}",
        response.answer
    );
    assert_eq!(
        response.answer,
        "Кубаторить / кубатурить (russian-colloquial-verb): Кубаторить, также кубатурить, — разговорно-жаргонный глагол: размышлять, думать или переживать о чем-то.\n\nSource: https://argo.academic.ru/2501/кубатурить (slang-dictionary)."
    );
    assert!(
        response.answer.contains("Кубаторить"),
        "answer should name the reported word form, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("размышлять") && response.answer.contains("думать"),
        "answer should explain the slang meaning, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("academic.ru"),
        "answer should cite the dictionary source, got: {}",
        response.answer
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "concept_lookup:hit:concept_kubaturit"),
        "concept hit should be traceable, got: {:?}",
        response.evidence_links
    );
}

#[test]
fn russian_colloquial_kubaturit_variant_resolves_to_same_concept() {
    let response = answer("Что такое кубатурить?");

    assert_eq!(response.intent, "concept_lookup");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "concept_lookup:hit:concept_kubaturit"),
        "variant should resolve to the same dictionary concept, got: {:?}",
        response.evidence_links
    );
}

#[test]
fn concept_lookup_evidence_records_context_match_event() {
    // Verbose/debug trail: an in-context hit must leave a
    // `concept_lookup:context-match:*` evidence link so we can root-cause
    // future regressions from the trace alone (maintainer requirement #5).
    let response = answer("что такое iir в ml");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("concept_lookup:context-match")),
        "expected a concept_lookup:context-match evidence link, got: {:?}",
        response.evidence_links,
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("concept_lookup:request")),
        "expected a concept_lookup:request evidence link, got: {:?}",
        response.evidence_links,
    );
}

// ---------------------------------------------------------------------------
// Issue #20 (maintainer follow-up): native-language body and full disambiguated
// context name. The maintainer asked for:
//   - В контексте «ml» (Машинное обучение) IIR ... [R8]
//   - Russian term: "Фильтр с бесконечной импульсной характеристикой ... или
//     IIR-фильтр" [R9]
//   - Russian summary verbatim from ru.wikipedia.org [R10]
//   - Prefer the user's prevailing language [R11]
// ---------------------------------------------------------------------------

#[test]
fn russian_iir_in_ml_body_uses_native_term_and_context_label() {
    // R8 + R9 + R11: when the prevailing language is Russian, the body must
    // (a) name the resolved context in Russian ("Машинное обучение") and
    // (b) use the Russian term ("Фильтр с бесконечной импульсной...").
    let response = answer("что такое iir в ml");
    assert_eq!(response.answer, RUSSIAN_IIR_IN_ML_ANSWER);
    let answer_text = &response.answer;
    assert!(
        answer_text.contains("«ml»"),
        "Russian answer should quote the user's literal context phrase, got: {answer_text}"
    );
    assert!(
        answer_text.contains("Машинное обучение"),
        "Russian answer should append the registry label «Машинное обучение», got: {answer_text}"
    );
    assert!(
        answer_text.contains("Фильтр с бесконечной импульсной характеристикой"),
        "Russian answer should use the native term, got: {answer_text}"
    );
    assert!(
        answer_text.contains("IIR-фильтр"),
        "Russian answer should reference the Russian-language alias \"IIR-фильтр\", got: {answer_text}"
    );
}

#[test]
fn russian_iir_in_ml_source_points_at_russian_wikipedia() {
    // R10: the cited source must be the Russian Wikipedia article body the
    // maintainer linked, not the English fallback.
    let response = answer("что такое iir в ml");
    assert_eq!(response.answer, RUSSIAN_IIR_IN_ML_ANSWER);
    assert!(
        response.answer.contains("ru.wikipedia.org"),
        "Russian answer should cite ru.wikipedia.org, got: {}",
        response.answer
    );
}

#[test]
fn russian_iir_when_context_is_typed_natively_drops_redundant_parens() {
    // R8 corollary: if the user types the localized label themselves, the
    // response should not duplicate it as `«Машинное обучение» (Машинное
    // обучение)`. The `concept_lookup_in_context_no_alias` template handles
    // this without committing to per-language Rust code.
    let response = answer("что такое iir в машинное обучение");
    assert_eq!(response.intent, "concept_lookup_in_context");
    assert_eq!(
        response.answer,
        "В контексте Машинное обучение Фильтр с бесконечной импульсной характеристикой (signal-processing) означает: Фильтр с бесконечной импульсной характеристикой (рекурсивный фильтр, БИХ-фильтр) или IIR-фильтр (IIR сокр. от англ. infinite impulse response — бесконечная импульсная характеристика) — линейный электронный фильтр, использующий один или более своих выходов в качестве входа, то есть образующий обратную связь. Основным свойством таких фильтров является то, что их импульсная переходная характеристика имеет бесконечную длину во временной области, а передаточная функция имеет дробно-рациональный вид. Такие фильтры могут быть как аналоговыми, так и цифровыми.\n\nИсточник: https://ru.wikipedia.org/wiki/Фильтр_с_бесконечной_импульсной_характеристикой (wikipedia)."
    );
    let answer_text = &response.answer;
    assert!(
        answer_text.contains("Машинное обучение"),
        "answer should mention the localized context label, got: {answer_text}"
    );
    assert!(
        !answer_text.contains("«машинное обучение» (Машинное обучение)"),
        "no_alias template should not render «label» (label) duplication, got: {answer_text}"
    );
}

#[test]
fn english_iir_in_ml_body_uses_english_native_term() {
    // R11: prevailing-language routing for English. The localized "en" block
    // expands "IIR" to "infinite impulse response (IIR)" for the long form.
    let response = answer("what is IIR in ML?");
    assert_eq!(response.answer, ENGLISH_IIR_IN_ML_ANSWER);
    let lower = response.answer.to_lowercase();
    assert!(
        lower.contains("infinite impulse response"),
        "English answer should expand the acronym, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("machine learning"),
        "English answer should mention the resolved context label, got: {}",
        response.answer
    );
}

#[test]
fn chinese_iir_in_ml_body_uses_chinese_context_label() {
    // R8 in Chinese: the resolved label «机器学习» (machine learning) must
    // appear in the response body.
    let response = answer("ML中的IIR是什么?");
    assert_eq!(response.answer, CHINESE_IIR_IN_ML_ANSWER);
    assert!(
        response.answer.contains("机器学习"),
        "Chinese answer should append the localized context label, got: {}",
        response.answer
    );
}

#[test]
fn hindi_iir_in_ml_body_uses_hindi_context_label() {
    // R8 in Hindi: the resolved label «मशीन लर्निंग» must appear.
    let response = answer("ML में IIR क्या है?");
    assert_eq!(response.answer, HINDI_IIR_IN_ML_ANSWER);
    assert!(
        response.answer.contains("मशीन लर्निंग"),
        "Hindi answer should append the localized context label, got: {}",
        response.answer
    );
}

#[test]
fn russian_iir_evidence_includes_wikidata_anchor() {
    // R13: the link network must carry the Wikidata Q-ID anchor so callers
    // can use it as a cross-language join key (this is how human-language and
    // meta-expression translate across the four target languages).
    let response = answer("что такое iir в ml");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.contains("Q740073") || link.contains("wikidata")),
        "expected a wikidata-anchored evidence link for cross-language joins, got: {:?}",
        response.evidence_links,
    );
}

// ---------------------------------------------------------------------------
// Issue #49: "что за дичь?" and "что ты умеешь?" must not fall through to
// intent: unknown. The user expressed frustration after the agent failed to
// handle a capabilities question in Russian. Both the capability query and
// the confusion/frustration expression must produce a meaningful intent.
// ---------------------------------------------------------------------------

#[test]
fn russian_capabilities_question_does_not_return_unknown() {
    let response = answer("что ты умеешь?");
    assert_ne!(
        response.intent, "unknown",
        "Russian capability question should not fall through to unknown, got: {}",
        response.answer,
    );
}

#[test]
fn russian_capabilities_question_returns_capabilities_intent() {
    let response = answer("что ты умеешь?");
    assert_eq!(
        response.intent, "capabilities",
        "Russian capability question should map to capabilities intent, got: {}",
        response.answer,
    );
}

#[test]
fn russian_confusion_phrase_does_not_return_unknown() {
    let response = answer("что за дичь?");
    assert_ne!(
        response.intent, "unknown",
        "Russian slang confusion phrase should not fall through to unknown, got: {}",
        response.answer,
    );
}

#[test]
fn russian_confusion_phrase_returns_capabilities_intent() {
    // "что за дичь?" literally means "what is this nonsense?" — a frustrated
    // reaction to getting an unhelpful answer. The agent should respond with a
    // capabilities overview so the user understands what the agent can handle.
    let response = answer("что за дичь?");
    assert_eq!(
        response.intent, "capabilities",
        "Russian confusion phrase should map to capabilities intent, got: {}",
        response.answer,
    );
}

#[test]
fn russian_capabilities_answer_is_in_russian() {
    let response = answer("что ты умеешь?");
    assert_eq!(
        response.answer,
        "Я formal-ai — детерминированный символьный ИИ. Вот что я умею:\n\n- **Приветствия**: отвечаю на «Привет», «Здравствуйте» и т.п.\n- **Hello World**: генерирую программы на Rust, Python, JavaScript, Go, C и других языках.\n- **Веб-поиск**: ищу в интернете через DuckDuckGo, Wikipedia и Wikidata, когда поиск доступен.\n- **Поиск понятий**: объясняю термины — попробуйте «Что такое Википедия?»\n- **Арифметика**: вычисляю выражения — например, «Сколько будет 2 + 2?»\n- **Перевод**: перевожу фразы между языками.\n- **Память**: помню контекст разговора в рамках сессии.\n- **Правила поведения**: отправьте `Покажи правила поведения`, чтобы увидеть встроенные правила, и `Покажи правило unknown`, чтобы прочитать одно правило.\n- **Обучение в диалоге**: отправьте «Когда я скажу `ваш запрос`, ответь `ваш ответ`», чтобы добавить правило, действующее только в этом диалоге.\n- **Факты о себе**: отправьте `List all facts you know about yourself`, чтобы увидеть, что я знаю о себе.\n- **Сообщение об ошибке**: используйте кнопку отчёта об ошибке сверху; для неизвестных запросов ссылка в сообщении добавит диагностическую трассировку.\n- **Настройки и действия**: через сообщения можно включать диагностику/демо/agent mode, менять тему, язык, стиль чата и экспортировать или импортировать память.\n\nЯ работаю на основе локальных символьных правил, без нейросетевого инференса."
    );
    assert!(
        response
            .answer
            .chars()
            .any(|c| ('\u{0400}'..='\u{04FF}').contains(&c)),
        "Russian capabilities answer should contain Cyrillic text, got: {}",
        response.answer,
    );
    assert!(
        response.answer.contains("Покажи правила поведения")
            && response.answer.contains("Покажи правило unknown")
            && response.answer.contains("Когда я скажу"),
        "Russian capabilities answer should show rule commands in Russian, got: {}",
        response.answer,
    );
    assert!(
        !response.answer.contains("List behavior rules")
            && !response.answer.contains("Show behavior rule unknown")
            && !response.answer.contains("When I say"),
        "Russian capabilities answer should not switch to English rule commands, got: {}",
        response.answer,
    );
}

#[test]
fn russian_more_capabilities_follow_up_uses_history_without_repeating_web_search() {
    let history = [
        ConversationTurn::user("Ты можешь искать в интернете?"),
        ConversationTurn::assistant(
            "Да. В этой конфигурации веб-поиск включен: я могу использовать DuckDuckGo.",
        ),
    ];
    let response = UniversalSolver::default().solve_with_history("Что ещё ты умеешь?", &history);
    assert_eq!(
        response.intent, "capabilities",
        "Russian follow-up capabilities question should map to capabilities, got {}: {}",
        response.intent, response.answer,
    );
    assert_eq!(
        response.answer,
        "Кроме уже названных возможностей, могу ещё:\n\n- **Арифметика**: вычислять выражения вроде «Сколько будет 2 + 2?»\n- **Перевод**: переводить короткие фразы между поддерживаемыми языками.\n- **Поиск понятий**: объяснять термины, например «Что такое Википедия?»\n- **Hello World**: генерировать минимальные программы на Rust, Python, JavaScript, Go, C и других языках.\n- **Память диалога**: использовать предыдущие сообщения текущей сессии.\n- **Правила поведения**: показывать встроенные правила через `Покажи правила поведения` и `Покажи правило unknown`.\n- **Настройки и действия**: включать диагностику/демо/agent mode, менять тему, язык, стиль чата, экспортировать и импортировать память."
    );
    assert!(
        response.answer.contains("Арифметика") && response.answer.contains("Перевод"),
        "follow-up should list additional capabilities, got: {}",
        response.answer,
    );
    assert!(
        !response.answer.contains("Веб-поиск")
            && !response.answer.to_lowercase().contains("интернет")
            && !response.answer.contains("DuckDuckGo"),
        "follow-up should not repeat the already discussed web-search capability, got: {}",
        response.answer,
    );
}

#[test]
fn english_capabilities_question_returns_capabilities_intent() {
    let response = answer("what can you do?");
    assert_eq!(
        response.intent, "capabilities",
        "English capability question should map to capabilities intent, got: {}",
        response.answer,
    );
}
