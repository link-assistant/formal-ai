// Browser twins of the remaining native-only coding/policy rows of the handler
// precedence seed: `algorithm`, `execution_failure`,
// `document_generation_plan`, `source_conflict` and `shell_refusal`. Each
// `handle*` function mirrors its Rust counterpart (named in the JSDoc); the
// `try*` bindings pass the lowercased prompt as `normalized`, exactly as the
// native dispatcher does (rust/src/meta_method_dispatch.rs).
//
// Like their Rust counterparts, the first four keep their wording inline (the
// Rust functions have not been moved to seed templates yet); `shell_refusal`
// is a seed rule (data/seed/handler-rules.lino) run by the shared rule
// interpreter, so its cues and response stay data.

// ---------------------------------------------------------------------------
// algorithm (rust/src/solver_handlers/mod.rs try_algorithm)
// ---------------------------------------------------------------------------

/**
 * The language a sorting-algorithm request names (Rust
 * `detect_algorithm_language`).
 * @param {string} normalized normalized prompt
 * @returns {string} language slug
 */
function algorithmDetectLanguage(normalized) {
  const languages = [
    ["python", "python"],
    [" py ", "python"],
    ["rust", "rust"],
    [" rs ", "rust"],
    ["javascript", "javascript"],
    ["typescript", "typescript"],
    ["go ", "go"],
    ["golang", "go"],
    ["java", "java"],
    ["ruby", "ruby"],
  ];
  for (const pair of languages) {
    if (normalized.includes(pair[0])) return pair[1];
  }
  return "python";
}

/**
 * The reviewable sorting snippet (Rust `build_sorting_algorithm_answer`).
 * @param {string} language language slug
 * @param {boolean} withTests whether to include a test
 * @returns {string} answer body
 */
function algorithmSortingAnswer(language, withTests) {
  let fence = "python";
  let code = "def sort(values):\n    return sorted(values)\n";
  let tests = "def test_sort_ascending():\n    assert sort([3, 1, 2]) == [1, 2, 3]\n";
  switch (language) {
    case "rust":
      fence = "rust";
      code = "fn sort(values: &mut Vec<i32>) {\n    values.sort();\n}";
      tests = "#[test]\nfn test_sort_ascending() {\n    let mut v = vec![3, 1, 2];\n    sort(&mut v);\n    assert_eq!(v, vec![1, 2, 3]);\n}";
      break;
    case "javascript":
    case "typescript":
      fence = language;
      code = "function sort(values) {\n  return [...values].sort((a, b) => a - b);\n}";
      tests = "function test_sort_ascending() {\n  assert.deepEqual(sort([3,1,2]), [1,2,3]);\n}";
      break;
    default:
      break;
  }
  const status = "Execution status: unavailable in this runtime. The snippet is intended to be copy-paste reviewable.";
  if (withTests) {
    return "Here is a reviewable sorting algorithm in " + language + " with a test:\n\n```" + fence + "\n" + code +
      "\n```\n\nTests:\n```" + fence + "\n" + tests + "\n```\n\n" + status;
  }
  return "Here is a reviewable sorting algorithm in " + language + ":\n\n```" + fence + "\n" + code + "\n```\n\n" + status;
}

/**
 * Answer a sorting-algorithm request with a reviewable snippet.
 * Mirrors `try_algorithm`.
 * @param {string} prompt raw prompt
 * @param {string} normalized normalized prompt
 * @returns {object|null} the worker answer, or null
 */
function handleAlgorithm(prompt, normalized) {
  if (!normalized.includes("algorithm") && !normalized.includes("sort")) return null;
  const language = algorithmDetectLanguage(normalized);
  const log = codeTaskLog();
  codeTaskLogAppend(log, "execution_status", "unavailable");
  codeTaskLogAppend(log, "execution_environment", "no compile/run sandbox configured for this generated snippet");
  return codeTaskAnswer(log, "algorithm_sort_" + language, "response:algorithm",
    algorithmSortingAnswer(language, normalized.includes("test")), 1.0);
}

/**
 * Browser binding for the `algorithm` precedence row.
 * @param {string} prompt raw prompt
 * @returns {object|null} the worker answer, or null
 */
function tryAlgorithm(prompt) {
  return handleAlgorithm(prompt, prompt.toLowerCase());
}

