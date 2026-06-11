const {
  createElement: h,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} = React;

// The meta tag is stamped with the published crate version by
// `scripts/stamp-pages-artifact.sh` during the GitHub Pages deploy. When the
// site is served straight from the source tree (e.g. local Playwright runs)
// the placeholder is preserved verbatim; we fall back to `"dev"` so issue
// reports never advertise a hardcoded stale version like `0.16.0`.
const APP_VERSION = (() => {
  const raw = document.querySelector('meta[name="formal-ai-version"]')?.content;
  if (!raw || raw.startsWith("__") || raw.endsWith("__")) {
    return "dev";
  }
  return raw;
})();
const ASSET_VERSION =
  typeof window !== "undefined" ? window.FORMAL_AI_ASSET_VERSION || "" : "";
const ISSUE_REPOSITORY = "link-assistant/formal-ai";
const ISSUE_LABELS = "bug";
const SOURCE_CODE_URL = `https://github.com/${ISSUE_REPOSITORY}`;
const UNKNOWN_ANSWER =
  "I don't know how to answer that yet. I cannot answer that from local Links Notation rules yet. To inspect what I can do, send `List behavior rules`, then `Show behavior rule unknown`. To teach this dialog a response, send: When I say `your prompt`, answer `your answer`. If this still needs a shared Links Notation seed fact or rule after those checks, use Report issue with the reasoning trace, or export memory to keep a dialog-local rule durable.";
const IDENTITY_ANSWER =
  "I am formal-ai, a deterministic symbolic AI implementation that answers from local Links Notation rules and OpenAI-compatible API shapes. I do not perform neural inference in this demo.";
const ASSISTANT_FREE_TIME_ANSWER =
  "I do not have free time the way a person does. Between prompts I am idle; when the dialog is active, I help with tasks, rules, and explanations.";
const ASSISTANT_NAME_ANSWER =
  "I'm formal AI, and currently I don't have a name. But you can name me as you like.";
const COURTESY_ACKNOWLEDGEMENTS = [
  "Glad to hear it.",
  "You're welcome.",
  "Good to hear.",
  "Happy to hear that.",
];
const COURTESY_FOLLOW_UPS = [
  "What would you like to do next?",
  "Do you want to discuss something else?",
  "Is there anything else you want to work on?",
  "Would you like to explore another topic?",
];

// Issue #27: the sidebar advertises every prompt family that has a deterministic
// symbolic rule or seed-backed answer in the engine. The list intentionally
// mirrors the multilingual + hello-world end-to-end tests so any regression in
// the seed surfaces immediately when a user clicks the prompt.
const EXAMPLE_PROMPTS = [
  { label: "Greeting (en)", text: "Hi" },
  { label: "Greeting (ru)", text: "Привет" },
  { label: "Greeting (hi)", text: "नमस्ते" },
  { label: "Greeting (zh)", text: "你好" },
  { label: "Farewell (en)", text: "Goodbye" },
  { label: "Farewell (ru)", text: "До свидания" },
  { label: "Farewell (hi)", text: "अलविदा" },
  { label: "Farewell (zh)", text: "再见" },
  { label: "Identity (en)", text: "Who are you?" },
  { label: "Identity (ru)", text: "Кто ты?" },
  { label: "Identity (hi)", text: "तुम कौन हो?" },
  { label: "Identity (zh)", text: "你是谁?" },
  { label: "Clarification (en)", text: "I don't understand" },
  { label: "Clarification (ru)", text: "не понял" },
  { label: "Clarification (hi)", text: "समझ नहीं आया" },
  { label: "Clarification (zh)", text: "我不明白" },
  { label: "Capabilities (en)", text: "What can you do?" },
  { label: "Capabilities (ru)", text: "Что ты умеешь?" },
  { label: "Behavior rules", text: "List behavior rules" },
  { label: "Self facts", text: "List all facts you know about yourself" },
  { label: "Hello world (Rust)", text: "Write me hello world program in Rust" },
  { label: "Hello world (Python)", text: "Create a hello world example in Python" },
  { label: "Hello world (JavaScript)", text: "Write hello world in JavaScript" },
  { label: "Hello world (TypeScript)", text: "Write hello world in TypeScript" },
  { label: "Hello world (Go)", text: "Show hello world in Go" },
  { label: "Hello world (C)", text: "Show hello world in C" },
  { label: "Calculation (en)", text: "What is 2 + 2?" },
  { label: "Calculation (ru)", text: "Сколько будет два плюс два?" },
  { label: "Concept (en)", text: "What is Rust?" },
  { label: "Concept (en/Wikipedia)", text: "Who is Donald Trump?" },
  { label: "Concept (ru/Wikipedia)", text: "Кто такой Илон Маск?" },
  { label: "Concept (ru)", text: "Что такое Википедия?" },
  { label: "Concept (hi)", text: "विकिपीडिया क्या है?" },
  { label: "Concept (zh)", text: "维基百科是什么?" },
  { label: "Concept in context", text: "What is IIR in machine learning?" },
  { label: "Summarization", text: "Summarize this conversation" },
  { label: "Brainstorming", text: "Brainstorm 5 small tools for link notation." },
  { label: "Fact Q&A (en)", text: "Who wrote The Lord of the Rings?" },
  { label: "Fact Q&A (ru)", text: "столица россии" },
  { label: "Fact Q&A (hi)", text: "जापान की राजधानी क्या है?" },
  { label: "Fact Q&A (zh)", text: "日本的首都是什么?" },
  { label: "Navigate URL", text: "Navigate to github.com" },
  { label: "Fetch URL", text: "Сделай запрос к google.com" },
  { label: "Web search", text: "Search the web for Nikola Tesla" },
  { label: "Coreference", text: "What features make it different from C?" },
  { label: "Roleplay", text: "Pretend you are Albert Einstein and explain relativity to a teenager." },
  { label: "Idiom (ru)", text: "Купи слона" },
  { label: "Recall (en)", text: "When did I ask about Rust?" },
  { label: "Recall (cross-conv)", text: "Find Wikipedia in another conversation" },
  { label: "Export memory", text: "Export memory" },
  { label: "Import memory", text: "Import memory" },
];

// Issue #27 R5: the demo iterates through the same Example prompts list so
// every advertised feature is exercised. The greeting variants come from
// `EXAMPLE_PROMPTS` (`Greeting (...)` rows) and feature prompts are the
// remainder, minus actions that trigger downloads / file pickers.
const DEMO_GREETING_LABELS = new Set([
  "Greeting (en)",
  "Greeting (ru)",
  "Greeting (hi)",
  "Greeting (zh)",
]);
const DEMO_EXCLUDED_LABELS = new Set(["Export memory", "Import memory"]);
const pendingMemoryWrites = new Set();

function demoGreetings() {
  return EXAMPLE_PROMPTS.filter((entry) => DEMO_GREETING_LABELS.has(entry.label));
}

function demoFeaturePrompts() {
  return EXAMPLE_PROMPTS.filter(
    (entry) =>
      !DEMO_GREETING_LABELS.has(entry.label) &&
      !DEMO_EXCLUDED_LABELS.has(entry.label),
  );
}

// Persistent cursors so each demo cycle advances through the lists rather
// than repeating the same prompts forever. Wraps when the cursor runs off
// the end.
let demoGreetingCursor = 0;
let demoFeatureCursor = 0;

// Issue #27 / #196: typing "Export memory", "Import memory", or "Reset
// memory" (or a translation) in the chat input should trigger the matching
// toolbar action so the deterministic chat surface stays in sync with the UI.
// Each phrase is normalised to lower-case ASCII spaces so punctuation and
// casing differences do not break the trigger.
const MEMORY_ACTION_PHRASES = {
  export: [
    "export memory",
    "export your memory",
    "export the memory",
    "export full memory",
    "экспорт памяти",
    "экспортировать память",
    "экспортируй память",
    "экспортируй свою память",
    "स्मृति निर्यात करें",
    "अपनी स्मृति निर्यात करें",
    "导出记忆",
    "导出你的记忆",
    "导出全部记忆",
  ],
  import: [
    "import memory",
    "import new memory",
    "import your new memory",
    "import your memory",
    "импорт памяти",
    "импортировать память",
    "импортируй память",
    "импортируй новую память",
    "स्मृति आयात करें",
    "नई स्मृति आयात करें",
    "अपनी नई स्मृति आयात करें",
    "导入记忆",
    "导入新记忆",
    "导入你的新记忆",
  ],
  reset: [
    "reset memory",
    "clear memory",
    "reset your memory",
    "clear your memory",
    "сброс памяти",
    "сбросить память",
    "очистить память",
    "сбрось память",
    "स्मृति रीसेट करें",
    "स्मृति साफ करें",
    "अपनी स्मृति रीसेट करें",
    "重置记忆",
    "清空记忆",
    "重置你的记忆",
  ],
};

function normalizeMemoryPrompt(text) {
  return String(text || "")
    .toLowerCase()
    .replace(/[\s  -​]+/g, " ")
    .replace(/[!?.,;:。!?,;:、]+$/g, "")
    .trim();
}

function recognizeMemoryAction(text) {
  const normalized = normalizeMemoryPrompt(text);
  if (!normalized) return null;
  if (MEMORY_ACTION_PHRASES.export.some((phrase) => normalized === phrase)) {
    return "export";
  }
  if (MEMORY_ACTION_PHRASES.import.some((phrase) => normalized === phrase)) {
    return "import";
  }
  if (MEMORY_ACTION_PHRASES.reset.some((phrase) => normalized === phrase)) {
    return "reset";
  }
  return null;
}

function includesAnyText(value, terms) {
  return terms.some((term) => value.includes(term));
}

function matchesAnyPattern(value, patterns) {
  return patterns.some((pattern) => pattern.test(value));
}

const COMMAND_ON_TERMS = [
  "turn on",
  "enable",
  "show",
  "start",
  "включи",
  "включить",
  "покажи",
  "запусти",
  "开启",
  "打开",
  "चालू",
  "enable",
];

const COMMAND_OFF_TERMS = [
  "turn off",
  "disable",
  "hide",
  "stop",
  "выключи",
  "выключить",
  "отключи",
  "скрой",
  "останови",
  "关闭",
  "隐藏",
  "बंद",
  "disable",
];

function detectToggleCommand(normalized, featureTerms) {
  if (!includesAnyText(normalized, featureTerms)) return null;
  if (includesAnyText(normalized, COMMAND_OFF_TERMS)) return false;
  if (includesAnyText(normalized, COMMAND_ON_TERMS)) return true;
  return null;
}

const UI_LANGUAGE_COMMAND_TERMS = [
  "switch",
  "change",
  "set",
  "use",
  "select",
  "configure",
  "переключи",
  "переключить",
  "смени",
  "сменить",
  "измени",
  "изменить",
  "установи",
  "установить",
  "поставь",
  "поставить",
  "выбери",
  "выбрать",
  "используй",
  "использовать",
  "поменяй",
  "поменять",
  "настрой",
  "настроить",
  "切换",
  "设置",
  "使用",
  "选择",
  "बदल",
  "सेट",
  "चुन",
];

const UI_LANGUAGE_OBJECT_TERMS = [
  "ui language",
  "interface language",
  "app language",
  "application language",
  "language",
  "язык интерфейса",
  "язык приложения",
  "язык ui",
  "язык",
  "语言",
  "भाषा",
];

const UI_LANGUAGE_SHORT_PATTERNS = [
  /^(?:ui language|interface language|app language|application language|language)\s*(?:=|:|to)?\s*(?:russian|english|chinese|hindi|auto|system|ru|en|zh|hi)$/u,
  /^язык(?:\s+интерфейса|\s+приложения)?\s*(?:=|:|на)?\s*(?:русский|английский|китайский|хинди|авто|системный|ru|en|zh|hi)$/u,
  /^(?:русский|английский|китайский|хинди|авто|системный)\s+язык(?:\s+интерфейса|\s+приложения)?$/u,
  /^(?:俄语|英语|中文|汉语|自动)\s*语言$/u,
  /^भाषा\s*(?:=|:)?\s*(?:हिन्दी|हिंदी|अंग्रेज़ी|अंग्रेजी|auto|system)$/u,
];

function isExplicitUiLanguageCommand(normalized) {
  if (matchesAnyPattern(normalized, UI_LANGUAGE_SHORT_PATTERNS)) return true;
  if (!includesAnyText(normalized, UI_LANGUAGE_OBJECT_TERMS)) return false;
  return includesAnyText(normalized, UI_LANGUAGE_COMMAND_TERMS);
}

function commandNumberValue(normalized, terms) {
  if (!includesAnyText(normalized, terms)) return null;
  const match = normalized.match(/(\d+(?:[.,]\d+)?)\s*%?/);
  if (!match) return null;
  const raw = Number(match[1].replace(",", "."));
  if (!Number.isFinite(raw)) return null;
  if (normalized.includes("%") || raw > 1) {
    return clampNumber(raw / 100, 0, 1, 0);
  }
  return clampNumber(raw, 0, 1, 0);
}

function sanitizeAssistantNameInput(value) {
  return String(value || "")
    .replace(/[\r\n\t]+/g, " ")
    .slice(0, 64);
}

