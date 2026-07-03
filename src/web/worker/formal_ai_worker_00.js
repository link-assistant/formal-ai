// Worker module 1 of 21. Loaded by ../formal_ai_worker.js.
let wasm;
let mode = "wasm worker";

// Hard-coded fallbacks. These are only used if `seed/*.lino` fails to load,
// e.g. when the worker runs from a `file://` URL. The shipped GitHub Pages
// build always fetches the seed successfully.
const FALLBACK_IDENTITY_ANSWER =
  "I am formal-ai, a deterministic symbolic AI implementation that answers from local Links Notation rules and OpenAI-compatible API shapes. I do not perform neural inference in this demo.";

const FALLBACK_ASSISTANT_NAME_ANSWER =
  "I'm formal AI, and currently I don't have a name. But you can name me as you like.";

const FALLBACK_GREETING_ANSWER = "Hi, how may I help you?";

const FALLBACK_TEST_STATUS_ANSWER = "Test passed. I'm here.";
const FALLBACK_COURTESY_RESPONSE_ANSWER =
  "Glad to hear it. What would you like to do next?";
const FALLBACK_ASSISTANT_FREE_TIME_ANSWER =
  "I do not have free time the way a person does. Between prompts I am idle; when the dialog is active, I help with tasks, rules, and explanations.";
const FALLBACK_COURTESY_ACKNOWLEDGEMENTS = [
  "Glad to hear it.",
  "You're welcome.",
];
const FALLBACK_COURTESY_FOLLOW_UPS = [
  "What would you like to do next?",
  "Do you want to discuss something else?",
];

const FALLBACK_UNKNOWN_ANSWER =
  "I don't know how to answer that yet. I cannot answer that from local links rules yet. To inspect what I can do, send `List behavior rules`, then `Show behavior rule unknown`. To teach this dialog a response, send: When I say `your prompt`, answer `your answer`. If this still needs a shared Links Notation seed fact or links rule after those checks, use Report issue with the reasoning trace, or export memory to keep a dialog-local rule durable.";

const FALLBACK_CLARIFICATION_ANSWER =
  "I'm sorry for the confusion. I am formal-ai, a deterministic symbolic AI. I can answer greetings, identity questions, concept lookups (what is X?), arithmetic, and parameterized program templates. If you'd like to ask about something specific, try one of those or add a fact in Links Notation.";