// ---------------------------------------------------------------------------
// execution_failure (rust/src/solver_handlers/mod.rs try_execution_failure)
// ---------------------------------------------------------------------------

/**
 * Surface the failure trace of an explicit failing-call prompt.
 * Mirrors `try_execution_failure`.
 * @param {string} prompt raw prompt
 * @param {string} normalized normalized prompt
 * @returns {object|null} the worker answer, or null
 */
function handleExecutionFailure(prompt, normalized) {
  if (!normalized.includes("undefined_function")) return null;
  const log = codeTaskLog();
  codeTaskLogAppend(log, "trace:execution_failure", "undefined_function");
  if (normalized.includes("[agent]")) {
    codeTaskLogAppend(log, "agent_mode:opted_in", prompt);
    codeTaskLogAppend(log, "action_log", prompt);
  }
  const body = "Execution status: failed in isolated sandbox.\n" +
    "```python\nundefined_function()\n```\n" +
    "Traceback (most recent call last):\n  File 'main.py', line 1, in <module>\n" +
    "NameError: name 'undefined_function' is not defined.\n" +
    "The failure trace is appended to the action log; see the trace link.";
  return codeTaskAnswer(log, "execution_failure", "response:execution_failure", body, 0.4);
}

/**
 * Browser binding for the `execution_failure` precedence row.
 * @param {string} prompt raw prompt
 * @returns {object|null} the worker answer, or null
 */
function tryExecutionFailure(prompt) {
  return handleExecutionFailure(prompt, prompt.toLowerCase());
}

// ---------------------------------------------------------------------------
// document_generation_plan (rust/src/solver_handlers/document_request.rs)
// ---------------------------------------------------------------------------

/**
 * True when any needle occurs in the text.
 * @param {string} text haystack
 * @param {Array<string>} needles needles
 * @returns {boolean} whether one occurs
 */
function documentPlanHasAny(text, needles) {
  for (const needle of needles) {
    if (text.includes(needle)) return true;
  }
  return false;
}

/**
 * True when the prompt names a software artifact (Rust `mentions_software_artifact`).
 * @param {string} lowercased lowercased prompt
 * @returns {boolean} whether it names one
 */
function documentPlanMentionsSoftware(lowercased) {
  return documentPlanHasAny(lowercased, [
    "app", "application", "extension", "plugin", "addon", "add-on", "bot", "program", "script",
    "website", "web site", "web app", "webapp", "game", "api", "library", "framework", "tool",
    "cli", "daemon", "server",
    "приложени", "программ", "скрипт", "расширени", "плагин", "бот", "сайт", "веб-сайт", "игр",
    "утилит", "сервер", "библиотек",
  ]);
}

/**
 * True when the prompt carries an authoring verb (Rust `mentions_authoring_action`).
 * @param {string} lowercased lowercased prompt
 * @returns {boolean} whether it carries one
 */
function documentPlanMentionsAuthoring(lowercased) {
  return documentPlanHasAny(lowercased, [
    "make me", "make a", "make an", "create", "generate", "produce", "compile", "prepare",
    "assemble", "draft", "build me", "export", "give me", "i need", "i want", "put together",
    "сдела", "созда", "сгенерир", "подготов", "сформир", "состав", "собери", "собрать", "оформ",
    "выгрузи", "экспортир", "сверста",
    "बना", "तैयार", "उत्पन्न", "चाहिए",
    "做", "制作", "生成", "创建", "编写", "整理", "给我", "帮我",
  ]);
}

/**
 * The requested document container label: "PDF", "DOCX", …, "" for a generic
 * document, or null when no document artifact is named.
 * @param {string} lowercased lowercased prompt
 * @returns {string|null} format label
 */
function documentPlanFormat(lowercased) {
  if (documentPlanHasAny(lowercased, ["pdf", "пдф"])) return "PDF";
  if (documentPlanHasAny(lowercased, ["docx", ".doc", "word document", "ворд", "документ word"])) return "DOCX";
  if (documentPlanHasAny(lowercased, [
    "xlsx", "csv", "spreadsheet", "excel", "эксель", "табличк", "电子表格", "表格", "स्प्रेडशीट",
  ])) return "CSV/XLSX";
  if (documentPlanHasAny(lowercased, [
    "pptx", "powerpoint", "slide deck", "презентаци", "слайд", "演示", "幻灯片", "प्रस्तुति",
  ])) return "PPTX";
  if (documentPlanHasAny(lowercased, ["epub", "e-book", "ebook", "электронн"]) &&
      documentPlanHasAny(lowercased, ["book", "книг"])) return "EPUB";
  if (documentPlanHasAny(lowercased, [
    "document", "документ", "report", "отчет", "отчёт", "dossier", "досье", "brochure", "брошюр",
    "booklet", "буклет", "memo", "записку", "записка", "दस्तावेज़", "दस्तावेज", "रिपोर्ट",
    "文档", "文件", "报告",
  ])) return "";
  return null;
}

