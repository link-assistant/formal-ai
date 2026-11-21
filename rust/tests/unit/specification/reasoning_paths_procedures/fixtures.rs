//! Source-backed procedure fixtures; assertion bodies stay in their original tests.

pub(super) fn documented_english_procedure(
    task: &str,
    action: &str,
    object: &str,
    candidate: &str,
    primary_query: &str,
    official_install_query: Option<&str>,
) -> String {
    let source_gate = official_install_query.map_or_else(String::new, |official| {
        format!(
            "\n\nFor install tasks, the first source gate prefers the product's official documentation or official repository install page before community how-to sources. It starts with the official-source web search query `{official}` and keeps the general how-to query `how to {task}` as fallback."
        )
    });
    format!(
        "Procedural discovery plan for `{task}` (action `{action}`, object `{object}`).{source_gate}\n\nI do not answer this from a memoized recipe. The solver first checks Wikipedia for topic context and Wikidata for entity/action/object hints. It then tries wikiHow's CORS-readable MediaWiki parse API candidate `{candidate}` via `https://www.wikihow.com/api.php?action=parse&page={candidate}&prop=text%7Csections%7Cdisplaytitle&format=json&origin=*`. If those sources do not expose usable steps, the fallback path runs web search for `{primary_query}` across duckduckgo, internet-archive, wikipedia, wikidata, wiktionary, wikinews and merges the top results with reciprocal rank fusion (k = 60). The final recursive fetch check only accepts pages that actually contain explicit ordered or instructional steps for `{task}`."
    )
}

pub(super) fn documented_connect_procedure(
    language: &str,
    task: &str,
    object: &str,
    candidate: &str,
) -> String {
    let api = format!(
        "https://{language}.wikihow.com/api.php?action=parse&page={candidate}&prop=text%7Csections%7Cdisplaytitle&format=json&origin=*"
    );
    let query = format!("how to {task}");
    let providers = "duckduckgo, internet-archive, wikipedia, wikidata, wiktionary, wikinews";
    match language {
        "en" => documented_english_procedure(task, "connect", object, candidate, &query, None),
        "ru" => format!(
            "План поиска процедуры для `{task}` (действие `connect`, объект `{object}`).\n\nЯ не отвечаю на это как на заученный рецепт. Сначала solver проверяет Wikipedia для контекста темы и Wikidata для подсказок сущности, действия и объекта. Затем он пробует CORS-readable MediaWiki parse API wikiHow для кандидата `{candidate}` через `{api}`. Если эти источники не дают пригодные шаги, fallback запускает web search по `{query}` через {providers} и объединяет верхние результаты reciprocal rank fusion (k = 60). Финальная recursive fetch check принимает только страницы с явными упорядоченными или инструкционными шагами для `{task}`."
        ),
        "hi" => format!(
            "`{task}` के लिए procedural discovery plan (action `connect`, object `{object}`).\n\nमैं इसे memorized recipe से answer नहीं करता. Solver पहले topic context के लिए Wikipedia और entity/action/object hints के लिए Wikidata जांचता है. फिर वह candidate `{candidate}` के लिए wikiHow का CORS-readable MediaWiki parse API `{api}` से आजमाता है. अगर ये sources usable steps नहीं देते, fallback `{query}` के लिए {providers} पर web search चलाता है और top results को reciprocal rank fusion (k = 60) से merge करता है. अंतिम recursive fetch check केवल उन pages को स्वीकार करता है जिनमें `{task}` के explicit ordered या instructional steps हों."
        ),
        "zh" => format!(
            "`{task}` 的过程发现计划（action `connect`, object `{object}`）。\n\n我不会把它当作记忆中的固定 recipe 来回答。Solver 先检查 Wikipedia 获取主题上下文，再检查 Wikidata 获取 entity/action/object 线索。然后它尝试 wikiHow 的 CORS-readable MediaWiki parse API candidate `{candidate}`，URL 为 `{api}`。如果这些来源没有可用步骤，fallback 会对 `{query}` 通过 {providers} 运行 web search，并用 reciprocal rank fusion (k = 60) 合并顶部结果。最后的 recursive fetch check 只接受真正包含 `{task}` 的明确有序步骤或 instructional steps 的页面。"
        ),
        other => panic!("missing documented connect-procedure language {other}"),
    }
}

pub(super) struct InstallCase {
    pub(super) language: &'static str,
    pub(super) prompt: &'static str,
    pub(super) task_fragment: &'static str,
    pub(super) object_fragment: &'static str,
    pub(super) official_marker: &'static str,
}

pub(super) const INSTALL_CASES: [InstallCase; 4] = [
    InstallCase {
        language: "en",
        prompt: "how install cursor",
        task_fragment: "install cursor",
        object_fragment: "cursor",
        official_marker: "official documentation",
    },
    InstallCase {
        language: "ru",
        prompt: "как установить cursor",
        task_fragment: "установить cursor",
        object_fragment: "cursor",
        official_marker: "официальн",
    },
    InstallCase {
        language: "hi",
        prompt: "कैसे इंस्टॉल करें cursor",
        task_fragment: "इंस्टॉल करें cursor",
        object_fragment: "cursor",
        official_marker: "आधिकारिक",
    },
    InstallCase {
        language: "zh",
        prompt: "如何安装 cursor",
        task_fragment: "安装 cursor",
        object_fragment: "cursor",
        official_marker: "官方",
    },
];

pub(super) struct SurfaceCase {
    pub(super) language: &'static str,
    pub(super) prompt: &'static str,
}

pub(super) const SURFACE_CASES: [SurfaceCase; 4] = [
    SurfaceCase {
        language: "en",
        prompt: "how to do 3d print ordering",
    },
    SurfaceCase {
        language: "ru",
        prompt: "как сделать 3d print ordering",
    },
    SurfaceCase {
        language: "hi",
        prompt: "कैसे करें 3d print ordering",
    },
    SurfaceCase {
        language: "zh",
        prompt: "如何做 3d print ordering",
    },
];

pub(super) struct TypoCase {
    pub(super) language: &'static str,
    pub(super) prompt: &'static str,
}

pub(super) const TYPO_CASES: [TypoCase; 4] = [
    TypoCase {
        language: "en",
        prompt: "How to do SPEC dirven development step by step?",
    },
    TypoCase {
        language: "ru",
        prompt: "как сделать SPEC dirven development? напиши по шагам",
    },
    TypoCase {
        language: "hi",
        prompt: "कैसे करें SPEC dirven development? चरणों में बताओ",
    },
    TypoCase {
        language: "zh",
        prompt: "如何做 SPEC dirven development？按步骤写",
    },
];