// Mutable runtime tables — populated from seed at init(). Each entry is
// `{ text, variants }` so the worker can return either the canonical phrase
// (for deterministic tests and tool calls) or a random variant (for greeting
// randomisation introduced in issue #27). Courtesy responses can also carry
// separated acknowledgement and follow-up fragments for issue #160.
let MULTILINGUAL_ANSWERS = {
  greeting: {
    en: { text: FALLBACK_GREETING_ANSWER, variants: [FALLBACK_GREETING_ANSWER] },
  },
  farewell: {
    en: { text: "Goodbye! Feel free to return any time.", variants: ["Goodbye! Feel free to return any time."] },
  },
  test_status: {
    en: { text: FALLBACK_TEST_STATUS_ANSWER, variants: [FALLBACK_TEST_STATUS_ANSWER] },
  },
  courtesy_response: {
    en: {
      text: FALLBACK_COURTESY_RESPONSE_ANSWER,
      variants: [FALLBACK_COURTESY_RESPONSE_ANSWER],
      acknowledgements: FALLBACK_COURTESY_ACKNOWLEDGEMENTS,
      followUps: FALLBACK_COURTESY_FOLLOW_UPS,
    },
  },
  assistant_free_time: {
    en: {
      text: FALLBACK_ASSISTANT_FREE_TIME_ANSWER,
      variants: [FALLBACK_ASSISTANT_FREE_TIME_ANSWER],
    },
  },
  identity: {
    en: { text: FALLBACK_IDENTITY_ANSWER, variants: [FALLBACK_IDENTITY_ANSWER] },
  },
  assistant_name: {
    en: {
      text: FALLBACK_ASSISTANT_NAME_ANSWER,
      variants: [FALLBACK_ASSISTANT_NAME_ANSWER],
    },
  },
  clarification: {
    en: {
      text: FALLBACK_CLARIFICATION_ANSWER,
      variants: [FALLBACK_CLARIFICATION_ANSWER],
    },
  },
  github_repository_traffic: {
    en: {
      text: "Partly. For a GitHub repository such as {repository}, GitHub can show aggregate traffic to people with push or write access: views, unique visitors, clones, referring sites, and popular content for the recent traffic window. It does not show the identity of an individual visitor. Check GitHub Insights > Traffic or the REST traffic endpoints: {traffic_ui_docs}; {traffic_api_docs}.",
      variants: [
        "Partly. For a GitHub repository such as {repository}, GitHub can show aggregate traffic to people with push or write access: views, unique visitors, clones, referring sites, and popular content for the recent traffic window. It does not show the identity of an individual visitor. Check GitHub Insights > Traffic or the REST traffic endpoints: {traffic_ui_docs}; {traffic_api_docs}.",
      ],
    },
    ru: {
      text: "Частично. Для репозитория GitHub, например {repository}, GitHub показывает агрегированный трафик пользователям с доступом push/write: просмотры, уникальных посетителей, клоны, источники переходов и популярные страницы за недавний период. Он не показывает личность отдельного посетителя. Проверять нужно через Insights > Traffic или REST traffic endpoints: {traffic_ui_docs}; {traffic_api_docs}.",
      variants: [
        "Частично. Для репозитория GitHub, например {repository}, GitHub показывает агрегированный трафик пользователям с доступом push/write: просмотры, уникальных посетителей, клоны, источники переходов и популярные страницы за недавний период. Он не показывает личность отдельного посетителя. Проверять нужно через Insights > Traffic или REST traffic endpoints: {traffic_ui_docs}; {traffic_api_docs}.",
      ],
    },
    hi: {
      text: "आंशिक रूप से। {repository} जैसे GitHub repository के लिए GitHub push या write access वाले लोगों को aggregate traffic दिखा सकता है: views, unique visitors, clones, referring sites, और popular content for the recent traffic window. यह किसी individual visitor की identity नहीं दिखाता। GitHub Insights > Traffic या REST traffic endpoints देखें: {traffic_ui_docs}; {traffic_api_docs}.",
      variants: [
        "आंशिक रूप से। {repository} जैसे GitHub repository के लिए GitHub push या write access वाले लोगों को aggregate traffic दिखा सकता है: views, unique visitors, clones, referring sites, और popular content for the recent traffic window. यह किसी individual visitor की identity नहीं दिखाता। GitHub Insights > Traffic या REST traffic endpoints देखें: {traffic_ui_docs}; {traffic_api_docs}.",
      ],
    },
    zh: {
      text: "部分可以。对于 {repository} 这样的 GitHub 仓库，GitHub 可以向有 push 或 write 权限的人显示聚合流量：views、unique visitors、clones、referring sites 以及近期流量窗口内的 popular content。它不会显示单个访问者的身份。可查看 GitHub Insights > Traffic 或 REST traffic endpoints: {traffic_ui_docs}; {traffic_api_docs}.",
      variants: [
        "部分可以。对于 {repository} 这样的 GitHub 仓库，GitHub 可以向有 push 或 write 权限的人显示聚合流量：views、unique visitors、clones、referring sites 以及近期流量窗口内的 popular content。它不会显示单个访问者的身份。可查看 GitHub Insights > Traffic 或 REST traffic endpoints: {traffic_ui_docs}; {traffic_api_docs}.",
      ],
    },
  },
  unknown: {
    en: { text: FALLBACK_UNKNOWN_ANSWER, variants: [FALLBACK_UNKNOWN_ANSWER] },
  },
};
let CONCEPTS = [];
let CONCEPT_CONTEXTS = [];
let FACTS = [];
let PROJECTS = [];
let BRAINSTORM_SEEDS = {
  triggers: [
    "brainstorm",
    "give me five ideas",
    "give me 5 ideas",
    "give me ten ideas",
    "give me 10 ideas",
    "suggest five",
    "suggest 5",
    "suggest ten",
    "suggest 10",
  ],
  categories: [
    {
      slug: "project_ideas",
      intent: "brainstorm_project_ideas",
      detectionKeywords: [],
      items: [
        "A local Links Notation notebook with searchable traces.",
        "A deterministic code-review checklist generator.",
        "A multilingual prompt-variation test corpus.",
        "A CLI that converts issue requirements into traceable tests.",
        "A source-cache inspector for reproducible agent runs.",
        "A changelog-fragment consistency checker.",
        "A prompt-matrix generator for four-language smoke tests.",
        "A Wikidata anchor verifier for local seed records.",
        "A trace viewer that groups events by solver phase.",
        "A small offline issue-to-test planning tool.",
      ],
    },
  ],
};
let PERSONA_SEEDS = {
  triggers: ["pretend you are", "act as", "roleplay", "explain like you are"],
  defaultPersona: "requested persona",
  bodyTemplate:
    "Roleplay frame recorded for <persona>. I will keep the persona explicit and factual: <body>",
  fallbackBody:
    "relativity says measurements of space and time depend on the observer's motion, while the laws of physics stay consistent.",
  personas: [
    { displayName: "Albert Einstein", aliases: ["einstein"], wikidata: "Q937" },
    { displayName: "Ada Lovelace", aliases: ["ada lovelace"], wikidata: "Q7259" },
    { displayName: "teacher", aliases: ["teacher"], wikidata: "" },
  ],
  topics: [
    {
      slug: "algorithm",
      detectionKeywords: ["algorithm", "algorithms"],
      body:
        "an algorithm is a precise sequence of steps, so a reliable explanation names the inputs, the ordered operations, and the expected result.",
    },
    {
      slug: "time_dilation",
      detectionKeywords: ["time dilation"],
      body:
        "time dilation means clocks can measure different elapsed times when observers move differently or sit in different gravitational fields.",
    },
  ],
};
let COREFERENCE_SEEDS = { pronouns: [], antecedents: [] };
let TOOLS = [];
let SEED_RAW = {};
let NUMERIC_LIST_OPERATIONS_LINO = "";
let CODING_IDIOMS_LINO = "";
let TERMINAL_COMMANDS_LINO = "";
let PROGRAM_PLAN_RULES_LINO = "";
let OPERATION_VOCABULARY_LINO = "";
let MARKET_PRICE_REFERENCES_LINO = "";
let MEANINGS_LINO = "";
let AGENT_INFO = {};
let LANGUAGE_RULES = [
  { language: "ru", start: 0x0400, end: 0x04ff },
  { language: "hi", start: 0x0900, end: 0x097f },
  { language: "zh", start: 0x4e00, end: 0x9fff },
];
let PROMPT_PATTERNS = [];

function seedFileBaseName(path) {
  return String(path || "").replace(/\\/g, "/").split("/").pop() || "";
}

function seedRawText(raw, fileName) {
  const entries = Object.entries(raw || {});
  for (const [path, text] of entries) {
    if (seedFileBaseName(path) === fileName && text) return String(text);
  }
  return "";
}