/**
 * The localized plan template pieces for one language.
 * @param {string} language language slug
 * @returns {object} {suffix, body} with `{label}` / `{format_suffix}` slots
 */
function documentPlanTemplate(language) {
  switch (language) {
    case "ru":
      return { suffix: " в формате {label}", body: "Это запрос на создание документа{format_suffix}. Я детерминированный символьный решатель: у меня нет доступа к произвольным актуальным данным в вебе и я не рендерю бинарные файлы напрямую, поэтому я раскладываю задачу на формальный план по универсальному алгоритму (декомпозиция → проверки → черновики → композиция):\n\n1. Уточнить объём и критерии документа: какие элементы включать и по каким признакам их различать.\n2. Собрать список элементов из проверяемых источников и зафиксировать ссылки на эти источники.\n3. Классифицировать каждый элемент по заявленным критериям.\n4. Собрать структуру документа: заголовок, разделы и таблицу или список.\n5. Экспортировать готовую структуру в запрошенный формат.\n\nПодтвердите план или уточните критерии и источники — и я продолжу с конкретными шагами. Если нужны фактические данные, которых нет в локальной памяти Links Notation, укажите источник, и я добавлю его как правило связей." };
    case "hi":
      return { suffix: " ({label} प्रारूप में)", body: "यह एक दस्तावेज़ बनाने का अनुरोध है{format_suffix}. मैं एक नियतात्मक प्रतीकात्मक हल करने वाला हूँ: मैं वेब पर मनमाना सजीव डेटा नहीं खोज सकता और बाइनरी फ़ाइलें सीधे नहीं बनाता, इसलिए मैं इस कार्य को सार्वभौमिक एल्गोरिदम की औपचारिक योजना में विभाजित करता हूँ (विभाजन → जाँच → मसौदे → रचना):\n\n1. दस्तावेज़ और उसके मानदंड का दायरा तय करें: कौन-सी वस्तुएँ शामिल करनी हैं और कौन-से गुण उन्हें अलग करते हैं।\n2. सत्यापन योग्य स्रोतों से वस्तुओं की सूची एकत्र करें और उन स्रोतों के लिंक दर्ज करें।\n3. प्रत्येक वस्तु को बताए गए मानदंड के अनुसार वर्गीकृत करें।\n4. दस्तावेज़ की संरचना बनाएँ: शीर्षक, अनुभाग और एक तालिका या सूची।\n5. तैयार संरचना को अनुरोधित प्रारूप में निर्यात करें।\n\nयोजना की पुष्टि करें या मानदंड और स्रोत स्पष्ट करें, और मैं ठोस चरणों के साथ आगे बढ़ूँगा।" };
    case "zh":
      return { suffix: "（{label} 格式）", body: "这是一个生成文档的请求{format_suffix}。我是一个确定性的符号求解器：我无法在网络上检索任意实时数据，也不会直接渲染二进制文件，因此我把任务分解为通用算法生成的形式化计划（分解 → 校验 → 草稿 → 组合）：\n\n1. 界定文档及其标准：包含哪些条目以及用哪些属性区分它们。\n2. 从可验证的来源收集条目清单，并记录这些来源的链接。\n3. 根据所述标准对每个条目进行分类。\n4. 组装文档结构：标题、章节以及表格或列表。\n5. 将完成的结构导出为所请求的格式。\n\n请确认计划或细化标准与来源，我将继续给出具体步骤。" };
    default:
      return { suffix: " in {label} format", body: "This is a document-generation request{format_suffix}. I am a deterministic symbolic solver: I cannot research arbitrary live data on the web and I do not render binary files directly, so I decompose the task into the formal plan the universal algorithm produces (decompose → tests → drafts → composition). The document workflow uses link-foundation/meta-language for txt, Markdown, HTML, PDF, and DOCX representation/conversion, with concept profiles for headings, paragraphs, lists, strong/bold text, emphasis, and hyperlinks:\n\n1. Scope the document and its criteria: which items to include and which attributes distinguish them.\n2. Collect the list of items from verifiable sources and record links to those sources.\n3. Classify each item against the stated criteria.\n4. Assemble the document structure: title, sections, and a table or list.\n5. Export the finished structure to the requested format.\n\nConfirm the plan or refine the criteria and sources and I will continue with concrete steps. If you need facts that are not in the local Links Notation memory, name a source and I will add it as a links rule." };
  }
}