function normalizeAssistantName(value) {
  return sanitizeAssistantNameInput(value)
    .replace(/\s+/g, " ")
    .trim()
    .replace(/^[`"']+|[`"']+$/g, "")
    .trim();
}

function extractAssistantNameCommand(text, normalized) {
  const clearPhrases = [
    "clear assistant name",
    "reset assistant name",
    "remove assistant name",
    "очисти имя ассистента",
    "сбрось имя ассистента",
    "убери имя ассистента",
    "清除助手名字",
    "重置助手名字",
    "सहायक नाम हटाएं",
  ];
  if (clearPhrases.includes(normalized)) {
    return {
      kind: "set_preference",
      key: "assistantName",
      value: "",
      intent: "configure_assistant_name",
      label: "Assistant name",
    };
  }

  const raw = String(text || "").trim();
  const patterns = [
    /^(?:set|change|configure)\s+(?:the\s+)?(?:assistant|your)\s+name\s+(?:to|as)\s+(.+)$/iu,
    /^(?:assistant\s+name|your\s+name)\s*(?:=|:|is)\s*(.+)$/iu,
    /^(?:call|name)\s+(?:yourself|you)\s+(.+)$/iu,
    /^(?:назови|зови)\s+себя\s+(.+)$/iu,
    /^(?:теперь\s+)?(?:тебя\s+зовут|тво[её]\s+имя|имя\s+ассистента)\s*(?:=|:)?\s*(.+)$/iu,
    /^(?:你的名字|助手名字|助理名字)\s*(?:设为|设置为|叫|=|:)\s*(.+)$/u,
    /^(?:अपना नाम|सहायक नाम)\s*(?:रखो|सेट करो|=|:)?\s*(.+)$/u,
  ];
  for (const pattern of patterns) {
    const match = raw.match(pattern);
    if (!match) continue;
    const value = normalizeAssistantName(match[1]);
    if (!value) continue;
    return {
      kind: "set_preference",
      key: "assistantName",
      value,
      intent: "configure_assistant_name",
      label: "Assistant name",
    };
  }
  return null;
}

function commandValueLabel(command) {
  if (command.kind === "report_issue") return command.label;
  if (command.kind === "trigger") return command.label;
  if (command.key === "assistantName" && !command.value) return "not set";
  if (typeof command.value === "boolean") return command.value ? "on" : "off";
  if (typeof command.value === "number") return command.value.toFixed(2);
  return String(command.value);
}

function interfaceCommandResponse(command, reportIssueUrl) {
  if (command.kind === "report_issue") {
    return `Report issue link: [Report issue](${reportIssueUrl}).`;
  }
  if (command.kind === "trigger" && command.action === "attach_files") {
    return "Opening the file picker.";
  }
  return `Done. ${command.label} is now ${commandValueLabel(command)}.`;
}

function recognizeInterfaceCommand(text) {
  const normalized = normalizeMemoryPrompt(text);
  if (!normalized) return null;

  const reportPhrases = [
    "report issue",
    "create issue",
    "open issue",
    "сообщить о проблеме",
    "создай issue",
    "报告问题",
    "समस्या रिपोर्ट करें",
  ];
  if (reportPhrases.some((phrase) => normalized === phrase)) {
    return { kind: "report_issue", intent: "report_issue", label: "Report issue" };
  }

  const attachPhrases = [
    "attach file",
    "attach files",
    "add attachment",
    "upload file",
    "прикрепи файл",
    "добавь файл",
    "附加文件",
    "फ़ाइल जोड़ें",
  ];
  if (attachPhrases.some((phrase) => normalized === phrase || normalized.includes(phrase))) {
    return { kind: "trigger", action: "attach_files", intent: "attach_files", label: "Attach files" };
  }

  const assistantName = extractAssistantNameCommand(text, normalized);
  if (assistantName) {
    return assistantName;
  }

  const diagnostics = detectToggleCommand(normalized, [
    "diagnostics",
    "diagnostic",
    "trace",
    "диагност",
    "трассиров",
    "诊断",
    "निदान",
  ]);
  if (diagnostics !== null) {
    return {
      kind: "set_preference",
      key: "diagnosticsMode",
      value: diagnostics,
      intent: "configure_diagnostics",
      label: "Diagnostics",
    };
  }

  const demo = detectToggleCommand(normalized, ["demo", "демо", "演示", "डेमो"]);
  if (demo !== null || normalized === "manual mode" || normalized === "ручной режим") {
    return {
      kind: "set_preference",
      key: "demoMode",
      value: demo === null ? false : demo,
      intent: "configure_demo_mode",
      label: "Demo mode",
    };
  }

  const agent = detectToggleCommand(normalized, ["agent mode", "агент", "代理", "एजेंट"]);
  if (agent !== null || normalized === "chat mode") {
    return {
      kind: "set_preference",
      key: "agentMode",
      value: agent === null ? false : agent,
      intent: "configure_agent_mode",
      label: "Agent mode",
    };
  }

  const variations = detectToggleCommand(normalized, [
    "greeting variations",
    "greeting variation",
    "вариации приветствий",
    "варианты приветствий",
  ]);
  if (variations !== null) {
    return {
      kind: "set_preference",
      key: "greetingVariations",
      value: variations,
      intent: "configure_greeting_variations",
      label: "Greeting variations",
    };
  }

  const definitionFusion = detectToggleCommand(normalized, [
    "definition fusion",
    "merge definitions",
    "слияние определений",
    "合并定义",
  ]);
  if (definitionFusion !== null) {
    return {
      kind: "set_preference",
      key: "definitionFusion",
      value: definitionFusion ? "auto" : "explicit",
      intent: "configure_definition_fusion",
      label: "Definition fusion",
    };
  }

  // Issue #340: switch the composite-program blueprint between the projected
  // ("composed", default) and fully annotated ("documented") strategies. The
  // toggle reads naturally — "documented programs on" pins every optional
  // region, "off" returns to projecting only the requested capabilities.
  const blueprintComposition = detectToggleCommand(normalized, [
    "documented programs",
    "documented program",
    "full programs",
    "verbatim programs",
    "program composition",
    "документированные программы",
    "完整程序",
    "पूर्ण प्रोग्राम",
  ]);
  if (blueprintComposition !== null) {
    return {
      kind: "set_preference",
      key: "blueprintComposition",
      value: blueprintComposition ? "documented" : "composed",
      intent: "configure_blueprint_composition",
      label: "Program composition",
    };
  }

  const experimentalOcr = detectToggleCommand(normalized, [
    "ocr",
    "image text",
    "image recognition",
    "optical character recognition",
    "tesseract",
    "распознавание текста",
    "图片文字",
    "छवि पाठ",
  ]);
  if (experimentalOcr !== null) {
    return {
      kind: "set_preference",
      key: "experimentalOcr",
      value: experimentalOcr,
      intent: "configure_experimental_ocr",
      label: "Experimental OCR",
    };
  }

  const projectPromotion = detectToggleCommand(normalized, [
    "project promotion",
    "repository promotion",
    "associative project promotion",
    "associative repository promotion",
    "продвижение проектов",
    "продвижение репозиториев",
  ]);
  if (projectPromotion !== null) {
    return {
      kind: "set_preference",
      key: "associativeProjectPromotion",
      value: projectPromotion,
      intent: "configure_project_promotion",
      label: "Project promotion",
    };
  }

  if (includesAnyText(normalized, ["theme", "dark mode", "light mode", "тема", "режим", "主题"])) {
    if (includesAnyText(normalized, ["dark", "темн", "тёмн", "深色", "dark mode"])) {
      return { kind: "set_preference", key: "theme", value: "dark", intent: "configure_theme", label: "Theme" };
    }
    if (includesAnyText(normalized, ["light", "светл", "浅色", "light mode"])) {
      return { kind: "set_preference", key: "theme", value: "light", intent: "configure_theme", label: "Theme" };
    }
    if (includesAnyText(normalized, ["auto", "system", "авто", "систем", "自动"])) {
      return { kind: "set_preference", key: "theme", value: "auto", intent: "configure_theme", label: "Theme" };
    }
  }

  if (isExplicitUiLanguageCommand(normalized)) {
    if (includesAnyText(normalized, ["russian", "рус", "俄语"])) {
      return { kind: "set_preference", key: "uiLanguage", value: "ru", intent: "configure_language", label: "UI language" };
    }
    if (includesAnyText(normalized, ["english", "англ", "英语"])) {
      return { kind: "set_preference", key: "uiLanguage", value: "en", intent: "configure_language", label: "UI language" };
    }
    if (includesAnyText(normalized, ["chinese", "китай", "中文", "汉语"])) {
      return { kind: "set_preference", key: "uiLanguage", value: "zh", intent: "configure_language", label: "UI language" };
    }
    if (includesAnyText(normalized, ["hindi", "хинди", "हिन्दी", "हिंदी"])) {
      return { kind: "set_preference", key: "uiLanguage", value: "hi", intent: "configure_language", label: "UI language" };
    }
    if (includesAnyText(normalized, ["auto", "system", "авто", "自动"])) {
      return { kind: "set_preference", key: "uiLanguage", value: "auto", intent: "configure_language", label: "UI language" };
    }
  }

  if (includesAnyText(normalized, ["ui skin", "skin", "оформление", "外观"])) {
    if (normalized.includes("glass")) {
      return { kind: "set_preference", key: "uiSkin", value: "glass", intent: "configure_ui_skin", label: "UI skin" };
    }
    if (normalized.includes("contrast") || normalized.includes("контраст")) {
      return { kind: "set_preference", key: "uiSkin", value: "contrast", intent: "configure_ui_skin", label: "UI skin" };
    }
    if (normalized.includes("flat") || normalized.includes("плоск")) {
      return { kind: "set_preference", key: "uiSkin", value: "flat", intent: "configure_ui_skin", label: "UI skin" };
    }
  }

  if (includesAnyText(normalized, ["chat style", "стиль чата", "聊天样式"])) {
    if (normalized.includes("compact")) {
      return { kind: "set_preference", key: "chatStyle", value: "compact", intent: "configure_chat_style", label: "Chat style" };
    }
    if (normalized.includes("bubble") || normalized.includes("bubbles")) {
      return { kind: "set_preference", key: "chatStyle", value: "bubbles", intent: "configure_chat_style", label: "Chat style" };
    }
    if (normalized.includes("card") || normalized.includes("cards")) {
      return { kind: "set_preference", key: "chatStyle", value: "cards", intent: "configure_chat_style", label: "Chat style" };
    }
  }

  if (includesAnyText(normalized, ["composer style", "input style", "стиль ввода", "输入样式"])) {
    if (normalized.includes("glass clear") || normalized.includes("glass-clear")) {
      return { kind: "set_preference", key: "composerStyle", value: "glass-clear", intent: "configure_composer_style", label: "Composer style" };
    }
    if (normalized.includes("glass")) {
      return { kind: "set_preference", key: "composerStyle", value: "glass-soft", intent: "configure_composer_style", label: "Composer style" };
    }
    if (normalized.includes("bubble")) {
      return { kind: "set_preference", key: "composerStyle", value: "bubble", intent: "configure_composer_style", label: "Composer style" };
    }
    if (normalized.includes("flat")) {
      return { kind: "set_preference", key: "composerStyle", value: "flat", intent: "configure_composer_style", label: "Composer style" };
    }
  }

  if (includesAnyText(normalized, ["composer action", "attach button", "plus button", "кнопка ввода"])) {
    if (normalized.includes("plus") || normalized.includes("плюс")) {
      return { kind: "set_preference", key: "composerAction", value: "plus", intent: "configure_composer_action", label: "Composer action" };
    }
    if (normalized.includes("attach") || normalized.includes("attachment") || normalized.includes("скреп")) {
      return { kind: "set_preference", key: "composerAction", value: "attach", intent: "configure_composer_action", label: "Composer action" };
    }
  }

  const temperature = commandNumberValue(normalized, ["temperature", "температур", "तापमान", "温度"]);
  if (temperature !== null) {
    return {
      kind: "set_preference",
      key: "temperature",
      value: temperature,
      intent: "configure_temperature",
      label: "Temperature",
    };
  }

  const guessProbability = commandNumberValue(normalized, [
    "guess probability",
    "ambiguity",
    "вероятность догадки",
    "угадыв",
  ]);
  if (guessProbability !== null) {
    return {
      kind: "set_preference",
      key: "guessProbability",
      value: guessProbability,
      intent: "configure_guess_probability",
      label: "Guess probability",
    };
  }

  const locationPrefixes = [
    "set location to ",
    "my location is ",
    "remember my location as ",
    "установи местоположение ",
    "мое местоположение ",
  ];
  const locationPrefix = locationPrefixes.find((prefix) => normalized.startsWith(prefix));
  if (locationPrefix) {
    const value = normalized.slice(locationPrefix.length).trim().slice(0, 80);
    if (value) {
      return {
        kind: "set_preference",
        key: "location",
        value,
        intent: "configure_location",
        label: "Location",
      };
    }
  }

  const sidebar = detectToggleCommand(normalized, ["sidebar", "side panel", "боковая панель"]);
  if (sidebar !== null) {
    return {
      kind: "set_preference",
      key: "sidebarCollapsed",
      value: !sidebar,
      intent: "configure_sidebar",
      label: "Sidebar",
    };
  }

  const deleted = detectToggleCommand(normalized, ["deleted conversations", "deleted chats", "удаленные беседы"]);
  if (deleted !== null) {
    return {
      kind: "set_preference",
      key: "showDeletedConversations",
      value: deleted,
      intent: "configure_deleted_conversations",
      label: "Deleted conversations",
    };
  }

  return null;
}

// Issue #27 R11: natural-language cross-conversation recall. The user types
// something like "when did I ask about Rust?" or "find Donald Trump in another
// conversation" and the assistant projects the append-only memory log onto
// matching events grouped by conversation. Patterns are prefix-based so the
// remainder of the prompt becomes the search term verbatim (after trimming
// quotes and trailing punctuation). `scope = 'other'` excludes the current
// conversation; `scope = 'all'` searches every conversation including the
// current one.
const RECALL_QUERY_PATTERNS = [
  { prefix: "when did i ask about ", scope: "all" },
  { prefix: "when did i ask ", scope: "all" },
  { prefix: "when did i mention ", scope: "all" },
  { prefix: "when did i talk about ", scope: "all" },
  { prefix: "have i asked about ", scope: "all" },
  { prefix: "have i mentioned ", scope: "all" },
  { prefix: "did i ask about ", scope: "all" },
  { prefix: "did i mention ", scope: "all" },
  { prefix: "search my conversations for ", scope: "all" },
  { prefix: "search conversations for ", scope: "all" },
  { prefix: "search my chats for ", scope: "all" },
  { prefix: "recall ", scope: "all" },
  { prefix: "когда я спрашивал про ", scope: "all" },
  { prefix: "когда я спрашивал о ", scope: "all" },
  { prefix: "когда я спрашивал ", scope: "all" },
  { prefix: "когда я упоминал ", scope: "all" },
  { prefix: "поиск по беседам ", scope: "all" },
  { prefix: "поиск в беседах ", scope: "all" },
  { prefix: "найди в беседах ", scope: "all" },
  { prefix: "我什么时候问过 ", scope: "all" },
  { prefix: "我什么时候问过", scope: "all" },
  { prefix: "我什么时候提到 ", scope: "all" },
  { prefix: "我什么时候提到", scope: "all" },
  { prefix: "搜索我的对话 ", scope: "all" },
  { prefix: "搜索我的对话", scope: "all" },
  { prefix: "在对话中搜索 ", scope: "all" },
  { prefix: "在对话中搜索", scope: "all" },
];

// Suffix forms ("...in another conversation", "...在其他对话中") that mark the
// recall as cross-conversation-only. The remainder before the suffix becomes
// the search term.
const RECALL_OTHER_SUFFIXES = [
  " in another conversation",
  " in other conversations",
  " in my other conversations",
  " in my conversations",
  " in another chat",
  " in other chats",
  " в другой беседе",
  " в других беседах",
  " в других чатах",
  "在其他对话中",
  "在另一个对话中",
];

// "find X in another conversation" — `find ` is the lead-in for the other-scope
// recall when paired with one of the suffixes above.
const RECALL_OTHER_PREFIXES = [
  "find ",
  "search for ",
  "look for ",
  "найди ",
  "поищи ",
  "查找 ",
  "查找",
  "搜索 ",
  "搜索",
];

function stripRecallTerm(term) {
  return String(term || "")
    .replace(/^["'«»『「]+/, "")
    .replace(/["'«»』」]+$/, "")
    .replace(/[!?.,;:。!?,;:、]+$/g, "")
    .trim();
}

// Extract the substring from `original` that corresponds to characters at
// positions [start, end) of the lowercased normalised form. We do not have a
// strict 1:1 character map because normalisation can collapse whitespace, so
// approximate by walking the original and skipping characters that the
// normaliser would also skip. The result is good enough to preserve user
// casing for terms like "Donald Trump" or "Илона Маска".
function recoverOriginalRange(original, normalized, start, end) {
  // Walk through `original` character by character, advancing a normalised
  // cursor whenever we emit a character that would survive normalisation.
  // When the normalised cursor enters [start, end), we capture characters
  // from `original` instead of from `normalized`.
  let nIdx = 0;
  let captured = "";
  let i = 0;
  let prevWasSpace = false;
  while (i < original.length && nIdx < end) {
    const ch = original[i];
    const lower = ch.toLowerCase();
    // Mirror normalizeMemoryPrompt's whitespace collapse: \s plus the
    // unicode-space block U+00A0 / U+2000–U+200B used by the seed corpus.
    if (/[\s\u00A0\u2000-\u200B]/.test(ch)) {
      if (!prevWasSpace) {
        if (nIdx >= start) captured += " ";
        nIdx++;
        prevWasSpace = true;
      }
      i++;
      continue;
    }
    prevWasSpace = false;
    if (nIdx >= start) captured += ch;
    nIdx += lower.length;
    i++;
  }
  return captured.trim();
}

function recognizeRecallQuery(text) {
  const original = String(text || "").trim();
  if (!original) return null;
  const normalized = normalizeMemoryPrompt(text);
  if (!normalized) return null;

  // Try "find X in another conversation" — prefix + suffix combo.
  for (const suffix of RECALL_OTHER_SUFFIXES) {
    const suffixIdx = normalized.lastIndexOf(suffix);
    if (suffixIdx < 0) continue;
    const beforeSuffix = normalized.slice(0, suffixIdx);
    for (const prefix of RECALL_OTHER_PREFIXES) {
      if (beforeSuffix.startsWith(prefix)) {
        const normalizedTerm = stripRecallTerm(beforeSuffix.slice(prefix.length));
        if (!normalizedTerm) continue;
        const originalTerm = stripRecallTerm(
          recoverOriginalRange(original, normalized, prefix.length, suffixIdx),
        );
        return { term: originalTerm || normalizedTerm, scope: "other" };
      }
    }
  }

  // Prefix-only patterns ("when did I ask about X", "recall X").
  for (const { prefix, scope } of RECALL_QUERY_PATTERNS) {
    if (normalized.startsWith(prefix)) {
      const normalizedTerm = stripRecallTerm(normalized.slice(prefix.length));
      if (!normalizedTerm) continue;
      const originalTerm = stripRecallTerm(
        recoverOriginalRange(original, normalized, prefix.length, normalized.length),
      );
      return { term: originalTerm || normalizedTerm, scope };
    }
  }
  return null;
}

// Build a Markdown report of every message whose lowercased content contains
// `term`, grouped by conversation. `scope === 'other'` filters out the active
// conversation. `triggerText` is the user's recall request itself — skip
// events whose content equals it so the recall never matches the prompt that
// triggered it. `events` is the full append-only log from FormalAiMemory.
function buildRecallReport({ events, term, scope, currentConversationId, triggerText }) {
  const safeEvents = Array.isArray(events) ? events : [];
  const needle = String(term || "").toLowerCase();
  if (!needle) {
    return {
      content: "No search term recognised in the recall request.",
      matches: [],
    };
  }
  const triggerNormalized = String(triggerText || "").trim().toLowerCase();
  const groups = new Map();
  for (const event of safeEvents) {
    if (!event || (event.kind && event.kind !== "message")) continue;
    const content = String(event.content || "");
    if (!content.toLowerCase().includes(needle)) continue;
    if (triggerNormalized && content.trim().toLowerCase() === triggerNormalized) {
      continue;
    }
    const id = event.conversationId || "legacy";
    if (scope === "other" && id === (currentConversationId || "")) continue;
    let entry = groups.get(id);
    if (!entry) {
      entry = { id, title: "", events: [] };
      groups.set(id, entry);
    }
    if (!entry.title && event.role === "user" && event.conversationTitle) {
      entry.title = event.conversationTitle;
    }
    entry.events.push(event);
  }

  const groupList = Array.from(groups.values());
  // Fill in titles from the first user message of each group when the recorded
  // title is missing (legacy events without a conversationTitle field).
  for (const group of groupList) {
    if (!group.title) {
      const firstUser = group.events.find((e) => e.role === "user");
      if (firstUser && firstUser.content) {
        group.title = deriveConversationTitle(firstUser.content);
      } else if (group.id === "legacy") {
        group.title = "Earlier conversation";
      } else {
        group.title = "Untitled conversation";
      }
    }
    group.events.sort((a, b) => String(a.sentAt || "").localeCompare(String(b.sentAt || "")));
  }
  groupList.sort((left, right) => {
    const lLast = left.events[left.events.length - 1]?.sentAt || "";
    const rLast = right.events[right.events.length - 1]?.sentAt || "";
    return String(rLast).localeCompare(String(lLast));
  });

  const totalMatches = groupList.reduce((sum, g) => sum + g.events.length, 0);
  if (totalMatches === 0) {
    const scopeNote = scope === "other" ? " in any other conversation" : "";
    return {
      content: `No mentions of "${term}" found${scopeNote}.`,
      matches: [],
    };
  }

  const lines = [];
  const conversationCount = groupList.length;
  lines.push(
    `Found **${totalMatches}** mention${totalMatches === 1 ? "" : "s"} of "${term}" across **${conversationCount}** conversation${conversationCount === 1 ? "" : "s"}.`,
  );
  for (const group of groupList) {
    lines.push("");
    lines.push(`### ${group.title}`);
    for (const event of group.events) {
      const stamp = event.sentAt ? event.sentAt : "(no timestamp)";
      const role = event.role === "user" ? "user" : "assistant";
      const snippet = String(event.content || "").replace(/\s+/g, " ").trim();
      const trimmed = snippet.length > 160 ? `${snippet.slice(0, 157)}…` : snippet;
      lines.push(`- ${stamp} — ${role}: ${trimmed}`);
    }
  }
  return { content: lines.join("\n"), matches: groupList };
}

const PREFERENCE_DEFAULTS = {
  demoMode: true,
  diagnosticsMode: false,
  contextPanelWidth: 300,
  // Issue #27: each sidebar section is a VS Code-style collapsible region; the
  // last expand/collapse state is persisted via FormalAiPreferences so opening
  // the demo never reshuffles the user's layout.
  sidebarMenuCollapsed: false,
  sidebarPromptsCollapsed: false,
  sidebarToolsCollapsed: false,
  sidebarTraceCollapsed: false,
  sidebarConversationsCollapsed: false,
  sidebarSettingsCollapsed: false,
  // Issue #153: the side panel is collapsible to give the chat full viewport
  // width on desktop. The drawer view on mobile stays controlled by the
  // separate `mobileMenuOpen` toggle so phones can still slide it in.
  sidebarCollapsed: false,
  showDeletedConversations: false,
  // Issue #27: random greeting variations are opt-in but default to on so
  // newcomers see the multilingual surface immediately.
  greetingVariations: true,
  // Issue #82: user-tunable assistant behavior. The default still guesses
  // likely typo matches, while the sliders let cautious users ask first and
  // deterministic users turn random response variation off with temperature=0.
  guessProbability: 0.8,
  temperature: 0.7,
  // Issue #160 follow-up: polite courtesy responses can either leave the
  // initiative with the user or ask/propose the next action. This probability
  // controls whether the next-action sentence is appended.
  followUpProbability: 0.75,
  // Issue #63: definition fusion remains explicit-only by default, with an
  // opt-in mode that treats plain "What is X?" prompts as merge requests.
  definitionFusion: "explicit",
  // Issue #340: how composite-program blueprints project their annotated recipe
  // template into the program shown to the user.
  //   "composed" (default) — emit only the regions the request actually named,
  //                          so the program is a projection of the decomposition;
  //   "documented"         — always emit the fully documented program with every
  //                          optional region (error handling, comments) present.
  blueprintComposition: "composed",
  experimentalOcr: false,
  associativeProjectPromotion: true,
  theme: "auto",
  location: "",
  assistantName: "",
  // Issue #27: id of the conversation the user last typed in; on reload the
  // demo restores its event log into the main transcript. Empty string means
  // "no conversation yet — start a fresh one on first user input".
  currentConversationId: "",
  // Issue #27: Chat (single-turn Q&A) vs Agent (multi-step plan + execute) mode.
  // Persisted so the user keeps their preferred operating surface across
  // reloads. Agent mode in the browser sandbox decomposes the prompt into
  // sequential sub-tasks and runs each through the existing solver; a future
  // iteration will wire it to docker / WebVM execution.
  agentMode: false,
  // Issue #94: "auto" follows navigator.languages; explicit values use the
  // supported UI language catalog.
  uiLanguage: "auto",
  // Issue #324: which language drives the assistant's responses.
  //   "last_message" (default) — answer in the detected language of the prompt;
  //   "preferred"             — pin responses to `preferredLanguage`;
  //   "ui"                    — follow the UI-language preference.
  // The default reproduces the deterministic "reply in the message's language"
  // behavior, so a Russian prompt is answered in Russian.
  responseLanguage: "last_message",
  // Issue #324: the explicit language used when `responseLanguage` is
  // "preferred". One of the supported response languages (en/ru/hi/zh).
  preferredLanguage: "en",
  // Issues #108/#110: UI, chat, and input surfaces are configurable while the
  // defaults stay flat and cheap to render.
  uiSkin: "flat",
  chatStyle: "cards",
  composerStyle: "flat",
  composerAction: "attach",
};

// Issue #386: precompute the formatted default values so the issue report can
// omit any User Context field that matches its shipped default. Keeping these
// derived from PREFERENCE_DEFAULTS means they stay in sync if a default moves.
const DEFAULT_GUESS_PROBABILITY_PERCENT = formatSliderValue(
  PREFERENCE_DEFAULTS.guessProbability,
);
const DEFAULT_TEMPERATURE_TEXT = String(
  normalizeSliderPreference(PREFERENCE_DEFAULTS.temperature, 0),
);
const DEFAULT_FOLLOW_UP_PROBABILITY_PERCENT = formatSliderValue(
  PREFERENCE_DEFAULTS.followUpProbability,
);

// Issue #386: the settings panel lets the user reset each setting (or all of
// them) back to the shipped default. A setting is "modified" when its current
// value differs from PREFERENCE_DEFAULTS; numeric sliders are compared
// numerically so 0.8 and "0.8" are treated as equal.
function settingIsDefault(key, value) {
  const fallback = PREFERENCE_DEFAULTS[key];
  if (typeof fallback === "number") {
    return Number(value) === fallback;
  }
  return value === fallback;
}

const UI_SKINS = ["flat", "glass", "contrast"];
const CHAT_STYLES = ["cards", "compact", "bubbles"];
const COMPOSER_STYLES = ["flat", "glass-soft", "glass-clear", "bubble"];
const COMPOSER_ACTIONS = ["attach", "plus"];
const DEFINITION_FUSION_MODES = ["explicit", "auto"];
// Issue #340: blueprint program-composition strategies. "composed" projects the
// program from the detected capabilities; "documented" always emits the full
// annotated program with every optional region present.
const BLUEPRINT_COMPOSITION_MODES = ["composed", "documented"];
// Issue #324: source that drives the assistant's response language.
const RESPONSE_LANGUAGE_MODES = ["last_message", "preferred", "ui"];
// Issue #324: languages the assistant can be pinned to via `preferredLanguage`.
const PREFERRED_RESPONSE_LANGUAGES = ["en", "ru", "hi", "zh"];
const CONTEXT_PANEL_MIN_WIDTH = 220;
const CONTEXT_PANEL_MAX_WIDTH = 560;
const CONTEXT_PANEL_MIN_CHAT_WIDTH = 360;
const CONTEXT_PANEL_RESIZER_WIDTH = 10;

const MEMORY_EXPORT_FILENAME = "formal-ai-memory.lino";
const OCR_BUNDLE_FILENAME = "ocr.bundle.js";
const OCR_DOWNLOAD_WARNING =
  "Downloads about 6 MB on first use: OCR wrapper, worker, WebAssembly core, and English traineddata.";

let ocrBundlePromise = null;

function withAssetVersion(path) {
  if (!ASSET_VERSION) {
    return path;
  }
  const separator = path.includes("?") ? "&" : "?";
  return `${path}${separator}v=${encodeURIComponent(ASSET_VERSION)}`;
}

function recordMemoryEvent(payload) {
  if (typeof window === "undefined" || !window.FormalAiMemory) {
    return Promise.resolve(null);
  }
  try {
    const write = window.FormalAiMemory.appendEvent(payload).catch(() => null);
    pendingMemoryWrites.add(write);
    return write.finally(() => {
      pendingMemoryWrites.delete(write);
    });
  } catch (_error) {
    return Promise.resolve(null);
  }
}

function waitForMemoryWrites() {
  if (pendingMemoryWrites.size === 0) {
    return Promise.resolve();
  }
  return Promise.allSettled(Array.from(pendingMemoryWrites)).then(() => null);
}

function downloadTextFile(filename, text) {
  if (typeof window === "undefined" || typeof document === "undefined") {
    return;
  }
  const blob = new Blob([text], { type: "text/plain;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}

function isImageAttachment(file) {
  if (!file) return false;
  const type = String(file.type || "").toLowerCase();
  if (type.startsWith("image/")) return true;
  return /\.(png|jpe?g|webp|gif|bmp|tiff?)$/i.test(String(file.name || ""));
}

function formatFileSize(bytes) {
  const value = Number(bytes);
  if (!Number.isFinite(value) || value <= 0) return "0 B";
  if (value < 1024) return `${value} B`;
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB`;
  return `${(value / (1024 * 1024)).toFixed(1)} MB`;
}

function readFileAsDataUrl(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result || ""));
    reader.onerror = () => reject(reader.error || new Error("Unable to read file"));
    reader.readAsDataURL(file);
  });
}

function loadOcrBundle() {
  if (typeof window === "undefined" || typeof document === "undefined") {
    return Promise.reject(new Error("OCR is only available in the browser"));
  }
  if (window.FormalAiOcr && typeof window.FormalAiOcr.recognizeImage === "function") {
    return Promise.resolve(window.FormalAiOcr);
  }
  if (ocrBundlePromise) {
    return ocrBundlePromise;
  }
  ocrBundlePromise = new Promise((resolve, reject) => {
    const script = document.createElement("script");
    script.src = withAssetVersion(OCR_BUNDLE_FILENAME);
    script.async = true;
    script.onload = () => {
      if (
        window.FormalAiOcr &&
        typeof window.FormalAiOcr.recognizeImage === "function"
      ) {
        resolve(window.FormalAiOcr);
      } else {
        ocrBundlePromise = null;
        reject(new Error("OCR bundle loaded without an OCR API"));
      }
    };
    script.onerror = () => {
      ocrBundlePromise = null;
      reject(new Error("Unable to load OCR bundle"));
    };
    document.head.appendChild(script);
  });
  return ocrBundlePromise;
}

function attachmentMemoryRecord(attachment) {
  const record = {
    name: String(attachment.name || "attachment"),
    size: Number(attachment.size || 0),
    type: String(attachment.type || "application/octet-stream"),
    kind: attachment.isImage ? "image" : "file",
  };
  if (attachment.dataUrl) record.dataUrl = attachment.dataUrl;
  if (attachment.ocrText) record.ocrText = attachment.ocrText;
  if (attachment.ocrConfidence !== undefined && attachment.ocrConfidence !== null) {
    record.ocrConfidence = attachment.ocrConfidence;
  }
  if (attachment.ocrError) record.ocrError = attachment.ocrError;
  return record;
}

function attachmentOnlyPrompt(attachments) {
  const count = attachments.length;
  if (count === 1) {
    return `Attached ${attachments[0].isImage ? "image" : "file"}: ${attachments[0].name}`;
  }
  return `Attached ${count} files`;
}

function attachmentContextText(attachments) {
  if (!attachments.length) return "";
  const lines = ["Attached files:"];
  attachments.forEach((attachment, index) => {
    lines.push(
      `${index + 1}. ${attachment.name} (${attachment.type}, ${formatFileSize(attachment.size)})`,
    );
    if (attachment.ocrText) {
      lines.push(`OCR text: ${attachment.ocrText}`);
    } else if (attachment.ocrError) {
      lines.push(`OCR unavailable: ${attachment.ocrError}`);
    } else if (attachment.isImage && attachment.dataUrl) {
      lines.push("Image data is stored in memory as a base64 data URL.");
    }
  });
  return lines.join("\n");
}

function buildPromptWithAttachments(text, attachments) {
  const context = attachmentContextText(attachments);
  if (!context) return text;
  const promptText = String(text || "").trim();
  return `${promptText}\n\n${context}`.trim();
}

function loadPreferences() {
  if (typeof window === "undefined" || !window.FormalAiPreferences) {
    return { ...PREFERENCE_DEFAULTS };
  }
  try {
    return window.FormalAiPreferences.load(PREFERENCE_DEFAULTS);
  } catch (_error) {
    return { ...PREFERENCE_DEFAULTS };
  }
}

function persistPreferences(values) {
  if (typeof window === "undefined" || !window.FormalAiPreferences) {
    return;
  }
  try {
    window.FormalAiPreferences.save(values);
  } catch (_error) {
    // localStorage may be unavailable (private mode, sandboxed iframe); ignore.
  }
}

function clampNumber(value, min, max, fallback) {
  const number = Number(value);
  if (!Number.isFinite(number)) return fallback;
  return Math.min(max, Math.max(min, number));
}

function normalizeSliderPreference(value, fallback) {
  return clampNumber(value, 0, 1, fallback);
}

function formatSliderValue(value) {
  return String(Math.round(normalizeSliderPreference(value, 0) * 100));
}

function contextPanelMaxWidth() {
  if (typeof window === "undefined") {
    return CONTEXT_PANEL_MAX_WIDTH;
  }
  const viewportWidth =
    window.visualViewport && window.visualViewport.width
      ? window.visualViewport.width
      : window.innerWidth;
  const available = Math.round(
    viewportWidth - CONTEXT_PANEL_MIN_CHAT_WIDTH - CONTEXT_PANEL_RESIZER_WIDTH,
  );
  return Math.max(
    CONTEXT_PANEL_MIN_WIDTH,
    Math.min(CONTEXT_PANEL_MAX_WIDTH, available),
  );
}

function normalizeContextPanelWidth(value) {
  return Math.round(
    clampNumber(
      value,
      CONTEXT_PANEL_MIN_WIDTH,
      contextPanelMaxWidth(),
      PREFERENCE_DEFAULTS.contextPanelWidth,
    ),
  );
}

function normalizeThemePreference(value) {
  return ["auto", "light", "dark"].includes(value) ? value : "auto";
}

function normalizeUiSkin(value) {
  return UI_SKINS.includes(value) ? value : PREFERENCE_DEFAULTS.uiSkin;
}

function normalizeChatStyle(value) {
  return CHAT_STYLES.includes(value) ? value : PREFERENCE_DEFAULTS.chatStyle;
}

function normalizeComposerStyle(value) {
  return COMPOSER_STYLES.includes(value) ? value : PREFERENCE_DEFAULTS.composerStyle;
}

function normalizeComposerAction(value) {
  return COMPOSER_ACTIONS.includes(value)
    ? value
    : PREFERENCE_DEFAULTS.composerAction;
}

function normalizeDefinitionFusion(value) {
  return DEFINITION_FUSION_MODES.includes(value)
    ? value
    : PREFERENCE_DEFAULTS.definitionFusion;
}

function normalizeBlueprintComposition(value) {
  return BLUEPRINT_COMPOSITION_MODES.includes(value)
    ? value
    : PREFERENCE_DEFAULTS.blueprintComposition;
}

function normalizeResponseLanguageMode(value) {
  return RESPONSE_LANGUAGE_MODES.includes(value)
    ? value
    : PREFERENCE_DEFAULTS.responseLanguage;
}

function normalizePreferredLanguage(value) {
  return PREFERRED_RESPONSE_LANGUAGES.includes(value)
    ? value
    : PREFERENCE_DEFAULTS.preferredLanguage;
}

function i18nApi() {
  return typeof window !== "undefined" && window.FormalAiI18n
    ? window.FormalAiI18n
    : null;
}

function normalizeUiLanguagePreference(value) {
  if (!value || value === "auto") return "auto";
  const api = i18nApi();
  const normalized = api && api.normalizeLanguageTag
    ? api.normalizeLanguageTag(value)
    : String(value).toLowerCase().split(/[-_]/)[0];
  return normalized || "auto";
}

function detectUiLanguage(preference) {
  const api = i18nApi();
  if (api && api.detectLanguage) {
    return api.detectLanguage(preference === "auto" ? "" : preference);
  }
  return "en";
}

function translateUi(key, language, params) {
  const api = i18nApi();
  if (api && api.t) {
    return api.t(key, language, params);
  }
  return key;
}

function browserLanguagesList() {
  if (typeof navigator === "undefined") return [];
  if (Array.isArray(navigator.languages) && navigator.languages.length > 0) {
    return Array.from(navigator.languages);
  }
  return navigator.language ? [navigator.language] : [];
}

function currentColorScheme(themePreference) {
  if (themePreference === "light" || themePreference === "dark") {
    return themePreference;
  }
  if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
    return "unknown";
  }
  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

function resolvedLocale() {
  try {
    return Intl.DateTimeFormat().resolvedOptions().locale || "";
  } catch (_error) {
    return "";
  }
}

function resolvedTimeZone() {
  try {
    return Intl.DateTimeFormat().resolvedOptions().timeZone || "";
  } catch (_error) {
    return "";
  }
}

function collectUserContext({
  uiLanguage,
  uiLanguagePreference,
  themePreference,
  uiSkin,
  chatStyle,
  composerStyle,
  composerAction,
  locationPreference,
  assistantName,
  guessProbability,
  temperature,
  followUpProbability,
  definitionFusion,
  experimentalOcr,
}) {
  const browserLanguages = browserLanguagesList();
  const nav = typeof navigator !== "undefined" ? navigator : {};
  const userAgent = nav.userAgent || "";
  const screenInfo =
    typeof screen !== "undefined"
      ? `${screen.width}x${screen.height} @${window.devicePixelRatio || 1}x`
      : "";
  const viewportInfo =
    typeof window !== "undefined" ? `${window.innerWidth}x${window.innerHeight}` : "";
  return {
    uiLanguage,
    uiLanguagePreference,
    themePreference,
    uiSkin,
    chatStyle,
    composerStyle,
    composerAction,
    browserLanguage: nav.language || "",
    browserLanguages: browserLanguages.join(", "),
    locale: resolvedLocale(),
    timeZone: resolvedTimeZone(),
    colorScheme: currentColorScheme(themePreference),
    viewport: viewportInfo,
    screen: screenInfo,
    userAgent,
    platform:
      (nav.userAgentData && nav.userAgentData.platform) ||
      nav.platform ||
      "",
    online: typeof nav.onLine === "boolean" ? (nav.onLine ? "yes" : "no") : "",
    preferredLocation: locationPreference || "",
    assistantName: normalizeAssistantName(assistantName) || "not set",
    guessProbability: formatSliderValue(guessProbability),
    temperature: String(normalizeSliderPreference(temperature, 0)),
    followUpProbability: formatSliderValue(followUpProbability),
    definitionFusion,
    experimentalOcr: experimentalOcr ? "on" : "off",
    locationInference:
      locationPreference
        ? `user-provided preference: ${locationPreference}`
        : "time zone / locale only; exact geolocation was not requested",
  };
}

// Issue #140: the prefilled `Report issue` URL is encoded as `?body=…` and
// GitHub caps the request line at 8192 chars. The verbose User Context block
// previously listed one field per line; now we combine related fields so a
// typical 5-turn dialog fits comfortably under the cap. Defaults and
// not-set values are omitted (UI Skin / Chat Style / Composer Style /
// Composer Action / Online status / Preferred Location), since they are
// uninteresting without the matching memory export.
function formatUiLanguagesField(active, browserLanguagesStr) {
  const browserLanguages = browserLanguagesStr
    ? String(browserLanguagesStr)
        .split(",")
        .map((entry) => entry.trim())
        .filter(Boolean)
    : [];
  const activeStr = String(active || "").trim();
  if (!activeStr && browserLanguages.length === 0) return "unknown";
  const lower = activeStr.toLowerCase();
  const primary = (lang) => String(lang).split(/[-_]/)[0].toLowerCase();
  const matchIndex = browserLanguages.findIndex(
    (lang) => primary(lang) === lower || lang.toLowerCase() === lower,
  );
  if (matchIndex >= 0) {
    return browserLanguages
      .map((lang, idx) => (idx === matchIndex ? `*${lang}*` : lang))
      .join(", ");
  }
  if (!activeStr) return browserLanguages.join(", ");
  if (browserLanguages.length === 0) return `*${activeStr}*`;
  return `*${activeStr}*, ${browserLanguages.join(", ")}`;
}

function formatUiField(context) {
  const parts = [];
  if (context.viewport) parts.push(`${context.viewport} viewport`);
  if (context.screen) parts.push(`${context.screen} screen`);
  if (context.userAgent) parts.push(`${context.userAgent} browser`);
  if (context.platform) parts.push(`${context.platform} platform`);
  return parts.join(", ");
}

function formatLocaleField(context) {
  const locale = context.locale ? String(context.locale).trim() : "";
  const timeZone = context.timeZone ? String(context.timeZone).trim() : "";
  if (locale && timeZone) return `${locale} (${timeZone})`;
  if (locale) return locale;
  if (timeZone) return timeZone;
  return "";
}

function formatThemeField(context) {
  const preference = context.themePreference || "auto";
  const scheme = context.colorScheme || "";
  if (scheme && scheme !== preference) return `${preference} (${scheme})`;
  return preference;
}

function appendUserContextBlock(lines, context) {
  const safe = context && typeof context === "object" ? context : {};
  const entries = [];
  const push = (label, value) => {
    if (value === undefined || value === null) return;
    const text = String(value).trim();
    if (!text) return;
    entries.push(`- **${label}**: ${text}`);
  };

  push("UI languages", formatUiLanguagesField(safe.uiLanguage, safe.browserLanguages));
  // Issue #386: omit settings that are set exactly to their default so the
  // report keeps space for the dialog itself. A field is reported only when it
  // differs from the shipped default (or carries an explicit user value).
  const themePreference = safe.themePreference || PREFERENCE_DEFAULTS.theme;
  if (themePreference !== PREFERENCE_DEFAULTS.theme) {
    push("Theme", formatThemeField(safe));
  }
  push("UI", formatUiField(safe));
  push("Locale", formatLocaleField(safe));
  if (safe.preferredLocation) {
    push("Preferred location", safe.preferredLocation);
  }
  if (safe.guessProbability !== DEFAULT_GUESS_PROBABILITY_PERCENT) {
    push("Guess probability", `${safe.guessProbability || "unknown"}%`);
  }
  if (safe.temperature !== DEFAULT_TEMPERATURE_TEXT) {
    push("Temperature", safe.temperature);
  }
  if (safe.followUpProbability !== DEFAULT_FOLLOW_UP_PROBABILITY_PERCENT) {
    push("Follow-up probability", `${safe.followUpProbability || "unknown"}%`);
  }
  // Issue #386: the inference-only location ("time zone / locale only") is the
  // default, so it is omitted. An explicit preference is reported above.

  if (entries.length === 0) return;
  lines.push("## User Context");
  lines.push("");
  for (const entry of entries) lines.push(entry);
  lines.push("");
}

function randomItem(items) {
  return items[Math.floor(Math.random() * items.length)];
}

// Issue #27: conversations are grouped slices of the append-only event log.
// Each event records the id of the conversation that produced it; the UI then
// filters events on read. New ids are generated locally so they stay portable
// across browsers (no server round-trip required).
function generateConversationId() {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return `conv-${crypto.randomUUID()}`;
  }
  const random = Math.random().toString(16).slice(2, 10);
  return `conv-${Date.now().toString(16)}-${random}`;
}

function deriveConversationTitle(text) {
  const trimmed = String(text || "").trim().replace(/\s+/g, " ");
  if (!trimmed) {
    return "New conversation";
  }
  if (trimmed.length <= 60) {
    return trimmed;
  }
  return `${trimmed.slice(0, 57)}…`;
}

// Group append-only events into per-conversation summaries (id, title,
// timestamps, message count). Events without a conversationId are aggregated
// under the synthetic "legacy" bucket so existing demos remain visible after
// the schema upgrade.
function groupConversations(events, options = {}) {
  const safe = Array.isArray(events) ? events : [];
  const map = new Map();

  const ensureEntry = (id, event = {}) => {
    let entry = map.get(id);
    if (!entry) {
      entry = {
        id,
        title: id === "legacy" ? "Earlier conversation" : "",
        firstAt: event.sentAt || "",
        lastAt: event.sentAt || "",
        deletedAt: "",
        messageCount: 0,
        deleted: false,
      };
      map.set(id, entry);
    }
    return entry;
  };

  for (let index = 0; index < safe.length; index += 1) {
    const event = safe[index];
    if (!event) {
      continue;
    }
    const kind = event.kind || "message";
    const id = event.conversationId || "legacy";
    if (kind === "conversation_deleted") {
      const entry = ensureEntry(id, event);
      entry.deleted = true;
      entry.deletedAt = event.sentAt || entry.deletedAt || "";
      if (!entry.title && event.conversationTitle) {
        entry.title = event.conversationTitle;
      }
      if (event.sentAt && (!entry.lastAt || event.sentAt > entry.lastAt)) {
        entry.lastAt = event.sentAt;
      }
      continue;
    }
    if (kind !== "message") {
      continue;
    }
    const entry = ensureEntry(id, event);
    if (event.role === "user" && !entry.title && event.conversationTitle) {
      entry.title = event.conversationTitle;
    } else if (event.role === "user" && !entry.title) {
      entry.title = deriveConversationTitle(event.content);
    }
    if (event.sentAt && (!entry.firstAt || event.sentAt < entry.firstAt)) {
      entry.firstAt = event.sentAt;
    }
    if (event.sentAt && (!entry.lastAt || event.sentAt > entry.lastAt)) {
      entry.lastAt = event.sentAt;
    }
    entry.messageCount += 1;
  }
  const showDeleted = Boolean(options.showDeleted);
  const list = Array.from(map.values()).filter((entry) =>
    showDeleted ? entry.deleted : !entry.deleted,
  );
  list.sort((left, right) => {
    if (left.lastAt && right.lastAt) {
      return right.lastAt.localeCompare(left.lastAt);
    }
    return 0;
  });
  return list;
}

function resizeComposerInput(element) {
  if (!element) return;
  element.style.height = "auto";
  const computed = getComputedStyle(element);
  const maxHeight = parseFloat(computed.maxHeight);
  const borderHeight =
    (parseFloat(computed.borderTopWidth) || 0) +
    (parseFloat(computed.borderBottomWidth) || 0);
  const scrollBorderBoxHeight = element.scrollHeight + borderHeight;
  const target = Number.isFinite(maxHeight)
    ? Math.min(scrollBorderBoxHeight, maxHeight)
    : scrollBorderBoxHeight;
  element.style.height = `${Math.max(target, 0)}px`;
  element.style.overflowY =
    element.scrollHeight > target - borderHeight + 1 ? "auto" : "hidden";
}

function localizeTool(tool, language) {
  if (!tool || !Array.isArray(tool.localized)) {
    return tool || {};
  }
  const normalized = normalizeUiLanguagePreference(language) || "en";
  const localized =
    tool.localized.find((entry) => entry.language === normalized) ||
    tool.localized.find((entry) => entry.language === "en");
  if (!localized) {
    return tool;
  }
  return {
    ...tool,
    name: localized.name || tool.name,
    description: localized.description || tool.description,
  };
}

// Issue #27: agent-mode task decomposition. Splits a multi-step prompt into
// sequential sub-tasks on a small, deterministic set of separators that span
// the languages the demo already supports. The split is intentionally
// conservative — if no separator is present we return [trimmedPrompt] so a
// single-step task still runs through the same code path.
const AGENT_STEP_SEPARATORS = [
  /\s*;\s+/,
  /\s*,\s+(?:and\s+then|then|next)\s+/i,
  /\s*,\s+after\s+that\s+/i,
  /\s+then(?:\s*,)?\s+/i,
  /\s+потом\s+/i,
  /\s+затем\s+/i,
  /\s+после\s+этого\s+/i,
  /\s+然后\s*/,
  /\s+接着\s*/,
];

// Issue #27: leading conjunctions ("then", "and then", "потом", "затем",
// "next", "after that", "然后", "接着") are linkers between steps, not part of
// the task itself. Strip them so each split segment is a clean instruction.
const AGENT_LEADING_CONJUNCTIONS =
  /^(?:and\s+then|then|next|after\s+that|потом|затем|после\s+этого|然后|接着)[\s,:]+/i;

function isAgentFormattingDirective(segment) {
  const normalized = normalizePrompt(segment);
  if (!normalized) return false;
  return /^(?:format|return|output|respond|write|show)\s+(?:(?:this|that|it|the result|the answer|the information|the output)\s+)?(?:as|in)\s+(?:a\s+|an\s+)?(?:json object|json|markdown table|table|csv|yaml|xml)$/.test(normalized);
}

const AGENT_QUOTED_PHRASE_PATTERN =
  /"([^"]+)"|'([^']+)'|`([^`]+)`|“([^”]+)”|«([^»]+)»/g;

function extractAgentQuotedPhrases(text) {
  const phrases = [];
  for (const match of String(text || "").matchAll(AGENT_QUOTED_PHRASE_PATTERN)) {
    const phrase = (match.slice(1).find((value) => value !== undefined) || "").trim();
    if (phrase) phrases.push(phrase);
  }
  return phrases;
}

function agentResearchCommandPrefix(segment) {
  const text = String(segment || "").trim().toLowerCase();
  if (!/^(?:search|find|look up|lookup|research)\b/.test(text)) return "";
  if (/\bwikipedia\b/.test(text)) return "Search Wikipedia for";
  if (/\bwikidata\b/.test(text)) return "Search Wikidata for";
  if (/\bwiktionary\b/.test(text)) return "Search Wiktionary for";
  return "Search the web for";
}

function agentComparisonFocus(segment) {
  const text = String(segment || "");
  if (!/\bcompare\b/i.test(text)) return "";
  const numberedTarget = text.match(
    /\bnumber\s+of\s+([A-Za-z][A-Za-z -]{0,40}?)(?:[.?!,;:]|$)/i,
  );
  if (numberedTarget) {
    const focus = numberedTarget[1]
      .replace(/\b(?:their|his|her|its|the)\b/gi, "")
      .replace(/\s+/g, " ")
      .trim();
    if (focus) return focus;
  }
  if (/\bpatents?\b/i.test(text)) return "patents";
  return "";
}

function expandAgentResearchStep(segment) {
  const commandPrefix = agentResearchCommandPrefix(segment);
  if (!commandPrefix) return [segment];
  const quotedPhrases = extractAgentQuotedPhrases(segment);
  if (quotedPhrases.length < 2) return [segment];
  const focus = agentComparisonFocus(segment);
  if (!focus) return [segment];
  const focusKey = focus.toLowerCase();
  return quotedPhrases.map((phrase) => {
    const query = phrase.toLowerCase().includes(focusKey)
      ? phrase
      : `${phrase} ${focus}`;
    return `${commandPrefix} "${query}"`;
  });
}

function decomposeAgentTask(text) {
  const trimmed = String(text || "").trim();
  if (!trimmed) return [];
  let segments = [trimmed];
  for (const sep of AGENT_STEP_SEPARATORS) {
    const next = [];
    for (const segment of segments) {
      const parts = segment.split(sep);
      for (const part of parts) {
        const cleaned = part.trim();
        if (cleaned) next.push(cleaned);
      }
    }
    segments = next;
  }
  const cleanedSegments = segments.map((segment) =>
    segment.replace(AGENT_LEADING_CONJUNCTIONS, "").trim(),
  ).filter((segment) => segment.length > 0);
  const mergedSegments = [];
  for (const segment of cleanedSegments) {
    if (isAgentFormattingDirective(segment) && mergedSegments.length > 0) {
      const previous = mergedSegments[mergedSegments.length - 1].replace(/\s*[.。]\s*$/u, "");
      mergedSegments[mergedSegments.length - 1] = `${previous}. Then ${segment}`;
    } else {
      mergedSegments.push(segment);
    }
  }
  return mergedSegments.flatMap((segment) => expandAgentResearchStep(segment));
}

function messagesForConversation(events, conversationId) {
  if (!conversationId) {
    return [];
  }
  const safe = Array.isArray(events) ? events : [];
  const out = [];
  for (let index = 0; index < safe.length; index += 1) {
    const event = safe[index];
    if (!event || event.kind && event.kind !== "message") continue;
    if ((event.conversationId || "legacy") !== conversationId) continue;
    const evidence = Array.isArray(event.evidence) ? event.evidence : [];
    out.push(
      createMessage(event.role || "assistant", String(event.content || ""), {
        intent: event.intent,
        evidence,
        iframeUrl: event.iframeUrl || null,
      }),
    );
  }
  return out;
}

// Issue #386: serialize a whole stored conversation to Markdown so it can be
// copied from the conversations list. Each turn becomes a `### <author>`
// section followed by the message body. When `includeReasoning` is set (the
// diagnostics surface is on) the per-turn reasoning steps — persisted as
// separate `reasoning` events recorded just before each assistant message —
// are appended after that AI message as a Markdown ordered list, so the export
// mirrors what the diagnostics panel shows on screen.
function conversationToMarkdown(events, conversationId, options = {}) {
  if (!conversationId) return "";
  const includeReasoning = options.includeReasoning === true;
  const userLabel = options.userLabel || "You";
  const assistantLabel = options.assistantLabel || "formal-ai";
  const reasoningLabel = options.reasoningLabel || "Reasoning";
  const safe = Array.isArray(events) ? events : [];
  const blocks = [];
  const title = (options.title || "").trim();
  if (title) {
    blocks.push(`# ${title}`);
  }
  let pendingReasoning = [];
  for (const event of safe) {
    if (!event) continue;
    if ((event.conversationId || "legacy") !== conversationId) continue;
    const kind = event.kind || "message";
    if (kind === "reasoning") {
      const detail = String(event.content || "").trim();
      if (detail) pendingReasoning.push(detail);
      continue;
    }
    if (kind !== "message") continue;
    const role = event.role || "assistant";
    const label = role === "user" ? userLabel : assistantLabel;
    const lines = [`### ${label}`, "", String(event.content || "")];
    if (role === "assistant" && includeReasoning && pendingReasoning.length > 0) {
      lines.push("", `#### ${reasoningLabel}`, "");
      pendingReasoning.forEach((step, index) => {
        lines.push(`${index + 1}. ${step}`);
      });
    }
    blocks.push(lines.join("\n"));
    pendingReasoning = [];
  }
  return blocks.join("\n\n");
}

function randomInt(min, max) {
  return Math.floor(Math.random() * (max - min + 1)) + min;
}

function timeLabel() {
  return new Date().toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
  });
}

// Render a single diagnostics detail value as JSON-ish text so the user can
// read the raw payload (PR #134 feedback 4489651616: "I want diagnostics to
// show exactly all steps with expandable requests/responses data, full lino
// data description and so on"). The function never throws — non-serializable
// values fall back to String() — so a diagnostics row can't crash the chat.
function formatDiagnosticPayload(value) {
  if (value === null || value === undefined) return "(empty)";
  if (typeof value === "string") return value;
  try {
    return JSON.stringify(value, null, 2);
  } catch (_error) {
    return String(value);
  }
}

function truncateDiagnosticDetail(value) {
  const text = formatDiagnosticPayload(value).replace(/\s+/g, " ").trim();
  if (text.length <= 64) return text;
  return `${text.slice(0, 61)}...`;
}

function summarizeToolCall(call) {
  if (!call || typeof call !== "object") return "";
  const parts = [];
  if (call.inputs && typeof call.inputs === "object") {
    const keys = Object.keys(call.inputs).slice(0, 3);
    if (keys.length > 0) parts.push(`in: ${keys.join(", ")}`);
  }
  if (call.outputs && typeof call.outputs === "object") {
    if (call.outputs.intent) parts.push(`out: ${call.outputs.intent}`);
    else {
      const keys = Object.keys(call.outputs).slice(0, 2);
      if (keys.length > 0) parts.push(`out: ${keys.join(", ")}`);
    }
  }
  return parts.join(" • ");
}

function createMessage(role, content, extra = {}) {
  return {
    id: `${role}-${Date.now()}-${Math.random().toString(16).slice(2)}`,
    role,
    author: role === "user" ? "You" : "formal-ai",
    content,
    sentAt: timeLabel(),
    ...extra,
  };
}

// Issue #153: dedicated renderer for the formalize / formalize_resolved
// diagnostics step. Keeps the SVO layout consistent regardless of source
// language and shows the canonical id prefixes (`Q`, `WP:`, `WT:`, `OP:`,
// `@USER`) so reviewers can verify the symbolic mapping. The verb slot
// labels the SVO triple in the user's UI language.
function FormalizationView({ formalization, t }) {
  if (!formalization) return null;
  return h(
    "div",
    { className: "formalization-view", "data-testid": "formalization" },
    formalization.raw
      ? h(
          "div",
          { className: "formalization-raw" },
          h("code", null, formalization.raw),
          h("span", { className: "formalization-arrow", "aria-hidden": "true" }, "→"),
          h("code", { className: "formalization-tuple" }, formalization.tuple),
        )
      : h("code", { className: "formalization-tuple" }, formalization.tuple),
    h(
      "div",
      { className: "formalization-svo" },
      h(
        "span",
        { className: "formalization-svo-label" },
        t("message.formalizationSubjectVerbObject"),
      ),
      h(
        "ol",
        { className: "formalization-svo-list" },
        h(
          "li",
          null,
          h("span", { className: "formalization-slot" }, "S"),
          h("code", null, formalization.subject || ""),
        ),
        h(
          "li",
          null,
          h("span", { className: "formalization-slot" }, "V"),
          h("code", null, formalization.verb || ""),
        ),
        h(
          "li",
          null,
          h("span", { className: "formalization-slot" }, "O"),
          h("code", null, formalization.object || ""),
        ),
      ),
    ),
  );
}

function escapeHtml(value) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

function isHttpExternalLink(href) {
  try {
    const url = new URL(href, window.location.href);
    return url.protocol === "http:" || url.protocol === "https:";
  } catch (_error) {
    return /^https?:\/\//i.test(String(href || ""));
  }
}

function enhanceMarkdownLinks(html) {
  if (typeof document === "undefined") return html;
  const template = document.createElement("template");
  template.innerHTML = html;
  template.content.querySelectorAll("a[href]").forEach((anchor) => {
    const href = anchor.getAttribute("href") || "";
    if (!isHttpExternalLink(href)) return;
    anchor.setAttribute("target", "_blank");
    anchor.setAttribute("rel", "noopener noreferrer");
    anchor.classList.add("external-link");
    if (!anchor.querySelector(".external-link-icon")) {
      anchor.appendChild(document.createTextNode(" "));
      const icon = document.createElement("span");
      icon.className = "external-link-icon";
      icon.setAttribute("aria-hidden", "true");
      anchor.appendChild(icon);
    }
  });
  return template.innerHTML;
}

function markdownHtml(value) {
  const text = String(value ?? "");
  if (window.marked && window.DOMPurify) {
    const html = window.marked.parse(text, {
      breaks: true,
      gfm: true,
    });
    return { __html: enhanceMarkdownLinks(window.DOMPurify.sanitize(html)) };
  }

  return { __html: escapeHtml(text).replaceAll("\n", "<br>") };
}

// Issue #330: copy helper shared by the per-code-block and per-message copy
// buttons. Prefers the async Clipboard API and falls back to a hidden textarea
// + execCommand so the feature still works in the Playwright/file:// contexts
// where the Clipboard API may be unavailable or permission-gated.
async function copyTextToClipboard(text) {
  const value = String(text ?? "");
  if (
    typeof navigator !== "undefined" &&
    navigator.clipboard &&
    typeof navigator.clipboard.writeText === "function"
  ) {
    try {
      await navigator.clipboard.writeText(value);
      return true;
    } catch (_error) {
      // Fall through to the legacy path below.
    }
  }
  if (typeof document === "undefined") return false;
  try {
    const textarea = document.createElement("textarea");
    textarea.value = value;
    textarea.setAttribute("readonly", "");
    textarea.style.position = "fixed";
    textarea.style.top = "-1000px";
    textarea.style.opacity = "0";
    document.body.appendChild(textarea);
    textarea.select();
    const ok = document.execCommand("copy");
    document.body.removeChild(textarea);
    return ok;
  } catch (_error) {
    return false;
  }
}

// Flash a transient "Copied!" label on a button, then restore the original.
function flashCopied(button, copiedLabel, restoreLabel) {
  if (!button) return;
  button.classList.add("is-copied");
  button.setAttribute("data-copied", "true");
  const labelNode = button.querySelector(".copy-button-label") || button;
  labelNode.textContent = copiedLabel;
  if (button._copyResetTimer) {
    clearTimeout(button._copyResetTimer);
  }
  button._copyResetTimer = setTimeout(() => {
    button.classList.remove("is-copied");
    button.removeAttribute("data-copied");
    labelNode.textContent = restoreLabel;
    button._copyResetTimer = null;
  }, 1600);
}

// Issue #330: progressively enhance the code fences rendered by marked. Each
// `<pre><code class="language-xxx">` is syntax-highlighted in place and wrapped
// in a `.code-block` shell carrying a language label and a per-block copy
// button. The function is idempotent so it can run on every effect pass without
// double-wrapping existing blocks.
function enhanceCodeBlocks(root, t) {
  if (!root || typeof document === "undefined") return;
  const highlighter =
    typeof window !== "undefined" ? window.FormalAiHighlight : null;
  const copyLabel = t ? t("message.copyCode") : "Copy";
  const copiedLabel = t ? t("message.copyCodeDone") : "Copied!";
  const copyTitle = t ? t("message.copyCodeTitle") : copyLabel;

  const blocks = root.querySelectorAll("pre > code");
  blocks.forEach((code) => {
    const pre = code.parentElement;
    if (!pre || pre.parentElement?.classList.contains("code-block")) {
      return; // already enhanced
    }

    const rawCode = code.textContent ?? "";
    const className = code.getAttribute("class") || "";
    const match = /language-([\w+#-]+)/i.exec(className);
    const requested = match ? match[1] : "";

    if (highlighter && typeof highlighter.highlight === "function") {
      const { value, language } = highlighter.highlight(rawCode, requested);
      code.innerHTML = value;
      code.classList.add("hljs");
      if (language) {
        code.setAttribute("data-language", language);
      }
    }

    const wrapper = document.createElement("div");
    wrapper.className = "code-block";

    const header = document.createElement("div");
    header.className = "code-block-header";

    const langLabel = document.createElement("span");
    langLabel.className = "code-block-lang";
    const resolved =
      highlighter && typeof highlighter.resolveLanguage === "function"
        ? highlighter.resolveLanguage(requested)
        : null;
    langLabel.textContent = (resolved || requested || "text").toLowerCase();

    const button = document.createElement("button");
    button.type = "button";
    button.className = "code-copy-button";
    button.setAttribute("data-testid", "code-copy-button");
    button.setAttribute("aria-label", copyTitle);
    button.setAttribute("title", copyTitle);
    const buttonLabel = document.createElement("span");
    buttonLabel.className = "copy-button-label";
    buttonLabel.textContent = copyLabel;
    button.appendChild(buttonLabel);
    button.addEventListener("click", async () => {
      const ok = await copyTextToClipboard(rawCode);
      if (ok) {
        flashCopied(button, copiedLabel, copyLabel);
      }
    });

    header.appendChild(langLabel);
    header.appendChild(button);

    pre.parentElement.insertBefore(wrapper, pre);
    wrapper.appendChild(header);
    wrapper.appendChild(pre);
  });
}

function normalizePrompt(prompt) {
  return String(prompt || "")
    .toLowerCase()
    .replace(/[^\p{L}\p{N}]+/gu, " ")
    .trim();
}

function isIdentityPrompt(normalized) {
  const tokens = normalized ? normalized.split(/\s+/) : [];
  const has = (token) => tokens.includes(token);
  return (
    [
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
      "चलो परिचय करते हैं",
      "आइए परिचय करें",
      "चलो एक दूसरे को जानें",
      "你是谁",
      "我们认识一下吧",
      "认识一下吧",
      "让我们认识一下",
    ].includes(normalized) ||
    (has("who") && has("you")) ||
    (has("what") && has("you")) ||
    ((has("who") || has("what")) && has("formal") && has("ai")) ||
    (has("tell") && has("yourself")) ||
    (has("introduce") && has("yourself")) ||
    (has("let") && has("s") && has("acquainted")) ||
    (has("lets") && has("acquainted")) ||
    (has("let") && has("us") && has("acquainted")) ||
    (has("know") && has("each") && has("other")) ||
    (has("кто") && has("ты")) ||
    (has("что") && has("ты")) ||
    (has("расскажи") && has("себе")) ||
    (has("опиши") && has("себя")) ||
    (has("давай") && has("знакомиться")) ||
    (has("давай") && has("познакомимся")) ||
    (has("давайте") && has("познакомимся")) ||
    (has("चलो") && has("परिचय")) ||
    (has("आइए") && has("परिचय"))
  );
}

function isLocalAssistantFreeTimePrompt(normalized) {
  return [
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
  ].includes(normalized);
}

function localPromptLanguage(prompt) {
  const raw = String(prompt || "");
  if (/[\u0400-\u04ff]/u.test(raw)) return "ru";
  if (/[\u0900-\u097f]/u.test(raw)) return "hi";
  if (/[\u3400-\u9fff]/u.test(raw)) return "zh";
  return "en";
}

function isAssistantNamePrompt(normalized) {
  const tokens = normalized ? normalized.split(/\s+/) : [];
  const has = (token) => tokens.includes(token);
  return (
    [
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
    ].includes(normalized) ||
    (has("what") && has("your") && has("name")) ||
    (has("you") && has("have") && has("name")) ||
    (has("call") && has("you")) ||
    (has("как") && has("тебя") && has("зовут"))
  );
}

function localAssistantNameAnswer(prompt, preferences = {}) {
  const name = normalizeAssistantName(preferences.assistantName);
  const raw = String(prompt || "");
  if (name && /[а-яё]/iu.test(raw)) {
    return `Меня зовут ${name}. Я formal AI.`;
  }
  if (name && /[\u0900-\u097f]/u.test(raw)) {
    return `मेरा नाम ${name} है। मैं formal AI हूँ।`;
  }
  if (name && /[\u3400-\u9fff]/u.test(raw)) {
    return `我的名字是 ${name}。我是 formal AI。`;
  }
  if (name) {
    return `My name is ${name}. I'm formal AI.`;
  }
  if (/[а-яё]/iu.test(raw)) {
    return "Я formal AI, и сейчас у меня нет имени. Но вы можете назвать меня как хотите.";
  }
  if (/[\u0900-\u097f]/u.test(raw)) {
    return "मैं formal AI हूँ, और अभी मेरा कोई नाम नहीं है। लेकिन आप मुझे अपनी पसंद का नाम दे सकते हैं।";
  }
  if (/[\u3400-\u9fff]/u.test(raw)) {
    return "我是 formal AI,目前还没有名字。不过您可以按自己的喜好给我起名。";
  }
  return ASSISTANT_NAME_ANSWER;
}

function localBehaviorRuleId(value) {
  let hash = 2166136261;
  const text = String(value || "");
  for (let index = 0; index < text.length; index += 1) {
    hash ^= text.charCodeAt(index);
    hash = Math.imul(hash, 16777619) >>> 0;
  }
  return `behavior_rule_runtime_${hash.toString(16)}`;
}

function localCodeSpans(text) {
  return String(text || "")
    .split("`")
    .map((part, index) => (index % 2 === 1 ? part.trim() : ""))
    .filter(Boolean);
}

// Issue #144: mirror the worker's multilingual `When X then Y` grammar so the
// local fallback recognizes the same teach forms even without WASM.
const LOCAL_BEHAVIOR_RULE_KEYWORD_PAIRS = [
  ["when ", " then "],
  ["when ", " do "],
  ["когда ", " тогда "],
  ["когда ", " делай "],
  ["когда ", " сделай "],
  ["когда ", " отвечай "],
  ["когда ", " отвечать "],
  ["если ", " то "],
  ["जब ", " तब "],
  ["जब ", " तो "],
  ["当 ", " 时 "],
  ["当 ", " 则 "],
  ["当 ", " 回答 "],
  ["当 ", "时回答 "],
  ["当 ", "则回答 "],
];

function localLooksLikeRuntimeRuleUpdate(text) {
  const raw = String(text || "");
  const lower = raw.toLowerCase();
  if (
    (lower.includes("when i say") && (lower.includes("answer") || lower.includes("reply"))) ||
    (lower.includes("if i ask") && (lower.includes("answer") || lower.includes("reply"))) ||
    lower.includes("add behavior rule") ||
    lower.includes("update behavior rule") ||
    (lower.includes("когда я скажу") && lower.includes("ответ")) ||
    (lower.includes("если я спрошу") && lower.includes("ответ")) ||
    lower.includes("добавь правило поведения") ||
    lower.includes("обнови правило поведения")
  ) {
    return true;
  }
  for (const [head, link] of LOCAL_BEHAVIOR_RULE_KEYWORD_PAIRS) {
    const headPos = lower.indexOf(head);
    if (headPos === -1) continue;
    const tail = lower.slice(headPos + head.length);
    const linkPos = tail.indexOf(link);
    if (linkPos === -1) continue;
    const absoluteLinkPos = headPos + head.length + linkPos;
    const beforeLink = raw.slice(headPos, absoluteLinkPos);
    const afterLink = raw.slice(absoluteLinkPos + link.length);
    if (beforeLink.includes("`") && afterLink.includes("`")) return true;
  }
  return false;
}

function localRuntimeRuleFromText(text) {
  if (!localLooksLikeRuntimeRuleUpdate(text)) return null;
  const spans = localCodeSpans(text);
  if (spans.length < 2) return null;
  const trigger = spans[0].trim();
  const answer = spans[1].trim();
  if (!trigger || !answer) return null;
  return {
    id: localBehaviorRuleId(`${trigger}\n${answer}`),
    trigger,
    answer,
  };
}

function localBehaviorRuleRecords() {
  return [
    {
      id: "rule_greeting",
      topic: "greetings",
      intent: "greeting",
      label: "Greeting rule",
      matches: "`Hi`, `Hello`, and `Hey`",
      response: "Hi, how may I help you?",
      source: "local fallback",
      whenThen:
        "When the user says `Hi`, `Hello`, or `Hey` then respond with `Hi, how may I help you?`.",
    },
    {
      id: "rule_identity",
      topic: "identity",
      intent: "identity",
      label: "Identity rule",
      matches: "`Who are you?`, `Кто ты?`, and equivalent identity prompts",
      response: IDENTITY_ANSWER,
      source: "local fallback",
      whenThen: `When the user asks \`Who are you?\` or \`Кто ты?\` then respond with the identity answer.`,
    },
    {
      id: "rule_assistant_free_time",
      topic: "small_talk",
      intent: "assistant_free_time",
      label: "Assistant free-time rule",
      matches:
        "`What do you do in your free time?`, `Что делаешь в свободное время?`, and equivalent small-talk seed phrases",
      response: ASSISTANT_FREE_TIME_ANSWER,
      source: "local fallback",
      whenThen: `When the user asks what I do in free time then respond with \`${ASSISTANT_FREE_TIME_ANSWER}\`.`,
    },
    {
      id: "rule_assistant_name",
      topic: "assistant_name",
      intent: "assistant_name",
      label: "Assistant name rule",
      matches: "`What is your name?`, `Как твое имя?`, and equivalent name prompts",
      response: ASSISTANT_NAME_ANSWER,
      source: "local fallback",
      whenThen:
        "When the user asks `What is your name?` or `Как твое имя?` then respond with the assistant-name answer, unless the assistant name setting is configured.",
    },
    {
      id: "rule_unknown",
      topic: "unknown_fallback",
      intent: "unknown",
      label: "Unknown fallback rule",
      matches: "Any prompt that no earlier rule can answer",
      response: UNKNOWN_ANSWER,
      source: "local fallback",
      whenThen:
        "When no earlier rule or handler matches the prompt then respond with the unknown-intent guide.",
    },
  ];
}

const LOCAL_BEHAVIOR_RULE_TOPIC_ORDER = [
  "greetings",
  "identity",
  "small_talk",
  "assistant_name",
  "unknown_fallback",
];

function localLocalizedText(language, values) {
  return values[language] || values.en;
}

function localBehaviorRuleTopicLabel(topic, language) {
  const labels = {
    greetings: { en: "Greetings", ru: "Приветствия", hi: "अभिवादन", zh: "问候" },
    identity: { en: "Identity", ru: "Идентичность", hi: "पहचान", zh: "身份" },
    small_talk: {
      en: "Small talk",
      ru: "Светская беседа",
      hi: "हल्की बातचीत",
      zh: "闲聊",
    },
    assistant_name: {
      en: "Assistant name",
      ru: "Имя ассистента",
      hi: "सहायक का नाम",
      zh: "助手名称",
    },
    unknown_fallback: {
      en: "Unknown fallback",
      ru: "Резервный ответ",
      hi: "अज्ञात अनुरोध का वैकल्पिक उत्तर",
      zh: "未知请求回退",
    },
  };
  return localLocalizedText(language, labels[topic] || {
    en: "Other",
    ru: "Другое",
    hi: "अन्य",
    zh: "其他",
  });
}

function localBehaviorRuleListIntro(language) {
  return localLocalizedText(language, {
    en: "Behavior rules I can inspect in this dialog (grouped by topic, each shown as a `When X then Y` statement):",
    ru: "Правила поведения, которые я могу показать в этом диалоге (сгруппированы по темам; каждое показано как инструкция `Когда X тогда Y`):",
    hi: "व्यवहार नियम जिन्हें मैं इस संवाद में दिखा सकता हूँ (विषय के अनुसार समूहित; हर नियम `जब X तब Y` कथन के रूप में है):",
    zh: "我可以查看的行为规则（按主题分组；每条都显示为 `当 X 时 Y` 语句）：",
  });
}

function localRuntimeRuleWhenThen(rule, language) {
  if (language === "ru") return `Когда пользователь говорит \`${rule.trigger}\`, ответь \`${rule.answer}\`.`;
  if (language === "hi") return `जब उपयोगकर्ता \`${rule.trigger}\` कहे, तब \`${rule.answer}\` उत्तर दें.`;
  if (language === "zh") return `当用户说 \`${rule.trigger}\` 时，回答 \`${rule.answer}\`。`;
  return `When the user says \`${rule.trigger}\` then respond with \`${rule.answer}\`.`;
}

function localRuleResponse(rule, language) {
  if (rule.id === "rule_greeting") {
    if (language === "ru") return "Здравствуйте! Чем могу помочь?";
    if (language === "hi") return "नमस्ते! मैं आपकी क्या मदद कर सकता हूँ?";
    if (language === "zh") return "你好，请问我可以帮你什么？";
  }
  if (rule.id === "rule_assistant_free_time") {
    return localLocalizedText(language, {
      en: ASSISTANT_FREE_TIME_ANSWER,
      ru: "У меня нет свободного времени в человеческом смысле. Между запросами я бездействую; когда диалог активен, помогаю с задачами, правилами и объяснениями.",
      hi: "मेरे पास मनुष्यों जैसा खाली समय नहीं है. prompts के बीच मैं निष्क्रिय रहता हूँ; dialog सक्रिय हो तो tasks, rules और explanations में मदद करता हूँ.",
      zh: "我没有人类意义上的空闲时间。两次提示之间我处于空闲状态；对话活跃时，我帮助处理任务、规则和解释。",
    });
  }
  if (rule.id === "rule_assistant_name") {
    return localLocalizedText(language, {
      en: "Returns the assistant-name answer; browser surfaces can override it from the assistant name setting.",
      ru: "Возвращает ответ об имени ассистента; браузерные поверхности могут переопределить его настройкой имени ассистента.",
      hi: "assistant-name उत्तर लौटाता है; browser surfaces assistant name setting से इसे बदल सकते हैं.",
      zh: "返回助手名称回答；浏览器界面可通过助手名称设置覆盖它。",
    });
  }
  return rule.response;
}

function localRuleLabel(rule, language) {
  const labels = {
    rule_greeting: {
      en: "Greeting rule",
      ru: "Правило приветствия",
      hi: "अभिवादन नियम",
      zh: "问候规则",
    },
    rule_identity: {
      en: "Identity rule",
      ru: "Правило идентичности",
      hi: "पहचान नियम",
      zh: "身份规则",
    },
    rule_assistant_free_time: {
      en: "Assistant free-time rule",
      ru: "Правило свободного времени ассистента",
      hi: "सहायक खाली समय नियम",
      zh: "助手空闲时间规则",
    },
    rule_assistant_name: {
      en: "Assistant name rule",
      ru: "Правило имени ассистента",
      hi: "सहायक नाम नियम",
      zh: "助手名称规则",
    },
    rule_unknown: {
      en: "Unknown fallback rule",
      ru: "Резервное правило для неизвестного запроса",
      hi: "अज्ञात अनुरोध का वैकल्पिक नियम",
      zh: "未知请求回退规则",
    },
  };
  return labels[rule.id] ? localLocalizedText(language, labels[rule.id]) : rule.label;
}

function localRuleMatches(rule, language) {
  const matches = {
    rule_greeting: {
      en: "`Hi`, `Hello`, and `Hey`",
      ru: "`Hi`, `Hello`, `Hey` и многоязычные seed-фразы приветствия",
      hi: "`Hi`, `Hello`, `Hey` और बहुभाषी greeting seed phrases",
      zh: "`Hi`、`Hello`、`Hey` 以及多语言问候 seed 短语",
    },
    rule_identity: {
      en: "`Who are you?`, `Кто ты?`, and equivalent identity prompts",
      ru: "`Who are you?`, `Кто ты?` и равнозначные вопросы об идентичности",
      hi: "`Who are you?`, `Кто ты?` और समान identity prompts",
      zh: "`Who are you?`、`Кто ты?` 以及等价身份提示",
    },
    rule_assistant_free_time: {
      en: "`What do you do in your free time?`, `Что делаешь в свободное время?`, and equivalent small-talk seed phrases",
      ru: "`What do you do in your free time?`, `Что делаешь в свободное время?` и равнозначные seed-фразы светской беседы",
      hi: "`What do you do in your free time?`, `Что делаешь в свободное время?` और समान small-talk seed phrases",
      zh: "`What do you do in your free time?`、`Что делаешь в свободное время?` 以及等价闲聊 seed 短语",
    },
    rule_assistant_name: {
      en: "`What is your name?`, `Как твое имя?`, and equivalent name prompts",
      ru: "`What is your name?`, `Как твое имя?` и равнозначные вопросы об имени",
      hi: "`What is your name?`, `Как твое имя?` और समान name prompts",
      zh: "`What is your name?`、`Как твое имя?` 以及等价名称提示",
    },
    rule_unknown: {
      en: "Any prompt that no earlier rule can answer",
      ru: "Любой запрос, на который не ответило более раннее правило",
      hi: "कोई भी prompt जिसका उत्तर पहले का rule नहीं दे सकता",
      zh: "任何前面的规则无法回答的提示",
    },
  };
  return matches[rule.id] ? localLocalizedText(language, matches[rule.id]) : rule.matches;
}

function localRuleWhenThen(rule, language) {
  const response = localRuleResponse(rule, language);
  if (rule.id === "rule_greeting") {
    if (language === "ru") return `Когда пользователь говорит \`Hi\`, \`Hello\`, \`Hey\` или многоязычную фразу приветствия, ответь \`${response}\`.`;
    if (language === "hi") return `जब उपयोगकर्ता \`Hi\`, \`Hello\`, \`Hey\` या बहुभाषी greeting phrase कहे, तब \`${response}\` उत्तर दें.`;
    if (language === "zh") return `当用户说 \`Hi\`、\`Hello\`、\`Hey\` 或多语言问候短语时，回答 \`${response}\`。`;
  }
  if (rule.id === "rule_identity") {
    if (language === "ru") return "Когда пользователь спрашивает `Who are you?` или `Кто ты?`, ответь сообщением об идентичности.";
    if (language === "hi") return "जब उपयोगकर्ता `Who are you?` या `Кто ты?` पूछे, तब identity answer दें.";
    if (language === "zh") return "当用户问 `Who are you?` 或 `Кто ты?` 时，回答身份说明。";
  }
  if (rule.id === "rule_assistant_free_time") {
    if (language === "ru") return `Когда пользователь спрашивает, что я делаю в свободное время, ответь \`${response}\`.`;
    if (language === "hi") return `जब उपयोगकर्ता पूछे कि मैं खाली समय में क्या करता हूँ, तब \`${response}\` उत्तर दें.`;
    if (language === "zh") return `当用户问我空闲时间做什么时，回答 \`${response}\`。`;
  }
  if (rule.id === "rule_assistant_name") {
    if (language === "ru") return "Когда пользователь спрашивает `What is your name?` или `Как твое имя?`, ответь сообщением об имени ассистента; если настройка имени есть, включи настроенное имя.";
    if (language === "hi") return "जब उपयोगकर्ता `What is your name?` या `Как твое имя?` पूछे, तब assistant-name उत्तर दें; अगर setting है, तो configured name शामिल करें.";
    if (language === "zh") return "当用户问 `What is your name?` 或 `Как твое имя?` 时，回答助手名称；如果有名称设置，则包含配置的名称。";
  }
  if (rule.id === "rule_unknown") {
    if (language === "ru") return "Когда ни одно более раннее правило не подходит к запросу, ответь подсказкой для неизвестного намерения.";
    if (language === "hi") return "जब कोई पहले का rule prompt से मेल न खाए, तब unknown-intent guide दें.";
    if (language === "zh") return "当前面的规则都不匹配提示时，回答未知意图指南。";
  }
  return rule.whenThen;
}

function localBehaviorRuleListFooter(language) {
  if (language === "ru") {
    return [
      "",
      "Прочитать одно правило можно командой `Покажи правило unknown`.",
      "Научить этот диалог можно так: ``Когда `ваш запрос` тогда `ваш ответ` ``. Также можно: ``Когда я скажу `ваш запрос`, ответь `ваш ответ` ``.",
      "Многоязычные формы: английская ``When `X` then `Y` ``, хинди ``जब `X` तब `Y` ``, китайская ``当 `X` 时 `Y` ``.",
      "Запись добавляется только в конец: экспортируйте память, чтобы сохранить сообщение с правилом вместе с диалогом.",
    ];
  }
  if (language === "hi") {
    return [
      "",
      "एक नियम पढ़ने के लिए `Show behavior rule unknown` भेजें.",
      "इस संवाद को सिखाएँ: ``जब `आपका प्रश्न` तब `आपका उत्तर` ``. दूसरा रूप: ``When I say `your prompt`, answer `your answer` ``.",
      "बहुभाषी रूप: रूसी ``Когда `X` тогда `Y` ``, अंग्रेज़ी ``When `X` then `Y` ``, चीनी ``当 `X` 时 `Y` ``.",
      "लेखन केवल append-only है: नियम संदेश को संवाद के साथ रखने के लिए memory export करें.",
    ];
  }
  if (language === "zh") {
    return [
      "",
      "要读取一条规则，请发送 `Show behavior rule unknown`。",
      "可以这样教当前对话：``当 `你的提示` 时 `你的回答` ``。也可以发送：``When I say `your prompt`, answer `your answer` ``。",
      "多语言形式：俄语 ``Когда `X` тогда `Y` ``，印地语 ``जब `X` तब `Y` ``，英语 ``When `X` then `Y` ``。",
      "写入是 append-only：导出 memory 可把这条规则消息随对话一起保存。",
    ];
  }
  return [
    "",
    "Read one with `Show behavior rule unknown`.",
    "Teach this dialog with: ``When `your prompt` then `your answer` ``. Equivalent: ``When I say `your prompt`, answer `your answer` ``.",
    "Multilingual forms: Russian ``Когда `X` тогда `Y` ``, Hindi ``जब `X` तब `Y` ``, Chinese ``当 `X` 时 `Y` ``.",
    "The write is append-only: export memory to preserve the rule message with the dialog.",
  ];
}

function localBehaviorRulesList(runtimeRules, language = "en") {
  const lines = [localBehaviorRuleListIntro(language), ""];
  const groups = new Map();
  for (const rule of localBehaviorRuleRecords()) {
    const order = LOCAL_BEHAVIOR_RULE_TOPIC_ORDER.indexOf(rule.topic);
    const safeOrder = order === -1 ? LOCAL_BEHAVIOR_RULE_TOPIC_ORDER.length : order;
    if (!groups.has(safeOrder)) {
      groups.set(safeOrder, {
        label: localBehaviorRuleTopicLabel(rule.topic, language),
        rules: [],
      });
    }
    groups.get(safeOrder).rules.push(rule);
  }
  const ordered = Array.from(groups.entries()).sort((a, b) => a[0] - b[0]);
  ordered.forEach(([, group], index) => {
    lines.push(`### ${group.label}`);
    for (const rule of group.rules) {
      lines.push(`- \`${rule.id}\` -> ${localRuleWhenThen(rule, language)}`);
    }
    if (index + 1 < ordered.length) lines.push("");
  });
  if (Array.isArray(runtimeRules) && runtimeRules.length > 0) {
    lines.push("", `### ${localLocalizedText(language, {
      en: "Dialog-local rules taught in this conversation",
      ru: "Правила, изученные в этом диалоге",
      hi: "इस संवाद में सिखाए गए स्थानीय नियम",
      zh: "本对话中学到的局部规则",
    })}`);
    for (const rule of runtimeRules) {
      lines.push(`- \`${rule.id}\` -> ${localRuntimeRuleWhenThen(rule, language)}`);
    }
  }
  lines.push(...localBehaviorRuleListFooter(language));
  return lines.join("\n");
}

function localBehaviorRuleDetail(rule, language = "en") {
  const label = localRuleLabel(rule, language);
  const whenThen = localRuleWhenThen(rule, language);
  const matches = localRuleMatches(rule, language);
  const response = localRuleResponse(rule, language);
  const changeHint = localLocalizedText(language, {
    en: "To change this behavior in the current dialog, send: ``When `your prompt` then `your answer` ``. Equivalent: ``When I say `your prompt`, answer `your answer` ``.",
    ru: "Чтобы изменить это поведение в текущем диалоге, отправьте: ``Когда `ваш запрос` тогда `ваш ответ` ``. Также можно: ``Когда я скажу `ваш запрос`, ответь `ваш ответ` ``.",
    hi: "इस व्यवहार को वर्तमान संवाद में बदलने के लिए भेजें: ``जब `आपका प्रश्न` तब `आपका उत्तर` ``. दूसरा रूप: ``When I say `your prompt`, answer `your answer` ``.",
    zh: "要在当前对话中改变此行为，请发送：``当 `你的提示` 时 `你的回答` ``。也可以发送：``When I say `your prompt`, answer `your answer` ``。",
  });
  return [
    label,
    "",
    whenThen || "",
    "",
    "```links",
    rule.id,
    `  topic "${(rule.topic || "").replaceAll('"', '\\"')}"`,
    `  intent "${rule.intent}"`,
    `  matches "${matches.replaceAll('"', '\\"')}"`,
    `  response "${response.replaceAll('"', '\\"')}"`,
    `  source "${rule.source}"`,
    `  when_then "${(whenThen || "").replaceAll('"', '\\"')}"`,
    "```",
    "",
    changeHint,
  ].join("\n");
}

function localAssistantNameStatus(preferences = {}) {
  const name = normalizeAssistantName(preferences.assistantName);
  return name ? `configured:${name}` : "browser_preference_when_set_else_not_configured";
}

function localLinoEscape(value) {
  return String(value || "").replaceAll("\\", "\\\\").replaceAll('"', '\\"').replaceAll("\n", "\\n");
}

const LOCAL_BROWSER_SURFACE = {
  slug: "browser",
  label: "browser demo with JavaScript and WebAssembly worker",
  runtime: "JavaScript UI plus a WebAssembly worker mirror of the solver",
  memory: "browser IndexedDB/local storage plus worker state and imported memory",
  webSearch: "available through browser CORS-readable providers when online and not blocked",
  limits: "browser settings, import/export controls, and IndexedDB-backed memory belong to this surface",
};

function localModeStatus(enabled) {
  return enabled ? "enabled" : "disabled";
}

function localDefinitionFusionStatus(preferences = {}) {
  return preferences.definitionFusion === "auto" ? "enabled_by_default" : "explicit_only";
}

function localBlueprintCompositionStatus(preferences = {}) {
  return normalizeBlueprintComposition(preferences.blueprintComposition);
}

function localSelfFacts(preferences = {}) {
  const assistantName = localAssistantNameStatus(preferences);
  const surface = LOCAL_BROWSER_SURFACE;
  return [
    "Facts I know about myself in this environment:",
    "",
    `- **Execution surface**: ${surface.label} (\`${surface.slug}\`).`,
    `- **Runtime**: ${surface.runtime}.`,
    `- **Memory**: ${surface.memory}.`,
    `- **Web search**: ${surface.webSearch}.`,
    `- **Surface limits**: ${surface.limits}.`,
    "- **Local rules**: local Links Notation rules and seed facts are checked first.",
    "",
    "```links",
    "self_fact_model",
    '  subject "formal-ai"',
    '  relation "model"',
    '  object "formal-symbolic-production"',
    "self_fact_policy",
    '  subject "formal-ai"',
    '  relation "policy"',
    '  object "deterministic symbolic AI; no neural network inference"',
    "self_fact_environment",
    '  subject "formal-ai"',
    '  relation "execution_surface"',
    `  object "${surface.slug}"`,
    "self_fact_runtime",
    '  subject "formal-ai"',
    '  relation "runtime"',
    `  object "${localLinoEscape(surface.runtime)}"`,
    "self_fact_memory",
    '  subject "formal-ai"',
    '  relation "memory"',
    `  object "${localLinoEscape(surface.memory)}"`,
    "self_fact_web_search",
    '  subject "formal-ai"',
    '  relation "web_search"',
    `  object "${localLinoEscape(surface.webSearch)}"`,
    "self_fact_assistant_name",
    '  subject "formal-ai"',
    '  relation "assistant_name"',
    `  object "${localLinoEscape(assistantName)}"`,
    "self_fact_agent_mode",
    '  subject "formal-ai"',
    '  relation "agent_mode"',
    `  object "${localModeStatus(preferences.agentMode)}"`,
    "self_fact_diagnostics",
    '  subject "formal-ai"',
    '  relation "diagnostic_mode"',
    `  object "${localModeStatus(preferences.diagnosticsMode)}"`,
    "self_fact_definition_fusion",
    '  subject "formal-ai"',
    '  relation "definition_fusion"',
    `  object "${localDefinitionFusionStatus(preferences)}"`,
    "self_fact_blueprint_composition",
    '  subject "formal-ai"',
    '  relation "blueprint_composition"',
    `  object "${localBlueprintCompositionStatus(preferences)}"`,
    "```",
    "",
    "Read behavior with `List behavior rules`; teach one with When `prompt` then `answer` (or When I say `prompt`, answer `answer`).",
  ].join("\n");
}

function localKnownFacts(language, preferences = {}) {
  const surface = LOCAL_BROWSER_SURFACE;
  const assistantName = localAssistantNameStatus(preferences);
  const links = [
    "```links",
    "known_fact_local_seed",
    '  source "local_links_notation_seed"',
    '  scope "built-in rules, concepts, facts, tools, and response templates"',
    "known_fact_internet",
    '  source "environment_aware_web_search"',
    `  scope "${localLinoEscape(surface.webSearch)}"`,
    "known_fact_memory",
    '  source "conversation_memory"',
    `  scope "${localLinoEscape(surface.memory)}"`,
    "known_fact_environment",
    '  subject "formal-ai"',
    '  relation "execution_surface"',
    `  object "${surface.slug}"`,
    "known_fact_self",
    '  subject "formal-ai"',
    '  relation "model"',
    '  object "formal-symbolic-production"',
    "known_fact_assistant_name",
    '  subject "formal-ai"',
    '  relation "assistant_name_setting"',
    `  object "${localLinoEscape(assistantName)}"`,
    "known_fact_surface_limits",
    '  source "environment_directory"',
    `  scope "${localLinoEscape(surface.limits)}"`,
    "```",
  ].join("\n");
  if (language === "ru") {
    return [
      `Я могу использовать несколько классов фактов в текущей среде \`${surface.slug}\`:`,
      "",
      "- **Локальные факты и правила**: встроенный seed Links Notation, включая правила, понятия, инструменты и ответы.",
      `- **Интернет**: ${surface.webSearch}; это не означает, что весь интернет предзагружен в локальную память.`,
      `- **Память диалога**: ${surface.memory}.`,
      "- **Факты о себе**: модель `formal-symbolic-production`, политика исполнения, поверхность и источники ответов.",
      `- **Ограничения среды**: ${surface.limits}.`,
      "",
      links,
      "",
      "Для конкретного факта задайте прямой вопрос; порядок проверки: локальные правила, память, затем веб-поиск, если он доступен в этой среде.",
    ].join("\n");
  }
  return [
    `I can use several classes of facts in the current \`${surface.slug}\` environment:`,
    "",
    "- **Local facts and rules**: built-in Links Notation seed data, including rules, concepts, tools, and response templates.",
    `- **Internet**: ${surface.webSearch}; the whole internet is not preloaded into local memory.`,
    `- **Conversation memory**: ${surface.memory}.`,
    "- **Self facts**: model `formal-symbolic-production`, execution policy, active surface, and answer sources.",
    `- **Surface limits**: ${surface.limits}.`,
    "",
    links,
    "",
    "Ask for a specific fact directly; I check local rules and memory first, then use web search only when this environment allows it.",
  ].join("\n");
}

function localContainsAny(normalized, needles) {
  return needles.some((needle) => normalized.includes(needle));
}

function localIsSelfFactQuery(normalized) {
  return (
    normalized.includes("facts you know about yourself") ||
    normalized.includes("self facts") ||
    normalized.includes("факты о себе") ||
    normalized.includes("какие факты ты знаешь о себе")
  );
}

function localIsSelfIntroductionQuery(normalized) {
  const cleaned = normalizePrompt(normalized);
  if (!cleaned || localIsSelfFactQuery(cleaned)) return false;
  return (
    cleaned === "tell me about yourself" ||
    cleaned === "introduce yourself" ||
    cleaned.includes("tell me about yourself") ||
    cleaned.includes("introduce yourself") ||
    cleaned.includes("let s get acquainted") ||
    cleaned.includes("lets get acquainted") ||
    cleaned.includes("let us get acquainted") ||
    cleaned.includes("let s get to know each other") ||
    cleaned.includes("расскажи о себе") ||
    cleaned.includes("расскажи мне о себе") ||
    cleaned.includes("расскажи про себя") ||
    cleaned.includes("опиши себя") ||
    cleaned.includes("представься") ||
    cleaned.includes("давай знакомиться") ||
    cleaned.includes("давай познакомимся") ||
    cleaned.includes("давайте познакомимся") ||
    cleaned.includes("चलो परिचय करते हैं") ||
    cleaned.includes("आइए परिचय करें") ||
    cleaned.includes("चलो एक दूसरे को जानें") ||
    cleaned.includes("我们认识一下") ||
    cleaned.includes("认识一下吧") ||
    cleaned.includes("让我们认识一下")
  );
}

function localSelfAwarenessLanguage(prompt, normalized) {
  const text = `${String(prompt || "").toLowerCase()} ${String(normalized || "")}`;
  if (/[\u0400-\u04ff]/u.test(text) || localContainsAny(text, ["ты", "теб", "у тебя"])) {
    return "ru";
  }
  if (/[\u0900-\u097f]/u.test(text)) return "hi";
  if (/[\u4e00-\u9fff]/u.test(text)) return "zh";
  return "en";
}

function localSelfIntroductionContent(language, preferences = {}) {
  const identity = IDENTITY_ANSWER;
  const name = normalizeAssistantName(preferences.assistantName);
  if (!name) return identity;
  if (language === "ru") return `Меня зовут ${name}. ${identity}`;
  if (language === "hi") return `मेरा नाम ${name} है। ${identity}`;
  if (language === "zh") return `我的名字是 ${name}。${identity}`;
  return `My name is ${name}. ${identity}`;
}

function localCleanConversationTopic(raw) {
  return String(raw || "")
    .trim()
    .replace(/^[`"':._,\-\s!?]+|[`"':._,\-\s!?]+$/gu, "");
}

function localConversationTopic(prompt, normalized) {
  const source = String(prompt || "");
  const lower = source.toLowerCase();
  for (const prefix of [
    "let's talk about ",
    "lets talk about ",
    "can we talk about ",
    "talk about ",
    "давай поговорим о ",
    "давай поговорим об ",
    "давайте поговорим о ",
    "давайте поговорим об ",
    "поговорим о ",
    "поговорим об ",
    "обсудим ",
    "चलो बात करें ",
    "बात करें ",
    "聊聊",
    "谈谈",
  ]) {
    if (String(normalized || "").startsWith(prefix)) {
      return localCleanConversationTopic(String(normalized || "").slice(prefix.length));
    }
  }
  const marker = "поговорим о ";
  const index = lower.indexOf(marker);
  if (index >= 0) return localCleanConversationTopic(lower.slice(index + marker.length));
  return "";
}

function localConversationTopicContent(topic, language) {
  if (language === "ru") {
    return `Можем. Тема: ${topic}. Я могу начать с краткого определения, контекста или конкретного вопроса; если веб-поиск доступен, публичные факты можно уточнить через внешний источник.`;
  }
  if (language === "hi") {
    return `हम बात कर सकते हैं. विषय: ${topic}. मैं छोटी परिभाषा, संदर्भ, या किसी конкрет प्रश्न से शुरू कर सकता हूँ; web search उपलब्ध हो तो public facts बाहरी स्रोत से जाँचे जा सकते हैं.`;
  }
  if (language === "zh") {
    return `可以聊。主题: ${topic}。我可以从简短定义、上下文或具体问题开始; 如果 web search 可用, 公开事实可以通过外部来源核对。`;
  }
  return `We can talk about ${topic}. I can start with a short definition, context, or a specific question; when web search is available, public facts can be checked against an external source.`;
}

function localIsKnownFactQuery(normalized) {
  const english =
    (normalized.includes("facts") &&
      localContainsAny(normalized, ["what", "which", "list", "show"]) &&
      localContainsAny(normalized, [
        "you know",
        "do you know",
        "you have",
        "available to you",
        "in your knowledge",
        "known to you",
      ])) ||
    localContainsAny(normalized, [
      "what do you know in general",
      "what do you know about the world",
      "what is known to you",
      "what knowledge do you have",
    ]);
  const russian =
    (normalized.includes("факт") &&
      localContainsAny(normalized, ["какие", "что", "перечисли", "покажи", "назови"]) &&
      localContainsAny(normalized, [
        "ты знаешь",
        "знаешь",
        "тебе извест",
        "у тебя есть",
        "твои знания",
        "что ты знаешь",
      ])) ||
    localContainsAny(normalized, [
      "что тебе вообще известно",
      "что тебе известно",
      "что ты вообще знаешь",
      "что ты знаешь об окружающем мире",
      "известно об окружающем мире",
      "знаешь про окружающий мир",
      "знаешь об окружающем мире",
    ]);
  const hindi = localContainsAny(normalized, [
    "आप क्या जानते हैं",
    "तुम क्या जानते हो",
    "आपको क्या पता है",
  ]);
  const chinese = localContainsAny(normalized, ["你知道什么", "您知道什么", "你知道哪些"]);
  return english || russian || hindi || chinese;
}

function localIsArchitectureQuestion(normalized) {
  const mentionsAssistant = localContainsAny(normalized, [
    "you",
    "your",
    "formal ai",
    "ты",
    "теб",
    "твоя",
    "твой",
    "тво",
    "вы",
  ]);
  if (!mentionsAssistant) return false;
  return localContainsAny(normalized, [
    "llm",
    "large language model",
    "language model",
    "openai api",
    "openai",
    "neural inference",
    "neural network",
    "links notation rules",
    "local rules",
    "world model",
    "model of the world",
    "бям",
    "языковая модель",
    "языковой моделью",
    "нейросет",
    "нейрон",
    "локальных правил",
    "локальных правилах",
    "область знаний",
    "модель окружающего мира",
    "модель мира",
    "принцип работы",
    "идея твоей разработки",
    "идея твоего проекта",
    "зачем тебя разработ",
    "ссылк",
  ]);
}

function localArchitectureExplanation(language) {
  const surface = LOCAL_BROWSER_SURFACE;
  if (language === "ru") {
    return `Я не LLM-рантайм и не выполняю нейросетевой инференс. Текущая среда: ${surface.label} (\`${surface.slug}\`). Рантайм: ${surface.runtime}. У проекта есть OpenAI-совместимые API-форматы, но ответы строит детерминированный solver: сначала он проверяет локальный seed Links Notation, правила и память (${surface.memory}); затем веб-поиск используется только с учетом среды: ${surface.webSearch}. Весь интернет не загружен в локальные правила целиком.`;
  }
  return `I am not an LLM runtime and I do not perform neural inference. Current environment: ${surface.label} (\`${surface.slug}\`). Runtime: ${surface.runtime}. The project exposes OpenAI-compatible API shapes, but answers come from a deterministic solver: it checks the local Links Notation seed, rules, and memory (${surface.memory}) first; web search is used only when this environment allows it: ${surface.webSearch}. The whole internet is not preloaded into local rules.`;
}

function localCleanRuleQuery(raw) {
  return String(raw || "")
    .trim()
    .replace(/^[\s`"':._,\-?!]+|[\s`"':._,\-?!]+$/g, "")
    .toLowerCase();
}

function localDetailQuery(prompt) {
  const lower = String(prompt || "").toLowerCase();
  for (const prefix of ["show behavior rule", "read behavior rule", "show rule", "read rule"]) {
    if (lower.startsWith(prefix)) {
      return localCleanRuleQuery(String(prompt || "").slice(prefix.length));
    }
  }
  if (lower.includes("rule_unknown")) return "unknown";
  return "";
}

function localFindBehaviorRule(query) {
  const cleaned = localCleanRuleQuery(query);
  const withoutPrefix = cleaned.startsWith("rule_") ? cleaned.slice(5) : cleaned;
  return localBehaviorRuleRecords().find(
    (rule) =>
      rule.id === cleaned ||
      rule.id === `rule_${withoutPrefix}` ||
      rule.intent === cleaned ||
      rule.intent === withoutPrefix,
  );
}

function localRuntimeRuleForPrompt(prompt, history) {
  const normalizedPrompt = normalizePrompt(prompt);
  const turns = Array.isArray(history) ? history : [];
  for (let index = turns.length - 1; index >= 0; index -= 1) {
    const turn = turns[index] || {};
    if (String(turn.role || "").toLowerCase() !== "user") continue;
    const rule = localRuntimeRuleFromText(turn.content);
    if (rule && normalizePrompt(rule.trigger) === normalizedPrompt) {
      return rule;
    }
  }
  return null;
}

function tryLocalBehaviorRules(prompt, normalized, history, preferences = {}) {
  const language = localSelfAwarenessLanguage(prompt, normalized);
  const updateRule = localRuntimeRuleFromText(prompt);
  if (updateRule) {
    const whenThen = localRuntimeRuleWhenThen(updateRule, language);
    const title = localLocalizedText(language, {
      en: "Behavior rule recorded for this dialog.",
      ru: "Правило поведения записано для этого диалога.",
      hi: "इस संवाद के लिए व्यवहार नियम record किया गया.",
      zh: "已为本对话记录行为规则。",
    });
    const sendHint =
      language === "ru"
        ? `Отправьте \`${updateRule.trigger}\` сейчас, и я отвечу настроенным ответом. Экспортируйте память, чтобы сохранить это правило вместе с диалогом.`
        : language === "hi"
          ? `\`${updateRule.trigger}\` अभी भेजें और मैं configured response से उत्तर दूँगा. इस rule message को dialog के साथ रखने के लिए memory export करें.`
          : language === "zh"
            ? `现在发送 \`${updateRule.trigger}\`，我会使用配置的回答。导出 memory 可把这条规则消息随对话一起保存。`
            : `Send \`${updateRule.trigger}\` now and I will answer with the configured response. Export memory to keep this rule message with the dialog.`;
    return {
      intent: "behavior_rule_update",
      content: [
        title,
        "",
        whenThen,
        "",
        "```links",
        updateRule.id,
        '  type "behavior_rule_runtime"',
        `  match_prompt "${updateRule.trigger.replaceAll('"', '\\"')}"`,
        `  answer "${updateRule.answer.replaceAll('"', '\\"')}"`,
        `  when_then "${whenThen.replaceAll('"', '\\"')}"`,
        '  source "user_message"',
        "```",
        "",
        sendHint,
      ].join("\n"),
    };
  }
  if (
    matchesLocalBehaviorRulesListPattern(normalized) ||
    normalized.includes("list behavior rules") ||
    normalized.includes("list all behavior rules") ||
    normalized.includes("show behavior rules") ||
    isSupportedLanguageBehaviorRulesListQuery(normalized) ||
    normalized.includes("список правил поведения")
  ) {
    return { intent: "behavior_rules_list", content: localBehaviorRulesList([], language) };
  }
  const query = localDetailQuery(prompt);
  if (query) {
    const rule = localFindBehaviorRule(query);
    if (rule) {
      return { intent: "behavior_rule_detail", content: localBehaviorRuleDetail(rule, language) };
    }
  }
  if (localIsSelfIntroductionQuery(normalized)) {
    const language = localSelfAwarenessLanguage(prompt, normalized);
    return {
      intent: "identity",
      content: localSelfIntroductionContent(language, preferences),
    };
  }
  if (localIsSelfFactQuery(normalized)) {
    return { intent: "self_facts", content: localSelfFacts(preferences) };
  }
  if (localIsKnownFactQuery(normalized)) {
    const language = localSelfAwarenessLanguage(prompt, normalized);
    return { intent: "known_facts", content: localKnownFacts(language, preferences) };
  }
  const topic = localConversationTopic(prompt, normalized);
  if (topic) {
    const language = localSelfAwarenessLanguage(prompt, normalized);
    return { intent: "conversation_topic", content: localConversationTopicContent(topic, language) };
  }
  const runtimeRule = localRuntimeRuleForPrompt(prompt, history);
  if (runtimeRule) {
    return { intent: "behavior_rule_custom", content: runtimeRule.answer };
  }
  return null;
}

const LOCAL_BEHAVIOR_RULES_LIST_PATTERNS = [
  "show behavior rules",
  "show list of your rules",
  "list your rules",
  "покажи правила поведения",
  "покажи список своих правил",
  "перечисли свои правила",
  "व्यवहार के नियम सूचीबद्ध करें",
  "अपने नियमों की सूची दिखाओ",
  "अपने नियम गिनाओ",
  "列出行为规则",
  "显示你的规则列表",
  "列出你的规则",
];

function matchesLocalBehaviorRulesListPattern(normalized) {
  return LOCAL_BEHAVIOR_RULES_LIST_PATTERNS.some((pattern) => {
    const text = normalizePrompt(pattern);
    return text && (normalized === text || normalized.includes(text));
  });
}

function isSupportedLanguageBehaviorRulesListQuery(normalized) {
  return (
    isEnglishBehaviorRulesListQuery(normalized) ||
    isRussianBehaviorRulesListQuery(normalized) ||
    isHindiBehaviorRulesListQuery(normalized) ||
    isChineseBehaviorRulesListQuery(normalized)
  );
}

function isEnglishBehaviorRulesListQuery(normalized) {
  const mentionsRules =
    normalized.includes("rules") ||
    normalized.includes("rule list") ||
    normalized.includes("rules list");
  const asksToList =
    normalized.includes("list") ||
    normalized.includes("show") ||
    normalized.includes("what") ||
    normalized.includes("which");
  const pointsAtAssistantRules =
    normalized.includes("behavior") ||
    normalized.includes("your") ||
    normalized.includes("own") ||
    normalized.includes("current") ||
    normalized.includes("existing");

  return mentionsRules && asksToList && pointsAtAssistantRules;
}

function isRussianBehaviorRulesListQuery(normalized) {
  const mentionsRules = normalized.includes("правил") || normalized.includes("правила");
  const asksToList =
    normalized.includes("список") ||
    normalized.includes("перечисли") ||
    normalized.includes("покажи") ||
    normalized.includes("какие");
  const pointsAtAssistantRules =
    normalized.includes("поведения") ||
    normalized.includes("своих") ||
    normalized.includes("свои") ||
    normalized.includes("твоих") ||
    normalized.includes("твои") ||
    normalized.includes("собственные") ||
    normalized.includes("список правил");

  return mentionsRules && asksToList && pointsAtAssistantRules;
}

function isHindiBehaviorRulesListQuery(normalized) {
  const mentionsRules = normalized.includes("नियम") || normalized.includes("नियमों");
  const asksToList =
    normalized.includes("सूची") ||
    normalized.includes("सूचीबद्ध") ||
    normalized.includes("दिखाओ") ||
    normalized.includes("दिखाएं") ||
    normalized.includes("बताओ") ||
    normalized.includes("गिनाओ") ||
    normalized.includes("कौन");
  const pointsAtAssistantRules =
    normalized.includes("व्यवहार") ||
    normalized.includes("अपने") ||
    normalized.includes("तुम्हारे") ||
    normalized.includes("आपके") ||
    normalized.includes("नियमों की सूची");

  return mentionsRules && asksToList && pointsAtAssistantRules;
}

function isChineseBehaviorRulesListQuery(normalized) {
  const mentionsRules = normalized.includes("规则") || normalized.includes("規則");
  const asksToList =
    normalized.includes("列出") ||
    normalized.includes("显示") ||
    normalized.includes("顯示") ||
    normalized.includes("展示") ||
    normalized.includes("哪些") ||
    normalized.includes("什么");
  const pointsAtAssistantRules =
    normalized.includes("行为") ||
    normalized.includes("行為") ||
    normalized.includes("你的") ||
    normalized.includes("您的") ||
    normalized.includes("自己") ||
    normalized.includes("规则列表") ||
    normalized.includes("規則列表");

  return mentionsRules && asksToList && pointsAtAssistantRules;
}

function chooseVariant(variants, randomize) {
  if (!Array.isArray(variants) || variants.length === 0) return "";
  if (!randomize || variants.length === 1) return variants[0];
  return variants[Math.floor(Math.random() * variants.length)] || variants[0];
}

function shouldIncludeCourtesyFollowUp(probability, randomize) {
  const normalized = normalizeSliderPreference(
    probability,
    PREFERENCE_DEFAULTS.followUpProbability,
  );
  if (normalized <= 0) return false;
  if (normalized >= 1) return true;
  if (!randomize) return normalized >= 0.5;
  return Math.random() < normalized;
}

function courtesyResponseContent(preferences = {}) {
  const temperature = normalizeSliderPreference(
    preferences.temperature,
    PREFERENCE_DEFAULTS.temperature,
  );
  const randomize = temperature > 0;
  const acknowledgement = chooseVariant(COURTESY_ACKNOWLEDGEMENTS, randomize);
  if (!shouldIncludeCourtesyFollowUp(preferences.followUpProbability, randomize)) {
    return acknowledgement;
  }
  const followUp = chooseVariant(COURTESY_FOLLOW_UPS, randomize);
  return `${acknowledgement} ${followUp}`;
}

function desktopBridge() {
  if (typeof window === "undefined" || !window.FormalAiDesktop) {
    return null;
  }
  return window.FormalAiDesktop;
}

function normalizeDesktopStatus(status) {
  if (!status || typeof status !== "object") {
    return null;
  }
  const apiBase = String(status.apiBase || "").replace(/\/+$/, "");
  return {
    shell: String(status.shell || "Electron"),
    mode: String(status.mode || (apiBase ? "server" : "in-process")),
    apiBase,
    staticBase: String(status.staticBase || ""),
    graphUrl: String(status.graphUrl || (apiBase ? `${apiBase}/v1/graph` : "")),
    traceUrl: String(status.traceUrl || (apiBase ? `${apiBase}/v1/graph?trace=answer_greeting_hi` : "")),
    memory: String(status.memory || "formal_ai_bundle"),
    agentModeDefault: Boolean(status.agentModeDefault),
    toolCallPolicy: String(status.toolCallPolicy || "explicit-permission"),
    apiReady: status.apiReady !== false && Boolean(apiBase),
    apiError: String(status.apiError || ""),
  };
}

function compactUrl(value) {
  if (!value) {
    return "unavailable";
  }
  try {
    const parsed = new URL(value);
    const pathName = parsed.pathname === "/" ? "" : parsed.pathname;
    return `${parsed.host}${pathName}`;
  } catch (_error) {
    return String(value);
  }
}

// Issue #353: the same chat UI now also runs inside a VS Code Webview. Label the
// host surface from the bridge's shell string ("VS Code" / "VS Code Web" from the
// extension; "Electron" from the desktop shell) so the status line and sidebar
// read correctly in either embedder.
function desktopSurfaceLabel(status) {
  return /code/i.test(String((status && status.shell) || "")) ? "VS Code" : "Desktop";
}

function desktopStatusLabel(status, agentMode) {
  if (!status) {
    return "";
  }
  const api = status.apiReady
    ? "API local"
    : status.apiError
      ? "API unavailable"
      : "in-process";
  const agent = agentMode ? "agent opted in" : "agent permission off";
  return `${desktopSurfaceLabel(status)} - ${api} - ${agent}`;
}

function desktopMessages(history, text) {
  const messages = [];
  for (const entry of Array.isArray(history) ? history : []) {
    if (!entry || !["user", "assistant"].includes(entry.role)) {
      continue;
    }
    const content = typeof entry.content === "string" ? entry.content : "";
    if (content.trim()) {
      messages.push({ role: entry.role, content });
    }
  }
  messages.push({ role: "user", content: String(text || "") });
  return messages;
}

// R5d: the agent-permission toggle is the explicit opt-in for the local tool
// router (default-deny). When the user opts in, the main process is allowed to
// run permitted tools through the local app / Docker sandbox; when off, every
// tool call is refused before anything executes.
function syncDesktopToolGrants(bridge, agentMode) {
  if (!bridge || typeof bridge.setToolGrants !== "function") {
    return;
  }
  Promise.resolve(bridge.setToolGrants({ all: Boolean(agentMode) })).catch(() => {});
}

// Route a single tool call through the desktop bridge to the local process /
// Docker sandbox. Returns a structured refusal when the bridge is unavailable so
// callers never silently fall back to executing in the browser.
async function requestDesktopToolCall(bridge, tool, input = {}) {
  if (!bridge || typeof bridge.invokeTool !== "function") {
    return {
      ok: false,
      tool: String(tool || ""),
      status: "unavailable",
      executed: false,
      reason: "desktop tool router is unavailable",
    };
  }
  return bridge.invokeTool({ tool: String(tool || ""), input: input || {} });
}

// R5c: reconcile the browser (IndexedDB) memory log with the native store over
// the local server's Links-Notation memory endpoints. Best-effort: a failed sync
// never blocks the conversation.
async function syncDesktopMemory(bridge, lino) {
  if (!bridge || typeof bridge.syncMemory !== "function") {
    return null;
  }
  try {
    return await bridge.syncMemory({ lino: String(lino || "") });
  } catch (_error) {
    return null;
  }
}

async function requestDesktopAnswer(text, history, desktopStatus, preferences = {}) {
  const apiBase = desktopStatus && desktopStatus.apiBase;
  if (!apiBase) {
    throw new Error("desktop API is unavailable");
  }

  const endpoint = `${apiBase}/v1/chat/completions`;
  const response = await fetch(endpoint, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      model: "formal-symbolic-production",
      messages: desktopMessages(history, text),
      temperature: normalizeSliderPreference(preferences.temperature, 0),
      stream: false,
    }),
  });

  if (!response.ok) {
    throw new Error(`desktop API returned ${response.status}`);
  }

  const payload = await response.json();
  const answerText =
    payload &&
    payload.choices &&
    payload.choices[0] &&
    payload.choices[0].message
      ? String(payload.choices[0].message.content || "")
      : "";

  return {
    intent: "desktop_http_chat",
    content: answerText || UNKNOWN_ANSWER,
    source: "desktop_http",
    evidence: [
      "surface:desktop",
      "api:/v1/chat/completions",
      desktopStatus.graphUrl ? "network:/v1/graph" : "",
    ].filter(Boolean),
    steps: [
      { step: "desktop_shell", detail: "Electron preload bridge supplied local API status" },
      { step: "http_chat", detail: "POST /v1/chat/completions on the local Rust server" },
      { step: "memory", detail: "UI import/export stays on formal_ai_bundle" },
    ],
    diagnostics: {
      providers: [
        {
          id: "formal_ai_desktop_http",
          status: "ok",
          endpoint,
        },
      ],
      http: [
        {
          provider: "formal_ai_desktop_http",
          url: endpoint,
          method: "POST",
          status: response.status,
          ok: response.ok,
        },
      ],
    },
  };
}

function localFallbackAnswer(prompt, history = [], preferences = {}) {
  const normalized = normalizePrompt(prompt);
  const behaviorRule = tryLocalBehaviorRules(prompt, normalized, history, preferences);
  if (behaviorRule) {
    return behaviorRule;
  }
  if (localIsArchitectureQuestion(normalized)) {
    const language = /[\u0400-\u04ff]/u.test(String(prompt || "")) ? "ru" : "en";
    return { intent: "meta_explanation", content: localArchitectureExplanation(language) };
  }
  if (["hi", "hello", "hey"].includes(normalized)) {
    return {
      intent: "greeting",
      content: "Hi, how may I help you?",
    };
  }

  if (isLocalAssistantFreeTimePrompt(normalized)) {
    return {
      intent: "assistant_free_time",
      content: localRuleResponse(
        { id: "rule_assistant_free_time", response: ASSISTANT_FREE_TIME_ANSWER },
        localPromptLanguage(prompt),
      ),
    };
  }

  const courtesyResponses = new Set([
    "thanks",
    "thank you",
    "i am fine thank you",
    "i am fine thanks",
    "i m fine thank you",
    "i m fine thanks",
    "ого чето начал соображать",
    "ого чёто начал соображать",
    "ого чё то начал соображать",
    "ого что то начал соображать",
  ]);
  if (courtesyResponses.has(normalized)) {
    return {
      intent: "courtesy_response",
      content: courtesyResponseContent(preferences),
    };
  }

  if (isAssistantNamePrompt(normalized)) {
    return {
      intent: "assistant_name",
      content: localAssistantNameAnswer(prompt, preferences),
    };
  }

  if (isIdentityPrompt(normalized)) {
    return {
      intent: "identity",
      content: IDENTITY_ANSWER,
    };
  }

  return {
    intent: "unknown",
    content: localUnknownAnswerWithVariation(prompt),
  };
}

// Mirrors `src/engine.rs::UNKNOWN_OPENERS_EN` so the React fallback (used when
// the worker is unavailable, e.g. on `file://`) presents the same set of
// variations as the worker and Rust solver. Only the English pool is kept
// here because the React fallback never reaches non-English seeds.
const LOCAL_UNKNOWN_OPENERS = [
  "I don't know how to answer that yet.",
  "I didn't understand you.",
  "I'm not sure how to respond to that yet.",
  "I haven't learned to answer that yet.",
  "That one is new to me.",
];

function localSelectUnknownOpener(prompt) {
  const trimmed = String(prompt || "").trim();
  if (trimmed === "") return LOCAL_UNKNOWN_OPENERS[0];
  const id = localBehaviorRuleId(`unknown_opener\n${trimmed}`);
  const hex = id.split("_").pop() || "0";
  const value = parseInt(hex, 16) || 0;
  return LOCAL_UNKNOWN_OPENERS[value % LOCAL_UNKNOWN_OPENERS.length];
}

function localUnknownAnswerWithVariation(prompt) {
  const opener = localSelectUnknownOpener(prompt);
  const body = String(UNKNOWN_ANSWER || "").trimStart();
  for (const known of LOCAL_UNKNOWN_OPENERS) {
    if (body.startsWith(known)) {
      const rest = body.slice(known.length).trimStart();
      return rest ? `${opener} ${rest}` : opener;
    }
  }
  const idx = body.indexOf(". ");
  if (idx >= 0) {
    return `${opener} ${body.slice(idx + 2).trimStart()}`;
  }
  return `${opener} ${body}`;
}

function createDemoTurns() {
  const greetings = demoGreetings();
  const features = demoFeaturePrompts();
  const turns = [];
  if (greetings.length > 0) {
    const greeting = greetings[demoGreetingCursor % greetings.length];
    demoGreetingCursor = (demoGreetingCursor + 1) % greetings.length;
    turns.push({ text: greeting.text, label: greeting.label });
  }
  if (features.length > 0) {
    const feature = features[demoFeatureCursor % features.length];
    demoFeatureCursor = (demoFeatureCursor + 1) % features.length;
    turns.push({ text: feature.text, label: feature.label });
  }
  return turns;
}

function appendCodeBlock(lines, value) {
  const text = String(value ?? "");
  const fence = text.includes("```") ? "````" : "```";
  lines.push(fence);
  lines.push(text);
  lines.push(fence);
}

// Issue #78: render the entire dialog as a single fenced block with `U:` /
// `A:` line prefixes, instead of one Markdown subsection per message. Keeps
// the prefilled GitHub issue body short enough to fit the `?body=` query
// string (which truncates around 8 KB) and easier for a maintainer to scan.
function pickDialogFence(messages) {
  let fence = "```";
  while (messages.some((message) => String(message.content ?? "").includes(fence))) {
    fence += "`";
  }
  return fence;
}

function appendDialogBlock(lines, messages, effectiveFocus, options = {}) {
  if (messages.length === 0) {
    lines.push("No messages have been sent yet.");
    return;
  }

  lines.push("Legend: `U` = user, `A` = agent.");
  lines.push("");
  const fence = pickDialogFence(messages);
  lines.push(fence);
  const earlierOmitted = Math.max(0, Number(options.earlierOmitted) || 0);
  if (earlierOmitted > 0) {
    lines.push(`... omitted ${earlierOmitted} earlier ${earlierOmitted === 1 ? "message" : "messages"} ...`);
  }
  messages.forEach((message) => {
    const prefix = message.role === "user" ? "U" : "A";
    const annotations = [];
    if (message.intent === "unknown") {
      annotations.push(`intent: ${message.intent}`);
    }
    if (effectiveFocus && effectiveFocus.id === message.id) {
      if (message.intent && message.intent !== "unknown") {
        annotations.push(`intent: ${message.intent}`);
      }
      annotations.push("reported");
    }
    const head = annotations.length > 0 ? `${prefix} (${annotations.join(", ")})` : prefix;
    const content = String(message.content ?? "");
    const [first, ...rest] = content.split("\n");
    lines.push(`${head}: ${first}`);
    rest.forEach((row) => lines.push(`   ${row}`));
  });
  lines.push(fence);
}

// Issue #140: GitHub caps the prefilled-issue URL at 8192 characters, so for
// chats that produce a long transcript we have to shrink the body. We keep
// the last two turns intact in shape and replace the rest with summary
// markers: "... omitted N earlier messages ..." for trimmed-out turns,
// "... omitted N lines ..." inside a multi-line message, and
// "... omitted N characters ..." inside a single long line. The exact ceiling
// is `GITHUB_URL_MAX_LENGTH` (documented limit); `URL_SAFETY_MARGIN` keeps a
// small buffer for the encoded `&labels=…` tail.
const GITHUB_URL_MAX_LENGTH = 8192;
const URL_SAFETY_MARGIN = 16;
const URL_BUDGET = GITHUB_URL_MAX_LENGTH - URL_SAFETY_MARGIN;

function truncateSingleLine(text, maxChars) {
  const str = String(text);
  if (str.length <= maxChars) return str;
  const markerTemplate = "... omitted XXXXX characters ...";
  const reservedForMarker = markerTemplate.length + 12;
  const half = Math.max(8, Math.floor((maxChars - reservedForMarker) / 2));
  if (half * 2 + reservedForMarker >= str.length) {
    // Not enough headroom for a useful trim — fall back to a head-only slice.
    const headOnly = str.slice(0, Math.max(8, maxChars - reservedForMarker));
    const omitted = str.length - headOnly.length;
    return `${headOnly}... omitted ${omitted} characters ...`;
  }
  const start = str.slice(0, half);
  const end = str.slice(str.length - half);
  const omitted = str.length - start.length - end.length;
  return `${start}... omitted ${omitted} characters ...${end}`;
}

function truncateMessageContent(content, maxChars) {
  const str = String(content ?? "");
  if (str.length <= maxChars) return str;
  const lines = str.split("\n");
  if (lines.length > 2) {
    const first = lines[0];
    const last = lines[lines.length - 1];
    const omitted = lines.length - 2;
    const combined = `${first}\n... omitted ${omitted} lines ...\n${last}`;
    if (combined.length <= maxChars) return combined;
    return `${truncateSingleLine(first, Math.floor((maxChars - 32) / 2))}\n... omitted ${omitted} lines ...\n${truncateSingleLine(last, Math.floor((maxChars - 32) / 2))}`;
  }
  return truncateSingleLine(str, maxChars);
}

const REPORT_TRACE_MAX_CHARS = 2400;
const REPORT_TRACE_ITEM_LIMIT = 20;

function compactReportTraceValue(value, limit = 180) {
  const raw =
    value !== null && typeof value === "object"
      ? formatDiagnosticPayload(value)
      : String(value ?? "");
  const compact = raw.replace(/\s+/g, " ").trim();
  return truncateSingleLine(compact, limit);
}

function appendLimitedTraceItems(lines, items, formatter) {
  const safeItems = Array.isArray(items) ? items : [];
  if (safeItems.length <= REPORT_TRACE_ITEM_LIMIT) {
    safeItems.forEach((item) => {
      lines.push(formatter(item));
    });
    return;
  }

  const headCount = Math.ceil(REPORT_TRACE_ITEM_LIMIT / 2);
  const tailCount = REPORT_TRACE_ITEM_LIMIT - headCount;
  safeItems.slice(0, headCount).forEach((item) => {
    lines.push(formatter(item));
  });
  lines.push(`- ... omitted ${safeItems.length - REPORT_TRACE_ITEM_LIMIT} middle trace items ...`);
  safeItems.slice(safeItems.length - tailCount).forEach((item) => {
    lines.push(formatter(item));
  });
}

function appendReasoningTraceBlock(lines, focusMessage) {
  if (!focusMessage || focusMessage.role !== "assistant") return;

  const trace = [];
  if (focusMessage.intent) {
    trace.push(`intent: ${focusMessage.intent}`);
  }

  if (Array.isArray(focusMessage.evidence) && focusMessage.evidence.length > 0) {
    trace.push("evidence:");
    appendLimitedTraceItems(
      trace,
      focusMessage.evidence,
      (item) => `- ${compactReportTraceValue(item)}`,
    );
  }

  if (
    Array.isArray(focusMessage.diagnosticsSteps) &&
    focusMessage.diagnosticsSteps.length > 0
  ) {
    trace.push("diagnostics_steps:");
    appendLimitedTraceItems(trace, focusMessage.diagnosticsSteps, (entry) => {
      const step = compactReportTraceValue(entry?.step || "step", 80);
      const detail = entry?.formalization?.tuple || entry?.detail || "";
      return `- ${step}: ${compactReportTraceValue(detail)}`;
    });
  } else if (
    Array.isArray(focusMessage.thinkingSteps) &&
    focusMessage.thinkingSteps.length > 0
  ) {
    trace.push("thinking_steps:");
    appendLimitedTraceItems(
      trace,
      focusMessage.thinkingSteps,
      (item) => `- ${compactReportTraceValue(item)}`,
    );
  }

  if (
    Array.isArray(focusMessage.diagnosticsToolCalls) &&
    focusMessage.diagnosticsToolCalls.length > 0
  ) {
    trace.push("tool_calls:");
    appendLimitedTraceItems(trace, focusMessage.diagnosticsToolCalls, (call) => {
      const tool = compactReportTraceValue(call?.tool || "tool", 80);
      const summary = summarizeToolCall(call || {});
      return `- ${tool}: ${compactReportTraceValue(summary)}`;
    });
  }

  if (trace.length === 0) return;

  lines.push("");
  lines.push("## Reasoning Trace");
  lines.push("");
  lines.push("Focused assistant turn:");
  lines.push("");
  appendCodeBlock(lines, truncateMessageContent(trace.join("\n"), REPORT_TRACE_MAX_CHARS));
  lines.push("");
}

function buildIssueUrl(title, body, labels) {
  const params = new URLSearchParams({ title, body, labels });
  return `https://github.com/${ISSUE_REPOSITORY}/issues/new?${params.toString()}`;
}

function buildIssueUrlForMessages(context, buildBody, title, labels, messages, earlierOmitted) {
  const body = buildBody({ ...context, messages, earlierOmitted });
  return buildIssueUrl(title, body, labels);
}

function fitIssueUrl(context, buildBody) {
  const title = createIssueTitle(context.messages, context.focusMessage);
  const labels = ISSUE_LABELS;
  const messages = Array.isArray(context.messages) ? context.messages : [];

  // Fast path: build with the full transcript and return when it already fits.
  let body = buildBody({ ...context, messages, earlierOmitted: 0 });
  let url = buildIssueUrl(title, body, labels);
  if (url.length <= URL_BUDGET) return url;

  // Step 1: keep the last two messages as the minimum useful reproduction,
  // then backfill older turns while URL budget remains.
  let includedMessages = messages.slice(-Math.min(2, messages.length));
  let earlierOmitted = messages.length - includedMessages.length;
  url = buildIssueUrlForMessages(
    context,
    buildBody,
    title,
    labels,
    includedMessages,
    earlierOmitted,
  );

  // If the final exchange itself is too large, shrink it first so the link
  // stays usable before trying to preserve any earlier context.
  if (url.length > URL_BUDGET) {
    for (const perMessageBudget of [4096, 2048, 1024, 512, 256, 128, 64, 32]) {
      const truncatedMessages = includedMessages.map((message) => ({
        ...message,
        content: truncateMessageContent(message.content, perMessageBudget),
      }));
      url = buildIssueUrlForMessages(
        context,
        buildBody,
        title,
        labels,
        truncatedMessages,
        earlierOmitted,
      );
      if (url.length <= URL_BUDGET) return url;
    }
    return url;
  }

  let bestUrl = url;

  while (earlierOmitted > 0) {
    const boundaryIndex = earlierOmitted - 1;
    const candidateMessages = [messages[boundaryIndex], ...includedMessages];
    const candidateOmitted = boundaryIndex;
    url = buildIssueUrlForMessages(
      context,
      buildBody,
      title,
      labels,
      candidateMessages,
      candidateOmitted,
    );
    if (url.length <= URL_BUDGET) {
      includedMessages = candidateMessages;
      earlierOmitted = candidateOmitted;
      bestUrl = url;
      continue;
    }

    // The next earlier turn does not fit in full. Keep a truncated version
    // instead of dropping all context before the last two messages.
    for (const perMessageBudget of [4096, 2048, 1024, 512, 256, 128, 64, 32]) {
      const truncatedBoundary = {
        ...messages[boundaryIndex],
        content: truncateMessageContent(messages[boundaryIndex].content, perMessageBudget),
      };
      url = buildIssueUrlForMessages(
        context,
        buildBody,
        title,
        labels,
        [truncatedBoundary, ...includedMessages],
        candidateOmitted,
      );
      if (url.length <= URL_BUDGET) return url;
    }

    return bestUrl;
  }

  // Final defensive pass: if the transcript had fewer than two messages and
  // still overflowed, shrink whatever was available.
  for (const perMessageBudget of [4096, 2048, 1024, 512, 256, 128, 64, 32]) {
    const truncatedMessages = includedMessages.map((message) => ({
      ...message,
      content: truncateMessageContent(message.content, perMessageBudget),
    }));
    url = buildIssueUrlForMessages(
      context,
      buildBody,
      title,
      labels,
      truncatedMessages,
      earlierOmitted,
    );
    if (url.length <= URL_BUDGET) return url;
  }

  return bestUrl;
}

function shortText(value, limit = 70) {
  const normalized = String(value ?? "").replace(/\s+/g, " ").trim();
  if (normalized.length <= limit) {
    return normalized;
  }

  return `${normalized.slice(0, limit - 3)}...`;
}

function promptBeforeMessage(messages, focusMessage) {
  let prompt = "";
  for (const message of messages) {
    if (message.role === "user") {
      prompt = message.content;
    }
    if (focusMessage && message.id === focusMessage.id) {
      break;
    }
  }
  return prompt;
}

function lastUnknownAssistantMessage(messages) {
  for (let i = messages.length - 1; i >= 0; i -= 1) {
    if (messages[i].role === "assistant" && messages[i].intent === "unknown") {
      return messages[i];
    }
  }
  return null;
}

function createIssueTitle(messages, focusMessage) {
  const effectiveFocus = focusMessage ?? lastUnknownAssistantMessage(messages);
  const prompt = promptBeforeMessage(messages, effectiveFocus);
  if (effectiveFocus?.intent === "unknown" && prompt) {
    return `Unknown prompt: ${shortText(prompt, 80)}`;
  }
  if (prompt) {
    return `Issue with dialog: ${shortText(prompt, 80)}`;
  }
  return "formal-ai demo issue report";
}

// Issue #386: the worker kind (`wasm worker`) used to occupy its own
// Environment line. Folding it into the version (`0.174.0 (wasm)`) keeps the
// header compact. The trailing " worker" word is redundant inside the parens.
function formatVersionWithWorker(version, workerState) {
  const worker = String(workerState || "").trim();
  if (!worker) return version;
  const short = worker.replace(/\s*workers?$/i, "").trim() || worker;
  return `${version} (${short})`;
}

function createIssueReportBody({
  messages,
  focusMessage,
  workerState,
  demoMode,
  demoStatus,
  diagnosticsMode,
  userContext,
  earlierOmitted = 0,
}) {
  const effectiveFocus = focusMessage ?? lastUnknownAssistantMessage(messages);
  // Issue #386: fold the worker into the version (`0.174.0 (wasm)`) and drop
  // settings that sit at their default. Manual mode is the interactive default,
  // so Mode/Status are only worth reporting while a demo is playing, and
  // Diagnostics is only reported when it has been turned on.
  const lines = [
    "## Environment",
    "",
    `- **Version**: ${formatVersionWithWorker(APP_VERSION, workerState)}`,
    `- **URL**: ${window.location.href}`,
  ];
  if (demoMode) {
    lines.push("- **Mode**: demo");
    lines.push(`- **Status**: ${demoStatus}`);
  }
  if (diagnosticsMode) {
    lines.push("- **Diagnostics**: on");
  }
  lines.push(`- **Timestamp**: ${new Date().toISOString()}`);
  lines.push("");

  appendUserContextBlock(lines, userContext);
  lines.push("## Reproduction of dialog");
  lines.push("");

  appendDialogBlock(lines, messages, effectiveFocus, { earlierOmitted });

  // Issue #386: the reasoning trace is only meaningful next to the full dialog.
  // When earlier turns had to be dropped to fit GitHub's URL cap the dialog is
  // no longer complete, so the trace is omitted to avoid misleading context.
  if (earlierOmitted === 0) {
    appendReasoningTraceBlock(lines, effectiveFocus);
  }

  lines.push("");
  lines.push("## Description");
  lines.push("");
  lines.push("<!-- Please describe what looked wrong or incomplete. -->");
  lines.push("");
  lines.push("## Attach full memory (optional)");
  lines.push("");
  lines.push(
    "Click **Export memory** to save `formal-ai-memory.lino`, redact it, and attach it (as a `.zip` if needed). See the [upload-memory guide](https://github.com/link-assistant/formal-ai/blob/main/docs/upload-memory.md).",
  );
  lines.push("");

  return lines.join("\n");
}

function createIssueUrl(context) {
  return fitIssueUrl(context, (effectiveContext) => createIssueReportBody(effectiveContext));
}

function shouldOfferMessageReport(message) {
  return message?.role === "assistant" && message.intent === "unknown";
}

// Issue #180: format the unified link-notation projection for an HTTP
// exchange so the user can see the formalization step alongside the raw
// request and response in diagnostics mode.
function formatHttpExchangeAsLinks(exchange) {
  if (!exchange || typeof exchange !== "object") return "";
  const lines = [];
  const id = exchange.id || `http:${exchange.method || "GET"}:${exchange.url || ""}`;
  lines.push(`(${id}: kind http_exchange)`);
  if (exchange.provider) lines.push(`(${id}: provider ${exchange.provider})`);
  if (exchange.phase) lines.push(`(${id}: phase ${exchange.phase})`);
  if (exchange.method) lines.push(`(${id}: method ${exchange.method})`);
  if (exchange.url) lines.push(`(${id}: url ${exchange.url})`);
  if (typeof exchange.status === "number") {
    lines.push(`(${id}: status ${exchange.status})`);
  }
  if (typeof exchange.elapsedMs === "number") {
    lines.push(`(${id}: elapsed_ms ${exchange.elapsedMs})`);
  }
  if (typeof exchange.responseBytes === "number") {
    lines.push(`(${id}: response_bytes ${exchange.responseBytes})`);
  }
  if (exchange.error) {
    const safeError = String(exchange.error).replace(/[()]/g, " ");
    lines.push(`(${id}: error ${safeError})`);
  }
  return lines.join("\n");
}

// Issue #180: render the worker's per-provider summary and the raw HTTP
// exchange list (request URL, status, elapsed time, response snippet,
// unified Links Notation projection) for each search-providing message.
function DiagnosticsHttpPanel({ providers, exchanges, t }) {
  if (
    (!Array.isArray(providers) || providers.length === 0) &&
    (!Array.isArray(exchanges) || exchanges.length === 0)
  ) {
    return null;
  }
  const safeExchanges = Array.isArray(exchanges) ? exchanges : [];
  return h(
    "div",
    {
      className: "diagnostics-http",
      "data-testid": "diagnostics-http",
    },
    Array.isArray(providers) && providers.length > 0
      ? h(
          "div",
          { className: "diagnostics-http-section" },
          h(
            "strong",
            { className: "diagnostics-section-label" },
            t("message.diagnosticsProviders"),
          ),
          h(
            "ul",
            { className: "diagnostics-http-provider-list" },
            providers.map((entry, index) =>
              h(
                "li",
                {
                  key: `${entry.id || "provider"}-${index}`,
                  className: `diagnostics-http-provider ${entry.ok ? "is-ok" : "is-error"}`,
                  "data-testid": "diagnostics-http-provider",
                },
                t("message.diagnosticsProviderRow", {
                  label: entry.label || entry.id || "(provider)",
                  status: entry.ok
                    ? t("message.diagnosticsProviderOk")
                    : `${t("message.diagnosticsProviderError")}: ${entry.error || "(unknown)"}`,
                  count: typeof entry.count === "number" ? entry.count : 0,
                  elapsed: typeof entry.elapsedMs === "number" ? entry.elapsedMs : 0,
                }),
              ),
            ),
          ),
        )
      : null,
    h(
      "div",
      { className: "diagnostics-http-section" },
      h(
        "strong",
        { className: "diagnostics-section-label" },
        t("message.diagnosticsHttp"),
      ),
      safeExchanges.length === 0
        ? h(
            "p",
            { className: "diagnostics-http-empty" },
            t("message.diagnosticsHttpEmpty"),
          )
        : h(
            "ol",
            { className: "diagnostics-http-list" },
            safeExchanges.map((exchange, index) =>
              h(
                "li",
                {
                  key: `${exchange.id || index}`,
                  className: "diagnostics-http-item",
                },
                h(
                  "details",
                  {
                    className: "diagnostics-detail",
                    "data-testid": "diagnostics-http-exchange",
                  },
                  h(
                    "summary",
                    null,
                    h(
                      "span",
                      { className: "diagnostics-step-name" },
                      `${exchange.method || "GET"} ${exchange.provider ? `[${exchange.provider}] ` : ""}`,
                    ),
                    h(
                      "span",
                      { className: "diagnostics-step-summary" },
                      exchange.url || "(no url)",
                    ),
                    h(
                      "span",
                      { className: "diagnostics-http-status" },
                      t("message.diagnosticsHttpStatus", {
                        status: typeof exchange.status === "number" ? exchange.status : "—",
                        elapsed: typeof exchange.elapsedMs === "number" ? exchange.elapsedMs : 0,
                        bytes: typeof exchange.responseBytes === "number" ? exchange.responseBytes : 0,
                      }),
                    ),
                  ),
                  h(
                    "div",
                    { className: "diagnostics-detail-body" },
                    h(
                      "div",
                      { className: "diagnostics-tool-section" },
                      h(
                        "span",
                        { className: "diagnostics-section-label" },
                        t("message.diagnosticsHttpRequest"),
                      ),
                      h(
                        "pre",
                        { className: "diagnostics-payload" },
                        formatDiagnosticPayload({
                          method: exchange.method || "GET",
                          url: exchange.url || "",
                          headers: exchange.requestHeaders || {},
                          body: exchange.requestBody || null,
                          provider: exchange.provider || "",
                          phase: exchange.phase || "",
                        }),
                      ),
                    ),
                    h(
                      "div",
                      { className: "diagnostics-tool-section" },
                      h(
                        "span",
                        { className: "diagnostics-section-label" },
                        t("message.diagnosticsHttpResponse"),
                      ),
                      h(
                        "pre",
                        { className: "diagnostics-payload" },
                        formatDiagnosticPayload({
                          status: exchange.status ?? null,
                          ok: !!exchange.ok,
                          elapsedMs: exchange.elapsedMs ?? null,
                          responseBytes: exchange.responseBytes ?? null,
                          finalUrl: exchange.finalUrl || "",
                          contentType: exchange.contentType || "",
                          responseSnippet: exchange.responseSnippet || "",
                          error: exchange.error || "",
                        }),
                      ),
                    ),
                    h(
                      "div",
                      { className: "diagnostics-tool-section" },
                      h(
                        "span",
                        { className: "diagnostics-section-label" },
                        t("message.diagnosticsHttpUnified"),
                      ),
                      h(
                        "pre",
                        { className: "diagnostics-payload diagnostics-http-links" },
                        formatHttpExchangeAsLinks(exchange),
                      ),
                    ),
                  ),
                ),
              ),
            ),
          ),
    ),
  );
}

function Message({ message, diagnosticsMode, reportIssueUrl, t }) {
  const evidence = diagnosticsMode ? (message.evidence ?? []) : [];
  const thinkingSteps = diagnosticsMode ? (message.thinkingSteps ?? []) : [];
  const diagnosticsSteps = diagnosticsMode
    ? (message.diagnosticsSteps ?? [])
    : [];
  const diagnosticsToolCalls = diagnosticsMode
    ? (message.diagnosticsToolCalls ?? [])
    : [];
  // Issue #180: surface raw HTTP request/response bodies and the per-provider
  // outcomes inside the diagnostics panel so the user can audit every network
  // call the worker performed on their behalf.
  const diagnosticsPayload = diagnosticsMode ? message.diagnostics : null;
  const diagnosticsProviders = Array.isArray(diagnosticsPayload?.providers)
    ? diagnosticsPayload.providers
    : [];
  const diagnosticsHttp = Array.isArray(diagnosticsPayload?.httpExchanges)
    ? diagnosticsPayload.httpExchanges
    : [];
  const reportLabel =
    message.intent === "unknown"
      ? t("buttons.reportMissingRule")
      : t("buttons.reportIssue");
  const [iframeFullscreen, setIframeFullscreen] = useState(false);
  // Issue #330: progressive syntax highlighting + per-code-block copy buttons.
  const markdownRef = useRef(null);
  const [markdownCopied, setMarkdownCopied] = useState(false);

  useEffect(() => {
    if (!iframeFullscreen) {
      return undefined;
    }
    const handleKeyDown = (event) => {
      if (event.key === "Escape") {
        setIframeFullscreen(false);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [iframeFullscreen]);

  // Highlight and wrap code fences after marked renders the message body. The
  // enhancement is idempotent, so re-running it on content changes is safe.
  useEffect(() => {
    enhanceCodeBlocks(markdownRef.current, t);
  }, [message.content, t]);

  const handleCopyMarkdown = useCallback(async () => {
    const ok = await copyTextToClipboard(message.content);
    if (ok) {
      setMarkdownCopied(true);
      setTimeout(() => setMarkdownCopied(false), 1600);
    }
  }, [message.content]);

  return h(
    "article",
    {
      className: `message ${message.role}`,
      "data-testid": "chat-message",
      "data-demo-label": message.demoLabel || null,
    },
    h("div", { className: "avatar", "aria-hidden": "true" }, message.role === "user" ? "Y" : "FA"),
    h(
      "div",
      { className: "message-body" },
      h(
        "div",
        { className: "message-meta" },
        h(
          "strong",
          null,
          message.role === "user" ? t("message.author.user") : message.author,
        ),
        h("time", null, message.sentAt),
        diagnosticsMode && message.intent
          ? h("span", { className: "intent" }, `intent:${message.intent}`)
          : null,
        h(
          "button",
          {
            type: "button",
            className: `message-copy-button${markdownCopied ? " is-copied" : ""}`,
            "data-testid": "copy-markdown-button",
            "data-copied": markdownCopied ? "true" : null,
            onClick: handleCopyMarkdown,
            "aria-label": t("message.copyMarkdownTitle"),
            title: t("message.copyMarkdownTitle"),
          },
          h(
            "span",
            { className: "copy-button-label" },
            markdownCopied ? t("message.copyMarkdownDone") : t("message.copyMarkdown"),
          ),
        ),
      ),
      h("div", {
        ref: markdownRef,
        className: "markdown-body",
        dangerouslySetInnerHTML: markdownHtml(message.content),
      }),
      message.iframeUrl
        ? h(
            "div",
            {
              className: `fetch-iframe-container${iframeFullscreen ? " is-fullscreen" : ""}`,
              "data-testid": "fetch-iframe-container",
            },
            h(
              "div",
              { className: "fetch-iframe-header" },
              h("span", { className: "fetch-iframe-url" }, message.iframeUrl),
              h(
                "div",
                { className: "fetch-iframe-actions" },
                h(
                  "a",
                  {
                    href: message.iframeUrl,
                    target: "_blank",
                    rel: "noopener noreferrer",
                    className: "fetch-iframe-open fetch-iframe-control",
                    "aria-label": t("fetch.openInNewTab"),
                    title: t("fetch.openInNewTab"),
                  },
                  "↗",
                ),
                h(
                  "button",
                  {
                    type: "button",
                    className: "fetch-iframe-toggle fetch-iframe-control",
                    onClick: () => setIframeFullscreen((prev) => !prev),
                    "aria-label": iframeFullscreen ? t("fetch.minimize") : t("fetch.fullscreen"),
                    "aria-pressed": iframeFullscreen ? "true" : "false",
                    title: iframeFullscreen ? t("fetch.minimize") : t("fetch.fullscreen"),
                  },
                  iframeFullscreen ? "⤡" : "⛶",
                ),
              ),
            ),
            h("iframe", {
              className: "fetch-iframe",
              src: message.iframeUrl,
              title: t("fetch.frameTitle", { url: message.iframeUrl }),
              sandbox: "allow-scripts allow-same-origin allow-forms allow-popups",
              loading: "lazy",
              "data-testid": "fetch-iframe",
            }),
          )
        : null,
      evidence.length
        ? h(
            "div",
            { className: "evidence-list" },
            evidence.map((item) => h("span", { key: item }, item)),
          )
        : null,
      thinkingSteps.length
        ? h(
            "div",
            { className: "thinking-steps" },
            h("strong", null, t("message.thinking")),
            h(
              "ol",
              null,
              thinkingSteps.map((item) => h("li", { key: item }, item)),
            ),
          )
        : null,
      diagnosticsSteps.length
        ? h(
            "div",
            {
              className: "diagnostics-steps",
              "data-testid": "diagnostics-steps",
            },
            h("strong", null, t("message.diagnosticsSteps")),
            h(
              "ol",
              { className: "diagnostics-step-list" },
              diagnosticsSteps.map((entry, index) =>
                h(
                  "li",
                  { key: `${entry.step}-${index}`, className: "diagnostics-step" },
                  h(
                    "details",
                    {
                      className: "diagnostics-detail",
                      "data-testid": "diagnostics-step",
                      "data-step": entry.step,
                    },
                    h(
                      "summary",
                      null,
                      h(
                        "span",
                        { className: "diagnostics-step-name" },
                        entry.formalization
                          ? t("message.formalization")
                          : entry.step,
                      ),
                      h(
                        "span",
                        { className: "diagnostics-step-summary" },
                        entry.formalization
                          ? truncateDiagnosticDetail(entry.formalization.tuple)
                          : truncateDiagnosticDetail(entry.detail),
                      ),
                    ),
                    h(
                      "div",
                      { className: "diagnostics-detail-body" },
                      entry.formalization
                        ? h(FormalizationView, {
                            formalization: entry.formalization,
                            t,
                          })
                        : h(
                            "pre",
                            { className: "diagnostics-payload" },
                            formatDiagnosticPayload(entry.detail),
                          ),
                    ),
                  ),
                ),
              ),
            ),
          )
        : null,
      diagnosticsToolCalls.length
        ? h(
            "div",
            {
              className: "diagnostics-tools",
              "data-testid": "diagnostics-tools",
            },
            h("strong", null, t("message.diagnosticsTools")),
            h(
              "ol",
              { className: "diagnostics-tool-list" },
              diagnosticsToolCalls.map((call, index) =>
                h(
                  "li",
                  {
                    key: `${call.tool || "tool"}-${index}`,
                    className: "diagnostics-tool",
                  },
                  h(
                    "details",
                    {
                      className: "diagnostics-detail",
                      "data-testid": "diagnostics-tool",
                    },
                    h(
                      "summary",
                      null,
                      h(
                        "span",
                        { className: "diagnostics-tool-name" },
                        call.tool || "(tool)",
                      ),
                      h(
                        "span",
                        { className: "diagnostics-tool-summary" },
                        summarizeToolCall(call),
                      ),
                    ),
                    h(
                      "div",
                      { className: "diagnostics-detail-body" },
                      h(
                        "div",
                        { className: "diagnostics-tool-section" },
                        h(
                          "span",
                          { className: "diagnostics-section-label" },
                          t("message.toolInputs"),
                        ),
                        h(
                          "pre",
                          { className: "diagnostics-payload" },
                          formatDiagnosticPayload(call.inputs),
                        ),
                      ),
                      h(
                        "div",
                        { className: "diagnostics-tool-section" },
                        h(
                          "span",
                          { className: "diagnostics-section-label" },
                          t("message.toolOutputs"),
                        ),
                        h(
                          "pre",
                          { className: "diagnostics-payload" },
                          formatDiagnosticPayload(call.outputs),
                        ),
                      ),
                      Array.isArray(call.steps) && call.steps.length > 0
                        ? h(
                            "div",
                            { className: "diagnostics-tool-section" },
                            h(
                              "span",
                              { className: "diagnostics-section-label" },
                              t("message.toolReasoning"),
                            ),
                            h(
                              "ol",
                              { className: "diagnostics-tool-reasoning" },
                              call.steps.map((s, j) =>
                                h(
                                  "li",
                                  { key: `${call.tool}-step-${j}` },
                                  `${s.step}: ${s.detail}`,
                                ),
                              ),
                            ),
                          )
                        : null,
                    ),
                  ),
                ),
              ),
            ),
          )
        : null,
      diagnosticsPayload
        ? h(DiagnosticsHttpPanel, {
            providers: diagnosticsProviders,
            exchanges: diagnosticsHttp,
            t,
          })
        : null,
      reportIssueUrl
        ? h(
            "div",
            { className: "message-actions" },
            h(
              "a",
              {
                href: reportIssueUrl,
                target: "_blank",
                rel: "noopener noreferrer",
              },
              reportLabel,
            ),
          )
        : null,
    ),
  );
}

// Issue #27: a VS Code-style collapsible sidebar section. When `collapsed` is
// false the section participates in the equal-share flex layout and scrolls
// independently; when true only the header remains visible.
function CollapsibleSection({
  title,
  collapsed,
  onToggle,
  testId,
  className = "",
  bodyClassName = "",
  children,
}) {
  const sectionClassName = [
    "sidebar-section",
    collapsed ? "is-collapsed" : "is-expanded",
    className,
  ]
    .filter(Boolean)
    .join(" ");
  const sectionBodyClassName = ["sidebar-section-body", bodyClassName]
    .filter(Boolean)
    .join(" ");
  return h(
    "section",
    {
      className: sectionClassName,
      "data-testid": testId,
      "data-collapsed": collapsed ? "true" : "false",
    },
    h(
      "button",
      {
        type: "button",
        className: "sidebar-section-header",
        "aria-expanded": collapsed ? "false" : "true",
        onClick: onToggle,
      },
      h("span", { className: "sidebar-section-caret", "aria-hidden": "true" }, collapsed ? "▶" : "▼"),
      h("h2", null, title),
    ),
    collapsed
      ? null
      : h("div", { className: sectionBodyClassName }, children),
  );
}

function MenuGlyph({ open }) {
  return h("span", {
    className: `btn-icon menu-icon ${open ? "menu-icon-close" : "menu-icon-hamburger"}`,
    "aria-hidden": "true",
  });
}

function SidebarToggleGlyph({ collapsed }) {
  return h(
    "span",
    {
      className: `btn-icon sidebar-toggle-icon ${collapsed ? "sidebar-toggle-icon-expand" : "sidebar-toggle-icon-collapse"}`,
      "aria-hidden": "true",
    },
    collapsed ? "▶" : "◀",
  );
}

function App() {
  const workerRef = useRef(null);
  const pendingResponses = useRef(new Map());
  const transcriptEndRef = useRef(null);
  const importInputRef = useRef(null);
  const attachmentInputRef = useRef(null);
  const composerInputRef = useRef(null);
  const [messages, setMessages] = useState([]);
  const [prompt, setPrompt] = useState("");
  const [pending, setPending] = useState(false);
  const [workerState, setWorkerState] = useState("wasm worker");
  const [memoryStatus, setMemoryStatus] = useState("");
  const [composerMenuOpen, setComposerMenuOpen] = useState(false);
  const [attachments, setAttachments] = useState([]);
  const [seed, setSeed] = useState({
    raw: {},
    tools: [],
    concepts: [],
    responses: {},
  });
  const initialPreferences = useRef(loadPreferences());
  const [uiLanguagePreference, setUiLanguagePreference] = useState(
    normalizeUiLanguagePreference(initialPreferences.current.uiLanguage),
  );
  // Issue #324: which language drives responses, and the pinned language used
  // when the mode is "preferred".
  const [responseLanguage, setResponseLanguage] = useState(
    normalizeResponseLanguageMode(initialPreferences.current.responseLanguage),
  );
  const [preferredLanguage, setPreferredLanguage] = useState(
    normalizePreferredLanguage(initialPreferences.current.preferredLanguage),
  );
  const [i18nRuntimeTick, setI18nRuntimeTick] = useState(0);
  const uiLanguage = detectUiLanguage(uiLanguagePreference);
  const t = useCallback(
    (key, params) => translateUi(key, uiLanguage, params),
    [uiLanguage, i18nRuntimeTick],
  );
  const [demoMode, setDemoMode] = useState(initialPreferences.current.demoMode);
  const [demoPhase, setDemoPhase] = useState("manual");
  const [demoCountdown, setDemoCountdown] = useState(null);
  const [diagnosticsMode, setDiagnosticsMode] = useState(
    initialPreferences.current.diagnosticsMode,
  );
  const [contextPanelWidth, setContextPanelWidth] = useState(
    normalizeContextPanelWidth(initialPreferences.current.contextPanelWidth),
  );
  // Issue #27: sidebar collapse/expand state per section.
  const [sidebarMenuCollapsed, setSidebarMenuCollapsed] = useState(
    initialPreferences.current.sidebarMenuCollapsed,
  );
  const [sidebarDesktopCollapsed, setSidebarDesktopCollapsed] = useState(false);
  const [sidebarPromptsCollapsed, setSidebarPromptsCollapsed] = useState(
    initialPreferences.current.sidebarPromptsCollapsed,
  );
  const [sidebarToolsCollapsed, setSidebarToolsCollapsed] = useState(
    initialPreferences.current.sidebarToolsCollapsed,
  );
  const [sidebarTraceCollapsed, setSidebarTraceCollapsed] = useState(
    initialPreferences.current.sidebarTraceCollapsed,
  );
  const [sidebarConversationsCollapsed, setSidebarConversationsCollapsed] = useState(
    initialPreferences.current.sidebarConversationsCollapsed,
  );
  const [sidebarSettingsCollapsed, setSidebarSettingsCollapsed] = useState(
    initialPreferences.current.sidebarSettingsCollapsed,
  );
  // Issue #153: persistent desktop sidebar collapse — separate from the
  // transient `mobileMenuOpen` drawer so wide-screen layouts can dedicate the
  // viewport to chat without losing the user's accordion state.
  const [sidebarCollapsed, setSidebarCollapsed] = useState(
    Boolean(initialPreferences.current.sidebarCollapsed),
  );
  const [showDeletedConversations, setShowDeletedConversations] = useState(
    Boolean(initialPreferences.current.showDeletedConversations),
  );
  const showDeletedConversationsRef = useRef(showDeletedConversations);
  const [greetingVariations, setGreetingVariations] = useState(
    initialPreferences.current.greetingVariations,
  );
  const [guessProbability, setGuessProbability] = useState(
    normalizeSliderPreference(
      initialPreferences.current.guessProbability,
      PREFERENCE_DEFAULTS.guessProbability,
    ),
  );
  const [temperature, setTemperature] = useState(
    normalizeSliderPreference(
      initialPreferences.current.temperature,
      PREFERENCE_DEFAULTS.temperature,
    ),
  );
  const [followUpProbability, setFollowUpProbability] = useState(
    normalizeSliderPreference(
      initialPreferences.current.followUpProbability,
      PREFERENCE_DEFAULTS.followUpProbability,
    ),
  );
  const [definitionFusion, setDefinitionFusion] = useState(
    normalizeDefinitionFusion(initialPreferences.current.definitionFusion),
  );
  const [blueprintComposition, setBlueprintComposition] = useState(
    normalizeBlueprintComposition(initialPreferences.current.blueprintComposition),
  );
  const [experimentalOcr, setExperimentalOcr] = useState(
    Boolean(initialPreferences.current.experimentalOcr),
  );
  const [associativeProjectPromotion, setAssociativeProjectPromotion] = useState(
    initialPreferences.current.associativeProjectPromotion !== false,
  );
  const [themePreference, setThemePreference] = useState(
    normalizeThemePreference(initialPreferences.current.theme),
  );
  const [uiSkin, setUiSkin] = useState(
    normalizeUiSkin(initialPreferences.current.uiSkin),
  );
  const [chatStyle, setChatStyle] = useState(
    normalizeChatStyle(initialPreferences.current.chatStyle),
  );
  const [composerStyle, setComposerStyle] = useState(
    normalizeComposerStyle(initialPreferences.current.composerStyle),
  );
  const [composerAction, setComposerAction] = useState(
    normalizeComposerAction(initialPreferences.current.composerAction),
  );
  const [locationPreference, setLocationPreference] = useState(
    String(initialPreferences.current.location || ""),
  );
  const [assistantName, setAssistantName] = useState(
    normalizeAssistantName(initialPreferences.current.assistantName),
  );
  const [desktopStatus, setDesktopStatus] = useState(null);
  // Issue #27: agent mode runs the user's prompt as a multi-step plan instead
  // of a single Q&A. Persisted across reloads via preferences.
  const [agentMode, setAgentMode] = useState(
    initialPreferences.current.agentMode,
  );
  // Issue #27: a mobile-friendly slide-out menu that hosts the entire sidebar
  // plus the topbar action buttons. On wide screens the menu is hidden via CSS.
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const [colorSchemeTick, setColorSchemeTick] = useState(0);
  // Issue #27: conversations. `currentConversationId` is the thread the user is
  // typing in right now; on first user message the demo lazily mints a new id
  // if none is set. `conversations` is the sidebar-visible list of all known
  // threads, derived from the append-only event log and refreshed after every
  // turn.
  const [currentConversationId, setCurrentConversationId] = useState(
    initialPreferences.current.currentConversationId || "",
  );
  const [conversations, setConversations] = useState([]);
  // Issue #386: id of the conversation whose "copy as Markdown" button last
  // succeeded, so the entry can flash a short confirmation label.
  const [copiedConversationId, setCopiedConversationId] = useState("");
  const currentConversationRef = useRef(currentConversationId);
  const conversationTitlesRef = useRef(new Map());
  const conversationEventsRef = useRef([]);

  useEffect(() => {
    if (typeof document === "undefined") return;
    document.documentElement.lang = uiLanguage;
    document.documentElement.dir = "ltr";
  }, [uiLanguage]);

  useEffect(() => {
    if (typeof document === "undefined") return;
    if (themePreference === "dark") {
      document.documentElement.setAttribute("data-theme", "dark");
    } else if (themePreference === "light") {
      document.documentElement.setAttribute("data-theme", "light");
    } else {
      document.documentElement.removeAttribute("data-theme");
    }
  }, [themePreference]);

  useEffect(() => {
    if (typeof window === "undefined" || typeof document === "undefined") {
      return undefined;
    }
    const root = document.documentElement;
    const updateViewport = () => {
      const visualViewport = window.visualViewport;
      const width =
        visualViewport && visualViewport.width
          ? visualViewport.width
          : window.innerWidth;
      const height =
        visualViewport && visualViewport.height
          ? visualViewport.height
          : window.innerHeight;
      const offsetLeft =
        visualViewport && visualViewport.offsetLeft
          ? visualViewport.offsetLeft
          : 0;
      const offsetTop =
        visualViewport && visualViewport.offsetTop
          ? visualViewport.offsetTop
          : 0;
      root.style.setProperty(
        "--formal-ai-viewport-width",
        `${Math.round(width)}px`,
      );
      root.style.setProperty(
        "--formal-ai-viewport-height",
        `${Math.round(height)}px`,
      );
      root.style.setProperty(
        "--formal-ai-viewport-offset-left",
        `${Math.round(offsetLeft)}px`,
      );
      root.style.setProperty(
        "--formal-ai-viewport-offset-top",
        `${Math.round(offsetTop)}px`,
      );
    };
    updateViewport();
    window.addEventListener("resize", updateViewport);
    window.addEventListener("orientationchange", updateViewport);
    if (window.visualViewport) {
      window.visualViewport.addEventListener("resize", updateViewport);
      window.visualViewport.addEventListener("scroll", updateViewport);
    }
    return () => {
      window.removeEventListener("resize", updateViewport);
      window.removeEventListener("orientationchange", updateViewport);
      if (window.visualViewport) {
        window.visualViewport.removeEventListener("resize", updateViewport);
        window.visualViewport.removeEventListener("scroll", updateViewport);
      }
    };
  }, []);

  useEffect(() => {
    if (typeof window === "undefined") return undefined;
    const clampContextPanel = () => {
      setContextPanelWidth((width) => normalizeContextPanelWidth(width));
    };
    window.addEventListener("resize", clampContextPanel);
    window.addEventListener("orientationchange", clampContextPanel);
    if (window.visualViewport) {
      window.visualViewport.addEventListener("resize", clampContextPanel);
    }
    return () => {
      window.removeEventListener("resize", clampContextPanel);
      window.removeEventListener("orientationchange", clampContextPanel);
      if (window.visualViewport) {
        window.visualViewport.removeEventListener("resize", clampContextPanel);
      }
    };
  }, []);

  useEffect(() => {
    if (typeof window === "undefined") return undefined;
    let cancelled = false;
    const update = () => {
      if (!cancelled) {
        setI18nRuntimeTick((value) => value + 1);
      }
    };
    window.addEventListener("formal-ai:i18n-ready", update);
    const api = i18nApi();
    if (api && api.ready && typeof api.ready.then === "function") {
      api.ready.then(update).catch(() => null);
    }
    return () => {
      cancelled = true;
      window.removeEventListener("formal-ai:i18n-ready", update);
    };
  }, []);

  useEffect(() => {
    if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
      return undefined;
    }
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const update = () => setColorSchemeTick((value) => value + 1);
    if (typeof media.addEventListener === "function") {
      media.addEventListener("change", update);
      return () => media.removeEventListener("change", update);
    }
    if (typeof media.addListener === "function") {
      media.addListener(update);
      return () => media.removeListener(update);
    }
    return undefined;
  }, []);

  useEffect(() => {
    currentConversationRef.current = currentConversationId;
  }, [currentConversationId]);

  useEffect(() => {
    const bridge = desktopBridge();
    if (!bridge || typeof bridge.getStatus !== "function") {
      return undefined;
    }
    let cancelled = false;
    bridge
      .getStatus()
      .then((status) => {
        if (!cancelled) {
          setDesktopStatus(normalizeDesktopStatus(status));
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setDesktopStatus(
            normalizeDesktopStatus({
              shell: "Electron",
              apiError: error && error.message ? error.message : String(error),
              apiReady: false,
            }),
          );
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    // Push the explicit-permission state to the local tool router whenever the
    // agent-permission toggle changes (default-deny until opted in).
    syncDesktopToolGrants(desktopBridge(), agentMode);
  }, [agentMode, desktopStatus]);

  useEffect(() => {
    // R5d: expose a thin hook so the local tool router (and the desktop e2e
    // suite) can route a tool call through the bridge to the local process /
    // Docker sandbox. The gate still applies — denied calls return a refusal.
    if (typeof window === "undefined") {
      return undefined;
    }
    window.formalAiDesktopToolCall = (tool, input) =>
      requestDesktopToolCall(desktopBridge(), tool, input);
    return () => {
      delete window.formalAiDesktopToolCall;
    };
  }, []);

  useEffect(() => {
    showDeletedConversationsRef.current = showDeletedConversations;
  }, [showDeletedConversations]);

  const userContext = useMemo(
    () =>
      collectUserContext({
        uiLanguage,
        uiLanguagePreference,
        themePreference,
        uiSkin,
        chatStyle,
        composerStyle,
        composerAction,
        locationPreference,
        assistantName,
        guessProbability,
        temperature,
        followUpProbability,
        definitionFusion,
        experimentalOcr,
      }),
    [
      uiLanguage,
      uiLanguagePreference,
      themePreference,
      uiSkin,
      chatStyle,
      composerStyle,
      composerAction,
      locationPreference,
      assistantName,
      guessProbability,
      temperature,
      followUpProbability,
      definitionFusion,
      experimentalOcr,
      colorSchemeTick,
    ],
  );
  const userContextRef = useRef(userContext);
  useEffect(() => {
    userContextRef.current = userContext;
  }, [userContext]);

  useEffect(() => {
    if (typeof window === "undefined" || !window.FormalAiSeed) return;
    let cancelled = false;
    window.FormalAiSeed.loadAll().then((loaded) => {
      if (cancelled) return;
      setSeed(loaded);
    });
    return () => {
      cancelled = true;
    };
  }, []);

  // Issue #27: on mount, hydrate the conversation list from the append-only
  // event log and restore the active thread's messages. Operates purely as a
  // projection — no events are mutated.
  const refreshConversations = useCallback(async (showDeletedOverride) => {
    if (typeof window === "undefined" || !window.FormalAiMemory) {
      return [];
    }
    try {
      const shouldShowDeleted =
        typeof showDeletedOverride === "boolean"
          ? showDeletedOverride
          : showDeletedConversationsRef.current;
      const events = await window.FormalAiMemory.listEvents();
      conversationEventsRef.current = events;
      const list = groupConversations(events, {
        showDeleted: shouldShowDeleted,
      });
      list.forEach((entry) => {
        if (entry.title) {
          conversationTitlesRef.current.set(entry.id, entry.title);
        }
      });
      setConversations(list);
      return events;
    } catch (_error) {
      conversationEventsRef.current = [];
      return [];
    }
  }, []);

  useEffect(() => {
    let cancelled = false;
    refreshConversations().then((events) => {
      if (cancelled || !Array.isArray(events) || events.length === 0) return;
      const initialId = initialPreferences.current.currentConversationId;
      if (!initialId) return;
      const restored = messagesForConversation(events, initialId);
      if (restored.length > 0) {
        setMessages(restored);
        setDemoMode(false);
      }
    });
    return () => {
      cancelled = true;
    };
  }, [refreshConversations]);

  const handleExportMemory = useCallback(async () => {
    if (typeof window === "undefined" || !window.FormalAiMemory) {
      setMemoryStatus(t("status.memoryUnavailable"));
      return;
    }
    try {
      await waitForMemoryWrites();
      const events = await window.FormalAiMemory.listEvents();
      const preferences = loadPreferences();
      const text = window.FormalAiMemory.exportFullMemory({
        seed,
        events,
        preferences,
        info: {
          version: APP_VERSION,
          url: window.location.href,
          userAgent: navigator.userAgent,
          workerState,
          mode: demoMode ? "demo" : "manual",
          ...userContext,
        },
      });
      downloadTextFile(MEMORY_EXPORT_FILENAME, text);
      const seedFileCount = seed && seed.raw ? Object.keys(seed.raw).length : 0;
      setMemoryStatus(
        t("status.memoryExported", {
          events: events.length,
          seedFiles: seedFileCount,
        }),
      );
    } catch (_error) {
      setMemoryStatus(t("status.exportFailed"));
    }
  }, [seed, workerState, demoMode, userContext, t]);

  // R5c (D1): reconcile the browser (IndexedDB) memory log with the native store
  // via the desktop bridge. Pushes the current `demo_memory` event log to the
  // local server and folds any pulled delta back into IndexedDB. Best-effort.
  const syncDesktopMemoryNow = useCallback(async () => {
    const bridge = desktopBridge();
    if (!bridge || typeof bridge.syncMemory !== "function") {
      return null;
    }
    if (typeof window === "undefined" || !window.FormalAiMemory) {
      return null;
    }
    try {
      await waitForMemoryWrites();
      const events = await window.FormalAiMemory.listEvents();
      const lino = window.FormalAiMemory.exportLinksNotation(events);
      const result = await syncDesktopMemory(bridge, lino);
      const delta = result && result.pulled ? result.pulled.delta : "";
      if (delta && delta.trim()) {
        const imported = window.FormalAiMemory.importFullMemory(delta);
        if (imported && Array.isArray(imported.events) && imported.events.length > 0) {
          await window.FormalAiMemory.importEvents(imported.events);
        }
      }
      return result;
    } catch (_error) {
      return null;
    }
  }, []);

  useEffect(() => {
    // R5c: keep the native store in step with the browser log after each turn
    // while the local server is the active surface. Declared after
    // `syncDesktopMemoryNow` so the dependency reference is initialized.
    if (!desktopStatus || !desktopStatus.apiReady) {
      return;
    }
    syncDesktopMemoryNow();
  }, [messages, desktopStatus, syncDesktopMemoryNow]);

  const handleImportMemory = useCallback(async (event) => {
    const file = event.target.files && event.target.files[0];
    event.target.value = "";
    if (!file || typeof window === "undefined" || !window.FormalAiMemory) {
      return;
    }
    try {
      const text = await file.text();
      const imported = window.FormalAiMemory.importFullMemory(text);
      const inserted = await window.FormalAiMemory.importEvents(imported.events);
      const current = {
        agentInfo: seed && seed.agentInfo ? seed.agentInfo : {},
        info: { version: APP_VERSION },
      };
      const suggestions = window.FormalAiMemory.suggestMigrations({
        imported,
        current,
      });
      const headline =
        imported.kind === "bundle"
          ? t("status.memoryImportedBundle", { inserted })
          : t("status.memoryImportedEvents", { inserted });
      if (suggestions.length > 0) {
        setMemoryStatus(
          t("status.migration", {
            headline,
            suggestions: suggestions.join(" / "),
          }),
        );
      } else {
        setMemoryStatus(headline);
      }
    } catch (_error) {
      setMemoryStatus(t("status.importFailed"));
    }
  }, [seed, t]);

  const triggerImportMemory = useCallback(() => {
    if (importInputRef.current) {
      importInputRef.current.click();
    }
  }, []);

  const confirmDangerousMemoryAction = useCallback(
    async (exportPrompt, confirmPrompt) => {
      if (typeof window === "undefined" || typeof window.confirm !== "function") {
        return true;
      }
      if (window.confirm(exportPrompt)) {
        await handleExportMemory();
        return false;
      }
      return window.confirm(confirmPrompt);
    },
    [handleExportMemory],
  );

  const handleResetMemory = useCallback(async () => {
    if (typeof window === "undefined" || !window.FormalAiMemory) {
      setMemoryStatus(t("status.memoryUnavailable"));
      return { cancelled: true, removed: 0 };
    }
    const proceed = await confirmDangerousMemoryAction(
      t("confirm.resetMemoryExportFirst"),
      t("confirm.resetMemory"),
    );
    if (!proceed) {
      return { cancelled: true, removed: 0 };
    }
    try {
      await waitForMemoryWrites();
      const removed = await window.FormalAiMemory.clearEvents();
      currentConversationRef.current = "";
      setCurrentConversationId("");
      setMessages([]);
      setPrompt("");
      setShowDeletedConversations(false);
      await refreshConversations(false);
      setMemoryStatus(t("status.memoryReset", { events: removed }));
      return { cancelled: false, removed };
    } catch (_error) {
      setMemoryStatus(t("status.memoryResetFailed"));
      return { cancelled: true, removed: 0 };
    }
  }, [confirmDangerousMemoryAction, refreshConversations, t]);

  const handlePurgeDeletedConversations = useCallback(async () => {
    if (typeof window === "undefined" || !window.FormalAiMemory) {
      setMemoryStatus(t("status.memoryUnavailable"));
      return;
    }
    const proceed = await confirmDangerousMemoryAction(
      t("confirm.purgeDeletedExportFirst"),
      t("confirm.purgeDeleted"),
    );
    if (!proceed) {
      return;
    }
    try {
      await waitForMemoryWrites();
      const events = await window.FormalAiMemory.listEvents();
      const deletedIds = new Set(
        groupConversations(events, { showDeleted: true }).map((entry) => entry.id),
      );
      const removed = await window.FormalAiMemory.purgeDeletedConversations();
      if (deletedIds.has(currentConversationRef.current)) {
        currentConversationRef.current = "";
        setCurrentConversationId("");
        setMessages([]);
        setPrompt("");
      }
      setShowDeletedConversations(true);
      await refreshConversations(true);
      setMemoryStatus(t("status.deletedConversationsPurged", { events: removed }));
    } catch (_error) {
      setMemoryStatus(t("status.memoryResetFailed"));
    }
  }, [confirmDangerousMemoryAction, refreshConversations, t]);

  const handlePurgeConversation = useCallback(
    async (entry) => {
      if (!entry || !entry.id || typeof window === "undefined" || !window.FormalAiMemory) {
        return;
      }
      const proceed = await confirmDangerousMemoryAction(
        t("confirm.deleteConversationPermanentExportFirst"),
        t("confirm.deleteConversationPermanent"),
      );
      if (!proceed) {
        return;
      }
      try {
        await waitForMemoryWrites();
        const removed = await window.FormalAiMemory.deleteEventsByConversationId(entry.id);
        if (entry.id === currentConversationRef.current) {
          currentConversationRef.current = "";
          setCurrentConversationId("");
          setMessages([]);
          setPrompt("");
        }
        setShowDeletedConversations(true);
        await refreshConversations(true);
        setMemoryStatus(t("status.conversationPurged", { events: removed }));
      } catch (_error) {
        setMemoryStatus(t("status.memoryResetFailed"));
      }
    },
    [confirmDangerousMemoryAction, refreshConversations, t],
  );

  const triggerAttachFiles = useCallback(() => {
    if (attachmentInputRef.current) {
      attachmentInputRef.current.click();
    }
    setComposerMenuOpen(false);
  }, []);

  const handleAttachFiles = useCallback((event) => {
    const files = Array.from(event.target.files || []);
    event.target.value = "";
    setAttachments(
      files.map((file) => ({
        id: `attachment-${Date.now()}-${Math.random().toString(16).slice(2)}`,
        sourceFile: file,
        name: file.name,
        size: file.size,
        type: file.type || "application/octet-stream",
        isImage: isImageAttachment(file),
      })),
    );
    setComposerMenuOpen(false);
  }, []);

  const prepareAttachmentsForSend = useCallback(
    async (items) => {
      const safe = Array.isArray(items) ? items : [];
      const prepared = [];
      for (const attachment of safe) {
        const next = {
          id: attachment.id,
          name: attachment.name,
          size: attachment.size,
          type: attachment.type || "application/octet-stream",
          isImage: Boolean(attachment.isImage),
        };
        if (experimentalOcr && next.isImage && attachment.sourceFile) {
          try {
            next.dataUrl = await readFileAsDataUrl(attachment.sourceFile);
            try {
              const ocr = await loadOcrBundle();
              const result = await ocr.recognizeImage(next.dataUrl, { language: "eng" });
              next.ocrText = result && result.text ? String(result.text).trim() : "";
              if (
                result &&
                typeof result.confidence === "number" &&
                Number.isFinite(result.confidence)
              ) {
                next.ocrConfidence = result.confidence;
              }
            } catch (error) {
              next.ocrError =
                error && error.message ? error.message : "OCR recognition failed";
            }
          } catch (error) {
            next.ocrError = error && error.message ? error.message : "File read failed";
          }
        }
        prepared.push(next);
      }
      return prepared;
    },
    [experimentalOcr],
  );

  const handleShowDeletedConversations = useCallback((event) => {
    const next = Boolean(event.target.checked);
    setShowDeletedConversations(next);
    refreshConversations(next);
  }, [refreshConversations]);

  const handleContextResizePointerDown = useCallback((event) => {
    if (event.button !== 0 || typeof window === "undefined") return;
    event.preventDefault();
    const startX = event.clientX;
    const startWidth = contextPanelWidth;
    const body = typeof document !== "undefined" ? document.body : null;
    const handlePointerMove = (moveEvent) => {
      const nextWidth = startWidth + moveEvent.clientX - startX;
      setContextPanelWidth(normalizeContextPanelWidth(nextWidth));
    };
    const stopResize = () => {
      if (body) {
        body.classList.remove("is-resizing-context");
      }
      window.removeEventListener("pointermove", handlePointerMove);
      window.removeEventListener("pointerup", stopResize);
      window.removeEventListener("pointercancel", stopResize);
    };
    if (body) {
      body.classList.add("is-resizing-context");
    }
    window.addEventListener("pointermove", handlePointerMove);
    window.addEventListener("pointerup", stopResize);
    window.addEventListener("pointercancel", stopResize);
  }, [contextPanelWidth]);

  const handleContextResizeKeyDown = useCallback((event) => {
    const step = event.shiftKey ? 40 : 16;
    let nextWidth = null;
    if (event.key === "ArrowLeft") {
      nextWidth = contextPanelWidth - step;
    } else if (event.key === "ArrowRight") {
      nextWidth = contextPanelWidth + step;
    } else if (event.key === "Home") {
      nextWidth = CONTEXT_PANEL_MIN_WIDTH;
    } else if (event.key === "End") {
      nextWidth = contextPanelMaxWidth();
    }
    if (nextWidth === null) return;
    event.preventDefault();
    setContextPanelWidth(normalizeContextPanelWidth(nextWidth));
  }, [contextPanelWidth]);

  const handleDeleteConversation = useCallback(async (entry) => {
    if (!entry || !entry.id) return;
    await recordMemoryEvent({
      kind: "conversation_deleted",
      role: "system",
      content: `Conversation deleted: ${entry.title || entry.id}`,
      sentAt: new Date().toISOString(),
      conversationId: entry.id,
      conversationTitle: entry.title || "",
    });
    if (entry.id === currentConversationRef.current) {
      currentConversationRef.current = "";
      setCurrentConversationId("");
      setMessages([]);
      setPrompt("");
      setDemoMode(false);
    }
    setShowDeletedConversations(false);
    await refreshConversations(false);
  }, [refreshConversations]);

  // Issue #386: copy a whole conversation to the clipboard as Markdown. When
  // diagnostics mode is on, the persisted reasoning steps are folded in after
  // each AI message so the copy matches the on-screen diagnostics surface.
  const handleCopyConversation = useCallback(
    async (entry) => {
      if (!entry || !entry.id) return;
      const events = conversationEventsRef.current;
      const markdown = conversationToMarkdown(events, entry.id, {
        title: entry.title || "",
        userLabel: t("message.author.user"),
        assistantLabel:
          normalizeAssistantName(assistantNameRef.current) || "formal-ai",
        reasoningLabel: t("message.diagnosticsSteps"),
        includeReasoning: diagnosticsModeRef.current,
      });
      const ok = await copyTextToClipboard(markdown);
      if (ok) {
        setCopiedConversationId(entry.id);
        refreshConversations();
        setTimeout(() => {
          setCopiedConversationId((current) =>
            current === entry.id ? "" : current,
          );
        }, 1600);
      }
    },
    [refreshConversations, t],
  );

  useEffect(() => {
    persistPreferences({
      demoMode,
      diagnosticsMode,
      contextPanelWidth,
      sidebarMenuCollapsed,
      sidebarPromptsCollapsed,
      sidebarToolsCollapsed,
      sidebarTraceCollapsed,
      sidebarConversationsCollapsed,
      sidebarSettingsCollapsed,
      sidebarCollapsed,
      showDeletedConversations,
      greetingVariations,
      guessProbability,
      temperature,
      followUpProbability,
      definitionFusion,
      blueprintComposition,
      experimentalOcr,
      associativeProjectPromotion,
      theme: themePreference,
      uiSkin,
      chatStyle,
      composerStyle,
      composerAction,
      location: locationPreference,
      assistantName: normalizeAssistantName(assistantName),
      currentConversationId,
      agentMode,
      uiLanguage: uiLanguagePreference,
      responseLanguage,
      preferredLanguage,
    });
  }, [
    demoMode,
    diagnosticsMode,
    contextPanelWidth,
    sidebarMenuCollapsed,
    sidebarPromptsCollapsed,
    sidebarToolsCollapsed,
    sidebarTraceCollapsed,
    sidebarConversationsCollapsed,
    sidebarSettingsCollapsed,
    sidebarCollapsed,
    showDeletedConversations,
    greetingVariations,
    guessProbability,
    temperature,
    followUpProbability,
    definitionFusion,
    blueprintComposition,
    experimentalOcr,
    associativeProjectPromotion,
    themePreference,
    uiSkin,
    chatStyle,
    composerStyle,
    composerAction,
    locationPreference,
    assistantName,
    currentConversationId,
    agentMode,
    uiLanguagePreference,
    responseLanguage,
    preferredLanguage,
  ]);

  useEffect(() => {
    const worker = new Worker(withAssetVersion("formal_ai_worker.js"));
    workerRef.current = worker;
    worker.onmessage = (event) => {
      if (event.data.kind === "ready") {
        setWorkerState(event.data.mode);
        return;
      }

      const requestId = event.data.requestId;
      const resolver = pendingResponses.current.get(requestId);
      if (resolver) {
        pendingResponses.current.delete(requestId);
        resolver(event.data);
      }
    };

    return () => worker.terminate();
  }, []);

  useEffect(() => {
    transcriptEndRef.current?.scrollIntoView({ block: "end" });
  }, [messages]);

  useEffect(() => {
    resizeComposerInput(composerInputRef.current);
  }, [prompt, demoMode]);

  const greetingVariationsRef = useRef(greetingVariations);
  useEffect(() => {
    greetingVariationsRef.current = greetingVariations;
  }, [greetingVariations]);

  const diagnosticsModeRef = useRef(diagnosticsMode);
  useEffect(() => {
    diagnosticsModeRef.current = diagnosticsMode;
  }, [diagnosticsMode]);

  const demoModeRef = useRef(demoMode);
  useEffect(() => {
    demoModeRef.current = demoMode;
  }, [demoMode]);

  const guessProbabilityRef = useRef(guessProbability);
  useEffect(() => {
    guessProbabilityRef.current = guessProbability;
  }, [guessProbability]);

  const temperatureRef = useRef(temperature);
  useEffect(() => {
    temperatureRef.current = temperature;
  }, [temperature]);

  const followUpProbabilityRef = useRef(followUpProbability);
  useEffect(() => {
    followUpProbabilityRef.current = followUpProbability;
  }, [followUpProbability]);

  const definitionFusionRef = useRef(definitionFusion);
  useEffect(() => {
    definitionFusionRef.current = definitionFusion;
  }, [definitionFusion]);

  const blueprintCompositionRef = useRef(blueprintComposition);
  useEffect(() => {
    blueprintCompositionRef.current = blueprintComposition;
  }, [blueprintComposition]);

  const experimentalOcrRef = useRef(experimentalOcr);
  useEffect(() => {
    experimentalOcrRef.current = experimentalOcr;
  }, [experimentalOcr]);

  const associativeProjectPromotionRef = useRef(associativeProjectPromotion);
  useEffect(() => {
    associativeProjectPromotionRef.current = associativeProjectPromotion;
  }, [associativeProjectPromotion]);

  const agentModeRef = useRef(agentMode);
  useEffect(() => {
    agentModeRef.current = agentMode;
  }, [agentMode]);

  const themePreferenceRef = useRef(themePreference);
  useEffect(() => {
    themePreferenceRef.current = themePreference;
  }, [themePreference]);

  const uiLanguagePreferenceRef = useRef(uiLanguagePreference);
  useEffect(() => {
    uiLanguagePreferenceRef.current = uiLanguagePreference;
  }, [uiLanguagePreference]);

  const responseLanguageRef = useRef(responseLanguage);
  useEffect(() => {
    responseLanguageRef.current = responseLanguage;
  }, [responseLanguage]);

  const preferredLanguageRef = useRef(preferredLanguage);
  useEffect(() => {
    preferredLanguageRef.current = preferredLanguage;
  }, [preferredLanguage]);

  const uiSkinRef = useRef(uiSkin);
  useEffect(() => {
    uiSkinRef.current = uiSkin;
  }, [uiSkin]);

  const chatStyleRef = useRef(chatStyle);
  useEffect(() => {
    chatStyleRef.current = chatStyle;
  }, [chatStyle]);

  const composerStyleRef = useRef(composerStyle);
  useEffect(() => {
    composerStyleRef.current = composerStyle;
  }, [composerStyle]);

  const composerActionRef = useRef(composerAction);
  useEffect(() => {
    composerActionRef.current = composerAction;
  }, [composerAction]);

  const locationPreferenceRef = useRef(locationPreference);
  useEffect(() => {
    locationPreferenceRef.current = locationPreference;
  }, [locationPreference]);

  const assistantNameRef = useRef(assistantName);
  useEffect(() => {
    assistantNameRef.current = assistantName;
  }, [assistantName]);

  const desktopStatusRef = useRef(desktopStatus);
  useEffect(() => {
    desktopStatusRef.current = desktopStatus;
  }, [desktopStatus]);

  const requestAnswer = useCallback((text, history = []) => {
    const worker = workerRef.current;
    const prefs = {
      greetingVariations: greetingVariationsRef.current,
      diagnosticsMode: diagnosticsModeRef.current,
      demoMode: demoModeRef.current,
      guessProbability: guessProbabilityRef.current,
      temperature: temperatureRef.current,
      followUpProbability: followUpProbabilityRef.current,
      definitionFusion: definitionFusionRef.current,
      blueprintComposition: blueprintCompositionRef.current,
      experimentalOcr: experimentalOcrRef.current,
      associativeProjectPromotion: associativeProjectPromotionRef.current,
      agentMode: agentModeRef.current,
      theme: themePreferenceRef.current,
      uiLanguage: uiLanguagePreferenceRef.current,
      responseLanguage: responseLanguageRef.current,
      preferredLanguage: preferredLanguageRef.current,
      uiSkin: uiSkinRef.current,
      chatStyle: chatStyleRef.current,
      composerStyle: composerStyleRef.current,
      composerAction: composerActionRef.current,
      location: locationPreferenceRef.current,
      assistantName: normalizeAssistantName(assistantNameRef.current),
    };
    const currentDesktopStatus = desktopStatusRef.current;
    if (currentDesktopStatus && currentDesktopStatus.apiReady && currentDesktopStatus.apiBase) {
      return requestDesktopAnswer(text, history, currentDesktopStatus, prefs).catch(() => {
        if (!worker) {
          return localFallbackAnswer(text, history, prefs);
        }
        return new Promise((resolve) => {
          const requestId = `request-${Date.now()}-${Math.random().toString(16).slice(2)}`;
          pendingResponses.current.set(requestId, resolve);
          worker.postMessage({
            prompt: text,
            requestId,
            history,
            prefs,
            userContext: userContextRef.current,
          });
        });
      });
    }
    if (!worker) {
      return Promise.resolve(localFallbackAnswer(text, history, prefs));
    }

    return new Promise((resolve) => {
      const requestId = `request-${Date.now()}-${Math.random().toString(16).slice(2)}`;
      pendingResponses.current.set(requestId, resolve);
      worker.postMessage({
        prompt: text,
        requestId,
        history,
        prefs,
        userContext: userContextRef.current,
      });
    });
  }, []);

  // Issue #27: assign every appended event to the current conversation, lazily
  // minting a fresh id on the first user message of a brand-new chat. The
  // returned object is { conversationId, conversationTitle } so the caller can
  // reuse it for follow-up records within the same turn (assistant reply,
  // reasoning steps, tool calls).
  const ensureConversation = useCallback((seedText) => {
    let id = currentConversationRef.current;
    let isNew = false;
    if (!id) {
      id = generateConversationId();
      isNew = true;
      currentConversationRef.current = id;
      setCurrentConversationId(id);
    }
    let title = conversationTitlesRef.current.get(id);
    if (!title && seedText) {
      title = deriveConversationTitle(seedText);
      conversationTitlesRef.current.set(id, title);
    }
    return { conversationId: id, conversationTitle: title || "", isNew };
  }, []);

  const appendUserMessage = useCallback((text, extra = {}) => {
    const { conversationId, conversationTitle } = ensureConversation(text);
    const message = createMessage("user", text, extra);
    const memoryAttachments = Array.isArray(extra.attachments)
      ? extra.attachments.map(attachmentMemoryRecord)
      : [];
    setMessages((current) => [...current, message]);
    recordMemoryEvent({
      kind: "message",
      role: "user",
      content: text,
      sentAt: new Date().toISOString(),
      demoLabel: extra.demoLabel,
      attachments:
        memoryAttachments.length > 0
          ? JSON.stringify(memoryAttachments)
          : undefined,
      conversationId,
      conversationTitle,
    });
  }, [ensureConversation]);

  const appendAssistantMessage = useCallback((answer) => {
    const source = answer.source || (workerRef.current ? "worker" : "fallback");
    const solverEvidence = Array.isArray(answer.evidence) ? answer.evidence : [];
    const evidence = answer.intent
      ? [`intent:${answer.intent}`, `source:${source}`, ...solverEvidence]
      : solverEvidence;
    const structuredSteps = Array.isArray(answer.steps) ? answer.steps : [];
    const structuredToolCalls = Array.isArray(answer.toolCalls)
      ? answer.toolCalls
      : [];
    const thinkingSteps = structuredSteps.length > 0
      ? structuredSteps.map((entry) => `${entry.step}: ${entry.detail}`)
      : [
          "Normalize prompt text",
          `Select symbolic intent ${answer.intent || "unknown"}`,
          `Render deterministic answer from ${source}`,
        ];
    const message = createMessage("assistant", answer.content, {
      intent: answer.intent,
      evidence,
      thinkingSteps,
      diagnosticsSteps: structuredSteps,
      diagnosticsToolCalls: structuredToolCalls,
      // Issue #180: forward the web_search diagnostics envelope so the
      // diagnostics panel can show raw HTTP request/response exchanges and
      // the per-provider success/failure status.
      diagnostics: answer.diagnostics || null,
      iframeUrl: answer.iframeUrl || null,
    });
    setMessages((current) => [...current, message]);
    const sentAt = new Date().toISOString();
    const { conversationId, conversationTitle } = ensureConversation("");
    if (Array.isArray(answer.steps)) {
      answer.steps.forEach((entry) => {
        recordMemoryEvent({
          kind: "reasoning",
          role: "assistant",
          content: `${entry.step}: ${entry.detail}`,
          intent: answer.intent,
          sentAt,
          conversationId,
          conversationTitle,
        });
      });
    }
    if (Array.isArray(answer.toolCalls)) {
      answer.toolCalls.forEach((call) => {
        recordMemoryEvent({
          kind: "tool_call",
          role: "assistant",
          tool: call.tool,
          inputs: call.inputs,
          outputs: call.outputs,
          content: `tool:${call.tool}`,
          sentAt,
          conversationId,
          conversationTitle,
        });
      });
    }
    recordMemoryEvent({
      kind: "message",
      role: "assistant",
      content: answer.content,
      intent: answer.intent,
      evidence,
      iframeUrl: answer.iframeUrl || null,
      sentAt,
      conversationId,
      conversationTitle,
    }).then(() => {
      // Refresh the sidebar so a brand-new conversation appears immediately.
      refreshConversations();
    });
  }, [ensureConversation, refreshConversations]);

  const conversationHistory = useCallback(
    () =>
      messages.map((message) => ({
        role: message.role,
        content: message.content,
        intent: message.intent,
        evidence: message.evidence,
      })),
    [messages],
  );

  const applyInterfaceCommand = useCallback(
    (command) => {
      if (!command) return;
      if (command.kind === "trigger" && command.action === "attach_files") {
        triggerAttachFiles();
        return;
      }
      if (command.kind !== "set_preference") {
        return;
      }
      switch (command.key) {
        case "diagnosticsMode":
          setDiagnosticsMode(Boolean(command.value));
          break;
        case "demoMode":
          setDemoMode(Boolean(command.value));
          break;
        case "agentMode":
          setAgentMode(Boolean(command.value));
          break;
        case "greetingVariations":
          setGreetingVariations(Boolean(command.value));
          break;
        case "definitionFusion":
          setDefinitionFusion(normalizeDefinitionFusion(command.value));
          break;
        case "blueprintComposition":
          setBlueprintComposition(
            normalizeBlueprintComposition(command.value),
          );
          break;
        case "experimentalOcr":
          setExperimentalOcr(Boolean(command.value));
          break;
        case "associativeProjectPromotion":
          setAssociativeProjectPromotion(Boolean(command.value));
          break;
        case "theme":
          setThemePreference(normalizeThemePreference(command.value));
          break;
        case "uiLanguage":
          setUiLanguagePreference(normalizeUiLanguagePreference(command.value));
          break;
        case "responseLanguage":
          setResponseLanguage(normalizeResponseLanguageMode(command.value));
          break;
        case "preferredLanguage":
          setPreferredLanguage(normalizePreferredLanguage(command.value));
          break;
        case "uiSkin":
          setUiSkin(normalizeUiSkin(command.value));
          break;
        case "chatStyle":
          setChatStyle(normalizeChatStyle(command.value));
          break;
        case "composerStyle":
          setComposerStyle(normalizeComposerStyle(command.value));
          break;
        case "composerAction":
          setComposerAction(normalizeComposerAction(command.value));
          break;
        case "temperature":
          setTemperature(
            normalizeSliderPreference(command.value, PREFERENCE_DEFAULTS.temperature),
          );
          break;
        case "guessProbability":
          setGuessProbability(
            normalizeSliderPreference(
              command.value,
              PREFERENCE_DEFAULTS.guessProbability,
            ),
          );
          break;
        case "location":
          setLocationPreference(String(command.value || "").slice(0, 80));
          break;
        case "assistantName":
          setAssistantName(normalizeAssistantName(command.value));
          break;
        case "sidebarCollapsed":
          setSidebarCollapsed(Boolean(command.value));
          break;
        case "showDeletedConversations":
          setShowDeletedConversations(Boolean(command.value));
          refreshConversations(Boolean(command.value));
          break;
        default:
          break;
      }
    },
    [refreshConversations, triggerAttachFiles],
  );

  // Issue #27: agent mode — run a decomposed task plan and merge the per-step
  // results into a single assistant message. Each step calls the same solver
  // the chat path uses, so deterministic intents (greeting, identity,
  // arithmetic, concept lookup, etc.) behave identically; the difference is
  // surface presentation, not solver semantics.
  const runAgentPlan = useCallback(
    async (steps, history) => {
      const lines = [];
      lines.push(`## Agent plan (${steps.length} steps)`);
      steps.forEach((step, index) => {
        lines.push(`${index + 1}. ${step}`);
      });
      lines.push("");
      const aggregatedSteps = [];
      const aggregatedToolCalls = [];
      const aggregatedEvidence = [];
      const workingHistory = Array.isArray(history) ? history.slice() : [];
      for (let index = 0; index < steps.length; index += 1) {
        const step = steps[index];
        aggregatedSteps.push({
          step: "agent_plan",
          detail: `${index + 1}/${steps.length} ${step}`,
        });
        const answer = await requestAnswer(step, workingHistory);
        lines.push(`### Step ${index + 1}: ${step}`);
        lines.push(answer.content || "(no output)");
        lines.push("");
        if (Array.isArray(answer.steps)) {
          answer.steps.forEach((entry) => {
            aggregatedSteps.push({
              step: `agent_${index + 1}_${entry.step}`,
              detail: entry.detail,
            });
          });
        }
        if (Array.isArray(answer.toolCalls)) {
          aggregatedToolCalls.push(...answer.toolCalls);
        }
        if (Array.isArray(answer.evidence)) {
          aggregatedEvidence.push(
            ...answer.evidence.map((item) => `step_${index + 1}:${item}`),
          );
        }
        workingHistory.push({ role: "user", content: step });
        workingHistory.push({ role: "assistant", content: answer.content || "" });
      }
      appendAssistantMessage({
        intent: "agent_plan",
        content: lines.join("\n").trim(),
        confidence: 0.85,
        evidence: ["rule:agent_mode", `steps:${steps.length}`, ...aggregatedEvidence],
        steps: aggregatedSteps,
        toolCalls: aggregatedToolCalls,
      });
    },
    [requestAnswer, appendAssistantMessage],
  );

  async function sendText(text, extra = {}) {
    const trimmed = text.trim();
    const displayText = String(extra.displayText || trimmed).trim();
    const hasAttachments =
      Array.isArray(extra.attachments) && extra.attachments.length > 0;
    if ((!trimmed && !displayText) || pending) {
      return;
    }

    setPending(true);
    const history = conversationHistory();
    appendUserMessage(displayText || trimmed, extra);

    // Issue #27: short-circuit memory-action phrases to the corresponding
    // toolbar button before invoking the worker so the chat surface and the
    // sidebar stay in lock-step.
    const memoryAction = hasAttachments ? null : recognizeMemoryAction(displayText);
    if (memoryAction === "export") {
      await handleExportMemory();
      appendAssistantMessage({
        intent: "memory_export",
        content: t("memory.exportTriggered"),
        confidence: 1.0,
        evidence: ["rule:memory_export"],
        steps: [{ step: "trigger_button", detail: "memory-export" }],
        toolCalls: [
          {
            tool: "export_memory",
            inputs: { prompt: displayText },
            outputs: { intent: "memory_export" },
          },
        ],
      });
      setPending(false);
      return;
    }
    if (memoryAction === "import") {
      triggerImportMemory();
      appendAssistantMessage({
        intent: "memory_import",
        content: t("memory.importTriggered"),
        confidence: 1.0,
        evidence: ["rule:memory_import"],
        steps: [{ step: "trigger_button", detail: "memory-import" }],
        toolCalls: [
          {
            tool: "import_memory",
            inputs: { prompt: displayText },
            outputs: { intent: "memory_import" },
          },
        ],
      });
      setPending(false);
      return;
    }
    if (memoryAction === "reset") {
      const result = await handleResetMemory();
      if (!result.cancelled) {
        setPending(false);
        return;
      }
      appendAssistantMessage({
        intent: "memory_reset",
        content: t("memory.resetCancelled"),
        confidence: 1.0,
        evidence: ["rule:memory_reset"],
        steps: [{ step: "trigger_button", detail: "memory-reset" }],
        toolCalls: [
          {
            tool: "reset_memory",
            inputs: { prompt: displayText },
            outputs: { intent: "memory_reset", events: result.removed },
          },
        ],
      });
      setPending(false);
      return;
    }

    const interfaceCommand = hasAttachments ? null : recognizeInterfaceCommand(displayText);
    if (interfaceCommand) {
      const valueLabel = commandValueLabel(interfaceCommand);
      if (interfaceCommand.kind !== "report_issue") {
        applyInterfaceCommand(interfaceCommand);
      }
      appendAssistantMessage({
        intent: interfaceCommand.intent,
        content: interfaceCommandResponse(interfaceCommand, currentReportUrl),
        confidence: 1.0,
        evidence: [
          `rule:${interfaceCommand.intent}`,
          `command:${interfaceCommand.kind}`,
          ...(interfaceCommand.key ? [`preference:${interfaceCommand.key}`] : []),
          `value:${valueLabel}`,
        ],
        steps: [
          {
            step:
              interfaceCommand.kind === "set_preference"
                ? "apply_message_command"
                : "trigger_message_action",
            detail: interfaceCommand.key
              ? `${interfaceCommand.key}=${valueLabel}`
              : interfaceCommand.label,
          },
        ],
        toolCalls: [
          {
            tool:
              interfaceCommand.kind === "set_preference"
                ? "configure_preference"
                : interfaceCommand.intent,
            inputs: { prompt: displayText },
            outputs: {
              kind: interfaceCommand.kind,
              key: interfaceCommand.key || interfaceCommand.action || "",
              value: interfaceCommand.value ?? interfaceCommand.label,
            },
          },
        ],
      });
      setPending(false);
      return;
    }

    // Issue #27 R11: cross-conversation recall. Phrases like "when did I ask
    // about Rust" / "find Donald Trump in another conversation" search the
    // append-only memory log on the main thread (where FormalAiMemory lives)
    // and emit a Markdown report grouped by conversation. The recognition
    // happens before the worker round-trip so we never have to ferry the full
    // event log across the worker boundary.
    const recallQuery = hasAttachments ? null : recognizeRecallQuery(displayText);
    if (recallQuery && typeof window !== "undefined" && window.FormalAiMemory) {
      let events = [];
      try {
        events = await window.FormalAiMemory.listEvents();
      } catch (_error) {
        events = [];
      }
      const report = buildRecallReport({
        events,
        term: recallQuery.term,
        scope: recallQuery.scope,
        currentConversationId: currentConversationRef.current,
        triggerText: displayText,
      });
      appendAssistantMessage({
        intent: "conversation_recall",
        content: report.content,
        confidence: 1.0,
        evidence: [
          "rule:conversation_recall",
          `scope:${recallQuery.scope}`,
          `matches:${report.matches.reduce((sum, g) => sum + g.events.length, 0)}`,
        ],
        steps: [
          { step: "extract_term", detail: recallQuery.term },
          { step: "scan_memory", detail: `${events.length} event(s)` },
          { step: "group_by_conversation", detail: `${report.matches.length} group(s)` },
        ],
        toolCalls: [
          {
            tool: "conversation_recall",
            inputs: { term: recallQuery.term, scope: recallQuery.scope },
            outputs: {
              conversations: report.matches.length,
              matches: report.matches.reduce((sum, g) => sum + g.events.length, 0),
            },
          },
        ],
      });
      setPending(false);
      return;
    }

    // Issue #27: agent mode decomposes the prompt into sub-tasks and executes
    // them sequentially, producing one consolidated assistant message with a
    // plan preamble and a per-step result list. Chat mode runs the single-step
    // path unchanged.
    if (agentModeRef.current && !hasAttachments) {
      const steps = decomposeAgentTask(displayText);
      if (steps.length > 1) {
        await runAgentPlan(steps, history);
        setPending(false);
        return;
      }
    }

    const answer = await requestAnswer(trimmed, history);
    appendAssistantMessage(answer);
    setPending(false);
  }

  async function send() {
    const text = prompt.trim();
    if (!text && attachments.length === 0) {
      return;
    }

    setPrompt("");
    setComposerMenuOpen(false);
    const queuedAttachments = attachments;
    setAttachments([]);
    const preparedAttachments = await prepareAttachmentsForSend(queuedAttachments);
    const displayText = text || attachmentOnlyPrompt(preparedAttachments);
    const solverText = buildPromptWithAttachments(displayText, preparedAttachments);
    await sendText(solverText, {
      displayText,
      attachments: preparedAttachments,
    });
  }

  function handleKeyDown(event) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      send();
    }
  }

  useEffect(() => {
    if (!demoMode) {
      setDemoPhase("manual");
      setDemoCountdown(null);
      return undefined;
    }

    let cancelled = false;
    let countdownTimer = 0;

    async function runCycle() {
      const turns = createDemoTurns();
      setMessages([]);
      setPending(true);
      setDemoPhase("playing");
      setDemoCountdown(null);

      for (const turn of turns) {
        if (cancelled) {
          return;
        }

        appendUserMessage(turn.text, { demoLabel: turn.label });
        await wait(randomInt(700, 1300));
        const answer = await requestAnswer(turn.text);
        if (cancelled) {
          return;
        }
        appendAssistantMessage(answer);
        await wait(randomInt(900, 1500));
      }

      setPending(false);
      const waitSeconds = randomInt(10, 20);
      let remainingSeconds = waitSeconds;
      setDemoPhase("waiting");
      setDemoCountdown(remainingSeconds);
      countdownTimer = window.setInterval(() => {
        remainingSeconds -= 1;
        if (remainingSeconds <= 0) {
          window.clearInterval(countdownTimer);
          if (!cancelled) {
            runCycle();
          }
          return;
        }
        setDemoCountdown(remainingSeconds);
      }, 1000);
    }

    runCycle();

    return () => {
      cancelled = true;
      window.clearInterval(countdownTimer);
      setPending(false);
    };
  }, [appendAssistantMessage, appendUserMessage, demoMode, requestAnswer]);

  const lastAssistant = useMemo(
    () => [...messages].reverse().find((message) => message.role === "assistant"),
    [messages],
  );

  const demoStatus = demoMode
    ? demoPhase === "waiting" && demoCountdown !== null
      ? t("status.nextDialogIn", { seconds: demoCountdown })
      : t("status.demoPlaying")
    : t("status.manual");
  const reportContext = {
    messages,
    workerState,
    demoMode,
    demoStatus,
    diagnosticsMode,
    userContext,
  };
  const currentReportUrl = createIssueUrl(reportContext);

  // Issue #386: registry of user-facing settings so the panel can reset each
  // one (or all of them) to its shipped default. Each entry pairs a
  // PREFERENCE_DEFAULTS key with the live value, its setter, and the i18n key
  // used as its label in the reset list.
  const settingDescriptors = [
    { key: "guessProbability", value: guessProbability, set: setGuessProbability, label: "settings.ambiguity" },
    { key: "followUpProbability", value: followUpProbability, set: setFollowUpProbability, label: "settings.followUpInitiative" },
    { key: "temperature", value: temperature, set: setTemperature, label: "settings.temperature" },
    { key: "greetingVariations", value: greetingVariations, set: setGreetingVariations, label: "settings.variations" },
    { key: "definitionFusion", value: definitionFusion, set: setDefinitionFusion, label: "settings.definitionFusion" },
    { key: "blueprintComposition", value: blueprintComposition, set: setBlueprintComposition, label: "settings.blueprintComposition" },
    { key: "experimentalOcr", value: experimentalOcr, set: setExperimentalOcr, label: "settings.experimentalOcr" },
    { key: "uiLanguage", value: uiLanguagePreference, set: setUiLanguagePreference, label: "settings.language" },
    { key: "responseLanguage", value: responseLanguage, set: setResponseLanguage, label: "settings.responseLanguage" },
    { key: "preferredLanguage", value: preferredLanguage, set: setPreferredLanguage, label: "settings.preferredLanguage" },
    { key: "theme", value: themePreference, set: setThemePreference, label: "settings.theme" },
    { key: "uiSkin", value: uiSkin, set: setUiSkin, label: "settings.uiSkin" },
    { key: "chatStyle", value: chatStyle, set: setChatStyle, label: "settings.chatStyle" },
    { key: "composerStyle", value: composerStyle, set: setComposerStyle, label: "settings.composerStyle" },
    { key: "composerAction", value: composerAction, set: setComposerAction, label: "settings.composerAction" },
    { key: "assistantName", value: assistantName, set: setAssistantName, label: "settings.assistantName" },
    { key: "location", value: locationPreference, set: setLocationPreference, label: "settings.location" },
  ];
  const modifiedSettings = settingDescriptors.filter(
    (descriptor) => !settingIsDefault(descriptor.key, descriptor.value),
  );
  const resetSetting = (descriptor) => {
    descriptor.set(PREFERENCE_DEFAULTS[descriptor.key]);
  };
  const resetAllSettings = () => {
    for (const descriptor of modifiedSettings) {
      resetSetting(descriptor);
    }
  };

  const composerActionIcon = composerAction === "plus" ? "+" : "📎";
  const attachmentStatus =
    attachments.length > 0
      ? t("composer.attachments", { count: attachments.length })
      : "";
  const desktopStatusText = desktopStatusLabel(desktopStatus, agentMode);
  const desktopAgentPermission = agentMode ? "Opted in" : "Off";
  const desktopToolPermission =
    desktopStatus && agentMode
      ? "Agent tools visible"
      : "Permission gated";

  return h(
    "main",
    {
      className: [
        "app",
        `ui-skin-${uiSkin}`,
        `chat-style-${chatStyle}`,
        `composer-style-${composerStyle}`,
        desktopStatus ? "desktop-shell" : "",
      ].filter(Boolean).join(" "),
    },
    h(
      "header",
      { className: "topbar" },
      h(
        "button",
        {
          type: "button",
          className: "mobile-menu-toggle topbar-menu-toggle",
          "data-testid": "mobile-menu-toggle",
          "aria-pressed": mobileMenuOpen,
          "aria-label": mobileMenuOpen
            ? t("buttons.closeMenu")
            : t("buttons.openMenu"),
          title: mobileMenuOpen
            ? t("titles.menuClose")
            : t("titles.menuOpen"),
          onClick: () => setMobileMenuOpen((value) => !value),
        },
        h(MenuGlyph, { open: mobileMenuOpen }),
      ),
      h(
        "button",
        {
          type: "button",
          className: `sidebar-toggle${sidebarCollapsed ? " is-collapsed" : ""}`,
          "data-testid": "sidebar-toggle",
          "aria-pressed": !sidebarCollapsed,
          "aria-label": sidebarCollapsed
            ? t("buttons.expandSidebar")
            : t("buttons.collapseSidebar"),
          title: sidebarCollapsed
            ? t("titles.expandSidebar")
            : t("titles.collapseSidebar"),
          onClick: () => setSidebarCollapsed((value) => !value),
        },
        h(SidebarToggleGlyph, { collapsed: sidebarCollapsed }),
      ),
      h(
        "div",
        { className: "brand" },
        h("span", { className: "mark" }, "FA"),
        h("strong", null, "formal-ai"),
        h("span", { className: "brand-version", "data-testid": "app-version" }, `v${APP_VERSION}`),
      ),
      h(
        "div",
        { className: "topbar-actions" },
        desktopStatus
          ? h(
              "span",
              {
                className: "desktop-status",
                "data-testid": "desktop-shell-status",
                "data-menu-priority": "7",
                role: "status",
                title: desktopStatus.apiError || desktopStatusText,
              },
              desktopStatusText,
            )
          : null,
        h(
          "span",
          {
            className: "demo-status",
            "data-testid": "demo-status",
            "data-menu-priority": "7",
            role: "status",
          },
          demoStatus,
        ),
        diagnosticsMode
          ? h("span", { className: "status", "data-menu-priority": "7" }, workerState)
          : null,
        h(
          "a",
          {
            className: "source-code-button",
            "data-testid": "source-code",
            "data-menu-priority": "5",
            href: SOURCE_CODE_URL,
            target: "_blank",
            rel: "noopener noreferrer",
            title: t("titles.sourceCode"),
            "aria-label": t("buttons.sourceCode"),
          },
          h("span", { className: "btn-icon", "aria-hidden": "true" }, "💻"),
          h("span", { className: "btn-label" }, t("buttons.sourceCode")),
        ),
        h(
          "a",
          {
            className: "download-button",
            "data-testid": "download-link",
            "data-menu-priority": "5",
            href: "download/",
            title: t("titles.download"),
            "aria-label": t("buttons.download"),
          },
          h("span", { className: "btn-icon", "aria-hidden": "true" }, "⬇️"),
          h("span", { className: "btn-label" }, t("buttons.download")),
        ),
        h(
          "a",
          {
            className: "report-button",
            "data-testid": "report-issue",
            "data-menu-priority": "1",
            href: currentReportUrl,
            target: "_blank",
            rel: "noopener noreferrer",
            title: t("titles.reportIssue"),
            "aria-label": t("buttons.reportIssue"),
          },
          h("span", { className: "btn-icon", "aria-hidden": "true" }, "🐛"),
          h("span", { className: "btn-label" }, t("buttons.reportIssue")),
        ),
        h(
          "button",
          {
            type: "button",
            className: "memory-button",
            "data-testid": "memory-export",
            "data-menu-priority": "6",
            onClick: handleExportMemory,
            title: t("titles.exportMemory"),
            "aria-label": t("buttons.exportMemory"),
          },
          h("span", { className: "btn-icon", "aria-hidden": "true" }, "📤"),
          h("span", { className: "btn-label" }, t("buttons.exportMemory")),
        ),
        h(
          "button",
          {
            type: "button",
            className: "memory-button",
            "data-testid": "memory-import",
            "data-menu-priority": "6",
            onClick: triggerImportMemory,
            title: t("titles.importMemory"),
            "aria-label": t("buttons.importMemory"),
          },
          h("span", { className: "btn-icon", "aria-hidden": "true" }, "📥"),
          h("span", { className: "btn-label" }, t("buttons.importMemory")),
        ),
        h(
          "button",
          {
            type: "button",
            className: "memory-button memory-reset-button",
            "data-testid": "memory-reset",
            "data-menu-priority": "6",
            onClick: handleResetMemory,
            title: t("titles.resetMemory"),
            "aria-label": t("buttons.resetMemory"),
          },
          h("span", { className: "btn-icon", "aria-hidden": "true" }, "🧹"),
          h("span", { className: "btn-label" }, t("buttons.resetMemory")),
        ),
        h("input", {
          ref: importInputRef,
          type: "file",
          accept: ".lino,text/plain",
          style: { display: "none" },
          "data-testid": "memory-import-input",
          onChange: handleImportMemory,
        }),
        memoryStatus
          ? h(
              "span",
              {
                className: "memory-status",
                role: "status",
                "data-testid": "memory-status",
                "data-menu-priority": "7",
              },
              memoryStatus,
            )
          : null,
        h(
          "button",
          {
            type: "button",
            className: "diagnostics-toggle",
            "data-menu-priority": "2",
            "aria-pressed": diagnosticsMode,
            onClick: () => setDiagnosticsMode((value) => !value),
            title: diagnosticsMode
              ? t("titles.diagnosticsHide")
              : t("titles.diagnosticsShow"),
            "aria-label": diagnosticsMode
              ? t("buttons.diagnosticsOn")
              : t("buttons.diagnostics"),
          },
          h("span", { className: "btn-icon", "aria-hidden": "true" }, "🧪"),
          h(
            "span",
            { className: "btn-label" },
            diagnosticsMode ? t("buttons.diagnosticsOn") : t("buttons.diagnostics"),
          ),
        ),
        h(
          "button",
          {
            type: "button",
            className: "agent-toggle",
            "data-testid": "agent-toggle",
            "data-menu-priority": "4",
            "aria-pressed": agentMode,
            title: agentMode
              ? t("titles.agentOn")
              : t("titles.agentOff"),
            "aria-label": agentMode ? t("buttons.agent") : t("buttons.chat"),
            onClick: () => setAgentMode((value) => !value),
          },
          h(
            "span",
            { className: "btn-icon", "aria-hidden": "true" },
            agentMode ? "🤖" : "💬",
          ),
          h(
            "span",
            { className: "btn-label" },
            agentMode ? t("buttons.agent") : t("buttons.chat"),
          ),
        ),
        h(
          "button",
          {
            type: "button",
            className: "mode-toggle",
            "data-menu-priority": "3",
            "aria-pressed": demoMode,
            onClick: () => setDemoMode((value) => !value),
            title: demoMode
              ? t("titles.demoOn")
              : t("titles.demoOff"),
            "aria-label": demoMode ? t("buttons.demoOn") : t("buttons.demo"),
          },
          h("span", { className: "btn-icon", "aria-hidden": "true" }, "🎬"),
          h(
            "span",
            { className: "btn-label" },
            demoMode ? t("buttons.demoOn") : t("buttons.demo"),
          ),
        ),
      ),
    ),
    mobileMenuOpen
      ? h("div", {
          className: "mobile-menu-backdrop",
          "data-testid": "mobile-menu-backdrop",
          onClick: () => setMobileMenuOpen(false),
        })
      : null,
    h(
      "section",
      {
        className: `workspace${sidebarCollapsed ? " sidebar-collapsed" : ""}`,
        style: { "--context-panel-width": `${contextPanelWidth}px` },
      },
      h(
        "aside",
        {
          className: `context-panel${mobileMenuOpen ? " is-mobile-open" : ""}${sidebarCollapsed ? " is-desktop-collapsed" : ""}`,
          "data-testid": "context-panel",
          "aria-hidden": sidebarCollapsed && !mobileMenuOpen ? "true" : "false",
        },
        h(
          "div",
          { className: "drawer-brand", "data-testid": "drawer-brand" },
          h(
            "div",
            { className: "drawer-brand-main" },
            h("span", { className: "mark" }, "FA"),
            h(
              "div",
              { className: "drawer-brand-copy" },
              h("strong", null, "formal-ai"),
              h("span", { className: "brand-version" }, `v${APP_VERSION}`),
            ),
          ),
          h(
            "button",
            {
              type: "button",
              className: "drawer-close",
              "data-testid": "drawer-close",
              "aria-label": t("buttons.closeMenu"),
              title: t("titles.menuClose"),
              onClick: () => setMobileMenuOpen(false),
            },
            h(MenuGlyph, { open: true }),
          ),
        ),
        h(CollapsibleSection, {
          title: t("sidebar.menu"),
          testId: "drawer-menu-actions",
          collapsed: sidebarMenuCollapsed,
          onToggle: () => setSidebarMenuCollapsed((value) => !value),
          className: "drawer-menu-section",
          bodyClassName: "drawer-menu-body",
          children: h(
            "div",
            { className: "drawer-action-list" },
            h(
              "a",
              {
                className: "drawer-action",
                "data-testid": "drawer-source-code",
                href: SOURCE_CODE_URL,
                target: "_blank",
                rel: "noopener noreferrer",
              },
              h("span", { className: "btn-icon", "aria-hidden": "true" }, "💻"),
              h("span", null, t("buttons.sourceCode")),
            ),
            h(
              "a",
              {
                className: "drawer-action",
                "data-testid": "drawer-report-issue",
                href: currentReportUrl,
                target: "_blank",
                rel: "noopener noreferrer",
              },
              h("span", { className: "btn-icon", "aria-hidden": "true" }, "🐛"),
              h("span", null, t("buttons.reportIssue")),
            ),
            h(
              "button",
              {
                type: "button",
                className: "drawer-action",
                "data-testid": "drawer-memory-export",
                onClick: handleExportMemory,
              },
              h("span", { className: "btn-icon", "aria-hidden": "true" }, "📤"),
              h("span", null, t("buttons.exportMemory")),
            ),
            h(
              "button",
              {
                type: "button",
                className: "drawer-action",
                "data-testid": "drawer-memory-import",
                onClick: triggerImportMemory,
              },
              h("span", { className: "btn-icon", "aria-hidden": "true" }, "📥"),
              h("span", null, t("buttons.importMemory")),
            ),
            h(
              "button",
              {
                type: "button",
                className: "drawer-action",
                "data-testid": "drawer-memory-reset",
                onClick: handleResetMemory,
              },
              h("span", { className: "btn-icon", "aria-hidden": "true" }, "🧹"),
              h("span", null, t("buttons.resetMemory")),
            ),
            h(
              "button",
              {
                type: "button",
                className: "drawer-action",
                "aria-pressed": diagnosticsMode,
                onClick: () => setDiagnosticsMode((value) => !value),
              },
              h("span", { className: "btn-icon", "aria-hidden": "true" }, "🧪"),
              h("span", null, diagnosticsMode ? t("buttons.diagnosticsOn") : t("buttons.diagnostics")),
            ),
            h(
              "button",
              {
                type: "button",
                className: "drawer-action",
                "aria-pressed": agentMode,
                onClick: () => setAgentMode((value) => !value),
              },
              h("span", { className: "btn-icon", "aria-hidden": "true" }, agentMode ? "🤖" : "💬"),
              h("span", null, agentMode ? t("buttons.agent") : t("buttons.chat")),
            ),
            h(
              "button",
              {
                type: "button",
                className: "drawer-action",
                "aria-pressed": demoMode,
                onClick: () => setDemoMode((value) => !value),
              },
              h("span", { className: "btn-icon", "aria-hidden": "true" }, "🎬"),
              h("span", null, demoMode ? t("buttons.demoOn") : t("buttons.demo")),
            ),
          ),
        }),
        desktopStatus
          ? h(CollapsibleSection, {
              title: desktopSurfaceLabel(desktopStatus),
              testId: "sidebar-desktop",
              collapsed: sidebarDesktopCollapsed,
              onToggle: () => setSidebarDesktopCollapsed((value) => !value),
              className: "desktop-shell-section",
              children: h(
                "dl",
                { className: "desktop-shell-panel", "data-testid": "desktop-shell-panel" },
                h(
                  "div",
                  null,
                  h("dt", null, "Shell"),
                  h("dd", null, desktopStatus.shell),
                ),
                h(
                  "div",
                  null,
                  h("dt", null, "API"),
                  h(
                    "dd",
                    { "data-testid": "desktop-api-base" },
                    compactUrl(desktopStatus.apiBase),
                  ),
                ),
                h(
                  "div",
                  null,
                  h("dt", null, "Network"),
                  h(
                    "dd",
                    null,
                    h(
                      "a",
                      {
                        href: desktopStatus.graphUrl || "#",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        "data-testid": "desktop-network-link",
                      },
                      compactUrl(desktopStatus.graphUrl),
                    ),
                  ),
                ),
                h(
                  "div",
                  null,
                  h("dt", null, "Memory"),
                  h("dd", { "data-testid": "desktop-memory-bundle" }, desktopStatus.memory),
                ),
                h(
                  "div",
                  null,
                  h("dt", null, "Agent"),
                  h(
                    "dd",
                    { "data-testid": "desktop-agent-permission" },
                    desktopAgentPermission,
                  ),
                ),
                h(
                  "div",
                  null,
                  h("dt", null, "Tool calls"),
                  h(
                    "dd",
                    { "data-testid": "desktop-tool-permission" },
                    desktopToolPermission,
                  ),
                ),
              ),
            })
          : null,
        h(CollapsibleSection, {
          title: t("sidebar.conversations"),
          testId: "sidebar-conversations",
          collapsed: sidebarConversationsCollapsed,
          onToggle: () => setSidebarConversationsCollapsed((value) => !value),
          children: h(
            "div",
            { className: "conversation-list", "data-testid": "conversation-list" },
            h(
              "button",
              {
                type: "button",
                className: "conversation-new",
                "data-testid": "conversation-new",
                disabled:
                  messages.length === 0 &&
                  !currentConversationId &&
                  prompt.trim().length === 0,
                onClick: () => {
                  currentConversationRef.current = "";
                  setCurrentConversationId("");
                  setMessages([]);
                  setDemoMode(false);
                  setPrompt("");
                },
              },
              t("conversation.new"),
            ),
            h(
              "label",
              { className: "conversation-deleted-toggle" },
              h("input", {
                type: "checkbox",
                checked: showDeletedConversations,
                "data-testid": "conversation-show-deleted",
                onChange: handleShowDeletedConversations,
              }),
              h("span", null, t("conversation.showDeleted")),
            ),
            showDeletedConversations
              ? h(
                  "button",
                  {
                    type: "button",
                    className: "conversation-purge-deleted",
                    "data-testid": "conversation-purge-deleted",
                    disabled: conversations.length === 0,
                    onClick: handlePurgeDeletedConversations,
                    title: t("conversation.purgeDeletedTitle"),
                  },
                  t("conversation.purgeDeleted"),
                )
              : null,
            conversations.length === 0
              ? h(
                  "p",
                  { className: "conversation-empty" },
                  showDeletedConversations
                    ? t("conversation.deletedEmpty")
                    : t("conversation.empty"),
                )
              : h(
                  "ul",
                  {
                    className: "conversation-entries",
                    "data-testid": "conversation-entries",
                  },
                  conversations.map((entry) => {
                    const active = entry.id === currentConversationId;
                    return h(
                      "li",
                      {
                        key: entry.id,
                        className: [
                          "conversation-entry",
                          active ? "is-active" : "",
                          entry.deleted ? "is-deleted" : "",
                        ].filter(Boolean).join(" "),
                      },
                      h(
                        "div",
                        { className: "conversation-entry-row" },
                        h(
                          "button",
                          {
                            type: "button",
                            className: "conversation-entry-button",
                            "data-conversation-id": entry.id,
                            "aria-pressed": active,
                            onClick: async () => {
                              if (entry.id === currentConversationRef.current) {
                                return;
                              }
                              currentConversationRef.current = entry.id;
                              setCurrentConversationId(entry.id);
                              setDemoMode(false);
                              try {
                                const events =
                                  await window.FormalAiMemory.listEvents();
                                setMessages(
                                  messagesForConversation(events, entry.id),
                                );
                              } catch (_error) {
                                setMessages([]);
                              }
                            },
                          },
                          h(
                            "span",
                            { className: "conversation-entry-title" },
                            entry.title || t("conversation.emptyTitle"),
                          ),
                          h(
                            "span",
                            { className: "conversation-entry-meta" },
                            t("conversation.messageCount", {
                              count: entry.messageCount,
                            }),
                          ),
                        ),
                        h(
                          "button",
                          {
                            type: "button",
                            className: `conversation-copy${
                              copiedConversationId === entry.id
                                ? " is-copied"
                                : ""
                            }`,
                            "data-testid": "conversation-copy",
                            "data-conversation-id": entry.id,
                            "data-copied":
                              copiedConversationId === entry.id ? "true" : null,
                            "aria-label": t("conversation.copyMarkdownTitle"),
                            title: t("conversation.copyMarkdownTitle"),
                            onClick: () => handleCopyConversation(entry),
                          },
                          copiedConversationId === entry.id
                            ? t("conversation.copyMarkdownDone")
                            : t("conversation.copyMarkdown"),
                        ),
                        entry.deleted
                          ? h(
                              "button",
                              {
                                type: "button",
                                className: "conversation-delete conversation-permanent-delete",
                                "data-testid": "conversation-purge-one",
                                "aria-label": t("conversation.deletePermanent"),
                                title: t("conversation.deletePermanent"),
                                onClick: () => handlePurgeConversation(entry),
                              },
                              "!",
                            )
                          : h(
                              "button",
                              {
                                type: "button",
                                className: "conversation-delete",
                                "data-testid": "conversation-delete",
                                "aria-label": t("conversation.delete"),
                                title: t("conversation.delete"),
                                onClick: () => handleDeleteConversation(entry),
                              },
                              "×",
                            ),
                      ),
                    );
                  }),
            ),
          ),
        }),
        h(CollapsibleSection, {
          title: t("sidebar.settings"),
          testId: "sidebar-settings",
          collapsed: sidebarSettingsCollapsed,
          onToggle: () => setSidebarSettingsCollapsed((value) => !value),
          children: h(
            "div",
            { className: "settings-panel" },
            // Issue #386: reset bar — reset every modified setting individually
            // or all at once back to the shipped defaults.
            h(
              "div",
              { className: "settings-reset", "data-testid": "settings-reset" },
              h(
                "div",
                { className: "settings-reset-header" },
                h(
                  "span",
                  { className: "settings-reset-title" },
                  t("settings.resetHeading"),
                ),
                h(
                  "button",
                  {
                    type: "button",
                    className: "settings-reset-all",
                    "data-testid": "settings-reset-all",
                    disabled: modifiedSettings.length === 0,
                    onClick: resetAllSettings,
                    title: t("settings.resetAll"),
                  },
                  t("settings.resetAll"),
                ),
              ),
              modifiedSettings.length === 0
                ? h(
                    "p",
                    {
                      className: "settings-reset-empty",
                      "data-testid": "settings-reset-empty",
                    },
                    t("settings.resetNone"),
                  )
                : h(
                    "ul",
                    { className: "settings-reset-list" },
                    modifiedSettings.map((descriptor) =>
                      h(
                        "li",
                        {
                          key: descriptor.key,
                          className: "settings-reset-item",
                        },
                        h(
                          "span",
                          { className: "settings-reset-label" },
                          t(descriptor.label),
                        ),
                        h(
                          "button",
                          {
                            type: "button",
                            className: "settings-reset-one",
                            "data-testid": `settings-reset-${descriptor.key}`,
                            onClick: () => resetSetting(descriptor),
                            title: t("settings.resetOne"),
                          },
                          t("settings.resetOne"),
                        ),
                      ),
                    ),
                  ),
            ),
            h(
              "div",
              { className: "setting-row setting-row-slider" },
              h(
                "label",
                { htmlFor: "setting-guess-probability" },
                t("settings.ambiguity"),
              ),
              h(
                "div",
                { className: "setting-poles" },
                h("span", null, t("settings.moreQuestions")),
                h("span", null, t("settings.moreGuessing")),
              ),
              h("input", {
                id: "setting-guess-probability",
                "data-testid": "setting-guess-probability",
                type: "range",
                min: "0",
                max: "1",
                step: "0.05",
                value: guessProbability,
                onChange: (event) =>
                  setGuessProbability(
                    normalizeSliderPreference(event.target.value, 0.8),
                  ),
              }),
              h(
                "output",
                { htmlFor: "setting-guess-probability" },
                `${formatSliderValue(guessProbability)}%`,
              ),
            ),
            h(
              "div",
              { className: "setting-row setting-row-slider" },
              h(
                "label",
                { htmlFor: "setting-follow-up-probability" },
                t("settings.followUpInitiative"),
              ),
              h(
                "div",
                { className: "setting-poles" },
                h("span", null, t("settings.userInitiative")),
                h("span", null, t("settings.assistantInitiative")),
              ),
              h("input", {
                id: "setting-follow-up-probability",
                "data-testid": "setting-follow-up-probability",
                type: "range",
                min: "0",
                max: "1",
                step: "0.05",
                value: followUpProbability,
                onChange: (event) =>
                  setFollowUpProbability(
                    normalizeSliderPreference(
                      event.target.value,
                      PREFERENCE_DEFAULTS.followUpProbability,
                    ),
                  ),
              }),
              h(
                "output",
                { htmlFor: "setting-follow-up-probability" },
                `${formatSliderValue(followUpProbability)}%`,
              ),
            ),
            h(
              "div",
              { className: "setting-row setting-row-slider" },
              h(
                "label",
                { htmlFor: "setting-temperature" },
                t("settings.temperature"),
              ),
              h(
                "div",
                { className: "setting-poles" },
                h("span", null, t("settings.deterministic")),
                h("span", null, t("settings.varied")),
              ),
              h("input", {
                id: "setting-temperature",
                "data-testid": "setting-temperature",
                type: "range",
                min: "0",
                max: "1",
                step: "0.05",
                value: temperature,
                onChange: (event) =>
                  setTemperature(
                    normalizeSliderPreference(event.target.value, 0),
                  ),
              }),
              h(
                "output",
                { htmlFor: "setting-temperature" },
                normalizeSliderPreference(temperature, 0).toFixed(2),
              ),
            ),
            h(
              "label",
              { className: "setting-check" },
              h("input", {
                type: "checkbox",
                checked: greetingVariations,
                onChange: (event) => setGreetingVariations(event.target.checked),
              }),
              h("span", null, t("settings.variations")),
            ),
            h(
              "label",
              { className: "setting-row" },
              h("span", null, t("settings.definitionFusion")),
              h(
                "select",
                {
                  "data-testid": "setting-definition-fusion",
                  value: definitionFusion,
                  onChange: (event) =>
                    setDefinitionFusion(
                      normalizeDefinitionFusion(event.target.value),
                    ),
                },
                h(
                  "option",
                  { value: "explicit" },
                  t("settings.definitionFusion.explicit"),
                ),
                h(
                  "option",
                  { value: "auto" },
                  t("settings.definitionFusion.auto"),
                ),
              ),
            ),
            h(
              "label",
              { className: "setting-row" },
              h("span", null, t("settings.blueprintComposition")),
              h(
                "select",
                {
                  "data-testid": "setting-blueprint-composition",
                  value: blueprintComposition,
                  onChange: (event) =>
                    setBlueprintComposition(
                      normalizeBlueprintComposition(event.target.value),
                    ),
                },
                h(
                  "option",
                  { value: "composed" },
                  t("settings.blueprintComposition.composed"),
                ),
                h(
                  "option",
                  { value: "documented" },
                  t("settings.blueprintComposition.documented"),
                ),
              ),
            ),
            h(
              "div",
              { className: "setting-row setting-row-ocr" },
              h(
                "label",
                { className: "setting-check" },
                h("input", {
                  type: "checkbox",
                  checked: experimentalOcr,
                  "data-testid": "setting-experimental-ocr",
                  onChange: (event) => setExperimentalOcr(event.target.checked),
                }),
                h("span", null, t("settings.experimentalOcr")),
              ),
              h(
                "p",
                {
                  className: "setting-warning",
                  "data-testid": "setting-experimental-ocr-warning",
                  title: OCR_DOWNLOAD_WARNING,
                },
                t("settings.experimentalOcr.warning"),
              ),
            ),
            h(
              "label",
              { className: "setting-row" },
              h("span", null, t("settings.language")),
              h(
                "select",
                {
                  "data-testid": "setting-ui-language",
                  value: uiLanguagePreference,
                  onChange: (event) =>
                    setUiLanguagePreference(
                      normalizeUiLanguagePreference(event.target.value),
                    ),
                },
                h("option", { value: "auto" }, t("settings.language.auto")),
                h("option", { value: "en" }, "English"),
                h("option", { value: "ru" }, "Русский"),
                h("option", { value: "zh" }, "中文"),
                h("option", { value: "hi" }, "हिन्दी"),
              ),
            ),
            h(
              "label",
              { className: "setting-row" },
              h("span", null, t("settings.responseLanguage")),
              h(
                "select",
                {
                  "data-testid": "setting-response-language",
                  value: responseLanguage,
                  onChange: (event) =>
                    setResponseLanguage(
                      normalizeResponseLanguageMode(event.target.value),
                    ),
                },
                h(
                  "option",
                  { value: "last_message" },
                  t("settings.responseLanguage.lastMessage"),
                ),
                h(
                  "option",
                  { value: "preferred" },
                  t("settings.responseLanguage.preferred"),
                ),
                h("option", { value: "ui" }, t("settings.responseLanguage.ui")),
              ),
            ),
            responseLanguage === "preferred"
              ? h(
                  "label",
                  { className: "setting-row" },
                  h("span", null, t("settings.preferredLanguage")),
                  h(
                    "select",
                    {
                      "data-testid": "setting-preferred-language",
                      value: preferredLanguage,
                      onChange: (event) =>
                        setPreferredLanguage(
                          normalizePreferredLanguage(event.target.value),
                        ),
                    },
                    h("option", { value: "en" }, "English"),
                    h("option", { value: "ru" }, "Русский"),
                    h("option", { value: "zh" }, "中文"),
                    h("option", { value: "hi" }, "हिन्दी"),
                  ),
                )
              : null,
            h(
              "label",
              { className: "setting-row" },
              h("span", null, t("settings.theme")),
              h(
                "select",
                {
                  "data-testid": "setting-theme",
                  value: themePreference,
                  onChange: (event) =>
                    setThemePreference(
                      normalizeThemePreference(event.target.value),
                    ),
                },
                h("option", { value: "auto" }, t("settings.theme.auto")),
                h("option", { value: "light" }, t("settings.theme.light")),
                h("option", { value: "dark" }, t("settings.theme.dark")),
              ),
            ),
            h(
              "label",
              { className: "setting-row" },
              h("span", null, t("settings.uiSkin")),
              h(
                "select",
                {
                  "data-testid": "setting-ui-skin",
                  value: uiSkin,
                  onChange: (event) =>
                    setUiSkin(normalizeUiSkin(event.target.value)),
                },
                h("option", { value: "flat" }, t("settings.uiSkin.flat")),
                h("option", { value: "glass" }, t("settings.uiSkin.glass")),
                h("option", { value: "contrast" }, t("settings.uiSkin.contrast")),
              ),
            ),
            h(
              "label",
              { className: "setting-row" },
              h("span", null, t("settings.chatStyle")),
              h(
                "select",
                {
                  "data-testid": "setting-chat-style",
                  value: chatStyle,
                  onChange: (event) =>
                    setChatStyle(normalizeChatStyle(event.target.value)),
                },
                h("option", { value: "cards" }, t("settings.chatStyle.cards")),
                h("option", { value: "compact" }, t("settings.chatStyle.compact")),
                h("option", { value: "bubbles" }, t("settings.chatStyle.bubbles")),
              ),
            ),
            h(
              "label",
              { className: "setting-row" },
              h("span", null, t("settings.composerStyle")),
              h(
                "select",
                {
                  "data-testid": "setting-composer-style",
                  value: composerStyle,
                  onChange: (event) =>
                    setComposerStyle(normalizeComposerStyle(event.target.value)),
                },
                h("option", { value: "flat" }, t("settings.composerStyle.flat")),
                h("option", { value: "glass-soft" }, t("settings.composerStyle.glassSoft")),
                h("option", { value: "glass-clear" }, t("settings.composerStyle.glassClear")),
                h("option", { value: "bubble" }, t("settings.composerStyle.bubble")),
              ),
            ),
            h(
              "label",
              { className: "setting-row" },
              h("span", null, t("settings.composerAction")),
              h(
                "select",
                {
                  "data-testid": "setting-composer-action",
                  value: composerAction,
                  onChange: (event) =>
                    setComposerAction(normalizeComposerAction(event.target.value)),
                },
                h("option", { value: "attach" }, t("settings.composerAction.attach")),
                h("option", { value: "plus" }, t("settings.composerAction.plus")),
              ),
            ),
            h(
              "label",
              { className: "setting-row" },
              h("span", null, t("settings.assistantName")),
              h("input", {
                "data-testid": "setting-assistant-name",
                type: "text",
                value: assistantName,
                maxLength: 64,
                placeholder: t("settings.assistantName.placeholder"),
                onChange: (event) =>
                  setAssistantName(sanitizeAssistantNameInput(event.target.value)),
              }),
            ),
            h(
              "label",
              { className: "setting-row" },
              h("span", null, t("settings.location")),
              h("input", {
                "data-testid": "setting-location",
                type: "text",
                value: locationPreference,
                placeholder: t("settings.location.placeholder"),
                onChange: (event) =>
                  setLocationPreference(event.target.value.slice(0, 80)),
              }),
            ),
          ),
        }),
        h(CollapsibleSection, {
          title: t("sidebar.examplePrompts"),
          testId: "sidebar-prompts",
          collapsed: sidebarPromptsCollapsed,
          onToggle: () => setSidebarPromptsCollapsed((value) => !value),
          children: h(
            "div",
            { className: "prompt-list", "data-testid": "example-prompts" },
            EXAMPLE_PROMPTS.map((entry) =>
              h(
                "button",
                {
                  key: entry.text,
                  type: "button",
                  "data-prompt-label": entry.label,
                  "data-prompt-text": entry.text,
                  onClick: () => {
                    setDemoMode(false);
                    setPrompt(entry.text);
                  },
                  title: entry.label,
                },
                entry.text,
              ),
            ),
          ),
        }),
        seed.tools && seed.tools.length > 0
          ? h(CollapsibleSection, {
              title: t("sidebar.tools"),
              testId: "sidebar-tools",
              collapsed: sidebarToolsCollapsed,
              onToggle: () => setSidebarToolsCollapsed((value) => !value),
              children: h(
                "div",
                { className: "tool-registry", "data-testid": "tool-registry" },
                h(
                  "ul",
                  { className: "tool-list" },
                  seed.tools.map((tool) => {
                    const displayTool = localizeTool(tool, uiLanguage);
                    return h(
                      "li",
                      {
                        key: tool.id,
                        className: `tool tool-mode-${tool.mode || "thinking"}`,
                        "data-testid": "tool-entry",
                        "data-tool-id": tool.id,
                        "data-tool-mode": tool.mode || "thinking",
                      },
                      h(
                        "div",
                        { className: "tool-head" },
                        h("strong", null, displayTool.name || tool.id),
                        h(
                          "span",
                          { className: "tool-mode" },
                          tool.mode === "agent"
                            ? t("toolMode.agent")
                            : t("toolMode.thinking"),
                        ),
                      ),
                      displayTool.description
                        ? h("p", { className: "tool-desc" }, displayTool.description)
                        : null,
                    );
                  }),
                ),
              ),
            })
          : null,
        diagnosticsMode
          ? h(CollapsibleSection, {
              title: t("sidebar.trace"),
              testId: "sidebar-trace",
              collapsed: sidebarTraceCollapsed,
              onToggle: () => setSidebarTraceCollapsed((value) => !value),
              children: h(
                "dl",
                { className: "trace-list" },
                h("div", null, h("dt", null, t("trace.model")), h("dd", null, "formal-symbolic-production")),
                h("div", null, h("dt", null, t("trace.mode")), h("dd", null, demoStatus)),
                h("div", null, h("dt", null, t("trace.intent")), h("dd", null, lastAssistant?.intent ?? "none")),
                h("div", null, h("dt", null, t("trace.data")), h("dd", null, "data/source-index.lino")),
                h(
                  "div",
                  null,
                  h("dt", null, t("trace.seedFiles")),
                  h(
                    "dd",
                    null,
                    Object.keys(seed.raw || {}).join(", ") || "(loading)",
                  ),
                ),
                h(
                  "div",
                  null,
                  h("dt", null, t("trace.toolsLoaded")),
                  h("dd", null, String((seed.tools || []).length)),
                ),
                h(
                  "div",
                  null,
                  h("dt", null, t("trace.conceptsLoaded")),
                  h("dd", null, String((seed.concepts || []).length)),
                ),
              ),
            })
          : null,
      ),
      h("div", {
        className: "context-resizer",
        "data-testid": "context-resizer",
        role: "separator",
        "aria-orientation": "vertical",
        "aria-label": t("titles.resizeSidebar"),
        "aria-valuemin": CONTEXT_PANEL_MIN_WIDTH,
        "aria-valuemax": contextPanelMaxWidth(),
        "aria-valuenow": contextPanelWidth,
        tabIndex: 0,
        title: t("titles.resizeSidebar"),
        onPointerDown: handleContextResizePointerDown,
        onKeyDown: handleContextResizeKeyDown,
      }),
      h(
        "section",
        { className: "chat-panel" },
        h(
          "section",
          { className: "messages", "aria-live": "polite", "data-testid": "message-list" },
          messages.map((message) =>
            h(Message, {
              key: message.id,
              message,
              diagnosticsMode,
              t,
              reportIssueUrl:
                shouldOfferMessageReport(message)
                  ? createIssueUrl({ ...reportContext, focusMessage: message })
                  : null,
            }),
          ),
          pending
            ? h(
                "article",
                { className: "message assistant pending" },
                h("div", { className: "avatar", "aria-hidden": "true" }, "FA"),
                h("div", { className: "message-body" }, h("div", { className: "typing" }, t("status.working"))),
              )
            : null,
          h("div", { ref: transcriptEndRef }),
        ),
        h(
          "form",
          {
            className: "composer",
            onSubmit: (event) => {
              event.preventDefault();
              send();
            },
          },
          h("input", {
            ref: attachmentInputRef,
            type: "file",
            multiple: true,
            style: { display: "none" },
            "data-testid": "composer-attachment-input",
            onChange: handleAttachFiles,
          }),
          demoMode
            ? h(
                "p",
                { className: "composer-demo-hint", "data-testid": "composer-demo-hint" },
                t("composer.demoHint.before"),
                h("span", { className: "composer-demo-hint-icon", "aria-hidden": "true" }, "🎬"),
                t("composer.demoHint.after"),
              )
            : null,
          composerMenuOpen
            ? h(
                "div",
                { className: "composer-menu", "data-testid": "composer-menu" },
                h(
                  "button",
                  {
                    type: "button",
                    className: "composer-menu-item",
                    onClick: triggerAttachFiles,
                  },
                  t("buttons.attachFiles"),
                ),
                h(
                  "button",
                  {
                    type: "button",
                    className: "composer-menu-item",
                    onClick: handleExportMemory,
                  },
                  t("buttons.exportMemory"),
                ),
                h(
                  "button",
                  {
                    type: "button",
                    className: "composer-menu-item",
                    onClick: triggerImportMemory,
                  },
                  t("buttons.importMemory"),
                ),
                h(
                  "a",
                  {
                    className: "composer-menu-item",
                    href: currentReportUrl,
                    target: "_blank",
                    rel: "noopener noreferrer",
                  },
                  t("buttons.reportIssue"),
                ),
              )
            : null,
          h(
            "div",
            { className: "composer-grid" },
            h(
              "button",
              {
                type: "button",
                className: "composer-action-button",
                "data-testid": "composer-menu-toggle",
                "aria-expanded": composerMenuOpen,
                "aria-label": t("buttons.composerMenu"),
                title: t("titles.composerMenu"),
                onClick: () => setComposerMenuOpen((value) => !value),
              },
              composerActionIcon,
            ),
            h("textarea", {
              ref: composerInputRef,
              value: prompt,
              rows: 1,
              placeholder: agentMode
                ? t("composer.placeholder.agent")
                : t("composer.placeholder.chat"),
              autoComplete: "off",
              autoCorrect: "off",
              autoCapitalize: "sentences",
              enterKeyHint: "send",
              inputMode: "text",
              spellCheck: true,
              onChange: (event) => setPrompt(event.target.value),
              onKeyDown: handleKeyDown,
              disabled: demoMode,
              "data-testid": "chat-composer-input",
            }),
            h(
              "button",
              {
                className: "send-button",
                type: "submit",
                disabled: pending || demoMode || (!prompt.trim() && attachments.length === 0),
                "data-testid": "chat-composer-submit",
              },
              pending
                ? h(
                    "span",
                    {
                      className: "send-spinner",
                      "aria-hidden": "true",
                      "data-testid": "send-spinner",
                    },
                  )
                : h(
                    "span",
                    { className: "send-icon", "aria-hidden": "true" },
                    "↑",
                  ),
              h(
                "span",
                { className: "send-label" },
                pending ? t("composer.sending") : t("composer.send"),
              ),
            ),
          ),
          attachmentStatus
            ? h(
                "p",
                { className: "composer-attachment-status", "data-testid": "composer-attachment-status" },
                attachmentStatus,
              )
            : null,
        ),
      ),
    ),
  );
}

function wait(milliseconds) {
  return new Promise((resolve) => {
    window.setTimeout(resolve, milliseconds);
  });
}

ReactDOM.createRoot(document.getElementById("root")).render(h(App));