function seedRawTexts(raw, predicate) {
  return Object.entries(raw || {})
    .filter(([path, text]) => text && predicate(seedFileBaseName(path)))
    .sort(([left], [right]) => seedFileBaseName(left).localeCompare(seedFileBaseName(right)))
    .map(([, text]) => String(text));
}

function hydrateLinoSeedText(raw) {
  NUMERIC_LIST_OPERATIONS_LINO = seedRawText(raw, "numeric-list-operations.lino");
  CODING_IDIOMS_LINO = seedRawText(raw, "coding-idioms.lino");
  TERMINAL_COMMANDS_LINO = seedRawText(raw, "terminal-commands.lino");
  PROGRAM_PLAN_RULES_LINO = seedRawText(raw, "program-plan-rules.lino");
  OPERATION_VOCABULARY_LINO = seedRawText(raw, "operation-vocabulary.lino");
  MARKET_PRICE_REFERENCES_LINO = seedRawText(raw, "market-price-references.lino");
  MEANINGS_LINO = seedRawTexts(
    raw,
    (fileName) => fileName === "meanings.lino" || /^meanings-[a-z0-9-]+\.lino$/.test(fileName),
  ).join("\n");

  cachedNumericListOntology = null;
  cachedCodingIdioms = null;
  cachedTerminalCommandVocabulary = null;
  cachedOperationVocabulary = null;
  cachedProgramPlanRules = null;
  cachedMeaningLexicon = null;
  cachedMarketPriceReferences = null;
}
// Intent routing rules loaded from `seed/intent-routing.lino` at init time.
// `intents` mirror `seed::IntentRoute` from the Rust crate, so the browser
// and the Rust solver behave identically when classifying prompts. The
// fallback below mirrors the contents of `data/seed/intent-routing.lino`
// so the worker remains functional even when the `.lino` fetch fails (for
// example when the demo is opened from `file://`).
let INTENT_ROUTING = {
  intents: [
    {
      id: "intent_greeting",
      slug: "greeting",
      responseLink: "response:greeting",
      keywords: [
        "hi",
        "hello",
        "hey",
        "привет",
        "здравствуйте",
        "шалом",
        "नमस्ते",
        "नमस्कार",
        "सलाम",
        "हाय",
        "你好",
        "您好",
        "嗨",
        "哈喽",
      ],
      phrases: [
        "how are you",
        "how are you doing",
        "how do you do",
        "how is it going",
        "how s it going",
        "how are things",
        "шабат шалом",
        "как дела",
        "как твои дела",
        "как ваши дела",
        "как у тебя дела",
        "как у вас дела",
        "привет как дела",
        "здравствуйте как ваши дела",
        "как поживаешь",
        "как вы поживаете",
        "राम राम",
        "कैसे हो",
        "आप कैसे हैं",
        "तुम कैसे हो",
        "क्या हाल है",
        "आपका क्या हाल है",
        "सब कैसा चल रहा है",
        "早上好",
        "早安",
        "你好吗",
        "你还好吗",
        "你怎么样",
        "您怎么样",
        "最近怎么样",
        "过得怎么样",
      ],
      tokens: ["greet"],
      combos: [],
    },
    {
      id: "intent_farewell",
      slug: "farewell",
      responseLink: "response:farewell",
      keywords: [
        "bye",
        "goodbye",
        "пока",
        "ciao",
        "tschüss",
        "再见",
        "拜拜",
        "回见",
        "अलविदा",
        "विदा",
        "बाय",
        "टाटा",
      ],
      phrases: ["до свидания", "досвидания", "改天见", "后会有期", "फिर मिलेंगे"],
      tokens: [],
      combos: [],
    },
    {
      id: "intent_courtesy_response",
      slug: "courtesy_response",
      responseLink: "response:courtesy_response",
      keywords: ["thanks", "спасибо", "благодарю", "धन्यवाद", "शुक्रिया", "谢谢"],
      phrases: [
        "thank you",
        "i am fine thank you",
        "i am fine thanks",
        "i m fine thank you",
        "i m fine thanks",
        "i am good thank you",
        "i am good thanks",
        "i m good thank you",
        "i m good thanks",
        "fine thank you",
        "fine thanks",
        "good thank you",
        "good thanks",
        "doing well thank you",
        "doing well thanks",
        "у меня все хорошо спасибо",
        "у меня всё хорошо спасибо",
        "все хорошо спасибо",
        "всё хорошо спасибо",
        "хорошо спасибо",
        "нормально спасибо",
        "ого чето начал соображать",
        "ого чёто начал соображать",
        "ого чё то начал соображать",
        "ого что то начал соображать",
        "मैं ठीक हूँ धन्यवाद",
        "ठीक हूँ धन्यवाद",
        "मैं अच्छा हूँ धन्यवाद",
        "我很好谢谢",
        "我很好 谢谢",
        "好的谢谢",
        "好的 谢谢",
      ],
      tokens: [],
      combos: [],
    },
    {
      id: "intent_assistant_free_time",
      slug: "assistant_free_time",
      responseLink: "response:assistant_free_time",
      keywords: [],
      phrases: [
        "what do you do in your free time",
        "what do you do in free time",
        "how do you spend your free time",
        "what do you do when you are not working",
        "что делаешь в свободное время",
        "что ты делаешь в свободное время",
        "чем занимаешься в свободное время",
        "чем ты занимаешься в свободное время",
        "что делаешь когда свободен",
        "खाली समय में क्या करते हो",
        "आप खाली समय में क्या करते हैं",
        "फुर्सत में क्या करते हो",
        "你空闲时间做什么",
        "你有空的时候做什么",
        "你业余时间做什么",
      ],
      tokens: [],
      combos: [],
    },
    {
      id: "intent_test_status",
      slug: "test_status",
      responseLink: "response:test_status",
      keywords: [
        "test",
        "ping",
        "pong",
        "testing",
        "тест",
        "пинг",
        "टेस्ट",
        "परीक्षण",
        "测试",
        "測試",
      ],
      phrases: [
        "test passed",
        "testing 123",
        "are you there",
        "you there",
        "i m here",
        "i am here",
        "я здесь",
        "тест пройден",
        "ты здесь",
        "вы здесь",
        "परीक्षण सफल रहा",
        "मैं यहाँ हूँ",
        "मैं यहां हूं",
        "क्या आप वहाँ हैं",
        "क्या आप वहां हैं",
        "测试通过",
        "測試通過",
        "我在这里",
        "我在這裡",
        "你在吗",
        "您在吗",
        "你在嗎",
        "您在嗎",
      ],
      tokens: [],
      combos: [
        ["test", "passed"],
        ["test", "here"],
        ["testing", "123"],
        ["ping", "test"],
        ["тест", "пройден"],
        ["тест", "здесь"],
        ["परीक्षण", "सफल"],
      ],
    },
    {
      id: "intent_assistant_name",
      slug: "assistant_name",
      responseLink: "response:assistant_name",
      keywords: [],
      phrases: [
        "what is your name",
        "what s your name",
        "what's your name",
        "do you have a name",
        "what should i call you",
        "как твое имя",
        "как твоё имя",
        "как тебя зовут",
        "у тебя есть имя",
        "आपका नाम क्या है",
        "तुम्हारा नाम क्या है",
        "你叫什么名字",
        "您叫什么名字",
        "你的名字是什么",
        "你有名字吗",
      ],
      tokens: [],
      combos: [
        ["what", "your", "name"],
        ["you", "have", "name"],
        ["call", "you"],
        ["как", "тебя", "зовут"],
      ],
    },
    {
      id: "intent_identity",
      slug: "identity",
      responseLink: "response:identity",
      keywords: [],
      phrases: [
        "who are you",
        "what are you",
        "who is formal ai",
        "what is formal ai",
        "who is formalai",
        "what is formalai",
        "tell me about yourself",
        "introduce yourself",
        "let s get acquainted",
        "lets get acquainted",
        "let us get acquainted",
        "let s get to know each other",
        "кто ты",
        "что ты",
        "расскажи о себе",
        "расскажи мне о себе",
        "расскажи про себя",
        "опиши себя",
        "представься",
        "давай знакомиться",
        "давай познакомимся",
        "давайте познакомимся",
        "तुम कौन हो",
        "तू कौन है",
        "आप कौन हैं",
        "अपना परिचय दो",
        "अपने बारे में बताओ",
        "चलो परिचय करते हैं",
        "आइए परिचय करें",
        "चलो एक दूसरे को जानें",
        "你是谁",
        "您是谁",
        "你是什么",
        "介绍一下你自己",
        "告诉我你自己",
        "你是誰",
        "我们认识一下吧",
        "认识一下吧",
        "让我们认识一下",
      ],
      tokens: [],
      combos: [
        ["who", "you"],
        ["what", "you"],
        ["tell", "yourself"],
        ["introduce", "yourself"],
        ["let", "s", "acquainted"],
        ["lets", "acquainted"],
        ["let", "us", "acquainted"],
        ["know", "each", "other"],
        ["кто", "ты"],
        ["что", "ты"],
        ["расскажи", "себе"],
        ["опиши", "себя"],
        ["давай", "знакомиться"],
        ["давай", "познакомимся"],
        ["давайте", "познакомимся"],
        ["चलो", "परिचय"],
        ["आइए", "परिचय"],
        ["who", "formal", "ai"],
        ["what", "formal", "ai"],
      ],
    },
  ],
  articlePrefixes: ["the ", "a ", "an "],
  tracePrefixes: ["answer_", "trace_"],
};