/**
 * Render the localized document-generation plan (Rust `render_document_plan`).
 * @param {string} language language slug
 * @param {string} label format label, "" for a generic document
 * @returns {string} the plan
 */
function documentPlanRender(language, label) {
  const template = documentPlanTemplate(language);
  const suffix = label === "" ? "" : template.suffix.split("{label}").join(label);
  return template.body.split("{format_suffix}").join(suffix);
}

/**
 * Answer a document-generation request with the universal-algorithm plan.
 * Mirrors `try_document_request` without its meta-language conversion branch.
 * @param {string} prompt raw prompt
 * @param {string} normalized normalized prompt
 * @returns {object|null} the worker answer, or null
 */
function handleDocumentGenerationPlan(prompt, normalized) {
  const lowercased = normalized.toLowerCase();
  if (documentPlanHasAny(lowercased, ["[agent]", "enable agent", "agent mode"])) return null;
  if (documentPlanMentionsSoftware(lowercased)) return null;
  if (!documentPlanMentionsAuthoring(lowercased)) return null;
  const label = documentPlanFormat(lowercased);
  if (label === null) return null;
  const log = codeTaskLog();
  codeTaskLogAppend(log, "document_request:format", label === "" ? "document" : label);
  return codeTaskAnswer(log, "document_generation_plan", "response:document_generation_plan",
    documentPlanRender(detectLanguage(prompt), label), 0.6);
}

/**
 * Browser binding for the `document_generation_plan` precedence row.
 * @param {string} prompt raw prompt
 * @returns {object|null} the worker answer, or null
 */
function tryDocumentGenerationPlan(prompt) {
  return handleDocumentGenerationPlan(prompt, prompt.toLowerCase());
}

// ---------------------------------------------------------------------------
// source_conflict (rust/src/retrieval_procedures.rs try_source_conflict)
// ---------------------------------------------------------------------------

/**
 * Record a source disagreement instead of resolving it silently.
 * Mirrors `try_source_conflict`.
 * @param {string} prompt raw prompt
 * @param {string} normalized normalized prompt
 * @returns {object|null} the worker answer, or null
 */
function handleSourceConflict(prompt, normalized) {
  if (!(normalized.includes("conflict") || (normalized.includes("born in") && normalized.includes(" or ")))) {
    return null;
  }
  const log = codeTaskLog();
  codeTaskLogAppend(log, "conflict:source_disagreement", "sources disagree on the answer");
  return codeTaskAnswer(log, "source_conflict", "response:source_conflict",
    "Sources disagree on this question. The disagreement is recorded as a conflict:source_disagreement link in the network rather than silently resolved.",
    0.3);
}

/**
 * Browser binding for the `source_conflict` precedence row.
 * @param {string} prompt raw prompt
 * @returns {object|null} the worker answer, or null
 */
function trySourceConflict(prompt) {
  return handleSourceConflict(prompt, prompt.toLowerCase());
}

// ---------------------------------------------------------------------------
// shell_refusal (data/seed/handler-rules.lino, rule shell_refusal)
// ---------------------------------------------------------------------------

/**
 * Browser binding for the `shell_refusal` precedence row: the seed rule runs
 * through the shared rule interpreter (formal_ai_worker_handler_rules.js) over
 * the lowercased prompt, which keeps the backticks its `run \`` cue needs —
 * the same subject the native dispatcher hands the rule.
 * @param {string} prompt raw prompt
 * @returns {object|null} the worker answer, or null
 */
function tryShellRefusal(prompt) {
  return runHandlerRuleSet("shell_refusal", prompt, prompt.toLowerCase(), []);
}