function fallbackEntry(intent) {
  if (intent === "greeting") {
    return { text: FALLBACK_GREETING_ANSWER, variants: [FALLBACK_GREETING_ANSWER] };
  }
  if (intent === "courtesy_response") {
    return {
      text: FALLBACK_COURTESY_RESPONSE_ANSWER,
      variants: [FALLBACK_COURTESY_RESPONSE_ANSWER],
      acknowledgements: FALLBACK_COURTESY_ACKNOWLEDGEMENTS,
      followUps: FALLBACK_COURTESY_FOLLOW_UPS,
    };
  }
  if (intent === "assistant_free_time") {
    return {
      text: FALLBACK_ASSISTANT_FREE_TIME_ANSWER,
      variants: [FALLBACK_ASSISTANT_FREE_TIME_ANSWER],
    };
  }
  if (intent === "identity") {
    return { text: FALLBACK_IDENTITY_ANSWER, variants: [FALLBACK_IDENTITY_ANSWER] };
  }
  if (intent === "assistant_name") {
    return {
      text: FALLBACK_ASSISTANT_NAME_ANSWER,
      variants: [FALLBACK_ASSISTANT_NAME_ANSWER],
    };
  }
  if (intent === "test_status") {
    return { text: FALLBACK_TEST_STATUS_ANSWER, variants: [FALLBACK_TEST_STATUS_ANSWER] };
  }
  if (intent === "clarification") {
    return {
      text: FALLBACK_CLARIFICATION_ANSWER,
      variants: [FALLBACK_CLARIFICATION_ANSWER],
    };
  }
  return { text: FALLBACK_UNKNOWN_ANSWER, variants: [FALLBACK_UNKNOWN_ANSWER] };
}

function normalizeEntry(value, intent) {
  if (value && typeof value === "object" && typeof value.text === "string") {
    const variants =
      Array.isArray(value.variants) && value.variants.length > 0
        ? value.variants
        : [value.text];
    const acknowledgements = Array.isArray(value.acknowledgements)
      ? value.acknowledgements.filter(Boolean)
      : [];
    const followUps = Array.isArray(value.followUps)
      ? value.followUps.filter(Boolean)
      : [];
    return {
      text: value.text,
      variants: variants,
      acknowledgements: acknowledgements,
      followUps: followUps,
    };
  }
  if (typeof value === "string") {
    return {
      text: value,
      variants: [value],
      acknowledgements: [],
      followUps: [],
    };
  }
  return fallbackEntry(intent);
}

function responseEntryFor(intent, language) {
  const table = MULTILINGUAL_ANSWERS[intent] || {};
  const raw = table[language] || table.en || fallbackEntry(intent);
  return normalizeEntry(raw, intent);
}

function answerFor(intent, language, options) {
  const opts = options || {};
  const entry = responseEntryFor(intent, language);
  if (opts.randomize && Array.isArray(entry.variants) && entry.variants.length > 1) {
    const idx = Math.floor(Math.random() * entry.variants.length);
    return entry.variants[idx] || entry.text;
  }
  return entry.text;
}

function normalizeAssistantNamePreference(value) {
  return String(value || "")
    .replace(/[\r\n\t]+/g, " ")
    .replace(/\s+/g, " ")
    .trim()
    .replace(/^[`"']+|[`"']+$/g, "")
    .trim()
    .slice(0, 64);
}

function assistantNameAnswer(language, preferences) {
  const name = normalizeAssistantNamePreference(
    preferences && preferences.assistantName,
  );
  if (!name) return answerFor("assistant_name", language);
  if (language === "ru") {
    return `Меня зовут ${name}. Я formal AI.`;
  }
  if (language === "hi") {
    return `मेरा नाम ${name} है। मैं formal AI हूँ।`;
  }
  if (language === "zh") {
    return `我的名字是 ${name}。我是 formal AI。`;
  }
  return `My name is ${name}. I'm formal AI.`;
}

// Mirrors `src/engine.rs::UNKNOWN_OPENERS_*`. The first entry of each pool
// equals the opener already embedded in the seed text so the "with-variations"
// answer is a strict superset of the seed. Different prompts get different
// openers; the same prompt always picks the same one (FNV-1a hash, mirrored
// from `stableBehaviorRuleId`).
const UNKNOWN_OPENERS_BY_LANGUAGE = {
  en: [
    "I don't know how to answer that yet.",
    "I didn't understand you.",
    "I'm not sure how to respond to that yet.",
    "I haven't learned to answer that yet.",
    "That one is new to me.",
  ],
  ru: [
    "Я пока не знаю, как ответить на это.",
    "Я тебя не понял.",
    "Я не уверен, как на это ответить.",
    "Я ещё не научился отвечать на это.",
    "Это для меня новое.",
  ],
  hi: [
    "मुझे अभी इसका उत्तर देना नहीं आता।",
    "मैं समझ नहीं पाया।",
    "मुझे यकीन नहीं है कि कैसे उत्तर दूँ।",
    "मैंने अभी तक यह उत्तर देना नहीं सीखा।",
    "यह मेरे लिए नया है।",
  ],
  zh: [
    "我还不知道如何回答这个问题。",
    "我不太明白你说的意思。",
    "我不确定该如何回答。",
    "我还没有学会回答这个问题。",
    "这对我来说是新的。",
  ],
};

function unknownOpenersFor(language) {
  return UNKNOWN_OPENERS_BY_LANGUAGE[language] || UNKNOWN_OPENERS_BY_LANGUAGE.en;
}

function selectUnknownOpener(prompt, language) {
  const fromWasm = wasmSelectUnknownOpener(prompt, language);
  if (fromWasm) return fromWasm;
  const pool = unknownOpenersFor(language);
  const trimmed = String(prompt || "").trim();
  if (trimmed === "") return pool[0];
  const id = stableBehaviorRuleId("unknown_opener", trimmed);
  const hex = id.split("_").pop() || "0";
  let value;
  try {
    value = BigInt(`0x${hex}`);
  } catch (_err) {
    value = 0n;
  }
  const index = Number(value % BigInt(pool.length));
  return pool[index] || pool[0];
}

function stripLeadingUnknownOpener(text, language) {
  const trimmed = String(text || "").trimStart();
  const openers = unknownOpenersFor(language);
  for (const known of openers) {
    if (trimmed.startsWith(known)) {
      return trimmed.slice(known.length).trimStart();
    }
  }
  for (const separator of [". ", "。", "। "]) {
    const idx = trimmed.indexOf(separator);
    if (idx >= 0) {
      return trimmed.slice(idx + separator.length).trimStart();
    }
  }
  return trimmed;
}

function unknownAnswerWithVariation(prompt, language) {
  const seedText = answerFor("unknown", language);
  const opener = selectUnknownOpener(prompt, language);
  const body = stripLeadingUnknownOpener(seedText, language);
  if (!body) return opener;
  return `${opener} ${body}`;
}

function numericPreference(value, fallback, min, max) {
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) return fallback;
  return Math.min(max, Math.max(min, parsed));
}

function pickVariant(values, randomize) {
  if (!Array.isArray(values) || values.length === 0) return "";
  if (!randomize || values.length === 1) return values[0];
  return values[Math.floor(Math.random() * values.length)] || values[0];
}

function includeFollowUpQuestion(probability, randomize) {
  if (probability <= 0) return false;
  if (probability >= 1) return true;
  if (!randomize) return probability >= 0.5;
  return Math.random() < probability;
}

function courtesyResponseFor(language, preferences) {
  const prefs = preferences || {};
  const entry = responseEntryFor("courtesy_response", language);
  const temperature = numericPreference(prefs.temperature, 0.7, 0, 1);
  const followUpProbability = numericPreference(
    prefs.followUpProbability,
    0.75,
    0,
    1,
  );
  const randomize = temperature > 0;
  const acknowledgements =
    entry.acknowledgements.length > 0 ? entry.acknowledgements : [entry.text];
  const followUps = entry.followUps;
  const acknowledgement = pickVariant(acknowledgements, randomize);
  const includeFollowUp =
    followUps.length > 0 &&
    includeFollowUpQuestion(followUpProbability, randomize);
  return {
    content: includeFollowUp
      ? `${acknowledgement} ${pickVariant(followUps, randomize)}`
      : acknowledgement,
    temperature: temperature,
    randomize: randomize,
    followUpProbability: followUpProbability,
    followUpIncluded: includeFollowUp,
  };
}

function definitionFusionByDefault(preferences) {
  const value = preferences && preferences.definitionFusion;
  if (value === true) return true;
  if (value === false) return false;
  const normalized = String(value || "").trim().toLowerCase();
  return ["auto", "on", "true", "1", "merge", "fusion"].includes(normalized);
}

// Language detection and prompt normalization are owned by the Rust core
// (`src/web_engine_core.rs`) and exposed to the worker through the WASM
// exports `engine_detect_language` and `engine_normalize_prompt`. The JS
// branches below are pre-WASM fallbacks used during init() and on browsers
// that could not instantiate the worker — they must stay byte-for-byte
// compatible with the Rust path so the offline trace and the live answer
// agree (PR #134 feedback 4489651616).
// Issue #556: during a response-language follow-up the solver replays the
// previous request with a language forced onto every localizable handler. The
// handlers derive their output language from detectLanguage(prompt), so the
// forced language is applied here — a single seam that every handler already
// reads, mirroring how SolverConfig.forced_response_language flows through the
// Rust dispatch. solve() sets this via setForcedResponseLanguage() and always
// restores the previous value, so nested replays remain balanced.
let FORCED_RESPONSE_LANGUAGE = null;

function setForcedResponseLanguage(language) {
  const previous = FORCED_RESPONSE_LANGUAGE;
  FORCED_RESPONSE_LANGUAGE = isKnownResponseLanguage(language) ? language : null;
  return previous;
}

function detectLanguage(prompt) {
  if (FORCED_RESPONSE_LANGUAGE) return FORCED_RESPONSE_LANGUAGE;
  const text = String(prompt || "");
  const fromWasm = wasmDetectLanguage(text);
  if (fromWasm !== null) {
    if (fromWasm === "unknown") {
      return AGENT_INFO.default_language || "en";
    }
    return fromWasm;
  }
  for (const ch of text) {
    const code = ch.codePointAt(0);
    for (const rule of LANGUAGE_RULES) {
      if (
        rule.language !== "en" &&
        code >= rule.start &&
        code <= rule.end
      ) {
        return rule.language;
      }
    }
  }
  if (/[a-zA-Z]/.test(text)) return "en";
  return AGENT_INFO.default_language || "en";
}

// Issue #324: the user can choose which language drives responses. The default
// ("last_message") answers in the detected language of the current message
// (fixing the Russian-prompt/English-answer bug). "preferred" pins responses to
// an explicitly selected language and "ui" follows the UI-language preference.
// Both fall back to the detected language when their source is "auto"/unset so
// the deterministic default behavior is never lost.
const RESPONSE_LANGUAGE_MODES = ["last_message", "preferred", "ui"];

function isKnownResponseLanguage(slug) {
  return slug === "en" || slug === "ru" || slug === "hi" || slug === "zh";
}

function responseLanguageFor(detected, preferences, userContext) {
  const prefs = preferences || {};
  const mode = RESPONSE_LANGUAGE_MODES.includes(prefs.responseLanguage)
    ? prefs.responseLanguage
    : "last_message";
  if (mode === "preferred" && isKnownResponseLanguage(prefs.preferredLanguage)) {
    return prefs.preferredLanguage;
  }
  if (mode === "ui") {
    if (isKnownResponseLanguage(prefs.uiLanguage)) return prefs.uiLanguage;
    // "auto" UI language follows the browser; fall back to the detected
    // message language when no concrete browser language is supplied.
    // `browserLanguages` may arrive as an array or a comma-joined string
    // (see `collectUserContext` in app.js).
    const raw = userContext ? userContext.browserLanguages : null;
    const browser = Array.isArray(raw)
      ? raw
      : typeof raw === "string"
        ? raw.split(",")
        : [];
    for (const tag of browser) {
      const slug = String(tag || "").slice(0, 2).toLowerCase();
      if (isKnownResponseLanguage(slug)) return slug;
    }
  }
  return detected;
}

// CONCEPTS is populated from `seed/concepts.lino` at init() time.

function normalizePrompt(prompt) {
  const text = String(prompt || "");
  const fromWasm = wasmNormalizePrompt(text);
  if (fromWasm !== null) return fromWasm;
  // Keep letters, numbers and every Unicode mark (category M): Devanagari
  // matras, the nukta and the virama are marks, so a bare \p{L}\p{N} filter
  // would strip them and corrupt Hindi words (issue #312). Mark-awareness via
  // \p{M} mirrors the Rust `normalize_prompt`, which keeps `is_alphanumeric()`
  // characters plus its script-combining-mark ranges. Crucially it does NOT
  // keep the whole U+0900–U+097F block: Indic punctuation such as the danda
  // "।" (U+0964) is category Po, so both sides collapse it to a space. The
  // boundary-aware role matcher (issue #386) depends on that parity — a
  // retained danda would defeat the whole-token match for phrases like
  // "अपना परिचय दो।".
  return text
    .toLowerCase()
    .replace(/[^\p{L}\p{N}\p{M}]+/gu, " ")
    .trim();
}

function normalizeConceptTerm(value) {
  let lower = String(value || "").toLowerCase();
  for (const prefix of ["the ", "a ", "an "]) {
    if (lower.startsWith(prefix)) {
      lower = lower.slice(prefix.length);
      break;
    }
  }
  return lower.trim().replace(/[?.!,;:]+$/g, "").trim();
}

function recordMatchesTerm(record, normalized) {
  return (
    normalizeConceptTerm(record.term) === normalized ||
    normalizeConceptTerm(record.slug) === normalized ||
    (Array.isArray(record.aliases) &&
      record.aliases.some(
        (alias) => normalizeConceptTerm(alias) === normalized,
      ))
  );
}

function recordMatchesQueryTerm(record, normalized, contextNormalized) {
  if (recordMatchesTerm(record, normalized)) return true;
  if (!contextNormalized) return false;
  return recordMatchesTerm(record, `${normalized} ${contextNormalized}`);
}

function contextRecordMatches(contextRecord, contextNormalized) {
  if (!contextRecord) return false;
  if (
    Array.isArray(contextRecord.aliases) &&
    contextRecord.aliases.some(
      (alias) => normalizeConceptTerm(alias) === contextNormalized,
    )
  ) {
    return true;
  }
  return (
    Array.isArray(contextRecord.labels) &&
    contextRecord.labels.some(
      (label) => normalizeConceptTerm(label.text) === contextNormalized,
    )
  );
}

function resolveContextRecord(contextNormalized) {
  if (!contextNormalized) return null;
  for (const record of CONCEPT_CONTEXTS) {
    if (contextRecordMatches(record, contextNormalized)) return record;
  }
  return null;
}

function recordHasContext(record, contextNormalized) {
  if (
    Array.isArray(record.contexts) &&
    record.contexts.some(
      (candidate) => normalizeConceptTerm(candidate) === contextNormalized,
    )
  ) {
    return true;
  }
  // Registry fallback: resolve the user-supplied context through the
  // concept-contexts registry and see whether the resolved record's slug is
  // referenced by the concept's `contextLinks` list. Matches the Rust
  // ranker (src/concepts.rs::record_has_context).
  const contextRecord = resolveContextRecord(contextNormalized);
  if (contextRecord && Array.isArray(record.contextLinks)) {
    return record.contextLinks.some(
      (slug) => String(slug).trim() === contextRecord.slug,
    );
  }
  return false;
}

function localizedConceptFor(record, language) {
  if (!record || !Array.isArray(record.localized)) return null;
  return (
    record.localized.find((loc) => loc && loc.language === language) ||
    record.localized.find((loc) => loc && loc.language === "en") ||
    null
  );
}

function contextLabelFor(contextRecord, language) {
  if (!contextRecord || !Array.isArray(contextRecord.labels)) {
    return null;
  }
  const exact = contextRecord.labels.find(
    (label) => label && label.language === language,
  );
  if (exact && exact.text) return exact.text;
  const english = contextRecord.labels.find(
    (label) => label && label.language === "en",
  );
  if (english && english.text) return english.text;
  return contextRecord.slug || null;
}

function rankConceptForPair(termRaw, contextRaw) {
  const normalized = normalizeConceptTerm(termRaw);
  if (!normalized) return null;
  const contextNormalized = contextRaw ? normalizeConceptTerm(contextRaw) : "";

  const termMatches = CONCEPTS.filter((record) =>
    recordMatchesQueryTerm(record, normalized, contextNormalized),
  );
  if (termMatches.length === 0) return null;

  if (contextNormalized) {
    const ctxHit = termMatches.find((record) =>
      recordHasContext(record, contextNormalized),
    );
    if (ctxHit) {
      return {
        record: ctxHit,
        contextMatch: true,
        context: contextNormalized,
      };
    }
  }

  // No context match: prefer records with no contexts declared.
  termMatches.sort((a, b) => {
    const ac = (Array.isArray(a.contexts) && a.contexts.length > 0) ? 1 : 0;
    const bc = (Array.isArray(b.contexts) && b.contexts.length > 0) ? 1 : 0;
    return ac - bc;
  });
  return {
    record: termMatches[0],
    contextMatch: false,
    context: contextNormalized || null,
  };
}

function lookupConceptQuery(query) {
  if (!query) return null;
  const direct = rankConceptForPair(query.term, query.context);
  if (query.context) {
    const reversed = rankConceptForPair(query.context, query.term);
    if (reversed && (!direct || (!direct.contextMatch && reversed.contextMatch))) {
      return reversed;
    }
  }
  return direct || null;
}

function lookupConcept(term) {
  const hit = lookupConceptQuery({ term: term, context: null });
  return hit ? hit.record : null;
}

// Default concept-lookup patterns when seed/prompt-patterns.lino is missing.
// Sorted longest-first so "what is a " beats "what is " when both match.
const DEFAULT_CONCEPT_SUFFIXES = [
  " का अर्थ बताओ",
  " क्या होता है",
  " क्या है",
  " कौन हैं",
  " कौन है",
  "的意思是什么",
  "是什么意思",
  "是甚麼",
  "是什么",
  "是誰",
  "是谁",
];
const DEFAULT_CONCEPT_PREFIXES = [
  "what is a ",
  "what is an ",
  "what is the ",
  "what is ",
  "what's a ",
  "what's an ",
  "what's the ",
  "what's ",
  "what do ",
  "what does ",
  "tell me about ",
  "tell me what ",
  "define ",
  "explain ",
  "describe ",
  "who is ",
  "who was ",
  "что такое ",
  "что это ",
  "что означает слово ",
  "кто такой ",
  "кто такая ",
  "кто это ",
  "расскажи о ",
  "расскажи про ",
  "назови ",
  "опиши ",
  "объясни ",
  "什么是",
  "甚麼是",
  "请解释",
  "请说说",
  "介绍一下",
];

function conceptPatternsByKind(kind) {
  const matches = PROMPT_PATTERNS.filter(
    (p) => p && p.intent === "concept_lookup" && p.kind === kind && p.text,
  ).map((p) => p.text);
  // Sort longest-first so more specific patterns win.
  matches.sort((a, b) => b.length - a.length);
  if (matches.length > 0) return matches;
  if (kind === "suffix") return DEFAULT_CONCEPT_SUFFIXES;
  if (kind === "prefix") return DEFAULT_CONCEPT_PREFIXES;
  return [];
}

let cachedConceptResponseLanguageMarkers = null;

function meaningDefinedLanguageCode(meaning) {
  for (const slug of meaning && Array.isArray(meaning.definedBy)
    ? meaning.definedBy
    : []) {
    if (slug === "language_english") return "en";
    if (slug === "language_russian") return "ru";
    if (slug === "language_hindi") return "hi";
    if (slug === "language_chinese") return "zh";
  }
  return null;
}

function conceptResponseLanguageMarkers() {
  if (cachedConceptResponseLanguageMarkers) {
    return cachedConceptResponseLanguageMarkers;
  }
  const markers = [];
  for (const meaning of meaningsWithRole(ROLE_RESPONSE_LANGUAGE_MARKER)) {
    const language = meaningDefinedLanguageCode(meaning);
    if (!language) continue;
    for (const word of meaning.words || []) {
      const marker = String(word || "").toLowerCase();
      if (marker) markers.push({ marker, language });
    }
  }
  markers.sort((a, b) => b.marker.length - a.marker.length);
  cachedConceptResponseLanguageMarkers = markers;
  return markers;
}

function stripTrailingResponseLanguageMarker(original, lower) {
  const sourceOriginal = String(original || "").trim();
  const sourceLower = String(
    lower || sourceOriginal.toLowerCase(),
  ).trim();
  for (const { marker, language } of conceptResponseLanguageMarkers()) {
    if (!sourceLower.endsWith(marker)) continue;
    const start = sourceLower.length - marker.length;
    if (start === 0) continue;
    const before = sourceLower.slice(0, start).slice(-1);
    if (!isResponseLanguageMarkerBoundary(before, marker)) continue;
    const stemOriginal = sourceOriginal
      .slice(0, start)
      .trim()
      .replace(/[,，;；:：、]+$/u, "")
      .trim();
    const stemLower = sourceLower
      .slice(0, start)
      .trim()
      .replace(/[,，;；:：、]+$/u, "")
      .trim();
    if (!stemLower) continue;
    return {
      original: stemOriginal || stemLower,
      lower: stemLower,
      language,
    };
  }
  return { original: sourceOriginal, lower: sourceLower, language: null };
}

function isResponseLanguageMarkerBoundary(before, marker) {
  return (
    /\s/u.test(before) ||
    /^[,，;；:：、]$/u.test(before) ||
    containsCjk(marker)
  );
}

function splitTermAndContext(bodyOriginal, bodyLower) {
  const delimiters = conceptPatternsByKind("context_delimiter");
  for (const delimiter of delimiters) {
    const idx = bodyLower.indexOf(delimiter);
    if (idx >= 0) {
      const term = bodyLower.slice(0, idx).trim();
      const context = bodyLower.slice(idx + delimiter.length).trim();
      const termOriginal = bodyOriginal.slice(0, idx).trim();
      const contextOriginal = bodyOriginal
        .slice(idx + delimiter.length)
        .trim();
      if (term && context) {
        return {
          term: term,
          context: context,
          termOriginal: termOriginal || term,
          contextOriginal: contextOriginal || context,
        };
      }
    }
  }
  return {
    term: bodyLower,
    context: null,
    termOriginal: bodyOriginal || bodyLower,
    contextOriginal: null,
  };
}

function stripLeadingRequest(input) {
  const lower = input.toLowerCase();
  const prefixes = [
    "please tell me,",
    "please tell me",
    "tell me,",
    "tell me",
  ];
  const questionStarts = ["who ", "what ", "what's ", "who's "];
  for (const prefix of prefixes) {
    if (!lower.startsWith(prefix)) continue;
    const rest = input.slice(prefix.length).trimStart();
    const restLower = rest.toLowerCase();
    if (
      questionStarts.some((questionStart) =>
        restLower.startsWith(questionStart),
      )
    ) {
      return rest;
    }
  }
  return input;
}
