// Issue #550: the front-end is now authored as JSX and bundled by the bun
// bundler into the served `src/web/app.js` (see package.json `build:web`).
// React and ReactDOM are imported here so they are bundled into the app — a
// single React instance shared with @chakra-ui/react and @emotion/react (the
// vendor bundle no longer needs to expose them as globals for the app). The
// JSX factory stays bound to `h` so the existing `h(tag, props, ...children)`
// render calls — and the static guards that parse them (check-web-tdz,
// check-web-hardcoded-ui-strings) — keep working unchanged during the
// incremental migration to Chakra primitives.
import React from "react";
import { createRoot } from "react-dom/client";
import { ChakraProvider, chakra } from "@chakra-ui/react";

// Issue #550: the Chakra system bridges the app's --fa-* CSS design tokens into
// Chakra semantic tokens with the global reset/body styling disabled, so
// styles.css stays authoritative while the UI migrates to Chakra primitives.
import { system as chakraSystem } from "./theme.js";

const {
  createElement: h,
  Fragment,
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

function recognizeSeedInterfaceCommand(text, capabilities) {
  const normalized = normalizeMemoryPrompt(text);
  if (!normalized || !Array.isArray(capabilities)) return null;
  for (const capability of capabilities) {
    const phrases = (capability.phrases || []).map(normalizeMemoryPrompt);
    if (!includesAnyText(normalized, phrases)) continue;
    let value = null;
    if (capability.kind === "enum") {
      const option = (capability.options || []).find((candidate) =>
        (candidate.aliases || [])
          .map(normalizeMemoryPrompt)
          .some((alias) => normalized.includes(alias)),
      );
      if (option) value = option.value;
    } else if (capability.kind === "number") {
      const match = normalized.match(/(\d+(?:[.,]\d+)?)/);
      if (match) {
        const number = Number(match[1].replace(",", "."));
        if (Number.isFinite(number)) value = number * Number(capability.scale || 1);
      }
    } else if (capability.kind === "boolean") {
      value = detectToggleCommand(normalized, phrases);
    }
    if (value === null) continue;
    return {
      kind: "set_preference",
      key: capability.key,
      value,
      intent: capability.intent,
      label: capability.label,
    };
  }
  return null;
}

function recognizeInterfaceCommand(text, capabilities = []) {
  const normalized = normalizeMemoryPrompt(text);
  if (!normalized) return null;

  const seedCommand = recognizeSeedInterfaceCommand(text, capabilities);
  if (seedCommand) return seedCommand;

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

// Issue #444: the assistant may consult a small set of external *trusted*
// services (wikiHow, Stack Exchange, the MediaWiki sister projects, GitHub) when
// it answers procedural "how to X" prompts and project lookups. The maintainer
// asked for a settings section to opt in or out of each one. This single
// data-driven catalog drives the preference defaults, the worker prefs payload,
// the reset descriptors, and the settings panel checkboxes, so adding a new
// trusted service later means appending one row here (plus an i18n label) rather
// than editing six call sites. `key` mirrors the `settings_key` recorded in
// `data/seed/sources-registry.lino`; every service is opt-out (default enabled),
// so existing behavior is preserved unless the user turns one off.
const EXTERNAL_TRUSTED_SERVICES = [
  { key: "externalServiceWikihow", label: "settings.externalServiceWikihow" },
  { key: "externalServiceStackExchange", label: "settings.externalServiceStackExchange" },
  { key: "externalServiceMediawikiFamily", label: "settings.externalServiceMediawikiFamily" },
  { key: "externalServiceGithub", label: "settings.externalServiceGithub" },
];

const LEGACY_EXPANDED_SIDEBAR_KEYS = [
  "sidebarSettingsCollapsed",
  "sidebarToolsCollapsed",
  "sidebarTraceCollapsed",
  "sidebarDesktopCollapsed",
  "sidebarServicesCollapsed",
];

const PREFERENCE_DEFAULTS = {
  demoMode: true,
  diagnosticsMode: false,
  contextPanelWidth: 300,
  // Issue #27: each sidebar section is a VS Code-style collapsible region; the
  // last expand/collapse state is persisted via FormalAiPreferences so opening
  // the demo never reshuffles the user's layout.
  sidebarMenuCollapsed: true,
  sidebarPromptsCollapsed: false,
  sidebarToolsCollapsed: true,
  sidebarTraceCollapsed: true,
  sidebarConversationsCollapsed: false,
  sidebarSettingsCollapsed: true,
  sidebarDesktopCollapsed: true,
  sidebarServicesCollapsed: true,
  // Issue #153: the side panel is collapsible to give the chat full viewport
  // width on desktop. The drawer view on mobile stays controlled by the
  // separate `mobileMenuOpen` toggle so phones can still slide it in.
  sidebarCollapsed: false,
  showDeletedConversations: false,
  // Issue #27: random greeting variations are opt-in but default to on so
  // newcomers see the multilingual surface immediately.
  greetingVariations: true,
  // Issue #488: user-facing thinking can be compact or detailed without
  // changing the raw diagnostics available to maintainers.
  // Issue #541 (R8): default to the 50% midpoint ("standard"), which surfaces
  // only the high-level human-readable steps (not the mechanical sub-steps), so
  // newcomers are not overwhelmed; "detailed" remains one notch away for power
  // users and still renders fully human-readable prose (no symbolic syntax).
  thinkingDetailLevel: "standard",
  // Issue #541 (R5): minimum wall-clock time, in milliseconds, that a freshly
  // produced assistant answer spends animating its reasoning + reveal so the
  // user *feels* the thinking happen even when the deterministic engine answers
  // instantly. 0 = immediate display; the shipped default is a relaxed 2s.
  minMessageAnimationMs: 2000,
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
  // Issue #444: external trusted-service opt-outs. Defined data-driven from
  // EXTERNAL_TRUSTED_SERVICES; every service ships enabled (opt-out model) so the
  // assistant keeps consulting them unless the user disables one in settings.
  ...Object.fromEntries(EXTERNAL_TRUSTED_SERVICES.map((service) => [service.key, true])),
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
  // Issue #513: three-way operating mode replacing the binary agent toggle.
  //   "chat"     — single-turn reasoning, no command execution;
  //   "agent"    — multi-step plan + execute, capabilities gated on grant;
  //   "fullAuto" — agent mode that runs permitted commands automatically.
  // The legacy `agentMode` boolean is derived as `mode !== "chat"` so existing
  // readers (worker prefs, desktop tool grants) keep working unchanged.
  mode: "chat",
  // Issue #514: first-run Agent/Full Auto onboarding and per-tool grant
  // decisions are persisted independently. The grant string is a compact
  // Links-friendly map such as `shell:on,http_fetch:off`.
  agentOnboardingSeen: false,
  desktopToolGrants: "",
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
  toolbarIconPack: "fontawesome",
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
const TOOLBAR_ICON_PACKS = [
  "fontawesome",
  "material-symbols",
  "bootstrap-icons",
  "ionicons",
  "remix-icon",
  "tabler-icons",
  "names",
];
const DEFINITION_FUSION_MODES = ["explicit", "auto"];
// Issue #340: blueprint program-composition strategies. "composed" projects the
// program from the detected capabilities; "documented" always emits the full
// annotated program with every optional region present.
const BLUEPRINT_COMPOSITION_MODES = ["composed", "documented"];
const THINKING_DETAIL_LEVELS = ["brief", "standard", "detailed"];
// Issue #513: the three-way operating modes shown in the toolbar radio group.
const MODE_OPTIONS = ["chat", "agent", "fullAuto"];
const MODE_LABEL_KEYS = {
  chat: "buttons.chat",
  agent: "buttons.agent",
  fullAuto: "buttons.fullAuto",
};
const MODE_TITLE_KEYS = {
  chat: "titles.agentOff",
  agent: "titles.agentOn",
  fullAuto: "titles.fullAuto",
};
// Issue #514: the renderer mirrors the desktop tool vocabulary so it can send a
// per-tool grant map to the native router instead of the old all-or-nothing
// grant. Keep this list in sync with desktop/lib/tool-router.cjs.
const DESKTOP_TOOL_OPTIONS = Object.freeze([
  "http_fetch",
  "url_navigate",
  "eval_js",
  "read_local_file",
  "code_exec",
  "shell",
]);
// Issue #511/#514: per-tool labels and descriptions live in the i18n catalog
// (permissions.tool.<tool>.{label,description}) so the desktop permission panel
// translates with the active UI language instead of shipping hardcoded English.
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

const TEXT_ATTACHMENT_CONTEXT_LIMIT = 12000;

function isTextAttachment(file) {
  if (!file) return false;
  const type = String(file.type || "").toLowerCase();
  if (type.startsWith("text/")) return true;
  if (
    [
      "application/json",
      "application/ld+json",
      "application/javascript",
      "application/xml",
      "application/x-ndjson",
      "application/yaml",
      "application/x-yaml",
    ].includes(type)
  ) {
    return true;
  }
  return /\.(txt|md|markdown|csv|tsv|json|jsonl|lino|log|xml|html?|css|js|jsx|ts|tsx|rs|py|java|c|cc|cpp|h|hpp|go|rb|php|sh|ps1|sql|ya?ml|toml|ini|tex|rtf)$/i.test(
    String(file.name || ""),
  );
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

function readFileAsText(file) {
  if (file && typeof file.text === "function") {
    return file.text();
  }
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result || ""));
    reader.onerror = () => reject(reader.error || new Error("Unable to read file"));
    reader.readAsText(file);
  });
}

function sampleTextAttachmentContent(text, limit = TEXT_ATTACHMENT_CONTEXT_LIMIT) {
  const normalized = String(text || "").replace(/\r\n?/g, "\n").trim();
  if (normalized.length <= limit) {
    return { text: normalized, truncated: false };
  }
  const segment = Math.max(1000, Math.floor(limit / 3));
  const middleStart = Math.max(0, Math.floor((normalized.length - segment) / 2));
  return {
    text: [
      normalized.slice(0, segment).trim(),
      "[... omitted middle of text attachment ...]",
      normalized.slice(middleStart, middleStart + segment).trim(),
      "[... omitted middle of text attachment ...]",
      normalized.slice(normalized.length - segment).trim(),
    ].join("\n\n"),
    truncated: true,
  };
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
  if (attachment.text) record.text = attachment.text;
  if (attachment.textTruncated) record.textTruncated = true;
  if (attachment.textError) record.textError = attachment.textError;
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
    if (attachment.text) {
      lines.push(`Text excerpt: ${attachment.text}`);
      if (attachment.textTruncated) {
        lines.push("Text omitted: attachment excerpt was truncated before solver context.");
      }
    } else if (attachment.textError) {
      lines.push(`Text unavailable: ${attachment.textError}`);
    } else if (attachment.ocrText) {
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

function readStoredPreferencesRecord() {
  if (typeof window === "undefined" || !window.FormalAiPreferences) {
    return null;
  }
  const parser = window.FormalAiPreferences.parse;
  if (typeof parser !== "function") {
    return null;
  }
  try {
    const storage = window.localStorage;
    if (!storage) return null;
    const key = window.FormalAiPreferences.STORAGE_KEY || "formal-ai.preferences.v1";
    return parser(storage.getItem(key));
  } catch (_error) {
    return null;
  }
}

function migrateStoredSidebarCollapsePreferences(preferences, storedRecord) {
  if (!storedRecord || typeof storedRecord !== "object") {
    return preferences;
  }
  let next = preferences;
  for (const key of LEGACY_EXPANDED_SIDEBAR_KEYS) {
    if (!Object.prototype.hasOwnProperty.call(storedRecord, key)) {
      if (next === preferences) next = { ...preferences };
      next[key] = false;
    }
  }
  return next;
}

function loadPreferences() {
  if (typeof window === "undefined" || !window.FormalAiPreferences) {
    return { ...PREFERENCE_DEFAULTS };
  }
  try {
    const storedRecord = readStoredPreferencesRecord();
    const preferences = window.FormalAiPreferences.load(PREFERENCE_DEFAULTS);
    return migrateStoredSidebarCollapsePreferences(preferences, storedRecord);
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

function normalizeToolbarIconPack(value) {
  return TOOLBAR_ICON_PACKS.includes(value)
    ? value
    : PREFERENCE_DEFAULTS.toolbarIconPack;
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

function normalizeThinkingDetailLevel(value) {
  return THINKING_DETAIL_LEVELS.includes(value)
    ? value
    : PREFERENCE_DEFAULTS.thinkingDetailLevel;
}

// Issue #541 (R5): the minimum message-animation budget is a millisecond count
// clamped to a sane range. 0 means "show the answer immediately" (no animation);
// the upper bound keeps a mis-set preference from freezing the UI for minutes.
// Non-numeric / NaN input falls back to the shipped 2s default.
const MIN_MESSAGE_ANIMATION_MAX_MS = 8000;
function normalizeAnimationBudgetMs(value) {
  const number = typeof value === "number" ? value : Number(value);
  if (!Number.isFinite(number)) {
    return PREFERENCE_DEFAULTS.minMessageAnimationMs;
  }
  const clamped = Math.min(Math.max(number, 0), MIN_MESSAGE_ANIMATION_MAX_MS);
  return Math.round(clamped);
}

function normalizeResponseLanguageMode(value) {
  return RESPONSE_LANGUAGE_MODES.includes(value)
    ? value
    : PREFERENCE_DEFAULTS.responseLanguage;
}

// Issue #513: resolve the persisted operating mode. Falls back to the legacy
// `agentMode` boolean so users who saved preferences before the radio existed
// keep their agent opt-in (true -> "agent"), and unknown values reset to chat.
function normalizeMode(value, legacyAgentMode) {
  // A newer client that writes mode="chat" also writes the derived agentMode as
  // false, so the only way to see mode="chat" alongside a truthy legacy
  // agentMode is a pre-#513 preference store that only had `agentMode "on"`
  // (mode defaulting to "chat"). In that case upgrade to the agent radio.
  if (MODE_OPTIONS.includes(value) && !(value === "chat" && legacyAgentMode)) {
    return value;
  }
  return legacyAgentMode ? "agent" : PREFERENCE_DEFAULTS.mode;
}

function normalizeDesktopToolGrants(value) {
  const normalized = {};
  const apply = (tool, granted) => {
    if (!DESKTOP_TOOL_OPTIONS.includes(tool) || typeof granted !== "boolean") {
      return;
    }
    normalized[tool] = granted;
  };
  if (value && typeof value === "object" && !Array.isArray(value)) {
    DESKTOP_TOOL_OPTIONS.forEach((tool) => {
      if (value[tool] === true || value[tool] === false) {
        apply(tool, value[tool]);
      }
    });
    return normalized;
  }
  const text = String(value || "").trim();
  if (!text) {
    return normalized;
  }
  text.split(/[,;\n]+/).forEach((entry) => {
    const match = /^\s*([a-z0-9_]+)\s*[:=]\s*([a-z0-9_-]+)\s*$/i.exec(entry);
    if (!match) {
      return;
    }
    const state = match[2].toLowerCase();
    if (["on", "true", "1", "grant", "granted"].includes(state)) {
      apply(match[1], true);
    } else if (["off", "false", "0", "decline", "declined", "deny", "denied"].includes(state)) {
      apply(match[1], false);
    }
  });
  return normalized;
}

function serializeDesktopToolGrants(grants) {
  const safe = grants && typeof grants === "object" ? grants : {};
  return DESKTOP_TOOL_OPTIONS
    .filter((tool) => safe[tool] === true || safe[tool] === false)
    .map((tool) => `${tool}:${safe[tool] ? "on" : "off"}`)
    .join(",");
}

function desktopToolRouterGrants(mode, grants) {
  const active = mode !== "chat";
  const safe = grants && typeof grants === "object" ? grants : {};
  const out = { all: false };
  DESKTOP_TOOL_OPTIONS.forEach((tool) => {
    out[tool] = active && safe[tool] === true;
  });
  return out;
}

function desktopToolGrantCount(grants) {
  const safe = grants && typeof grants === "object" ? grants : {};
  return DESKTOP_TOOL_OPTIONS.filter((tool) => safe[tool] === true).length;
}

function desktopToolGrantState(grants, tool) {
  const safe = grants && typeof grants === "object" ? grants : {};
  if (safe[tool] === true) return "granted";
  if (safe[tool] === false) return "declined";
  return "undecided";
}

function normalizePreferredLanguage(value) {
  return PREFERRED_RESPONSE_LANGUAGES.includes(value)
    ? value
    : PREFERENCE_DEFAULTS.preferredLanguage;
}

const TOOLBAR_ICON_FONT_NAMES = {
  fontawesome: {
    sourceCode: "fa-code",
    download: "fa-download",
    reportIssue: "fa-bug",
    exportMemory: "fa-file-export",
    importMemory: "fa-file-import",
    resetMemory: "fa-broom",
    diagnostics: "fa-flask-vial",
    chat: "fa-comment-dots",
    agent: "fa-robot",
    demo: "fa-clapperboard",
    attachFiles: "fa-paperclip",
    isolateSection: "fa-up-right-and-down-left-from-center",
  },
  "material-symbols": {
    sourceCode: "code",
    download: "download",
    reportIssue: "bug_report",
    exportMemory: "upload_file",
    importMemory: "file_download",
    resetMemory: "cleaning_services",
    diagnostics: "science",
    chat: "chat_bubble",
    agent: "smart_toy",
    demo: "movie",
    attachFiles: "attach_file",
    isolateSection: "open_in_full",
  },
  "bootstrap-icons": {
    sourceCode: "bi-code-slash",
    download: "bi-download",
    reportIssue: "bi-bug",
    exportMemory: "bi-file-earmark-arrow-up",
    importMemory: "bi-file-earmark-arrow-down",
    resetMemory: "bi-eraser",
    diagnostics: "bi-flask",
    chat: "bi-chat-dots",
    agent: "bi-robot",
    demo: "bi-play-btn",
    attachFiles: "bi-paperclip",
    isolateSection: "bi-arrows-fullscreen",
  },
  ionicons: {
    sourceCode: "code-slash-outline",
    download: "download-outline",
    reportIssue: "bug-outline",
    exportMemory: "cloud-upload-outline",
    importMemory: "cloud-download-outline",
    resetMemory: "brush-outline",
    diagnostics: "flask-outline",
    chat: "chatbubble-ellipses-outline",
    agent: "hardware-chip-outline",
    demo: "videocam-outline",
    attachFiles: "attach-outline",
    isolateSection: "expand-outline",
  },
  "remix-icon": {
    sourceCode: "ri-code-s-slash-line",
    download: "ri-download-line",
    reportIssue: "ri-bug-line",
    exportMemory: "ri-file-upload-line",
    importMemory: "ri-file-download-line",
    resetMemory: "ri-brush-3-line",
    diagnostics: "ri-flask-line",
    chat: "ri-chat-3-line",
    agent: "ri-robot-2-line",
    demo: "ri-movie-line",
    attachFiles: "ri-attachment-line",
    isolateSection: "ri-fullscreen-line",
  },
  "tabler-icons": {
    sourceCode: "IconCode",
    download: "IconDownload",
    reportIssue: "IconBug",
    exportMemory: "IconFileExport",
    importMemory: "IconFileImport",
    resetMemory: "IconEraser",
    diagnostics: "IconFlask",
    chat: "IconMessageCircle",
    agent: "IconRobot",
    demo: "IconMovie",
    attachFiles: "IconPaperclip",
    isolateSection: "IconArrowsMaximize",
  },
  names: {
    sourceCode: "Code",
    download: "Download",
    reportIssue: "Bug",
    exportMemory: "Export",
    importMemory: "Import",
    resetMemory: "Reset",
    diagnostics: "Diagnostics",
    chat: "Chat",
    agent: "Agent",
    demo: "Demo",
    attachFiles: "Attach",
    isolateSection: "Only",
  },
};

const TOOLBAR_ICON_SHORT_NAMES = {
  sourceCode: "Code",
  download: "Down",
  reportIssue: "Bug",
  exportMemory: "Out",
  importMemory: "In",
  resetMemory: "Clear",
  diagnostics: "Diag",
  chat: "Chat",
  agent: "Agent",
  demo: "Demo",
  attachFiles: "File",
  isolateSection: "One",
};

const TOOLBAR_ICON_SHAPES = {
  sourceCode: [
    ["path", { d: "M8.5 8.5 5 12l3.5 3.5" }],
    ["path", { d: "m15.5 8.5 3.5 3.5-3.5 3.5" }],
    ["path", { d: "m13.5 6-3 12" }],
  ],
  download: [
    ["path", { d: "M12 5v10" }],
    ["path", { d: "m8 11 4 4 4-4" }],
    ["path", { d: "M5 19h14" }],
  ],
  reportIssue: [
    ["path", { d: "M8 9h8v6a4 4 0 0 1-8 0V9Z" }],
    ["path", { d: "M9 9 7 6" }],
    ["path", { d: "m15 9 2-3" }],
    ["path", { d: "M12 8V5" }],
    ["path", { d: "M6 13H3.5" }],
    ["path", { d: "M20.5 13H18" }],
    ["path", { d: "M7 18l-2 2" }],
    ["path", { d: "m17 18 2 2" }],
    ["path", { d: "M10 13h.01" }],
    ["path", { d: "M14 13h.01" }],
  ],
  exportMemory: [
    ["path", { d: "M6 3h8l4 4v14H6V3Z" }],
    ["path", { d: "M14 3v5h4" }],
    ["path", { d: "M12 17V9" }],
    ["path", { d: "m8.5 12.5 3.5-3.5 3.5 3.5" }],
  ],
  importMemory: [
    ["path", { d: "M6 3h8l4 4v14H6V3Z" }],
    ["path", { d: "M14 3v5h4" }],
    ["path", { d: "M12 9v8" }],
    ["path", { d: "m8.5 13.5 3.5 3.5 3.5-3.5" }],
  ],
  resetMemory: [
    ["path", { d: "m15 4-7 7" }],
    ["path", { d: "m7 12 5 5" }],
    ["path", { d: "m5 14 5 5" }],
    ["path", { d: "m9 10 5 5" }],
    ["path", { d: "M4 20h16" }],
  ],
  diagnostics: [
    ["path", { d: "M10 4h4" }],
    ["path", { d: "M11 4v5l-5 8a3 3 0 0 0 2.5 4h7a3 3 0 0 0 2.5-4l-5-8V4" }],
    ["path", { d: "M8 16h8" }],
  ],
  chat: [
    ["path", { d: "M5 6h14v9H9l-4 4V6Z" }],
    ["path", { d: "M8.5 10.5h7" }],
    ["path", { d: "M8.5 13h4" }],
  ],
  agent: [
    ["path", { d: "M8 9h8v8H8V9Z" }],
    ["path", { d: "M12 9V5" }],
    ["path", { d: "M9.5 5h5" }],
    ["path", { d: "M6 12H4" }],
    ["path", { d: "M20 12h-2" }],
    ["path", { d: "M10 12h.01" }],
    ["path", { d: "M14 12h.01" }],
    ["path", { d: "M10 15h4" }],
  ],
  demo: [
    ["path", { d: "M4 7h16v12H4V7Z" }],
    ["path", { d: "M4 11h16" }],
    ["path", { d: "m7 7 2-4" }],
    ["path", { d: "m12 7 2-4" }],
    ["path", { d: "m17 7 2-4" }],
    ["path", { d: "m10 14 4 2-4 2v-4Z" }],
  ],
  attachFiles: [
    ["path", { d: "m8 12 5.5-5.5a3 3 0 0 1 4.25 4.25l-7.25 7.25a5 5 0 0 1-7.07-7.07L10 4.36" }],
    ["path", { d: "m9.5 14.5 6-6" }],
  ],
  isolateSection: [
    ["path", { d: "M8 3H3v5" }],
    ["path", { d: "M3 3l6 6" }],
    ["path", { d: "M16 3h5v5" }],
    ["path", { d: "M21 3l-6 6" }],
    ["path", { d: "M8 21H3v-5" }],
    ["path", { d: "M3 21l6-6" }],
    ["path", { d: "M16 21h5v-5" }],
    ["path", { d: "M21 21l-6-6" }],
  ],
};

function toolbarIconFontName(action, pack) {
  const normalizedPack = normalizeToolbarIconPack(pack);
  return (
    TOOLBAR_ICON_FONT_NAMES[normalizedPack]?.[action] ||
    TOOLBAR_ICON_FONT_NAMES.fontawesome[action] ||
    action
  );
}

function toolbarIconFontClass(action, pack) {
  const normalizedPack = normalizeToolbarIconPack(pack);
  const name = toolbarIconFontName(action, normalizedPack);
  if (normalizedPack === "fontawesome") return `fa-solid ${name}`;
  if (normalizedPack === "material-symbols") return `material-symbols-outlined ${name}`;
  if (normalizedPack === "bootstrap-icons") return `bi ${name}`;
  if (normalizedPack === "remix-icon") return `ri ${name}`;
  if (normalizedPack === "tabler-icons") return `ti ${name}`;
  return name;
}

function ToolbarIcon({ action, pack, className = "btn-icon" }) {
  const normalizedPack = normalizeToolbarIconPack(pack);
  const fontName = toolbarIconFontName(action, normalizedPack);
  const fontClass = toolbarIconFontClass(action, normalizedPack);
  const baseClass = `${className} toolbar-icon icon-pack-${normalizedPack}`;
  if (normalizedPack === "names") {
    return <span className={`${baseClass} toolbar-icon-name`} aria-hidden="true" data-icon-pack={normalizedPack} data-icon-font-name={fontName} data-icon-font-class={fontClass}>{TOOLBAR_ICON_SHORT_NAMES[action] || fontName}</span>;
  }
  const shape = TOOLBAR_ICON_SHAPES[action] || TOOLBAR_ICON_SHAPES.chat;
  return <span className={baseClass} aria-hidden="true" data-icon-pack={normalizedPack} data-icon-font-name={fontName} data-icon-font-class={fontClass}><svg className="toolbar-icon-svg" viewBox="0 0 24 24" focusable="false">{shape.map(([tag, attrs], index) => {
      // The SVG primitive tag (path/circle/rect/…) is data-driven, so alias it
      // to a capitalised name: JSX treats lowercase element names as literal
      // string tags, only Capitalised names resolve to a variable. <Shape …/>
      // compiles to h(Shape, {…}) === h(tag, {…}).
      const Shape = tag;
      return <Shape {...attrs} key={`${action}-${index}`} />;
    })}</svg></span>;
}

// Reusable top-menu control (issue #550). Every topbar button/link was hand
// written as the same icon + localized `.btn-label` span pair, so they drifted:
// some gained a hover/focus treatment and some didn't (the P5 "only some
// buttons react to hover" defect). Routing them all through ONE component means
// they share the same markup contract — `.btn-label`, the icon, and the
// className that the single CSS hover/focus rule targets — so a new control
// cannot silently miss the shared affordance. Renders an <a> when `href` is a
// string, otherwise a <button type="button">. Localized text stays at the call
// site as `label`/`title`/`ariaLabel` props (still real `t(...)` calls, so the
// hardcoded-UI check keeps passing); this component never embeds prose.
// Pairs with the `--fa-control-*` design tokens in styles.css (the shared
// hover/active/focus treatment); see docs/case-studies/issue-550 for the full
// rationale (M2 reusable-component requirement + the Chakra/CSP ADR).
function ToolbarButton({
  className,
  label,
  icon,
  iconPack,
  href,
  onClick,
  title,
  ariaLabel,
  testId,
  menuPriority,
  target,
  rel,
  type = "button",
  extraProps = null,
  children = null,
}) {
  const isLink = typeof href === "string";
  const props = { className };
  if (title !== undefined) props.title = title;
  if (ariaLabel !== undefined) props["aria-label"] = ariaLabel;
  if (testId !== undefined) props["data-testid"] = testId;
  if (menuPriority !== undefined) props["data-menu-priority"] = menuPriority;
  if (isLink) {
    props.href = href;
    if (target !== undefined) props.target = target;
    if (rel !== undefined) props.rel = rel;
  } else {
    props.type = type;
    if (onClick) props.onClick = onClick;
  }
  // Caller-supplied attributes (aria-pressed, role, data-mode, key, …) for the
  // segmented/toggle controls. Merged last so a control can extend the shared
  // contract without forking it.
  if (extraProps) Object.assign(props, extraProps);
  // Render through the Chakra styled factory (chakra.a / chakra.button). These
  // are the low-level primitives — they carry no component recipe, so no Chakra
  // styling is imposed; the element keeps its className and styles.css stays
  // authoritative (preflight is off). This is the safe first step of the
  // h() → JSX + Chakra migration: identical DOM and computed styles.
  const Tag = isLink ? chakra.a : chakra.button;
  return (
    <Tag {...props}>
      {icon ? <ToolbarIcon action={icon} pack={iconPack} /> : null}
      {label !== undefined && label !== null ? (
        <chakra.span className="btn-label">{label}</chakra.span>
      ) : null}
      {children}
    </Tag>
  );
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
  toolbarIconPack,
  locationPreference,
  assistantName,
  guessProbability,
  temperature,
  followUpProbability,
  definitionFusion,
  thinkingDetailLevel,
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
    toolbarIconPack,
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
    thinkingDetailLevel,
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
  const thinkingDetailLevel =
    safe.thinkingDetailLevel || PREFERENCE_DEFAULTS.thinkingDetailLevel;
  if (thinkingDetailLevel !== PREFERENCE_DEFAULTS.thinkingDetailLevel) {
    push("Thinking detail", thinkingDetailLevel);
  }
  const toolbarIconPack = safe.toolbarIconPack || PREFERENCE_DEFAULTS.toolbarIconPack;
  if (toolbarIconPack !== PREFERENCE_DEFAULTS.toolbarIconPack) {
    push("Toolbar icon pack", toolbarIconPack);
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
    // Issue #541 (R4): demo turns live in their own dedicated conversation
    // and must never surface in the user's sidebar — listing them would
    // suggest the user can navigate into and edit them, breaking the
    // "demo never overwrites your work" guarantee.
    if (event.isDemo) {
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
    // Issue #541 (R4): defensive — if a caller ever asks for a demo
    // conversation id, only return its demo-flagged events; conversely a
    // real conversation id must skip any stray demo-flagged event that
    // somehow shares the id.
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

function humanizeThinkingIdentifier(value) {
  return String(value || "")
    .replace(/^agent_\d+_/i, "")
    .replace(/^try(?=[A-Z])/u, "")
    .replace(/^handle(?=[A-Z])/u, "")
    .replace(/([a-z0-9])([A-Z])/gu, "$1 $2")
    .replace(/[_:.-]+/gu, " ")
    .replace(/\s+/gu, " ")
    .trim()
    .toLowerCase();
}

function thinkingLanguageLabel(value, t) {
  const code = String(value || "").toLowerCase().split(/[-_]/u)[0];
  if (["en", "ru", "zh", "hi"].includes(code)) {
    return t(`message.thinkingLanguage.${code}`);
  }
  return code || t("message.thinkingLanguage.unknown");
}

function thinkingRouteLabel(value, t) {
  const label = humanizeThinkingIdentifier(value);
  if (!label) return t("message.thinkingRoute.reply");
  if (label === "greeting") return t("message.thinkingRoute.greeting");
  if (label === "farewell") return t("message.thinkingRoute.farewell");
  if (label === "unknown") return t("message.thinkingRoute.unknown");
  return t("message.thinkingRoute.generic", { route: label });
}

function thinkingRuleLabel(value, t) {
  const label = humanizeThinkingIdentifier(value);
  if (!label) return t("message.thinkingRule.selected");
  if (label === "greeting") return t("message.thinkingRule.greeting");
  if (label === "farewell") return t("message.thinkingRule.farewell");
  if (label === "unknown") return t("message.thinkingRule.unknown");
  return label;
}

function thinkingToolLabel(value) {
  const label = humanizeThinkingIdentifier(value);
  return label || "local";
}

function truncateThinkingSummary(value) {
  const text = String(value || "").trim();
  if (text.length <= 96) return text;
  return `${text.slice(0, 93).trimEnd()}...`;
}

function summarizeThinkingDetail(value) {
  if (value === null || value === undefined) return "";
  if (typeof value === "string") {
    return truncateThinkingSummary(humanizeThinkingIdentifier(value));
  }
  if (typeof value === "number" || typeof value === "boolean") {
    return String(value);
  }
  if (Array.isArray(value)) {
    return `${value.length} item(s)`;
  }
  if (typeof value === "object") {
    const keys = Object.keys(value).slice(0, 3).map(humanizeThinkingIdentifier);
    return keys.length > 0 ? keys.join(", ") : "structured data";
  }
  return truncateThinkingSummary(value);
}

// Preserve a concrete detail value verbatim (the user's prompt, the computed
// result, the composed answer) while bounding its length, mirroring the Rust
// `truncate_thinking_detail` helper (600 chars, ellipsis suffix). Unlike
// `summarizeThinkingDetail` this does NOT lowercase or strip punctuation, so the
// real content survives into the naturalized sentence.
//
// Issue #1963 (P2 "Thinking steps are not fully written, some parts are
// omitted."): the cap was raised 120 -> 600 so realistic single-step detail
// renders in full instead of being clipped mid-sentence. Keep this constant in
// sync with the Rust `truncate_thinking_detail` helper.
function thinkingDetailText(detail) {
  if (detail === null || detail === undefined) return "";
  const text = String(detail).trim();
  if (text.length === 0) return "";
  const chars = Array.from(text);
  if (chars.length <= 600) return text;
  return `${chars.slice(0, 599).join("").trimEnd()}…`;
}

// English indefinite article for a phrase, mirroring the Rust `indefinite_article`
// helper so the English "Formalize the request as {article} {task} task." reads
// grammatically. Languages without articles simply ignore the {article} param.
function thinkingIndefiniteArticle(phrase) {
  const first = String(phrase || "").trimStart().charAt(0).toLowerCase();
  return ["a", "e", "i", "o", "u"].includes(first) ? "an" : "a";
}

// Issue #541 (R8): map the formalization operation (the `OP:*` verb) to a plain,
// localized task noun ("greeting", "calculation", "search", …) so the human
// reasoning view can describe what the request was understood as WITHOUT leaking
// the raw Links-notation tuple. The symbolic tuple stays in the diagnostics
// panel; the default trace stays human-readable per R8 ("no special syntax").
const FORMALIZATION_OP_LABEL_KEYS = {
  greet: "formalizeOpGreet",
  farewell: "formalizeOpFarewell",
  express: "formalizeOpExpress",
  compute: "formalizeOpCompute",
  define: "formalizeOpDefine",
  lookup: "formalizeOpLookup",
  search: "formalizeOpSearch",
  procedure: "formalizeOpProcedure",
  identify: "formalizeOpIdentify",
};
function formalizationOpLabel(formalization, t) {
  if (!formalization || typeof formalization !== "object") return "";
  const op = String(formalization.verb || formalization.op || "")
    .replace(/^OP:/i, "")
    .trim()
    .toLowerCase();
  const key = FORMALIZATION_OP_LABEL_KEYS[op];
  return key ? t(`message.thinkingStep.${key}`) : "";
}

// Translate a single structured thinking step into one concrete, human-readable
// sentence in the active UI language. This is stage 2 of the issue #488 pipeline
// ("translate the meta-language description into the target user language"):
// every known step kind threads its *concrete* detail (the prompt, the computed
// result, the looked-up entity, the composed answer) into a localized template,
// so the trace reads as real reasoning rather than generic category labels.
// Unknown kinds fall back to the meta-language `summary` the Rust solver already
// computed, then to a generic humanized label.
function naturalizeThinkingStep(entry, t) {
  const rawStep = String(entry?.step || "step");
  const step = rawStep.replace(/^agent_\d+_/i, "");
  const detail = entry?.detail;
  const value = thinkingDetailText(detail);
  const hasDetail = value.length > 0;

  if (rawStep !== step) {
    return t("message.thinkingStep.agentSubstep", {
      summary: naturalizeThinkingStep({ ...entry, step }, t),
    });
  }

  switch (step) {
    case "impulse":
      return hasDetail
        ? t("message.thinkingStep.impulse", { prompt: value })
        : t("message.thinkingStep.impulsePlain");
    case "detect_language":
      return t("message.thinkingStep.detectLanguage", {
        language: thinkingLanguageLabel(detail, t),
      });
    case "resolve_response_language":
      return t("message.thinkingStep.resolveResponseLanguage", {
        language: thinkingLanguageLabel(detail, t),
      });
    case "formalize": {
      // Issue #541 (R8): keep the human reasoning view free of symbolic syntax.
      // The browser solver formalizes into a Links-notation tuple before the
      // route is known — that tuple lives only in the diagnostics panel. Here we
      // project the operation to a plain task noun ("greeting", "calculation",
      // "search", …). The Rust solver instead reports the resolved task route in
      // `detail` (e.g. "greeting"), which we humanize directly.
      const opLabel = formalizationOpLabel(entry?.formalization, t);
      if (opLabel) {
        return t("message.thinkingStep.formalize", {
          task: opLabel,
          article: thinkingIndefiniteArticle(opLabel),
        });
      }
      if (!hasDetail) return t("message.thinkingStep.formalizePlain");
      const task = humanizeThinkingIdentifier(detail);
      return t("message.thinkingStep.formalize", {
        task,
        article: thinkingIndefiniteArticle(task),
      });
    }
    case "formalize_resolved": {
      // Issue #541 (R8): never surface the resolved (@USER OP:… Q-id) tuple in
      // the human trace. The browser solver only has an opaque resolved id here
      // (its `detail` still embeds the tuple), so fall back to the plain
      // phrasing; a solver that reports a concrete, syntax-free entity name in
      // `detail` keeps it.
      if (entry?.formalization) {
        return t("message.thinkingStep.formalizeResolvedPlain");
      }
      const looksSymbolic = /[()@?]|OP:|->|⇒/.test(value);
      return hasDetail && !looksSymbolic
        ? t("message.thinkingStep.formalizeResolved", {
            entity: humanizeThinkingIdentifier(detail),
          })
        : t("message.thinkingStep.formalizeResolvedPlain");
    }
    case "clarify_formalization":
      return hasDetail
        ? t("message.thinkingStep.clarifyFormalization", { options: value })
        : t("message.thinkingStep.clarifyFormalizationPlain");
    case "dispatch_handler":
      return hasDetail
        ? t("message.thinkingStep.dispatchHandler", {
            route: thinkingRouteLabel(detail, t),
          })
        : t("message.thinkingStep.dispatchHandlerPlain");
    case "route_attempt":
      return hasDetail
        ? t("message.thinkingStep.routeAttempt", {
            route: thinkingRouteLabel(detail, t),
          })
        : t("message.thinkingStep.routeAttemptPlain");
    case "match_rule":
      return hasDetail
        ? t("message.thinkingStep.matchRule", {
            rule: thinkingRuleLabel(detail, t),
          })
        : t("message.thinkingStep.matchRulePlain");
    case "compute":
      return hasDetail
        ? t("message.thinkingStep.compute", { expression: value })
        : t("message.thinkingStep.computePlain");
    case "compute_engine":
      return hasDetail
        ? t("message.thinkingStep.computeEngine", {
            engine: humanizeThinkingIdentifier(detail),
          })
        : t("message.thinkingStep.computeEnginePlain");
    case "compute_expression":
      return t("message.thinkingStep.computeExpression", { expression: value });
    case "compute_steps":
      return t("message.thinkingStep.computeSteps", { count: value });
    case "lookup_fact":
      return hasDetail
        ? t("message.thinkingStep.lookupFact", {
            fact: humanizeThinkingIdentifier(detail),
          })
        : t("message.thinkingStep.lookupFactPlain");
    case "invoke_tool":
      return hasDetail
        ? t("message.thinkingStep.invokeTool", {
            tool: thinkingToolLabel(detail),
          })
        : t("message.thinkingStep.invokeToolPlain");
    case "rule_verification":
      return hasDetail
        ? t("message.thinkingStep.ruleVerification", {
            rule: humanizeThinkingIdentifier(detail),
          })
        : t("message.thinkingStep.ruleVerificationPlain");
    case "policy_refusal":
      return hasDetail
        ? t("message.thinkingStep.policyRefusal", {
            policy: humanizeThinkingIdentifier(detail),
          })
        : t("message.thinkingStep.policyRefusalPlain");
    case "rule_construction":
      return t("message.thinkingStep.ruleConstruction");
    case "coreference_binding":
      return t("message.thinkingStep.coreferenceBinding");
    case "modifier_detection":
      return t("message.thinkingStep.modifierDetection");
    case "program_plan":
      return hasDetail
        ? t("message.thinkingStep.programPlan", {
            plan: humanizeThinkingIdentifier(detail),
          })
        : t("message.thinkingStep.programPlanPlain");
    case "scan_memory":
      return hasDetail
        ? t("message.thinkingStep.scanMemory", { term: value })
        : t("message.thinkingStep.scanMemoryPlain");
    case "deformalize": {
      // Prefer the clean composed answer the browser solver attaches as
      // `answer`; the Rust/API solver already sends the answer text as the
      // detail. The raw worker `detail` is the symbolic projection summary
      // (with the ⇒ glyph) reserved for the diagnostics panel, so it is not
      // used here.
      const answerText = thinkingDetailText(
        entry?.answer !== undefined && entry?.answer !== null
          ? entry.answer
          : detail,
      );
      return answerText
        ? t("message.thinkingStep.deformalize", { answer: answerText })
        : t("message.thinkingStep.deformalizePlain");
    }
    case "agent_plan":
      return hasDetail
        ? t("message.thinkingStep.agentPlan", {
            task: humanizeThinkingIdentifier(detail),
          })
        : t("message.thinkingStep.agentPlanPlain");
    case "fallback":
      return t("message.thinkingStep.fallback");
    case "http_chat":
      return t("message.thinkingStep.httpChat");
    case "memory":
      return t("message.thinkingStep.memory");
    case "extract_term":
      return t("message.thinkingStep.extractTerm");
    case "group_by_conversation":
      return t("message.thinkingStep.groupByConversation");
    // ---- Browser-only steps (no Rust solver counterpart) ----
    case "user_context":
      return t("message.thinkingStep.userContext", {
        context:
          summarizeThinkingDetail(detail) ||
          t("message.thinkingStep.userContextDefault"),
      });
    case "desktop_shell":
      return t("message.thinkingStep.desktopShell");
    case "trigger_button":
      return t("message.thinkingStep.triggerButton", {
        action: summarizeThinkingDetail(detail) || "button",
      });
    case "apply_message_command":
      return t("message.thinkingStep.applyMessageCommand", {
        command: summarizeThinkingDetail(detail) || "setting",
      });
    case "trigger_message_action":
      return t("message.thinkingStep.triggerMessageAction", {
        action: summarizeThinkingDetail(detail) || "action",
      });
    default: {
      // Unknown step kind: prefer the concrete meta-language summary the Rust
      // solver already computed (issue #488 pipeline stage 1), then fall back to
      // a generic humanized label so nothing renders as a bare identifier.
      const summary = String(entry?.summary || "").trim();
      if (summary) return summary;
      const readableStep = humanizeThinkingIdentifier(step) || "step";
      const readableDetail = summarizeThinkingDetail(detail);
      return t("message.thinkingStep.generic", {
        step: readableStep,
        detail: readableDetail ? `: ${readableDetail}` : "",
      });
    }
  }
}

function thinkingStepKey(entry) {
  return String(entry?.step || "").replace(/^agent_\d+_/i, "");
}

function filterThinkingEntriesForDetail(entries, detailLevel) {
  const safeEntries = Array.isArray(entries) ? entries.filter(Boolean) : [];
  if (safeEntries.length <= 1) return safeEntries;
  const level = normalizeThinkingDetailLevel(detailLevel);
  if (level === "detailed") return safeEntries;
  if (level === "brief") return safeEntries.slice(-1);

  // Medium (default) granularity: show the high-level universal-algorithm
  // phases plus the final step, recursively folding composite internals (the
  // calculator trace, memory scans, tool calls) out of view. Prefer the
  // structured `level` field the solver now emits ("high" for phases,
  // "detailed" for nested children); fall back to a step-name allowlist for
  // legacy entries that predate the level metadata.
  const lastIndex = safeEntries.length - 1;
  const hasLevels = safeEntries.some(
    (entry) => typeof entry?.level === "string" && entry.level.length > 0,
  );
  if (hasLevels) {
    const filtered = safeEntries.filter(
      (entry, index) => index === lastIndex || entry?.level === "high",
    );
    return filtered.length > 0 ? filtered : safeEntries.slice(-1);
  }

  const standardSteps = new Set([
    "impulse",
    "detect_language",
    "resolve_response_language",
    "clarify_formalization",
    "match_rule",
    "fallback",
    "user_context",
    "deformalize",
    "program_plan",
    "desktop_shell",
    "http_chat",
    "memory",
    "agent_plan",
  ]);
  const filtered = safeEntries.filter(
    (entry, index) =>
      index === lastIndex || standardSteps.has(thinkingStepKey(entry)),
  );
  return filtered.length > 0 ? filtered : safeEntries.slice(-1);
}

function filterThinkingSummariesForDetail(summaries, detailLevel) {
  const safeSummaries = Array.isArray(summaries)
    ? summaries.map((step) => String(step || "").trim()).filter(Boolean)
    : [];
  if (safeSummaries.length <= 1) return safeSummaries;
  const level = normalizeThinkingDetailLevel(detailLevel);
  if (level === "detailed") return safeSummaries;
  if (level === "brief") return safeSummaries.slice(-1);
  return safeSummaries.length > 4
    ? [safeSummaries[0], ...safeSummaries.slice(-3)]
    : safeSummaries;
}

function buildThinkingPreviewSteps(
  structuredSteps,
  answer,
  source,
  t,
  detailLevel,
) {
  if (Array.isArray(structuredSteps) && structuredSteps.length > 0) {
    return filterThinkingEntriesForDetail(structuredSteps, detailLevel)
      .map((entry) => naturalizeThinkingStep(entry, t))
      .filter(Boolean);
  }
  return filterThinkingSummariesForDetail(
    [
      t("message.thinkingStep.fallbackNormalize"),
      t("message.thinkingStep.fallbackIntent", {
        intent: humanizeThinkingIdentifier(answer?.intent || "unknown"),
      }),
      t("message.thinkingStep.fallbackRender", {
        source: humanizeThinkingIdentifier(source || "fallback"),
      }),
    ],
    detailLevel,
  );
}

function buildMessageThinkingPreviewSteps(message, t, detailLevel) {
  if (message?.role !== "assistant") return [];
  const diagnosticsSteps = Array.isArray(message.diagnosticsSteps)
    ? message.diagnosticsSteps
    : [];
  if (diagnosticsSteps.length > 0) {
    return buildThinkingPreviewSteps(
      diagnosticsSteps,
      message,
      message.thinkingPreviewSource || message.intent || "local",
      t,
      detailLevel,
    );
  }
  return filterThinkingSummariesForDetail(
    message.thinkingPreviewSteps ?? [],
    detailLevel,
  );
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
  return <div className="formalization-view" data-testid="formalization">{formalization.raw ? <div className="formalization-raw"><code>{formalization.raw}</code><span className="formalization-arrow" aria-hidden="true">{"→"}</span><code className="formalization-tuple">{formalization.tuple}</code></div> : <code className="formalization-tuple">{formalization.tuple}</code>}<div className="formalization-svo"><span className="formalization-svo-label">{t("message.formalizationSubjectVerbObject")}</span><ol className="formalization-svo-list"><li><span className="formalization-slot">{"S"}</span><code>{formalization.subject || ""}</code></li><li><span className="formalization-slot">{"V"}</span><code>{formalization.verb || ""}</code></li><li><span className="formalization-slot">{"O"}</span><code>{formalization.object || ""}</code></li></ol></div></div>;
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

function localBehaviorRulesCount(runtimeRules, language = "en") {
  const runtimeRuleCount = Array.isArray(runtimeRules) ? runtimeRules.length : 0;
  const builtInCount = localBehaviorRuleRecords().length;
  const total = builtInCount + runtimeRuleCount;
  const summary = localLocalizedText(language, {
    en: `Total behavior rules: ${total} (built-in: ${builtInCount}; dialog-local: ${runtimeRuleCount}).`,
    ru: `Всего правил: ${total} (встроенных: ${builtInCount}; изученных в этом диалоге: ${runtimeRuleCount}).`,
    hi: `कुल व्यवहार नियम: ${total} (built-in: ${builtInCount}; dialog-local: ${runtimeRuleCount}).`,
    zh: `行为规则总数：${total}（内置：${builtInCount}；本对话：${runtimeRuleCount}）。`,
  });
  const reasoning = localLocalizedText(language, {
    en: "Reasoning: I count the built-in behavior-rule catalog and add dialog-local rules compiled from earlier user turns.",
    ru: "Рассуждение: я считаю встроенный каталог правил поведения и добавляю правила, скомпилированные из предыдущих сообщений пользователя.",
    hi: "Reasoning: मैं built-in behavior-rule catalog गिनता हूँ और पहले user turns से compiled dialog-local rules जोड़ता हूँ.",
    zh: "Reasoning：我统计内置行为规则目录，并加上从此前用户消息编译出的本对话规则。",
  });
  return [
    summary,
    "",
    reasoning,
    "",
    "```links",
    "behavior_rules_count",
    `  built_in_rules "${builtInCount}"`,
    `  dialog_local_rules "${runtimeRuleCount}"`,
    `  total_rules "${total}"`,
    '  algorithm "localBehaviorRuleRecords + localCollectRuntimeRules(history:user)"',
    "```",
  ].join("\n");
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
    '  object "formal-ai"',
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
    '  object "formal-ai"',
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
      "- **Факты о себе**: модель `formal-ai`, политика исполнения, поверхность и источники ответов.",
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
    "- **Self facts**: model `formal-ai`, execution policy, active surface, and answer sources.",
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

function localCollectRuntimeRules(history) {
  const turns = Array.isArray(history) ? history : [];
  const seen = new Set();
  const rules = [];
  for (const turn of turns) {
    if (String((turn || {}).role || "").toLowerCase() !== "user") continue;
    const rule = localRuntimeRuleFromText((turn || {}).content);
    if (rule && !seen.has(rule.id)) {
      seen.add(rule.id);
      rules.push(rule);
    }
  }
  return rules;
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
  const runtimeRules = localCollectRuntimeRules(history);
  if (localIsBehaviorRulesCount(normalized, history)) {
    return {
      intent: "behavior_rules_count",
      content: localBehaviorRulesCount(runtimeRules, language),
    };
  }
  if (localIsBehaviorRulesList(normalized)) {
    return { intent: "behavior_rules_list", content: localBehaviorRulesList(runtimeRules, language) };
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
  "show rules",
  "show list of your rules",
  "list your rules",
  "покажи правила поведения",
  "покажи правила",
  "покажи список своих правил",
  "перечисли свои правила",
  "व्यवहार के नियम सूचीबद्ध करें",
  "नियम दिखाओ",
  "अपने नियमों की सूची दिखाओ",
  "अपने नियम गिनाओ",
  "列出行为规则",
  "显示规则",
  "显示你的规则列表",
  "列出你的规则",
];

function matchesLocalBehaviorRulesListPattern(normalized) {
  return LOCAL_BEHAVIOR_RULES_LIST_PATTERNS.some((pattern) => {
    const text = normalizePrompt(pattern);
    return text && (normalized === text || normalized.includes(text));
  });
}

function localIsBehaviorRulesList(normalized) {
  return (
    matchesLocalBehaviorRulesListPattern(normalized) ||
    normalized.includes("list behavior rules") ||
    normalized.includes("list all behavior rules") ||
    normalized.includes("show behavior rules") ||
    isSupportedLanguageBehaviorRulesListQuery(normalized) ||
    normalized.includes("список правил поведения")
  );
}

function localIsBehaviorRulesCount(normalized, history) {
  const priorRuleListContext = localPriorBehaviorRulesListContext(history);
  const english =
    localContainsAny(normalized, ["rules", "rule list", "rules list"]) &&
    localContainsAny(normalized, ["how many", "number of", "count"]) &&
    (localContainsAny(normalized, ["all", "total", "there", "existing", "current", "behavior"]) ||
      priorRuleListContext);
  const russian =
    localContainsAny(normalized, ["правил", "правила"]) &&
    normalized.includes("сколько") &&
    (localContainsAny(normalized, ["всего", "все", "текущих", "поведения"]) ||
      priorRuleListContext);
  const hindi =
    localContainsAny(normalized, ["नियम", "नियमों"]) &&
    normalized.includes("कितने") &&
    (localContainsAny(normalized, ["कुल", "सभी", "व्यवहार"]) || priorRuleListContext);
  const chinese =
    localContainsAny(normalized, ["规则", "規則"]) &&
    normalized.includes("多少") &&
    (localContainsAny(normalized, ["总共", "总共有", "所有", "行为", "行為"]) ||
      priorRuleListContext);

  return english || russian || hindi || chinese;
}

function localPriorBehaviorRulesListContext(history) {
  const turns = Array.isArray(history) ? history : [];
  return turns.some((turn) => {
    const role = String((turn || {}).role || "").toLowerCase();
    const content = String((turn || {}).content || "");
    if (role === "user") return localIsBehaviorRulesList(normalizePrompt(content));
    return (
      role === "assistant" &&
      content.includes("rule_greeting") &&
      content.includes("rule_unknown")
    );
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

// Issue #438 (follow-up): only the Electron desktop shell exposes the
// start/stop service handlers. The browser/VS Code surfaces lack them, so the
// Services panel is gated on the bridge actually carrying serviceStatus.
function desktopServiceBridge() {
  const bridge = desktopBridge();
  if (!bridge || typeof bridge.serviceStatus !== "function") {
    return null;
  }
  return bridge;
}

function normalizeAppVersion(value) {
  const raw = String(value || "").trim();
  if (!raw || raw.startsWith("__") || raw.endsWith("__")) {
    return "";
  }
  return raw.replace(/^v/i, "");
}

function desktopAppVersionLabel(status) {
  const desktopVersion = normalizeAppVersion(status && status.appVersion);
  const fallback = normalizeAppVersion(APP_VERSION) || APP_VERSION;
  const version = desktopVersion || fallback;
  return /^v/i.test(version) ? version : `v${version}`;
}

function normalizeDesktopUpdaterStatus(updater, currentVersion) {
  if (!updater || typeof updater !== "object") {
    return null;
  }
  const state = String(updater.state || (updater.updateAvailable ? "available" : "idle"));
  const progressPercent = Math.max(
    0,
    Math.min(100, Number(updater.progressPercent || 0) || 0),
  );
  return {
    supported: updater.supported !== false,
    enabled: updater.enabled !== false && updater.supported !== false,
    platform: String(updater.platform || ""),
    currentVersion: normalizeAppVersion(updater.currentVersion) || currentVersion || "",
    state,
    updateAvailable: Boolean(updater.updateAvailable),
    downloaded: Boolean(updater.downloaded),
    latestVersion: normalizeAppVersion(updater.latestVersion),
    progressPercent,
    checkedAt: String(updater.checkedAt || ""),
    error: String(updater.error || ""),
    message: String(updater.message || ""),
  };
}

function mergeDesktopUpdateStatus(current, payload) {
  if (!payload || typeof payload !== "object") {
    return current;
  }
  if (payload.updater && typeof payload.updater === "object") {
    return normalizeDesktopStatus({ ...(current || {}), ...payload });
  }
  return normalizeDesktopStatus({
    ...(current || {}),
    appVersion:
      normalizeAppVersion(payload.currentVersion)
      || (current && current.appVersion)
      || "",
    updater: payload,
  });
}

function desktopUpdaterStateLabel(updater, t) {
  const tr = typeof t === "function" ? t : (key) => key;
  if (!updater) {
    return "";
  }
  if (updater.error) {
    return `${tr("updates.state.error")}: ${updater.error}`;
  }
  if (updater.message && updater.state === "disabled") {
    return updater.message;
  }
  const key = {
    idle: "updates.state.idle",
    checking: "updates.state.checking",
    available: "updates.state.available",
    "not-available": "updates.state.notAvailable",
    downloading: "updates.state.downloading",
    downloaded: "updates.state.downloaded",
    installing: "updates.state.installing",
    disabled: "updates.state.disabled",
    error: "updates.state.error",
  }[updater.state] || "updates.state.idle";
  return tr(key, {
    version: updater.latestVersion || updater.currentVersion || "",
    percent: Math.round(updater.progressPercent || 0),
  });
}

function desktopUpdaterBusy(updater) {
  return updater && ["checking", "downloading", "installing"].includes(updater.state);
}

// Human-readable summary for a single managed service state so the UI label and
// the indicator dot stay in lockstep.
function serviceStateLabel(state, t) {
  const tr = typeof t === "function" ? t : (key) => key;
  const key = {
    running: "services.state.running",
    stopped: "services.state.stopped",
    absent: "services.state.stopped",
    "missing-config": "services.state.needsToken",
    "docker-unavailable": "services.state.dockerUnavailable",
    ready: "services.state.ready",
    error: "services.state.error",
  }[String(state || "")];
  if (key) {
    return tr(key);
  }
  return String(state || "") || tr("services.state.unknown");
}

// Issue #554 (R2): map the structured VS Code install result the main process
// returns to a localized status line shown under the one-click button.
function vscodeInstallStateLabel(result, t) {
  const tr = typeof t === "function" ? t : (key) => key;
  if (!result || typeof result !== "object") {
    return "";
  }
  if (result.ok) {
    return tr("vscodeInstall.installed", { cli: String(result.cli || "code") });
  }
  const key = {
    "no-vscode-cli": "vscodeInstall.noCli",
    "no-release-asset": "vscodeInstall.noAsset",
    "release-lookup-failed": "vscodeInstall.lookupFailed",
    "download-failed": "vscodeInstall.downloadFailed",
    "install-failed": "vscodeInstall.installFailed",
    error: "vscodeInstall.error",
  }[String(result.state || "")];
  return key ? tr(key) : tr("vscodeInstall.error");
}

function normalizeDesktopStatus(status) {
  if (!status || typeof status !== "object") {
    return null;
  }
  const apiBase = String(status.apiBase || "").replace(/\/+$/, "");
  const appVersion = normalizeAppVersion(status.appVersion || status.version);
  const agentProvider =
    status.agentProvider && typeof status.agentProvider === "object"
      ? {
          type: String(status.agentProvider.type || "local-openai-compatible"),
          apiBase: String(status.agentProvider.apiBase || apiBase).replace(/\/+$/, ""),
          openAiBaseUrl: String(
            status.agentProvider.openAiBaseUrl || (apiBase ? `${apiBase}/v1` : ""),
          ).replace(/\/+$/, ""),
          model: String(status.agentProvider.model || "formal-ai"),
        }
      : null;
  return {
    shell: String(status.shell || "Electron"),
    mode: String(status.mode || (apiBase ? "server" : "in-process")),
    apiBase,
    staticBase: String(status.staticBase || ""),
    graphUrl: String(status.graphUrl || (apiBase ? `${apiBase}/v1/graph` : "")),
    traceUrl: String(status.traceUrl || (apiBase ? `${apiBase}/v1/graph?trace=answer_greeting_hi` : "")),
    memory: String(status.memory || "formal_ai_bundle"),
    appVersion,
    agentModeDefault: Boolean(status.agentModeDefault),
    toolCallPolicy: String(status.toolCallPolicy || "explicit-permission"),
    apiReady: status.apiReady !== false && Boolean(apiBase),
    apiError: String(status.apiError || ""),
    agentProvider,
    updater: normalizeDesktopUpdaterStatus(status.updater, appVersion),
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

// R5d / Issue #514: push the explicit per-tool grants to the local router. Chat
// mode always sends false grants; Agent and Full Auto activate only the tools
// the user has individually granted.
function syncDesktopToolGrants(bridge, mode, grants) {
  if (!bridge || typeof bridge.setToolGrants !== "function") {
    return;
  }
  Promise.resolve(bridge.setToolGrants(desktopToolRouterGrants(mode, grants))).catch(() => {});
}

async function ensureDesktopAgentServer(bridge) {
  if (!bridge || typeof bridge.ensureAgentServer !== "function") {
    return null;
  }
  return normalizeDesktopStatus(await bridge.ensureAgentServer());
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
  if (typeof bridge.ensureAgentServer === "function") {
    await bridge.ensureAgentServer();
  }
  return bridge.invokeTool({ tool: String(tool || ""), input: input || {} });
}

async function requestDesktopAgentProvider(bridge, request = {}) {
  if (!bridge || typeof bridge.runAgentProvider !== "function") {
    return null;
  }
  try {
    return await bridge.runAgentProvider(request || {});
  } catch (error) {
    return {
      ok: false,
      provider: "desktop",
      status: "error",
      executed: false,
      reason: error && error.message ? error.message : String(error),
    };
  }
}

function chatAnswerFromAgentProviderResult(result) {
  if (!result || !result.answer || typeof result.answer !== "object") {
    return null;
  }
  return result.answer;
}

function terminalCommandFromAnswer(answer) {
  const evidence = Array.isArray(answer && answer.evidence) ? answer.evidence : [];
  for (const item of evidence) {
    const text = String(item || "");
    if (text.startsWith("terminal:command:")) {
      const command = text.slice("terminal:command:".length).trim();
      if (command) {
        return command;
      }
    }
  }
  return "";
}

function shellOutputMarkdown(body, t) {
  const noOutput = typeof t === "function"
    ? t("permissions.message.noOutput")
    : "(no output)";
  const text = String(body || "").trim() || noOutput;
  const safe = text.replace(/```/g, "` ` `");
  return `\`\`\`text\n${safe}\n\`\`\``;
}

function desktopToolResultReason(result, t) {
  const translate = (key, fallback) =>
    typeof t === "function" ? t(key) : fallback;
  if (!result) {
    return translate(
      "permissions.message.reasonNoResult",
      "desktop tool router returned no result",
    );
  }
  return (
    result.reason ||
    result.error ||
    result.status ||
    translate(
      "permissions.message.reasonRefused",
      "desktop tool router refused the request",
    )
  );
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

function normalizeApiThinkingStep(entry) {
  if (!entry || typeof entry !== "object") return null;
  const step = String(entry.step || entry.kind || entry.source_event || "fallback").trim();
  const detail = String(entry.detail || entry.payload || entry.source_event || "").trim();
  if (!step && !detail) return null;
  const normalized = {
    step: step || "fallback",
    detail,
  };
  if (entry.summary !== undefined) normalized.summary = String(entry.summary);
  if (entry.id !== undefined) normalized.id = String(entry.id);
  if (entry.order !== undefined) normalized.order = entry.order;
  if (entry.level !== undefined) normalized.level = String(entry.level);
  if (entry.source_event !== undefined) normalized.sourceEvent = String(entry.source_event);
  if (entry.parent_id !== undefined && entry.parent_id !== null) {
    normalized.parentId = String(entry.parent_id);
  }
  return normalized;
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
      model: "formal-ai",
      messages: desktopMessages(history, text),
      temperature: normalizeSliderPreference(preferences.temperature, 0),
      stream: false,
    }),
  });

  if (!response.ok) {
    throw new Error(`desktop API returned ${response.status}`);
  }

  const payload = await response.json();
  const message =
    payload &&
    payload.choices &&
    payload.choices[0] &&
    payload.choices[0].message
      ? payload.choices[0].message
      : {};
  const answerText =
    message && message.content !== undefined ? String(message.content || "") : "";
  const apiThinkingSteps = Array.isArray(message.thinking_steps)
    ? message.thinking_steps.map(normalizeApiThinkingStep).filter(Boolean)
    : [];
  const fallbackDesktopSteps = [
    { step: "desktop_shell", detail: "Electron preload bridge supplied local API status" },
    { step: "http_chat", detail: "POST /v1/chat/completions on the local Rust server" },
    { step: "memory", detail: "UI import/export stays on formal_ai_bundle" },
  ];

  return {
    intent: "desktop_http_chat",
    content: answerText || UNKNOWN_ANSWER,
    source: "desktop_http",
    evidence: [
      "surface:desktop",
      "api:/v1/chat/completions",
      desktopStatus.graphUrl ? "network:/v1/graph" : "",
    ].filter(Boolean),
    steps: apiThinkingSteps.length > 0 ? apiThinkingSteps : fallbackDesktopSteps,
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
  return <div className="diagnostics-http" data-testid="diagnostics-http">{Array.isArray(providers) && providers.length > 0 ? <div className="diagnostics-http-section"><strong className="diagnostics-section-label">{t("message.diagnosticsProviders")}</strong><ul className="diagnostics-http-provider-list">{providers.map((entry, index) => <li key={`${entry.id || "provider"}-${index}`} className={`diagnostics-http-provider ${entry.ok ? "is-ok" : "is-error"}`} data-testid="diagnostics-http-provider">{t("message.diagnosticsProviderRow", {
          label: entry.label || entry.id || "(provider)",
          status: entry.ok ? t("message.diagnosticsProviderOk") : `${t("message.diagnosticsProviderError")}: ${entry.error || "(unknown)"}`,
          count: typeof entry.count === "number" ? entry.count : 0,
          elapsed: typeof entry.elapsedMs === "number" ? entry.elapsedMs : 0
        })}</li>)}</ul></div> : null}<div className="diagnostics-http-section"><strong className="diagnostics-section-label">{t("message.diagnosticsHttp")}</strong>{safeExchanges.length === 0 ? <p className="diagnostics-http-empty">{t("message.diagnosticsHttpEmpty")}</p> : <ol className="diagnostics-http-list">{safeExchanges.map((exchange, index) => <li key={`${exchange.id || index}`} className="diagnostics-http-item"><details className="diagnostics-detail" data-testid="diagnostics-http-exchange"><summary><span className="diagnostics-step-name">{`${exchange.method || "GET"} ${exchange.provider ? `[${exchange.provider}] ` : ""}`}</span><span className="diagnostics-step-summary">{exchange.url || "(no url)"}</span><span className="diagnostics-http-status">{t("message.diagnosticsHttpStatus", {
                status: typeof exchange.status === "number" ? exchange.status : "—",
                elapsed: typeof exchange.elapsedMs === "number" ? exchange.elapsedMs : 0,
                bytes: typeof exchange.responseBytes === "number" ? exchange.responseBytes : 0
              })}</span></summary><div className="diagnostics-detail-body"><div className="diagnostics-tool-section"><span className="diagnostics-section-label">{t("message.diagnosticsHttpRequest")}</span><pre className="diagnostics-payload">{formatDiagnosticPayload({
                  method: exchange.method || "GET",
                  url: exchange.url || "",
                  headers: exchange.requestHeaders || {},
                  body: exchange.requestBody || null,
                  provider: exchange.provider || "",
                  phase: exchange.phase || ""
                })}</pre></div><div className="diagnostics-tool-section"><span className="diagnostics-section-label">{t("message.diagnosticsHttpResponse")}</span><pre className="diagnostics-payload">{formatDiagnosticPayload({
                  status: exchange.status ?? null,
                  ok: !!exchange.ok,
                  elapsedMs: exchange.elapsedMs ?? null,
                  responseBytes: exchange.responseBytes ?? null,
                  finalUrl: exchange.finalUrl || "",
                  contentType: exchange.contentType || "",
                  responseSnippet: exchange.responseSnippet || "",
                  error: exchange.error || ""
                })}</pre></div><div className="diagnostics-tool-section"><span className="diagnostics-section-label">{t("message.diagnosticsHttpUnified")}</span><pre className="diagnostics-payload diagnostics-http-links">{formatHttpExchangeAsLinks(exchange)}</pre></div></div></details></li>)}</ol>}</div></div>;
}

// Issue #488: while the worker is still computing the answer there is no live
// per-step stream to display, but the pending bubble should still feel alive —
// an expert reasoning out loud surfaces "what they're working on right now"
// every couple of seconds. The hook accumulates a fixed sequence of generic
// expert-shaped phases (read → formalize → look up → compose) over ~2 s steps
// and stops once the answer arrives. Each new phase appends to the visible
// steps array so the rotated-scrolling animation in `ThinkingPreview` re-fires
// the same way it would for real solver steps.
// Issue #541 (R6): honour the OS "reduce motion" accessibility preference. When
// the user has asked for reduced motion we skip the staged reveal entirely and
// show the answer at once, matching the existing prefers-reduced-motion CSS.
function usePrefersReducedMotion() {
  const query = "(prefers-reduced-motion: reduce)";
  const getInitial = () =>
    typeof window !== "undefined" && typeof window.matchMedia === "function"
      ? window.matchMedia(query).matches
      : false;
  const [reduced, setReduced] = useState(getInitial);
  useEffect(() => {
    if (
      typeof window === "undefined" ||
      typeof window.matchMedia !== "function"
    ) {
      return undefined;
    }
    const media = window.matchMedia(query);
    const handler = (event) => setReduced(event.matches);
    if (typeof media.addEventListener === "function") {
      media.addEventListener("change", handler);
      return () => media.removeEventListener("change", handler);
    }
    // Safari < 14 fallback.
    media.addListener(handler);
    return () => media.removeListener(handler);
  }, []);
  return reduced;
}

// Issue #541 (R5/R6): stage the reveal of a freshly produced assistant message —
// reasoning steps first (each new step re-triggers the rotated-scroll animation
// in ThinkingPreview), then the answer body — across a minimum animation budget
// so the deterministic engine's instant answers still *feel* considered. Returns
// the count of currently revealed steps and whether the body is shown yet. With
// budgetMs<=0, reduced motion, or no steps it is an immediate no-op (everything
// shown at once). The first ~72% of the budget unveils the steps; the body is
// held back until the full budget elapses, satisfying R6's "only when we
// scrolled to the last thinking step can we show the message itself".
function useMessageReveal(stepCount, budgetMs) {
  const reducedMotion = usePrefersReducedMotion();
  const active = budgetMs > 0 && stepCount > 0 && !reducedMotion;
  // The staged reveal plays exactly once — when the freshly produced message
  // first appears. Once it has played out (or if it never applied) we latch
  // "done" so that a later change in step count — e.g. the user toggling the
  // reasoning-detail setting on an already-revealed message — snaps straight to
  // "show everything" instead of replaying the animation. Replaying would set
  // `bodyShown` back to false (the `.is-revealing` rule is `display:none`, so
  // the answer would briefly vanish) and re-scroll the steps from the first
  // one, which is jarring when the user is just adjusting how much detail to see.
  const doneRef = useRef(!active);
  const [revealedSteps, setRevealedSteps] = useState(active ? 1 : stepCount);
  const [bodyShown, setBodyShown] = useState(!active);
  useEffect(() => {
    if (!active || doneRef.current) {
      setRevealedSteps(stepCount);
      setBodyShown(true);
      return undefined;
    }
    // Reserve the final slice of the budget for the body fade; spread the rest
    // across the steps so even a single step occupies a perceptible beat.
    const stepsWindow = budgetMs * 0.72;
    const perStep = stepsWindow / stepCount;
    setRevealedSteps(1);
    setBodyShown(false);
    const timers = [];
    for (let index = 1; index < stepCount; index += 1) {
      timers.push(
        setTimeout(
          () => setRevealedSteps(index + 1),
          Math.round(perStep * index),
        ),
      );
    }
    timers.push(
      setTimeout(() => {
        setBodyShown(true);
        doneRef.current = true;
      }, Math.round(budgetMs)),
    );
    return () => timers.forEach((timer) => clearTimeout(timer));
  }, [active, stepCount, budgetMs]);
  return { active, revealedSteps, bodyShown };
}

function usePendingThinkingPhases(isActive, t) {
  const [phaseIndex, setPhaseIndex] = useState(0);
  const phrases = useMemo(
    () => [
      t("message.thinkingStep.pendingReading"),
      t("message.thinkingStep.pendingFormalizing"),
      t("message.thinkingStep.pendingDispatching"),
      t("message.thinkingStep.pendingComposing"),
      t("message.thinkingStep.working"),
    ],
    [t],
  );
  useEffect(() => {
    if (!isActive) {
      setPhaseIndex(0);
      return undefined;
    }
    if (phaseIndex >= phrases.length - 1) {
      return undefined;
    }
    const timer = setTimeout(() => setPhaseIndex((value) => value + 1), 1800);
    return () => clearTimeout(timer);
  }, [isActive, phaseIndex, phrases.length]);
  if (!isActive) return [];
  return phrases.slice(0, phaseIndex + 1);
}

// Issue #488: render the pending assistant message — while processing, the
// thinking preview IS the visible part of the message (no separate "working"
// caption), and the preview pulls from a hook that adds expert-shaped phases
// over time so the rotated-scrolling animation actually has something to rotate
// even though the worker itself does not yet stream per-step messages.
function PendingAssistantBubble({ t }) {
  const pendingPhases = usePendingThinkingPhases(true, t);
  return <article className="message assistant pending"><div className="avatar" aria-hidden="true">{"FA"}</div><div className="message-body"><ThinkingPreview steps={pendingPhases} t={t} isPending={true} /></div></article>;
}

// Issue #676 (R8): map the resolved intent to a per-intent narrative catalog
// key, mirroring the Rust `thinking_narrative`. Grouped so related routes share
// one human headline (all lookups read the same, all web tools read the same).
const THINKING_NARRATIVE_KEYS = {
  greeting: "narrativeGreeting",
  wellbeing: "narrativeWellbeing",
  assistant_free_time: "narrativeAssistantFreeTime",
  farewell: "narrativeFarewell",
  gratitude: "narrativeGratitude",
  thanks: "narrativeGratitude",
  courtesy_response: "narrativeGratitude",
  courtesy: "narrativeGratitude",
  identity: "narrativeIdentity",
  assistant_name: "narrativeIdentity",
  set_assistant_name: "narrativeIdentity",
  recall_name: "narrativeIdentity",
  naming: "narrativeIdentity",
  assistant_naming: "narrativeIdentity",
  self_facts: "narrativeIdentity",
  who_is_question: "narrativeIdentity",
  calculation: "narrativeCalculation",
  arithmetic: "narrativeCalculation",
  calculation_error: "narrativeCalculation",
  object_counting: "narrativeCalculation",
  fact_lookup: "narrativeLookup",
  fact_query: "narrativeLookup",
  concept_lookup: "narrativeLookup",
  concept_lookup_in_context: "narrativeLookup",
  known_facts: "narrativeLookup",
  wikipedia_lookup: "narrativeLookup",
  wikipedia_article_question: "narrativeLookup",
  definition_merge: "narrativeLookup",
  translation: "narrativeTranslation",
  web_search: "narrativeWeb",
  http_fetch: "narrativeWeb",
  url_navigate: "narrativeWeb",
  write_program: "narrativeCode",
  software_project_plan: "narrativeCode",
  software_project_implementation: "narrativeCode",
  algorithm: "narrativeCode",
  test_status: "narrativeTests",
  self_healing: "narrativeSelfHealing",
  self_heal: "narrativeSelfHealing",
  meta_explanation: "narrativeMetaExplanation",
  learn_from_source: "narrativeLearn",
  clarification: "narrativeClarification",
  unknown: "narrativeUnknown",
  fallback: "narrativeUnknown",
  no_match: "narrativeUnknown",
};

// Issue #676 (R8): produce the single human, first-person headline that leads a
// thinking trace ("You asked how I'm doing, so I told you and offered to
// help."). Unknown routes still get a human sentence via the generic template.
// Returns "" only when there is no intent to summarize (e.g. the pending
// placeholder), so callers can skip the headline entirely.
function thinkingNarrative(intent, t) {
  const route = String(intent || "").trim().toLowerCase();
  if (!route) return "";
  const key = THINKING_NARRATIVE_KEYS[route];
  if (key) return t(`message.thinkingStep.${key}`);
  return t("message.thinkingStep.narrativeGeneric", {
    task: humanizeThinkingIdentifier(route),
  });
}

function ThinkingPreview({ steps, t, isPending = false, narrative = "" }) {
  const [expanded, setExpanded] = useState(false);
  const safeSteps = Array.isArray(steps)
    ? steps.map((step) => String(step || "").trim()).filter(Boolean)
    : [];
  // Issue #488: track the index of the current step so a change in the latest
  // step triggers the rotated-scrolling CSS animation (current step slides up
  // into place; the previous step half-shows above with the gradient fade).
  const lastIndex = safeSteps.length - 1;
  const current = lastIndex >= 0 ? safeSteps[lastIndex] : "";
  const previous = lastIndex > 0 ? safeSteps[lastIndex - 1] : "";
  // Use a stable but per-step key so React re-mounts the current/previous
  // <p> nodes when the step changes — that re-mount is what re-triggers the
  // CSS `@keyframes thinking-rotate-in` animation.
  const animationKey = `${lastIndex}-${current}`;
  if (safeSteps.length === 0) return null;

  return <section className={["thinking-preview", expanded ? "is-expanded" : "is-collapsed", isPending ? "is-pending" : ""].filter(Boolean).join(" ")} data-testid="thinking-preview" data-pending={isPending ? "true" : null} aria-label={t("message.thinking")} aria-live={isPending ? "polite" : null}><div className="thinking-preview-header"><strong className="thinking-preview-title">{
      // Issue #488: show a subtle "live" affordance while pending so the user
      // understands the trace is updating in real time (the dot pulses via
      // CSS; the visible label stays unchanged for screen readers).
      isPending ? <span className="thinking-preview-live-dot" aria-hidden="true" data-testid="thinking-preview-live-dot" /> : null}{t("message.thinking")}</strong><button type="button" className="thinking-preview-toggle" data-testid="thinking-preview-toggle" aria-expanded={expanded ? "true" : "false"} onClick={() => setExpanded(value => !value)}>{expanded ? t("message.thinkingCollapse") : t("message.thinkingExpand")}</button></div>{narrative ? <p className="thinking-preview-narrative" data-testid="thinking-narrative">{narrative}</p> : null}{expanded ? <ol className="thinking-preview-list" data-testid="thinking-expanded-list">{safeSteps.map((step, index) => <li key={`${index}-${step}`}>{step}</li>)}</ol> : <div className="thinking-preview-collapsed" data-testid="thinking-collapsed">{previous ? <p key={`prev-${animationKey}`} className="thinking-preview-previous" data-testid="thinking-preview-previous" aria-label={t("message.thinkingPrevious")}>{previous}</p> : null}<p key={`curr-${animationKey}`} className="thinking-preview-current" data-testid="thinking-preview-current" aria-label={t("message.thinkingCurrent")}>{current}</p></div>}</section>;
}

function DesktopPermissionPanel({
  grants,
  mode,
  onDecision,
  onGrantAll,
  hasPendingTask = false,
  testId = "desktop-permission-panel",
  t,
}) {
  const active = mode !== "chat";
  const tr = typeof t === "function" ? t : (key) => key;
  const stateLabel = (state) =>
    state === "granted"
      ? tr("permissions.state.granted")
      : state === "declined"
        ? tr("permissions.state.declined")
        : tr("permissions.state.undecided");
  // Issue #541 (R9): the original issue text says "After permissions are
  // granted nothing happens, the message for granting permissions should also
  // include button to grant all permissions and switch to agent mode, which
  // when clicked should actually evaluate pending task for execution."
  //
  // We render that affordance as a primary CTA above the per-tool rows so the
  // user can opt-in with a single click without scrolling through six
  // grant/decline buttons. The button label changes when a task is queued
  // ("...and run pending task") so it is honest about what will happen.
  const grantAllLabel = hasPendingTask
    ? tr("permissions.action.grantAllAndRun")
    : tr("permissions.action.grantAll");
  return <section className="permission-panel" data-testid={testId} data-mode={mode}><div className="permission-panel-header"><strong>{tr("permissions.panel.title")}</strong><span>{active ? tr("permissions.panel.active") : tr("permissions.panel.saved")}</span></div>{onGrantAll ? <div className="permission-panel-grant-all"><button type="button" className="permission-button permission-button-grant-all" data-testid={`${testId}-grant-all`} data-has-pending-task={hasPendingTask ? "true" : "false"} onClick={() => onGrantAll()}>{grantAllLabel}</button></div> : null}<div className="permission-tool-list">{DESKTOP_TOOL_OPTIONS.map(tool => {
      const state = desktopToolGrantState(grants, tool);
      const granted = state === "granted";
      const declined = state === "declined";
      return <div key={tool} className="permission-tool-row" data-testid={`${testId}-row-${tool}`}><div className="permission-tool-copy"><strong>{tr(`permissions.tool.${tool}.label`)}</strong><span>{tr(`permissions.tool.${tool}.description`)}</span></div><span className={`permission-state permission-state-${state}`} data-testid={`${testId}-state-${tool}`}>{stateLabel(state)}</span><div className="permission-actions"><button type="button" className="permission-button" data-testid={`${testId}-grant-${tool}`} aria-pressed={granted ? "true" : "false"} onClick={() => onDecision && onDecision(tool, true)}>{tr("permissions.action.grant")}</button><button type="button" className="permission-button permission-button-secondary" data-testid={`${testId}-decline-${tool}`} aria-pressed={declined ? "true" : "false"} onClick={() => onDecision && onDecision(tool, false)}>{tr("permissions.action.decline")}</button></div></div>;
    })}</div></section>;
}

function CommandApprovalPanel({ approval, status, onApprove, onDeny, t }) {
  if (!approval) {
    return null;
  }
  const tr = typeof t === "function" ? t : (key) => key;
  const currentStatus = status || approval.status || "pending";
  const pending = currentStatus === "pending";
  const command = String(approval.command || "");
  const statusKeys = {
    pending: "permissions.command.status.pending",
    running: "permissions.command.status.running",
    approved: "permissions.command.status.approved",
    denied: "permissions.command.status.denied",
  };
  const statusLabel = statusKeys[currentStatus]
    ? tr(statusKeys[currentStatus])
    : currentStatus;
  return <section className="command-approval-panel" data-testid="command-approval" data-status={currentStatus}><div className="command-approval-copy"><strong>{tr("permissions.command.title")}</strong><code>{command}</code><span className={`command-approval-status command-approval-status-${currentStatus}`}>{statusLabel}</span></div><div className="command-approval-actions"><button type="button" className="permission-button" data-testid="command-approve" disabled={!pending} onClick={() => pending && onApprove && onApprove(approval)}>{tr("permissions.command.approve")}</button><button type="button" className="permission-button permission-button-secondary" data-testid="command-deny" disabled={!pending} onClick={() => pending && onDeny && onDeny(approval)}>{tr("permissions.command.deny")}</button></div></section>;
}

function Message({
  message,
  diagnosticsMode,
  reportIssueUrl,
  thinkingDetailLevel,
  minMessageAnimationMs = 0,
  renderPermissionPanel,
  commandApprovals,
  onApproveCommand,
  onDenyCommand,
  t,
}) {
  const evidence = diagnosticsMode ? (message.evidence ?? []) : [];
  const thinkingSteps = diagnosticsMode ? (message.thinkingSteps ?? []) : [];
  const thinkingPreviewSteps = buildMessageThinkingPreviewSteps(
    message,
    t,
    thinkingDetailLevel,
  );
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

  // Issue #541 (R5/R6): stage the reveal of this message — reasoning steps
  // first, then the answer body — across the minimum animation budget. Only a
  // freshly produced answer carries `animateReveal`; hydrated history shows at
  // once (budget 0 -> immediate no-op).
  const revealBudgetMs = message.animateReveal ? minMessageAnimationMs : 0;
  const reveal = useMessageReveal(thinkingPreviewSteps.length, revealBudgetMs);
  const revealedThinkingSteps = reveal.active
    ? thinkingPreviewSteps.slice(0, reveal.revealedSteps)
    : thinkingPreviewSteps;
  const bodyRevealClass = reveal.active
    ? reveal.bodyShown
      ? " is-revealed"
      : " is-revealing"
    : "";

  // React 19 compares `dangerouslySetInnerHTML` by object identity (React 18
  // compared the inner `__html` string). A fresh `markdownHtml(...)` object on
  // every render would therefore make React re-assign `innerHTML` each pass,
  // wiping the `.code-block` wrappers that `enhanceCodeBlocks` grafts in below.
  // Memoising by `message.content` keeps the object stable while the text is
  // unchanged, so the out-of-band enhancements survive unrelated re-renders.
  const markdownContent = useMemo(
    () => markdownHtml(message.content),
    [message.content],
  );

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

  return <article className={`message ${message.role}`} data-testid="chat-message" data-demo-label={message.demoLabel || null}><div className="avatar" aria-hidden="true">{message.role === "user" ? "Y" : "FA"}</div><div className="message-body"><div className="message-meta"><strong>{message.role === "user" ? t("message.author.user") : message.author}</strong><time>{message.sentAt}</time>{diagnosticsMode && message.intent ? <span className="intent">{`intent:${message.intent}`}</span> : null}<button type="button" className={`message-copy-button${markdownCopied ? " is-copied" : ""}`} data-testid="copy-markdown-button" data-copied={markdownCopied ? "true" : null} onClick={handleCopyMarkdown} aria-label={t("message.copyMarkdownTitle")} title={t("message.copyMarkdownTitle")}><span className="copy-button-label">{markdownCopied ? t("message.copyMarkdownDone") : t("message.copyMarkdown")}</span></button></div>{
    // Issue #488: render thinking ABOVE the answer body. Reasoning logically
    // precedes the answer (and during streaming it is the only visible part of
    // the message), so it belongs at the top of the message body, not below it.
    // Issue #541 (R6): during the staged reveal only the steps unveiled so far
    // are shown, so the trace visibly fills in before the answer appears.
    revealedThinkingSteps.length ? <ThinkingPreview steps={revealedThinkingSteps} t={t} narrative={thinkingNarrative(message.intent, t)} /> : null}<div ref={markdownRef} className={`markdown-body${bodyRevealClass}`} aria-hidden={reveal.active && !reveal.bodyShown ? "true" : null} data-testid="message-markdown-body" dangerouslySetInnerHTML={markdownContent} />{message.permissionPanel && typeof renderPermissionPanel === "function" ? <div className="message-permission-panel">{renderPermissionPanel("desktop-permission-panel-message")}</div> : null}{message.commandApproval ? <CommandApprovalPanel approval={message.commandApproval} status={commandApprovals && commandApprovals[message.commandApproval.id] && commandApprovals[message.commandApproval.id].status} onApprove={onApproveCommand} onDeny={onDenyCommand} t={t} /> : null}{message.iframeUrl ? <div className={`fetch-iframe-container${iframeFullscreen ? " is-fullscreen" : ""}`} data-testid="fetch-iframe-container"><div className="fetch-iframe-header"><span className="fetch-iframe-url">{message.iframeUrl}</span><div className="fetch-iframe-actions"><a href={message.iframeUrl} target="_blank" rel="noopener noreferrer" className="fetch-iframe-open fetch-iframe-control" aria-label={t("fetch.openInNewTab")} title={t("fetch.openInNewTab")}>{"↗"}</a><button type="button" className="fetch-iframe-toggle fetch-iframe-control" onClick={() => setIframeFullscreen(prev => !prev)} aria-label={iframeFullscreen ? t("fetch.minimize") : t("fetch.fullscreen")} aria-pressed={iframeFullscreen ? "true" : "false"} title={iframeFullscreen ? t("fetch.minimize") : t("fetch.fullscreen")}>{iframeFullscreen ? "⤡" : "⛶"}</button></div></div><iframe className="fetch-iframe" src={message.iframeUrl} title={t("fetch.frameTitle", {
        url: message.iframeUrl
      })} sandbox="allow-scripts allow-same-origin allow-forms allow-popups" loading="lazy" data-testid="fetch-iframe" /></div> : null}{evidence.length ? <div className="evidence-list">{evidence.map(item => <span key={item}>{item}</span>)}</div> : null}{thinkingSteps.length ? <div className="thinking-steps"><strong>{t("message.thinking")}</strong><ol>{thinkingSteps.map(item => <li key={item}>{item}</li>)}</ol></div> : null}{diagnosticsSteps.length ? <div className="diagnostics-steps" data-testid="diagnostics-steps"><strong>{t("message.diagnosticsSteps")}</strong><ol className="diagnostics-step-list">{diagnosticsSteps.map((entry, index) => <li key={`${entry.step}-${index}`} className="diagnostics-step"><details className="diagnostics-detail" data-testid="diagnostics-step" data-step={entry.step}><summary><span className="diagnostics-step-name">{entry.formalization ? t("message.formalization") : entry.step}</span><span className="diagnostics-step-summary">{entry.formalization ? truncateDiagnosticDetail(entry.formalization.tuple) : truncateDiagnosticDetail(entry.detail)}</span></summary><div className="diagnostics-detail-body">{entry.formalization ? <FormalizationView formalization={entry.formalization} t={t} /> : <pre className="diagnostics-payload">{formatDiagnosticPayload(entry.detail)}</pre>}</div></details></li>)}</ol></div> : null}{diagnosticsToolCalls.length ? <div className="diagnostics-tools" data-testid="diagnostics-tools"><strong>{t("message.diagnosticsTools")}</strong><ol className="diagnostics-tool-list">{diagnosticsToolCalls.map((call, index) => <li key={`${call.tool || "tool"}-${index}`} className="diagnostics-tool"><details className="diagnostics-detail" data-testid="diagnostics-tool"><summary><span className="diagnostics-tool-name">{call.tool || "(tool)"}</span><span className="diagnostics-tool-summary">{summarizeToolCall(call)}</span></summary><div className="diagnostics-detail-body"><div className="diagnostics-tool-section"><span className="diagnostics-section-label">{t("message.toolInputs")}</span><pre className="diagnostics-payload">{formatDiagnosticPayload(call.inputs)}</pre></div><div className="diagnostics-tool-section"><span className="diagnostics-section-label">{t("message.toolOutputs")}</span><pre className="diagnostics-payload">{formatDiagnosticPayload(call.outputs)}</pre></div>{Array.isArray(call.steps) && call.steps.length > 0 ? <div className="diagnostics-tool-section"><span className="diagnostics-section-label">{t("message.toolReasoning")}</span><ol className="diagnostics-tool-reasoning">{call.steps.map((s, j) => <li key={`${call.tool}-step-${j}`}>{`${s.step}: ${s.detail}`}</li>)}</ol></div> : null}</div></details></li>)}</ol></div> : null}{diagnosticsPayload ? <DiagnosticsHttpPanel providers={diagnosticsProviders} exchanges={diagnosticsHttp} t={t} /> : null}{reportIssueUrl ? <div className="message-actions"><a href={reportIssueUrl} target="_blank" rel="noopener noreferrer">{reportLabel}</a></div> : null}</div></article>;
}

// Issue #27: a VS Code-style collapsible sidebar section. When `collapsed` is
// false the section participates in the equal-share flex layout and scrolls
// independently; when true only the header remains visible.
const SIDEBAR_SECTION_TEST_IDS = [
  "drawer-menu-actions",
  "sidebar-desktop",
  "sidebar-services",
  "sidebar-conversations",
  "sidebar-settings",
  "sidebar-prompts",
  "sidebar-tools",
  "sidebar-trace",
];

function CollapsibleSection({
  title,
  collapsed,
  onToggle,
  testId,
  className = "",
  bodyClassName = "",
  expandOnlyLabel,
  expandOnlyTitle,
  iconPack,
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
  const isolateLabel = expandOnlyLabel || title;
  const isolateTitle = expandOnlyTitle || isolateLabel;
  const handleHeaderClick = (event) => {
    const target = event.target;
    if (
      target &&
      typeof target.closest === "function" &&
      target.closest("[data-sidebar-section-action]")
    ) {
      return;
    }
    if (typeof onToggle === "function") onToggle();
  };
  const handleToggleClick = (event) => {
    event.stopPropagation();
    if (typeof onToggle === "function") onToggle();
  };
  return <section className={sectionClassName} data-testid={testId} data-collapsed={collapsed ? "true" : "false"}><div className="sidebar-section-header" onClick={handleHeaderClick}><button type="button" className="sidebar-section-toggle" aria-expanded={collapsed ? "false" : "true"} onClick={handleToggleClick}><span className="sidebar-section-caret" aria-hidden="true">{collapsed ? "▶" : "▼"}</span><h2>{title}</h2></button><button type="button" className="sidebar-section-isolate" data-testid="sidebar-section-isolate" data-sidebar-section-action="isolate" aria-label={isolateLabel} title={isolateTitle}><ToolbarIcon action="isolateSection" pack={iconPack} /></button></div>{collapsed ? null : <div className={sectionBodyClassName}>{children}</div>}</section>;
}

function MenuGlyph({ open }) {
  return <span className={`btn-icon menu-icon ${open ? "menu-icon-close" : "menu-icon-hamburger"}`} aria-hidden="true" />;
}

function SidebarToggleGlyph({ collapsed }) {
  return <span className={`btn-icon sidebar-toggle-icon ${collapsed ? "sidebar-toggle-icon-expand" : "sidebar-toggle-icon-collapse"}`} aria-hidden="true">{collapsed ? "▶" : "◀"}</span>;
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
    interfaceCapabilities: [],
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
  const [thinkingDetailLevel, setThinkingDetailLevel] = useState(
    normalizeThinkingDetailLevel(initialPreferences.current.thinkingDetailLevel),
  );
  // Issue #541 (R5): minimum wall-clock budget for the reasoning + reveal
  // animation of a freshly produced answer.
  const [minMessageAnimationMs, setMinMessageAnimationMs] = useState(
    normalizeAnimationBudgetMs(initialPreferences.current.minMessageAnimationMs),
  );
  const [contextPanelWidth, setContextPanelWidth] = useState(
    normalizeContextPanelWidth(initialPreferences.current.contextPanelWidth),
  );
  // Issue #27: sidebar collapse/expand state per section.
  const [sidebarMenuCollapsed, setSidebarMenuCollapsed] = useState(
    initialPreferences.current.sidebarMenuCollapsed,
  );
  const [sidebarDesktopCollapsed, setSidebarDesktopCollapsed] = useState(
    initialPreferences.current.sidebarDesktopCollapsed,
  );
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
  // Issue #444: one boolean per external trusted service, kept in a single map so
  // the catalog stays the only place that enumerates the services. A missing
  // stored value defaults to enabled (opt-out model).
  const [externalServices, setExternalServices] = useState(() =>
    Object.fromEntries(
      EXTERNAL_TRUSTED_SERVICES.map((service) => [
        service.key,
        initialPreferences.current[service.key] !== false,
      ]),
    ),
  );
  const setExternalService = useCallback((key, value) => {
    setExternalServices((prev) => ({ ...prev, [key]: Boolean(value) }));
  }, []);
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
  const [toolbarIconPack, setToolbarIconPack] = useState(
    normalizeToolbarIconPack(initialPreferences.current.toolbarIconPack),
  );
  const sidebarExpandOnlyLabel = t("buttons.expandOnlySection");
  const sidebarExpandOnlyTitle = t("titles.expandOnlySection");
  const SidebarSection = useCallback((props) => (
    <CollapsibleSection
      {...props}
      expandOnlyLabel={sidebarExpandOnlyLabel}
      expandOnlyTitle={sidebarExpandOnlyTitle}
      iconPack={toolbarIconPack}
    />
  ), [sidebarExpandOnlyLabel, sidebarExpandOnlyTitle, toolbarIconPack]);
  const [locationPreference, setLocationPreference] = useState(
    String(initialPreferences.current.location || ""),
  );
  const [assistantName, setAssistantName] = useState(
    normalizeAssistantName(initialPreferences.current.assistantName),
  );
  const [desktopStatus, setDesktopStatus] = useState(null);
  // Issue #438 (follow-up): one-click start/stop of the prepared Docker
  // containers (Telegram bot + OpenAI-compatible server). `serviceStatus` holds
  // the latest snapshot from the desktop bridge; `serviceBusy` names the service
  // currently starting/stopping so its buttons disable; `telegramToken` backs the
  // inline token field the bot needs before it can start.
  const [serviceStatus, setServiceStatus] = useState(null);
  const [serviceBusy, setServiceBusy] = useState("");
  const [serviceError, setServiceError] = useState("");
  const [telegramToken, setTelegramToken] = useState("");
  const [sidebarServicesCollapsed, setSidebarServicesCollapsed] = useState(
    initialPreferences.current.sidebarServicesCollapsed,
  );
  const [updateBusy, setUpdateBusy] = useState("");
  const isolateSidebarSection = useCallback((testId) => {
    const activeSection = String(testId || "");
    if (!SIDEBAR_SECTION_TEST_IDS.includes(activeSection)) return;
    setSidebarMenuCollapsed(activeSection !== "drawer-menu-actions");
    setSidebarDesktopCollapsed(activeSection !== "sidebar-desktop");
    setSidebarServicesCollapsed(activeSection !== "sidebar-services");
    setSidebarConversationsCollapsed(activeSection !== "sidebar-conversations");
    setSidebarSettingsCollapsed(activeSection !== "sidebar-settings");
    setSidebarPromptsCollapsed(activeSection !== "sidebar-prompts");
    setSidebarToolsCollapsed(activeSection !== "sidebar-tools");
    setSidebarTraceCollapsed(activeSection !== "sidebar-trace");
  }, []);
  const handleSidebarSectionClickCapture = useCallback((event) => {
    const target = event.target;
    if (!target || typeof target.closest !== "function") return;
    const isolateButton = target.closest("[data-sidebar-section-action='isolate']");
    const shiftedHeader = event.shiftKey
      ? target.closest(".sidebar-section-header")
      : null;
    if (!isolateButton && !shiftedHeader) return;
    const section = target.closest(".sidebar-section");
    if (!section) return;
    event.preventDefault();
    event.stopPropagation();
    isolateSidebarSection(section.getAttribute("data-testid"));
  }, [isolateSidebarSection]);
  // Issue #554 (R2): one-click install of the formal-ai VS Code extension from
  // the desktop app. `vscodeInstallBusy` gates the button; `vscodeInstallResult`
  // holds the last {ok,state,reason} the main process returned.
  const [vscodeInstallBusy, setVscodeInstallBusy] = useState(false);
  const [vscodeInstallResult, setVscodeInstallResult] = useState(null);
  // Issue #27 / #513: the operating mode runs the user's prompt as a single
  // Q&A ("chat"), a multi-step plan ("agent"), or an auto-executing agent
  // ("fullAuto"). Persisted across reloads via preferences. The legacy
  // `agentMode` boolean is derived so existing readers keep working.
  const [mode, setMode] = useState(
    normalizeMode(
      initialPreferences.current.mode,
      initialPreferences.current.agentMode,
    ),
  );
  const agentMode = mode !== "chat";
  const [agentOnboardingSeen, setAgentOnboardingSeen] = useState(
    Boolean(initialPreferences.current.agentOnboardingSeen),
  );
  const [desktopToolGrants, setDesktopToolGrants] = useState(() =>
    normalizeDesktopToolGrants(initialPreferences.current.desktopToolGrants),
  );
  const [commandApprovals, setCommandApprovals] = useState({});
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
  // Issue #541 (R4): demo mode runs in its own conversation so user
  // conversations are never deleted or overwritten. The id lives for the
  // lifetime of the React app and is reused across cycles so the "last
  // example" persists even when the user toggles demo off and back on within
  // a session. It is intentionally NOT persisted as the "current" conversation
  // — `currentConversationRef` keeps pointing at the user's real thread, so
  // restoring the UI on demo-off is a single lookup against IndexedDB.
  const demoConversationIdRef = useRef("");

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
    const bridge = desktopBridge();
    if (!bridge || typeof bridge.onUpdateStatus !== "function") {
      return undefined;
    }
    const unsubscribe = bridge.onUpdateStatus((status) => {
      setDesktopStatus((current) => mergeDesktopUpdateStatus(current, status));
    });
    return typeof unsubscribe === "function" ? unsubscribe : undefined;
  }, []);

  useEffect(() => {
    // Push the explicit per-tool grant map to the local router whenever either
    // the operating mode or a grant decision changes.
    syncDesktopToolGrants(desktopBridge(), mode, desktopToolGrants);
  }, [mode, desktopToolGrants, desktopStatus]);

  useEffect(() => {
    if (mode === "chat") {
      return undefined;
    }
    const bridge = desktopBridge();
    if (!bridge || typeof bridge.ensureAgentServer !== "function") {
      return undefined;
    }
    let cancelled = false;
    ensureDesktopAgentServer(bridge)
      .then((status) => {
        if (!cancelled && status) {
          setDesktopStatus(status);
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setDesktopStatus((current) =>
            normalizeDesktopStatus({
              ...(current || {}),
              shell: (current && current.shell) || "Electron",
              apiReady: false,
              apiError: error && error.message ? error.message : String(error),
            }),
          );
        }
      });
    return () => {
      cancelled = true;
    };
  }, [mode]);

  // Issue #438 (follow-up): poll the desktop bridge for the prepared-container
  // status so the Services panel reflects running/stopped without a manual
  // refresh, and expose it for the one-click buttons.
  const refreshServiceStatus = useCallback(async () => {
    const bridge = desktopServiceBridge();
    if (!bridge) {
      return null;
    }
    try {
      const snapshot = await bridge.serviceStatus();
      setServiceStatus(snapshot && typeof snapshot === "object" ? snapshot : null);
      return snapshot;
    } catch (error) {
      setServiceError(error && error.message ? error.message : String(error));
      return null;
    }
  }, []);

  useEffect(() => {
    const bridge = desktopServiceBridge();
    if (!bridge) {
      return undefined;
    }
    let active = true;
    const tick = () => {
      if (active) {
        refreshServiceStatus();
      }
    };
    tick();
    const timer = setInterval(tick, 5000);
    return () => {
      active = false;
      clearInterval(timer);
    };
  }, [refreshServiceStatus]);

  const handleStartService = useCallback(
    async (key) => {
      const bridge = desktopServiceBridge();
      if (!bridge || typeof bridge.startService !== "function") {
        return;
      }
      setServiceBusy(key);
      setServiceError("");
      try {
        const request = { service: key };
        if (key === "telegram") {
          request.token = telegramToken.trim();
        }
        const result =
          key === "agent" && typeof bridge.installAgentEnvironment === "function"
            ? await bridge.installAgentEnvironment()
            : await bridge.startService(request);
        if (result && result.ok === false && result.reason) {
          setServiceError(result.reason);
        }
      } catch (error) {
        setServiceError(error && error.message ? error.message : String(error));
      } finally {
        setServiceBusy("");
        await refreshServiceStatus();
      }
    },
    [telegramToken, refreshServiceStatus],
  );

  const handleStopService = useCallback(
    async (key) => {
      const bridge = desktopServiceBridge();
      if (!bridge || typeof bridge.stopService !== "function") {
        return;
      }
      setServiceBusy(key);
      setServiceError("");
      try {
        const result = await bridge.stopService({ service: key });
        if (result && result.ok === false && result.reason) {
          setServiceError(result.reason);
        }
      } catch (error) {
        setServiceError(error && error.message ? error.message : String(error));
      } finally {
        setServiceBusy("");
        await refreshServiceStatus();
      }
    },
    [refreshServiceStatus],
  );

  const handleCheckForUpdates = useCallback(async () => {
    const bridge = desktopBridge();
    if (!bridge || typeof bridge.checkForUpdates !== "function") {
      return;
    }
    setUpdateBusy("check");
    try {
      const status = await bridge.checkForUpdates();
      setDesktopStatus((current) => mergeDesktopUpdateStatus(current, status));
    } finally {
      setUpdateBusy("");
    }
  }, []);

  const handleInstallUpdate = useCallback(async () => {
    const bridge = desktopBridge();
    if (!bridge || typeof bridge.installUpdate !== "function") {
      return;
    }
    setUpdateBusy("install");
    try {
      const status = await bridge.installUpdate();
      setDesktopStatus((current) => mergeDesktopUpdateStatus(current, status));
    } finally {
      setUpdateBusy("");
    }
  }, []);

  // Issue #554 (R2): one-click VS Code extension install. The main process
  // detects a VS Code CLI, downloads the latest release `.vsix`, and runs
  // `code --install-extension`; we surface the structured result inline.
  const handleInstallVsCodeExtension = useCallback(async () => {
    const bridge = desktopBridge();
    if (!bridge || typeof bridge.installVsCodeExtension !== "function") {
      return;
    }
    setVscodeInstallBusy(true);
    setVscodeInstallResult(null);
    try {
      const result = await bridge.installVsCodeExtension();
      setVscodeInstallResult(result || { ok: false, state: "error" });
    } catch (error) {
      setVscodeInstallResult({
        ok: false,
        state: "error",
        reason: error && error.message ? error.message : String(error),
      });
    } finally {
      setVscodeInstallBusy(false);
    }
  }, []);

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
        toolbarIconPack,
        locationPreference,
        assistantName,
        guessProbability,
        temperature,
        followUpProbability,
        definitionFusion,
        thinkingDetailLevel,
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
      toolbarIconPack,
      locationPreference,
      assistantName,
      guessProbability,
      temperature,
      followUpProbability,
      definitionFusion,
      thinkingDetailLevel,
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
        if (!next.isImage && attachment.sourceFile && isTextAttachment(attachment.sourceFile)) {
          try {
            const fullText = await readFileAsText(attachment.sourceFile);
            const sample = sampleTextAttachmentContent(fullText);
            next.text = sample.text;
            next.textTruncated = sample.truncated;
          } catch (error) {
            next.textError = error && error.message ? error.message : "File read failed";
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
      sidebarDesktopCollapsed,
      sidebarServicesCollapsed,
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
      thinkingDetailLevel,
      minMessageAnimationMs,
      experimentalOcr,
      ...externalServices,
      associativeProjectPromotion,
      theme: themePreference,
      uiSkin,
      chatStyle,
      composerStyle,
      composerAction,
      toolbarIconPack,
      location: locationPreference,
      assistantName: normalizeAssistantName(assistantName),
      currentConversationId,
      mode,
      agentMode,
      agentOnboardingSeen,
      desktopToolGrants: serializeDesktopToolGrants(desktopToolGrants),
      uiLanguage: uiLanguagePreference,
      responseLanguage,
      preferredLanguage,
    });
  }, [
    demoMode,
    diagnosticsMode,
    contextPanelWidth,
    sidebarMenuCollapsed,
    sidebarDesktopCollapsed,
    sidebarServicesCollapsed,
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
    thinkingDetailLevel,
    minMessageAnimationMs,
    experimentalOcr,
    externalServices,
    associativeProjectPromotion,
    themePreference,
    uiSkin,
    chatStyle,
    composerStyle,
    composerAction,
    toolbarIconPack,
    locationPreference,
    assistantName,
    currentConversationId,
    mode,
    agentMode,
    agentOnboardingSeen,
    desktopToolGrants,
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

  // Issue #444: mirror the external trusted-service toggles into a ref so the
  // worker prefs payload (assembled outside React render) reads the live values.
  const externalServicesRef = useRef(externalServices);
  useEffect(() => {
    externalServicesRef.current = externalServices;
  }, [externalServices]);

  const associativeProjectPromotionRef = useRef(associativeProjectPromotion);
  useEffect(() => {
    associativeProjectPromotionRef.current = associativeProjectPromotion;
  }, [associativeProjectPromotion]);

  const agentModeRef = useRef(agentMode);
  useEffect(() => {
    agentModeRef.current = agentMode;
  }, [agentMode]);

  // Issue #513: mirror the three-way operating mode for the worker prefs payload.
  const modeRef = useRef(mode);
  useEffect(() => {
    modeRef.current = mode;
  }, [mode]);

  const agentOnboardingSeenRef = useRef(agentOnboardingSeen);
  useEffect(() => {
    agentOnboardingSeenRef.current = agentOnboardingSeen;
  }, [agentOnboardingSeen]);

  const desktopToolGrantsRef = useRef(desktopToolGrants);
  useEffect(() => {
    desktopToolGrantsRef.current = desktopToolGrants;
  }, [desktopToolGrants]);

  const commandApprovalsRef = useRef(commandApprovals);
  useEffect(() => {
    commandApprovalsRef.current = commandApprovals;
  }, [commandApprovals]);

  // Issue #541 (R9): when the chat surface refuses to execute a shell command
  // because the user is not in Agent mode or has not granted `shell`, we stash
  // the original command here so a single click on the permission panel's
  // "Grant all and switch to Agent mode" button can replay it. The ref holds
  // the live value (read inside async callbacks); the state mirrors whether
  // a task is queued so the panel can change its button copy.
  const pendingAgentTaskRef = useRef(null);
  const [hasPendingAgentTask, setHasPendingAgentTask] = useState(false);

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

  const requestAnswer = useCallback(async (text, history = []) => {
    const worker = workerRef.current;
    // Issue #529: snapshot every searchable persistent-memory value so the
    // worker can report how many occurrences a natural-language substitution
    // rewrites. The actual read+write transform is applied back to IndexedDB
    // when the answer returns (see handleMemoryOperation).
    let memory = [];
    if (typeof window !== "undefined" && window.FormalAiMemory) {
      try {
        memory = await window.FormalAiMemory.collectSearchableValues();
      } catch (_error) {
        memory = [];
      }
    }
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
      // Issue #444: forward every external trusted-service opt-out so the worker
      // can skip a disabled service's live fetch.
      ...externalServicesRef.current,
      associativeProjectPromotion: associativeProjectPromotionRef.current,
      agentMode: agentModeRef.current,
      mode: modeRef.current,
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
            memory,
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
        memory,
      });
    });
  }, []);

  // Issue #27: assign every appended event to the current conversation, lazily
  // minting a fresh id on the first user message of a brand-new chat. The
  // returned object is { conversationId, conversationTitle } so the caller can
  // reuse it for follow-up records within the same turn (assistant reply,
  // reasoning steps, tool calls).
  const ensureConversation = useCallback((seedText) => {
    // Issue #541 (R4): when demo mode is active, route every persisted event
    // into a dedicated demo conversation. We never touch
    // `currentConversationRef`/`setCurrentConversationId` from this path so the
    // user's real thread is preserved exactly as they left it — toggling demo
    // off restores their conversation untouched.
    if (demoModeRef.current) {
      if (!demoConversationIdRef.current) {
        demoConversationIdRef.current = generateConversationId();
      }
      const id = demoConversationIdRef.current;
      let title = conversationTitlesRef.current.get(id);
      if (!title) {
        title = t("buttons.demoOn") || "Demo";
        conversationTitlesRef.current.set(id, title);
      }
      return {
        conversationId: id,
        conversationTitle: title,
        isNew: false,
        isDemo: true,
      };
    }
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
  }, [t]);

  const appendUserMessage = useCallback((text, extra = {}) => {
    const { conversationId, conversationTitle, isDemo } = ensureConversation(text);
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
      // Issue #541 (R4): flag demo turns so the sidebar can hide the demo
      // conversation and never list it alongside the user's real threads.
      isDemo: isDemo ? true : undefined,
    });
  }, [ensureConversation]);

  const appendSystemMessage = useCallback((content, extra = {}) => {
    const { conversationId, conversationTitle, isDemo } = ensureConversation("");
    const message = createMessage("system", content, {
      author: "formal-ai system",
      ...extra,
    });
    setMessages((current) => [...current, message]);
    recordMemoryEvent({
      kind: "message",
      role: "system",
      content,
      intent: extra.intent,
      sentAt: new Date().toISOString(),
      conversationId,
      conversationTitle,
      // Issue #541 (R4): flag demo turns so the sidebar can hide them.
      isDemo: isDemo ? true : undefined,
    }).then(() => {
      refreshConversations();
    });
    return message;
  }, [ensureConversation, refreshConversations]);

  const showAgentOnboarding = useCallback(() => {
    if (agentOnboardingSeenRef.current) {
      return false;
    }
    agentOnboardingSeenRef.current = true;
    setAgentOnboardingSeen(true);
    appendSystemMessage(
      [
        t("permissions.onboarding.intro"),
        t("permissions.onboarding.perTool"),
        t("permissions.onboarding.modes"),
      ].join("\n\n"),
      {
        intent: "agent_permission_onboarding",
        permissionPanel: true,
      },
    );
    return true;
  }, [appendSystemMessage, t]);

  const setDesktopToolGrant = useCallback((tool, granted) => {
    if (!DESKTOP_TOOL_OPTIONS.includes(tool)) {
      return;
    }
    setDesktopToolGrants((current) => ({
      ...current,
      [tool]: Boolean(granted),
    }));
  }, []);

  // Issue #541 (R9): record a shell command that was deferred because the user
  // was not in Agent mode (or had not granted `shell`). The "Grant all" button
  // on the permission panel reads this and replays the command.
  const capturePendingAgentTask = useCallback((command) => {
    const safeCommand = String(command || "").trim();
    if (!safeCommand) {
      return;
    }
    pendingAgentTaskRef.current = { kind: "shell", command: safeCommand };
    setHasPendingAgentTask(true);
  }, []);

  const clearPendingAgentTask = useCallback(() => {
    pendingAgentTaskRef.current = null;
    setHasPendingAgentTask(false);
  }, []);

  useEffect(() => {
    if (agentMode) {
      showAgentOnboarding();
    }
  }, [agentMode, showAgentOnboarding]);

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
    const thinkingPreviewSteps = buildThinkingPreviewSteps(
      structuredSteps,
      answer,
      source,
      t,
      thinkingDetailLevel,
    );
    const message = createMessage("assistant", answer.content, {
      intent: answer.intent,
      evidence,
      thinkingSteps,
      thinkingPreviewSteps,
      thinkingPreviewSource: source,
      diagnosticsSteps: structuredSteps,
      diagnosticsToolCalls: structuredToolCalls,
      // Issue #180: forward the web_search diagnostics envelope so the
      // diagnostics panel can show raw HTTP request/response exchanges and
      // the per-provider success/failure status.
      diagnostics: answer.diagnostics || null,
      iframeUrl: answer.iframeUrl || null,
      // Issue #541 (R5/R6): mark this as a freshly produced answer so the
      // Message component stages its reasoning-then-body reveal across the
      // minimum animation budget. Hydrated history (rebuilt from memory events)
      // never carries this flag, so reloads render instantly.
      animateReveal: true,
    });
    setMessages((current) => [...current, message]);
    const sentAt = new Date().toISOString();
    const { conversationId, conversationTitle, isDemo } = ensureConversation("");
    // Issue #541 (R4): flag every persisted record of this turn (reasoning
    // step, tool call, and the message itself) so the sidebar can hide the
    // demo conversation. Set undefined rather than false for non-demo turns
    // so we don't litter the IndexedDB log with redundant keys.
    const demoFlag = isDemo ? true : undefined;
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
          isDemo: demoFlag,
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
          isDemo: demoFlag,
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
      isDemo: demoFlag,
    }).then(() => {
      // Refresh the sidebar so a brand-new conversation appears immediately.
      refreshConversations();
    });
  }, [ensureConversation, refreshConversations, t, thinkingDetailLevel]);

  // Issue #529: apply a natural-language memory write returned by the worker to
  // the persistent associative memory. This is the *write* half of the
  // Turing-complete memory primitive, the browser mirror of try_memory_write in
  // the Rust runtime: an append stores the bare statement as a new, queryable
  // memory event; a substitution rewrites every matching stored value in place
  // (an explicit, user-initiated departure from the passive append-only log) and
  // records an audit event. The user thereby has full read+write control over
  // the associative memory through ordinary chat messages.
  const handleMemoryOperation = useCallback(async (operation) => {
    if (
      !operation ||
      typeof window === "undefined" ||
      !window.FormalAiMemory
    ) {
      return;
    }
    const { conversationId, conversationTitle, isDemo } = ensureConversation("");
    const sentAt = new Date().toISOString();
    const demoFlag = isDemo ? true : undefined;
    if (operation.action === "append" && operation.statement) {
      await recordMemoryEvent({
        kind: "message",
        role: "user",
        intent: "memory_write",
        content: operation.statement,
        evidence: ["memory_write:natural_language"],
        sentAt,
        conversationId,
        conversationTitle,
        isDemo: demoFlag,
      });
      refreshConversations();
      return;
    }
    if (operation.action === "substitute" && operation.oldValue) {
      let applied = 0;
      try {
        applied = await window.FormalAiMemory.applySubstitution(
          operation.oldValue,
          operation.newValue,
        );
      } catch (_error) {
        applied = 0;
      }
      await recordMemoryEvent({
        kind: "memory_substitution",
        role: "user",
        intent: "memory_substitution",
        inputs: `replace:${operation.oldValue}`,
        outputs: `with:${operation.newValue}`,
        content: `replace ${operation.oldValue} with ${operation.newValue} in memory`,
        evidence: ["substitution_event:update", `substitution:applied=${applied}`],
        sentAt,
        conversationId,
        conversationTitle,
        isDemo: demoFlag,
      });
      refreshConversations();
    }
  }, [ensureConversation, refreshConversations]);

  const executeTerminalCommand = useCallback(async (command, executionMode = "agent") => {
    const bridge = desktopBridge();
    const providerResult = await requestDesktopAgentProvider(bridge, {
      mode: executionMode === "fullAuto" ? "fullAuto" : "agent",
      tool: "shell",
      command,
      grants: desktopToolRouterGrants(modeRef.current, desktopToolGrantsRef.current),
      transcript: true,
    });
    const providerAnswer = chatAnswerFromAgentProviderResult(providerResult);
    if (providerAnswer) {
      appendAssistantMessage({
        ...providerAnswer,
        content: String(providerAnswer.content || desktopToolResultReason(providerResult, t)),
        evidence: [
          ...(Array.isArray(providerAnswer.evidence) ? providerAnswer.evidence : []),
          "desktop_agent_provider",
          `mode:${executionMode}`,
        ],
      });
      return providerResult;
    }

    const result = providerResult || (await requestDesktopToolCall(bridge, "shell", { command }));
    const ok = result && result.ok === true && result.executed === true;
    const content = ok
      ? [
          t("permissions.message.shellRan", {
            mode:
              executionMode === "fullAuto"
                ? t("buttons.fullAuto")
                : t("buttons.agent"),
            command,
          }),
          "",
          shellOutputMarkdown(result.body, t),
        ].join("\n")
      : [
          t("permissions.message.shellNotRun", { command }),
          "",
          desktopToolResultReason(result, t),
        ].join("\n");
    appendAssistantMessage({
      intent: ok ? "desktop_shell_result" : "desktop_shell_refused",
      content,
      confidence: ok ? 0.9 : 1.0,
      evidence: [
        "desktop_tool:shell",
        `mode:${executionMode}`,
        ok ? "desktop_tool:executed" : "desktop_tool:refused",
      ],
      steps: [
        {
          step: ok ? "execute_shell" : "refuse_shell",
          detail: command,
        },
      ],
      toolCalls: [
        {
          tool: "shell",
          inputs: { command, mode: executionMode },
          outputs: result || { ok: false, executed: false, status: "unavailable" },
        },
      ],
    });
    return result;
  }, [appendAssistantMessage, t]);

  // Issue #541 (R9): single-click escalation from the permission panel.
  // 1. Mode flips to "agent" so `requestTerminalCommandExecution` will route
  //    the shell command instead of bouncing it back to chat.
  // 2. Every desktop tool grant flips to true so the router approves the call.
  // 3. Refs are mirrored synchronously (React state lands on the next render,
  //    but `executeTerminalCommand` reads `modeRef.current` and
  //    `desktopToolGrantsRef.current` synchronously inside this callback —
  //    without the manual mirror the replay would race the React update).
  // 4. The replayed command runs in "agent" mode (per-command prompt) so the
  //    user still sees the approve/deny step for the deferred shell command
  //    rather than skipping straight to autorun. If they wanted skip-prompt
  //    behaviour they would choose Full Auto mode instead.
  const grantAllAndRunPending = useCallback(async () => {
    const allGranted = {};
    DESKTOP_TOOL_OPTIONS.forEach((tool) => { allGranted[tool] = true; });
    desktopToolGrantsRef.current = allGranted;
    setDesktopToolGrants(allGranted);
    modeRef.current = "agent";
    agentModeRef.current = true;
    setMode("agent");
    const task = pendingAgentTaskRef.current;
    clearPendingAgentTask();
    if (task && task.kind === "shell" && task.command) {
      await executeTerminalCommand(task.command, "agent");
    }
  }, [clearPendingAgentTask, executeTerminalCommand]);

  const requestTerminalCommandExecution = useCallback(
    async (command, answer) => {
      const safeCommand = String(command || "").trim();
      if (!safeCommand) {
        appendAssistantMessage(answer);
        return;
      }
      if (!agentModeRef.current) {
        // Issue #541 (R9): stash the command so the panel's "Grant all" button
        // can replay it without the user having to retype the prompt.
        capturePendingAgentTask(safeCommand);
        appendAssistantMessage(answer);
        showAgentOnboarding();
        return;
      }
      showAgentOnboarding();
      if (desktopToolGrantsRef.current.shell !== true) {
        // Same as above — user opted in to Agent mode but hasn't granted
        // `shell` yet. Stash the command for one-click recovery.
        capturePendingAgentTask(safeCommand);
        appendAssistantMessage({
          intent: "desktop_shell_not_granted",
          content: t("permissions.message.shellNotGranted"),
          confidence: 1.0,
          evidence: ["desktop_tool:shell", "desktop_tool:not_granted"],
          steps: [{ step: "check_tool_grant", detail: "shell=false" }],
          toolCalls: [
            {
              tool: "shell",
              inputs: { command: safeCommand },
              outputs: {
                ok: false,
                executed: false,
                status: "refused",
                reason: "shell tool is not granted",
              },
            },
          ],
        });
        return;
      }
      if (modeRef.current === "fullAuto") {
        await executeTerminalCommand(safeCommand, "fullAuto");
        return;
      }
      const approval = {
        id: `command-${Date.now()}-${Math.random().toString(16).slice(2)}`,
        tool: "shell",
        command: safeCommand,
        status: "pending",
      };
      setCommandApprovals((current) => ({
        ...current,
        [approval.id]: approval,
      }));
      appendSystemMessage(
        `${t("permissions.message.approvalPrompt")}\n\n\`${safeCommand}\``,
        {
          intent: "desktop_command_approval",
          commandApproval: approval,
        },
      );
    },
    [appendAssistantMessage, appendSystemMessage, capturePendingAgentTask, executeTerminalCommand, showAgentOnboarding, t],
  );

  const approveDesktopCommand = useCallback(
    async (approval) => {
      if (!approval || !approval.id) {
        return;
      }
      const existing = commandApprovalsRef.current[approval.id] || approval;
      if (existing.status !== "pending") {
        return;
      }
      const running = { ...existing, status: "running" };
      commandApprovalsRef.current = {
        ...commandApprovalsRef.current,
        [approval.id]: running,
      };
      setCommandApprovals(commandApprovalsRef.current);
      await executeTerminalCommand(approval.command, "agent");
      const approved = { ...running, status: "approved" };
      commandApprovalsRef.current = {
        ...commandApprovalsRef.current,
        [approval.id]: approved,
      };
      setCommandApprovals(commandApprovalsRef.current);
    },
    [executeTerminalCommand],
  );

  const denyDesktopCommand = useCallback(
    (approval) => {
      if (!approval || !approval.id) {
        return;
      }
      const existing = commandApprovalsRef.current[approval.id] || approval;
      if (existing.status !== "pending") {
        return;
      }
      const denied = { ...existing, status: "denied" };
      commandApprovalsRef.current = {
        ...commandApprovalsRef.current,
        [approval.id]: denied,
      };
      setCommandApprovals(commandApprovalsRef.current);
      appendAssistantMessage({
        intent: "desktop_shell_denied",
        content: t("permissions.message.commandDeclined", {
          command: approval.command,
        }),
        confidence: 1.0,
        evidence: ["desktop_tool:shell", "desktop_tool:user_denied"],
        steps: [{ step: "user_denied_shell", detail: approval.command }],
        toolCalls: [
          {
            tool: "shell",
            inputs: { command: approval.command, mode: "agent" },
            outputs: { ok: false, executed: false, status: "denied" },
          },
        ],
      });
    },
    [appendAssistantMessage, t],
  );

  const conversationHistory = useCallback(
    () =>
      messages
        .filter((message) => ["user", "assistant"].includes(message.role))
        .map((message) => ({
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
          // Issue #513: the legacy natural-language "agent mode on/off" command
          // maps onto the three-way mode (preserving full-auto when already set).
          setMode((current) =>
            command.value
              ? current === "fullAuto"
                ? "fullAuto"
                : "agent"
              : "chat",
          );
          break;
        case "mode":
          setMode(normalizeMode(command.value));
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
        case "thinkingDetailLevel":
          setThinkingDetailLevel(normalizeThinkingDetailLevel(command.value));
          break;
        case "minMessageAnimationMs":
          setMinMessageAnimationMs(normalizeAnimationBudgetMs(command.value));
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
        case "followUpProbability":
          setFollowUpProbability(
            normalizeSliderPreference(
              command.value,
              PREFERENCE_DEFAULTS.followUpProbability,
            ),
          );
          break;
        case "toolbarIconPack":
          setToolbarIconPack(normalizeToolbarIconPack(command.value));
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

    const interfaceCommand = hasAttachments
      ? null
      : recognizeInterfaceCommand(displayText, seed.interfaceCapabilities);
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
    const terminalCommand = terminalCommandFromAnswer(answer);
    if (terminalCommand) {
      await requestTerminalCommandExecution(terminalCommand, answer);
      setPending(false);
      return;
    }
    appendAssistantMessage(answer);
    // Issue #529: persist any natural-language memory write (append/substitution)
    // the worker recognised, giving the user full read+write control over the
    // associative memory through chat.
    if (answer.memoryOperation) {
      await handleMemoryOperation(answer.memoryOperation);
    }
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

  // Issue #541 (R4): tracks whether the previous render was in demo mode so we
  // can detect the on→off transition and restore the user's real conversation
  // back into the UI. Demo writes go to a dedicated demo conversation (see
  // `demoConversationIdRef`), so restoration is a single IndexedDB lookup
  // against the still-pointed-at `currentConversationRef`.
  const demoWasActiveRef = useRef(demoMode);
  useEffect(() => {
    const wasActive = demoWasActiveRef.current;
    demoWasActiveRef.current = demoMode;

    if (!demoMode) {
      setDemoPhase("manual");
      setDemoCountdown(null);
      if (wasActive) {
        // Demo just turned off — restore whatever the user had open before
        // demo took over. If they had no prior conversation, fall back to an
        // empty composer rather than leaking the last demo turn into the UI.
        const userId = currentConversationRef.current;
        const cachedEvents = conversationEventsRef.current;
        const restored = userId
          ? messagesForConversation(cachedEvents, userId)
          : [];
        setMessages(restored);
        setPending(false);
      }
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
  // Issue #513: localized labels for the three-way operating-mode radio group
  // and the status indicator that reflects the active mode.
  const modeLabel = (option) => t(MODE_LABEL_KEYS[option]);
  const modeTitle = (option) => t(MODE_TITLE_KEYS[option]);
  const modeStatusText = t("status.mode", { mode: modeLabel(mode) });
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
    { key: "thinkingDetailLevel", value: thinkingDetailLevel, set: setThinkingDetailLevel, label: "settings.thinkingDetail" },
    { key: "minMessageAnimationMs", value: minMessageAnimationMs, set: setMinMessageAnimationMs, label: "settings.minMessageAnimation" },
    { key: "experimentalOcr", value: experimentalOcr, set: setExperimentalOcr, label: "settings.experimentalOcr" },
    // Issue #444: one reset descriptor per external trusted service so the
    // "modified settings" reset bar lists any service the user turned off.
    ...EXTERNAL_TRUSTED_SERVICES.map((service) => ({
      key: service.key,
      value: externalServices[service.key],
      set: (next) => setExternalService(service.key, next),
      label: service.label,
    })),
    { key: "uiLanguage", value: uiLanguagePreference, set: setUiLanguagePreference, label: "settings.language" },
    { key: "responseLanguage", value: responseLanguage, set: setResponseLanguage, label: "settings.responseLanguage" },
    { key: "preferredLanguage", value: preferredLanguage, set: setPreferredLanguage, label: "settings.preferredLanguage" },
    { key: "theme", value: themePreference, set: setThemePreference, label: "settings.theme" },
    { key: "uiSkin", value: uiSkin, set: setUiSkin, label: "settings.uiSkin" },
    { key: "chatStyle", value: chatStyle, set: setChatStyle, label: "settings.chatStyle" },
    { key: "composerStyle", value: composerStyle, set: setComposerStyle, label: "settings.composerStyle" },
    { key: "composerAction", value: composerAction, set: setComposerAction, label: "settings.composerAction" },
    { key: "toolbarIconPack", value: toolbarIconPack, set: setToolbarIconPack, label: "settings.toolbarIconPack" },
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

  const composerActionIcon =
    composerAction === "plus"
      ? "+"
      : <ToolbarIcon action="attachFiles" pack={toolbarIconPack} className="composer-action-icon" />;
  const attachmentStatus =
    attachments.length > 0
      ? t("composer.attachments", { count: attachments.length })
      : "";
  const desktopStatusText = desktopStatusLabel(desktopStatus, agentMode);
  const desktopAgentPermission = agentMode ? "Opted in" : "Off";
  const desktopGrantedToolCount = desktopToolGrantCount(desktopToolGrants);
  const desktopToolPermission = t("permissions.toolCount", {
    granted: desktopGrantedToolCount,
    total: DESKTOP_TOOL_OPTIONS.length,
  });
  const appVersionLabel = desktopAppVersionLabel(desktopStatus);
  const updater = desktopStatus && desktopStatus.updater;
  const updateInFlight = updateBusy || (updater && desktopUpdaterBusy(updater));
  const canCheckForUpdates = Boolean(
    updater
      && updater.supported
      && updater.enabled
      && !updateInFlight,
  );
  const canInstallUpdate = Boolean(
    updater
      && updater.supported
      && updater.enabled
      && (updater.updateAvailable || updater.downloaded)
      && !updateInFlight,
  );
  const renderDesktopPermissionPanel = (testId) =>
    <DesktopPermissionPanel grants={desktopToolGrants} mode={mode} onDecision={setDesktopToolGrant} onGrantAll={grantAllAndRunPending} hasPendingTask={hasPendingAgentTask} testId={testId} t={t} />;

  return <main className={["app", `ui-skin-${uiSkin}`, `chat-style-${chatStyle}`, `composer-style-${composerStyle}`, `toolbar-icon-pack-${toolbarIconPack}`, desktopStatus ? "desktop-shell" : ""].filter(Boolean).join(" ")}><chakra.header className="topbar">
      <ToolbarButton className="mobile-menu-toggle topbar-menu-toggle" testId="mobile-menu-toggle" ariaLabel={mobileMenuOpen ? t("buttons.closeMenu") : t("buttons.openMenu")} title={mobileMenuOpen ? t("titles.menuClose") : t("titles.menuOpen")} onClick={() => setMobileMenuOpen(value => !value)} extraProps={{
      "aria-pressed": mobileMenuOpen
    }}>
        <MenuGlyph open={mobileMenuOpen} />
      </ToolbarButton>
      <ToolbarButton className={`sidebar-toggle${sidebarCollapsed ? " is-collapsed" : ""}`} testId="sidebar-toggle" ariaLabel={sidebarCollapsed ? t("buttons.expandSidebar") : t("buttons.collapseSidebar")} title={sidebarCollapsed ? t("titles.expandSidebar") : t("titles.collapseSidebar")} onClick={() => setSidebarCollapsed(value => !value)} extraProps={{
      "aria-pressed": !sidebarCollapsed
    }}>
        <SidebarToggleGlyph collapsed={sidebarCollapsed} />
      </ToolbarButton>
      <chakra.div className="brand">
        <chakra.span className="mark">FA</chakra.span>
        <chakra.strong>formal-ai</chakra.strong>
        <chakra.span className="brand-version" data-testid="app-version">
          {appVersionLabel}
        </chakra.span>
      </chakra.div>
      <chakra.div className="topbar-actions">
        {desktopStatus ? <chakra.span className="desktop-status" data-testid="desktop-shell-status" data-menu-priority="7" role="status" title={desktopStatus.apiError || desktopStatusText}>
            {desktopStatusText}
          </chakra.span> : null}
        <chakra.span className="demo-status" data-testid="demo-status" data-menu-priority="7" role="status">
          {demoStatus}
        </chakra.span>
        <chakra.span className={`mode-status mode-status-${mode}`} data-testid="mode-status" data-mode={mode} data-menu-priority="7" role="status">
          {modeStatusText}
        </chakra.span>
        {diagnosticsMode ? <chakra.span className="status" data-menu-priority="7">
            {workerState}
          </chakra.span> : null}
        <ToolbarButton className="source-code-button" testId="source-code" menuPriority="5" href={SOURCE_CODE_URL} target="_blank" rel="noopener noreferrer" title={t("titles.sourceCode")} ariaLabel={t("buttons.sourceCode")} icon="sourceCode" iconPack={toolbarIconPack} label={t("buttons.sourceCode")} />
        <ToolbarButton className="download-button" testId="download-link" menuPriority="5" href="download/" title={t("titles.download")} ariaLabel={t("buttons.download")} icon="download" iconPack={toolbarIconPack} label={t("buttons.download")} />
        <ToolbarButton className="report-button" testId="report-issue" menuPriority="1" href={currentReportUrl} target="_blank" rel="noopener noreferrer" title={t("titles.reportIssue")} ariaLabel={t("buttons.reportIssue")} icon="reportIssue" iconPack={toolbarIconPack} label={t("buttons.reportIssue")} />
        <ToolbarButton className="memory-button" testId="memory-export" menuPriority="6" onClick={handleExportMemory} title={t("titles.exportMemory")} ariaLabel={t("buttons.exportMemory")} icon="exportMemory" iconPack={toolbarIconPack} label={t("buttons.exportMemory")} />
        <ToolbarButton className="memory-button" testId="memory-import" menuPriority="6" onClick={triggerImportMemory} title={t("titles.importMemory")} ariaLabel={t("buttons.importMemory")} icon="importMemory" iconPack={toolbarIconPack} label={t("buttons.importMemory")} />
        <ToolbarButton className="memory-button memory-reset-button" testId="memory-reset" menuPriority="6" onClick={handleResetMemory} title={t("titles.resetMemory")} ariaLabel={t("buttons.resetMemory")} icon="resetMemory" iconPack={toolbarIconPack} label={t("buttons.resetMemory")} />
        <chakra.input ref={importInputRef} type="file" accept=".lino,text/plain" style={{
        display: "none"
      }} data-testid="memory-import-input" onChange={handleImportMemory} />
        {memoryStatus ? <chakra.span className="memory-status" role="status" data-testid="memory-status" data-menu-priority="7">
            {memoryStatus}
          </chakra.span> : null}
        <ToolbarButton className="diagnostics-toggle" menuPriority="2" onClick={() => setDiagnosticsMode(value => !value)} title={diagnosticsMode ? t("titles.diagnosticsHide") : t("titles.diagnosticsShow")} ariaLabel={diagnosticsMode ? t("buttons.diagnosticsOn") : t("buttons.diagnostics")} icon="diagnostics" iconPack={toolbarIconPack} label={diagnosticsMode ? t("buttons.diagnosticsOn") : t("buttons.diagnostics")} extraProps={{
        "aria-pressed": diagnosticsMode
      }} />
        <chakra.div className="mode-radio" data-testid="mode-radio" data-menu-priority="4" role="radiogroup" aria-label={t("titles.modeGroup")}>
          {MODE_OPTIONS.map(option => <ToolbarButton key={option} className={`mode-option mode-option-${option}${mode === option ? " is-active" : ""}`} testId={`mode-option-${option}`} title={modeTitle(option)} ariaLabel={modeLabel(option)} icon={option === "chat" ? "chat" : "agent"} iconPack={toolbarIconPack} label={modeLabel(option)} onClick={() => setMode(option)} extraProps={{
          "data-mode": option,
          role: "radio",
          "aria-checked": mode === option
        }} />)}
        </chakra.div>
        <ToolbarButton className="mode-toggle" menuPriority="3" onClick={() => setDemoMode(value => !value)} title={demoMode ? t("titles.demoOn") : t("titles.demoOff")} ariaLabel={demoMode ? t("buttons.demoOn") : t("buttons.demo")} icon="demo" iconPack={toolbarIconPack} label={demoMode ? t("buttons.demoOn") : t("buttons.demo")} extraProps={{
        "aria-pressed": demoMode
      }} />
      </chakra.div>
    </chakra.header>{mobileMenuOpen ? <div className="mobile-menu-backdrop" data-testid="mobile-menu-backdrop" onClick={() => setMobileMenuOpen(false)} /> : null}<section className={`workspace${sidebarCollapsed ? " sidebar-collapsed" : ""}`} style={{
    "--context-panel-width": `${contextPanelWidth}px`
  }}><aside className={`context-panel${mobileMenuOpen ? " is-mobile-open" : ""}${sidebarCollapsed ? " is-desktop-collapsed" : ""}`} data-testid="context-panel" aria-hidden={sidebarCollapsed && !mobileMenuOpen ? "true" : "false"} onClickCapture={handleSidebarSectionClickCapture}><div className="drawer-brand" data-testid="drawer-brand"><div className="drawer-brand-main"><span className="mark">{"FA"}</span><div className="drawer-brand-copy"><strong>{"formal-ai"}</strong><span className="brand-version">{appVersionLabel}</span></div></div><button type="button" className="drawer-close" data-testid="drawer-close" aria-label={t("buttons.closeMenu")} title={t("titles.menuClose")} onClick={() => setMobileMenuOpen(false)}><MenuGlyph open={true} /></button></div><SidebarSection title={t("sidebar.menu")} testId="drawer-menu-actions" collapsed={sidebarMenuCollapsed} onToggle={() => setSidebarMenuCollapsed(value => !value)} className="drawer-menu-section" bodyClassName="drawer-menu-body" children={<div className="drawer-action-list"><a className="drawer-action" data-testid="drawer-source-code" href={SOURCE_CODE_URL} target="_blank" rel="noopener noreferrer"><ToolbarIcon action="sourceCode" pack={toolbarIconPack} /><span>{t("buttons.sourceCode")}</span></a><a className="drawer-action" data-testid="drawer-report-issue" href={currentReportUrl} target="_blank" rel="noopener noreferrer"><ToolbarIcon action="reportIssue" pack={toolbarIconPack} /><span>{t("buttons.reportIssue")}</span></a><button type="button" className="drawer-action" data-testid="drawer-memory-export" onClick={handleExportMemory}><ToolbarIcon action="exportMemory" pack={toolbarIconPack} /><span>{t("buttons.exportMemory")}</span></button><button type="button" className="drawer-action" data-testid="drawer-memory-import" onClick={triggerImportMemory}><ToolbarIcon action="importMemory" pack={toolbarIconPack} /><span>{t("buttons.importMemory")}</span></button><button type="button" className="drawer-action" data-testid="drawer-memory-reset" onClick={handleResetMemory}><ToolbarIcon action="resetMemory" pack={toolbarIconPack} /><span>{t("buttons.resetMemory")}</span></button><button type="button" className="drawer-action" aria-pressed={diagnosticsMode} onClick={() => setDiagnosticsMode(value => !value)}><ToolbarIcon action="diagnostics" pack={toolbarIconPack} /><span>{diagnosticsMode ? t("buttons.diagnosticsOn") : t("buttons.diagnostics")}</span></button><div className="drawer-action drawer-mode-radio" data-testid="drawer-mode-radio" role="radiogroup" aria-label={t("titles.modeGroup")}>{MODE_OPTIONS.map(option => <button key={option} type="button" className={`mode-option mode-option-${option}${mode === option ? " is-active" : ""}`} data-testid={`drawer-mode-option-${option}`} data-mode={option} role="radio" aria-checked={mode === option} title={modeTitle(option)} onClick={() => setMode(option)}><ToolbarIcon action={option === "chat" ? "chat" : "agent"} pack={toolbarIconPack} /><span>{modeLabel(option)}</span></button>)}</div><button type="button" className="drawer-action" aria-pressed={demoMode} onClick={() => setDemoMode(value => !value)}><ToolbarIcon action="demo" pack={toolbarIconPack} /><span>{demoMode ? t("buttons.demoOn") : t("buttons.demo")}</span></button></div>} />{desktopStatus ? <SidebarSection title={desktopSurfaceLabel(desktopStatus)} testId="sidebar-desktop" collapsed={sidebarDesktopCollapsed} onToggle={() => setSidebarDesktopCollapsed(value => !value)} className="desktop-shell-section" children={<dl className="desktop-shell-panel" data-testid="desktop-shell-panel"><div><dt>{"Shell"}</dt><dd>{desktopStatus.shell}</dd></div><div><dt>{t("updates.currentVersion")}</dt><dd data-testid="desktop-app-version">{appVersionLabel}</dd></div><div><dt>{"API"}</dt><dd data-testid="desktop-api-base">{compactUrl(desktopStatus.apiBase)}</dd></div><div><dt>{"Network"}</dt><dd><a href={desktopStatus.graphUrl || "#"} target="_blank" rel="noopener noreferrer" data-testid="desktop-network-link">{compactUrl(desktopStatus.graphUrl)}</a></dd></div><div><dt>{"Memory"}</dt><dd data-testid="desktop-memory-bundle">{desktopStatus.memory}</dd></div><div><dt>{"Agent"}</dt><dd data-testid="desktop-agent-permission">{desktopAgentPermission}</dd></div><div><dt>{"Tool calls"}</dt><dd data-testid="desktop-tool-permission">{desktopToolPermission}</dd></div><div className="desktop-permission-row"><dt>{t("permissions.panel.rowLabel")}</dt><dd>{renderDesktopPermissionPanel("desktop-permission-panel-sidebar")}</dd></div>{updater ? <div className="desktop-update-row"><dt>{t("updates.title")}</dt><dd><div className="desktop-update-panel" data-testid="desktop-update-panel" data-state={updater.state}><span className="desktop-update-state" data-testid="desktop-update-state" role={updater.updateAvailable || updater.downloaded ? "status" : undefined}>{desktopUpdaterStateLabel(updater, t)}</span>{updater.state === "downloading" ? <progress className="desktop-update-progress" data-testid="desktop-update-progress" max="100" value={String(Math.round(updater.progressPercent || 0))} aria-label={t("updates.progress", {
                percent: Math.round(updater.progressPercent || 0)
              })} /> : null}<div className="desktop-update-actions"><button type="button" data-testid="desktop-update-check" disabled={!canCheckForUpdates} onClick={handleCheckForUpdates}>{updateBusy === "check" || updater && updater.state === "checking" ? t("updates.checking") : t("updates.check")}</button><button type="button" className="desktop-update-install" data-testid="desktop-update-install" disabled={!canInstallUpdate} onClick={handleInstallUpdate}>{updateBusy === "install" || updater && updater.state === "installing" ? t("updates.updating") : t("updates.update")}</button></div></div></dd></div> : null}<div className="desktop-vscode-row" data-testid="desktop-vscode-install-row"><dt>{t("vscodeInstall.title")}</dt><dd><div className="desktop-vscode-panel" data-testid="desktop-vscode-install-panel"><p className="desktop-vscode-summary">{t("vscodeInstall.summary")}</p><div className="desktop-vscode-actions"><button type="button" className="desktop-vscode-install" data-testid="desktop-vscode-install" disabled={vscodeInstallBusy} onClick={handleInstallVsCodeExtension}>{vscodeInstallBusy ? t("vscodeInstall.installing") : t("vscodeInstall.install")}</button></div>{vscodeInstallResult ? <p className={`desktop-vscode-status${vscodeInstallResult.ok ? " is-ok" : " is-error"}`} data-testid="desktop-vscode-install-status" role="status">{vscodeInstallStateLabel(vscodeInstallResult, t)}{vscodeInstallResult.ok || !vscodeInstallResult.reason ? "" : ` — ${vscodeInstallResult.reason}`}</p> : null}</div></dd></div></dl>} /> : null}{serviceStatus ? <SidebarSection title={t("services.title")} testId="sidebar-services" collapsed={sidebarServicesCollapsed} onToggle={() => setSidebarServicesCollapsed(value => !value)} className="desktop-services-section" children={<div className="desktop-services-panel" data-testid="desktop-services-panel">{serviceStatus.dockerAvailable === false ? <p className="desktop-services-note" data-testid="desktop-services-docker-missing">{t("services.dockerMissing")}</p> : null}{(Array.isArray(serviceStatus.services) ? serviceStatus.services : []).map(service => {
        const running = Boolean(service.running);
        const busy = serviceBusy === service.key;
        const dockerReady = serviceStatus.dockerAvailable !== false;
        const isAgentEnvironment = service.key === "agent";
        const serviceLabel = service.labelKey ? t(service.labelKey) : service.label;
        return <div key={service.key} className="desktop-service" data-testid={`desktop-service-${service.key}`} data-state={service.state}><div className="desktop-service-head"><span className={`desktop-service-dot${running ? " is-running" : ""}`} data-testid={`desktop-service-dot-${service.key}`} /><span className="desktop-service-label">{serviceLabel}</span><span className="desktop-service-state" data-testid={`desktop-service-state-${service.key}`}>{serviceStateLabel(service.state, t)}</span></div>{service.key === "telegram" && !running ? <input type="password" className="desktop-service-token" data-testid="desktop-service-telegram-token" placeholder="TELEGRAM_BOT_TOKEN" value={telegramToken} autoComplete="off" spellCheck={false} onChange={event => setTelegramToken(event.target.value)} /> : null}{running && service.url ? <a className="desktop-service-url" href={service.url} target="_blank" rel="noopener noreferrer" data-testid={`desktop-service-url-${service.key}`}>{compactUrl(service.url)}</a> : null}<div className="desktop-service-actions"><button type="button" className="desktop-service-start" data-testid={`desktop-service-start-${service.key}`} disabled={!isAgentEnvironment && running || busy || !dockerReady} onClick={() => handleStartService(service.key)}>{isAgentEnvironment ? busy ? t("services.installing") : t("services.installAgent") : busy ? t("services.starting") : t("services.start")}</button><button type="button" className="desktop-service-stop" data-testid={`desktop-service-stop-${service.key}`} disabled={!running || busy} onClick={() => handleStopService(service.key)}>{busy ? t("services.stopping") : t("services.stop")}</button></div></div>;
      })}{serviceError ? <p className="desktop-services-error" data-testid="desktop-services-error">{serviceError}</p> : null}</div>} /> : null}<SidebarSection title={t("sidebar.conversations")} testId="sidebar-conversations" collapsed={sidebarConversationsCollapsed} onToggle={() => setSidebarConversationsCollapsed(value => !value)} children={<div className="conversation-list" data-testid="conversation-list"><button type="button" className="conversation-new" data-testid="conversation-new" disabled={messages.length === 0 && !currentConversationId && prompt.trim().length === 0} onClick={() => {
          currentConversationRef.current = "";
          setCurrentConversationId("");
          setMessages([]);
          setDemoMode(false);
          setPrompt("");
        }}>{t("conversation.new")}</button><label className="conversation-deleted-toggle"><input type="checkbox" checked={showDeletedConversations} data-testid="conversation-show-deleted" onChange={handleShowDeletedConversations} /><span>{t("conversation.showDeleted")}</span></label>{showDeletedConversations ? <button type="button" className="conversation-purge-deleted" data-testid="conversation-purge-deleted" disabled={conversations.length === 0} onClick={handlePurgeDeletedConversations} title={t("conversation.purgeDeletedTitle")}>{t("conversation.purgeDeleted")}</button> : null}{conversations.length === 0 ? <p className="conversation-empty">{showDeletedConversations ? t("conversation.deletedEmpty") : t("conversation.empty")}</p> : <ul className="conversation-entries" data-testid="conversation-entries">{conversations.map(entry => {
            const active = entry.id === currentConversationId;
            return <li key={entry.id} className={["conversation-entry", active ? "is-active" : "", entry.deleted ? "is-deleted" : ""].filter(Boolean).join(" ")}><div className="conversation-entry-row"><button type="button" className="conversation-entry-button" data-conversation-id={entry.id} aria-pressed={active} onClick={async () => {
                  if (entry.id === currentConversationRef.current) {
                    return;
                  }
                  currentConversationRef.current = entry.id;
                  setCurrentConversationId(entry.id);
                  setDemoMode(false);
                  try {
                    const events = await window.FormalAiMemory.listEvents();
                    setMessages(messagesForConversation(events, entry.id));
                  } catch (_error) {
                    setMessages([]);
                  }
                }}><span className="conversation-entry-title">{entry.title || t("conversation.emptyTitle")}</span><span className="conversation-entry-meta">{t("conversation.messageCount", {
                      count: entry.messageCount
                    })}</span></button><button type="button" className={`conversation-copy${copiedConversationId === entry.id ? " is-copied" : ""}`} data-testid="conversation-copy" data-conversation-id={entry.id} data-copied={copiedConversationId === entry.id ? "true" : null} aria-label={t("conversation.copyMarkdownTitle")} title={t("conversation.copyMarkdownTitle")} onClick={() => handleCopyConversation(entry)}>{copiedConversationId === entry.id ? t("conversation.copyMarkdownDone") : t("conversation.copyMarkdown")}</button>{entry.deleted ? <button type="button" className="conversation-delete conversation-permanent-delete" data-testid="conversation-purge-one" aria-label={t("conversation.deletePermanent")} title={t("conversation.deletePermanent")} onClick={() => handlePurgeConversation(entry)}>{"!"}</button> : <button type="button" className="conversation-delete" data-testid="conversation-delete" aria-label={t("conversation.delete")} title={t("conversation.delete")} onClick={() => handleDeleteConversation(entry)}>{"×"}</button>}</div></li>;
          })}</ul>}</div>} /><SidebarSection title={t("sidebar.settings")} testId="sidebar-settings" collapsed={sidebarSettingsCollapsed} onToggle={() => setSidebarSettingsCollapsed(value => !value)} children={<div className="settings-panel"><div className="settings-reset" data-testid="settings-reset"><div className="settings-reset-header"><span className="settings-reset-title">{t("settings.resetHeading")}</span><button type="button" className="settings-reset-all" data-testid="settings-reset-all" disabled={modifiedSettings.length === 0} onClick={resetAllSettings} title={t("settings.resetAll")}>{t("settings.resetAll")}</button></div>{modifiedSettings.length === 0 ? <p className="settings-reset-empty" data-testid="settings-reset-empty">{t("settings.resetNone")}</p> : <ul className="settings-reset-list">{modifiedSettings.map(descriptor => <li key={descriptor.key} className="settings-reset-item"><span className="settings-reset-label">{t(descriptor.label)}</span><button type="button" className="settings-reset-one" data-testid={`settings-reset-${descriptor.key}`} onClick={() => resetSetting(descriptor)} title={t("settings.resetOne")}>{t("settings.resetOne")}</button></li>)}</ul>}</div><div className="setting-row setting-row-slider"><label htmlFor="setting-guess-probability">{t("settings.ambiguity")}</label><div className="setting-poles"><span>{t("settings.moreQuestions")}</span><span>{t("settings.moreGuessing")}</span></div><input id="setting-guess-probability" data-testid="setting-guess-probability" type="range" min="0" max="1" step="0.05" value={guessProbability} onChange={event => setGuessProbability(normalizeSliderPreference(event.target.value, 0.8))} /><output htmlFor="setting-guess-probability">{`${formatSliderValue(guessProbability)}%`}</output></div><div className="setting-row setting-row-slider"><label htmlFor="setting-follow-up-probability">{t("settings.followUpInitiative")}</label><div className="setting-poles"><span>{t("settings.userInitiative")}</span><span>{t("settings.assistantInitiative")}</span></div><input id="setting-follow-up-probability" data-testid="setting-follow-up-probability" type="range" min="0" max="1" step="0.05" value={followUpProbability} onChange={event => setFollowUpProbability(normalizeSliderPreference(event.target.value, PREFERENCE_DEFAULTS.followUpProbability))} /><output htmlFor="setting-follow-up-probability">{`${formatSliderValue(followUpProbability)}%`}</output></div><div className="setting-row setting-row-slider"><label htmlFor="setting-temperature">{t("settings.temperature")}</label><div className="setting-poles"><span>{t("settings.deterministic")}</span><span>{t("settings.varied")}</span></div><input id="setting-temperature" data-testid="setting-temperature" type="range" min="0" max="1" step="0.05" value={temperature} onChange={event => setTemperature(normalizeSliderPreference(event.target.value, 0))} /><output htmlFor="setting-temperature">{normalizeSliderPreference(temperature, 0).toFixed(2)}</output></div><label className="setting-check"><input type="checkbox" checked={greetingVariations} onChange={event => setGreetingVariations(event.target.checked)} /><span>{t("settings.variations")}</span></label><label className="setting-row"><span>{t("settings.definitionFusion")}</span><select data-testid="setting-definition-fusion" value={definitionFusion} onChange={event => setDefinitionFusion(normalizeDefinitionFusion(event.target.value))}><option value="explicit">{t("settings.definitionFusion.explicit")}</option><option value="auto">{t("settings.definitionFusion.auto")}</option></select></label><label className="setting-row"><span>{t("settings.blueprintComposition")}</span><select data-testid="setting-blueprint-composition" value={blueprintComposition} onChange={event => setBlueprintComposition(normalizeBlueprintComposition(event.target.value))}><option value="composed">{t("settings.blueprintComposition.composed")}</option><option value="documented">{t("settings.blueprintComposition.documented")}</option></select></label><label className="setting-row"><span>{t("settings.thinkingDetail")}</span><select data-testid="setting-thinking-detail" value={thinkingDetailLevel} onChange={event => setThinkingDetailLevel(normalizeThinkingDetailLevel(event.target.value))}><option value="brief">{t("settings.thinkingDetail.brief")}</option><option value="standard">{t("settings.thinkingDetail.standard")}</option><option value="detailed">{t("settings.thinkingDetail.detailed")}</option></select></label><div className="setting-row setting-row-slider"><label htmlFor="setting-min-message-animation">{t("settings.minMessageAnimation")}</label><div className="setting-poles"><span>{t("settings.animationImmediate")}</span><span>{t("settings.animationRelaxed")}</span></div><input id="setting-min-message-animation" data-testid="setting-min-message-animation" type="range" min="0" max="6000" step="250" value={minMessageAnimationMs} onChange={event => setMinMessageAnimationMs(normalizeAnimationBudgetMs(event.target.value))} /><output htmlFor="setting-min-message-animation">{minMessageAnimationMs === 0 ? t("settings.animationImmediate") : t("settings.animationSeconds", {
              seconds: (minMessageAnimationMs / 1000).toFixed(1)
            })}</output></div><div className="setting-row setting-row-ocr"><label className="setting-check"><input type="checkbox" checked={experimentalOcr} data-testid="setting-experimental-ocr" onChange={event => setExperimentalOcr(event.target.checked)} /><span>{t("settings.experimentalOcr")}</span></label><p className="setting-warning" data-testid="setting-experimental-ocr-warning" title={OCR_DOWNLOAD_WARNING}>{t("settings.experimentalOcr.warning")}</p></div>{
        // Issue #444: external trusted-services opt-in/opt-out section. The
        // checkbox list is generated from EXTERNAL_TRUSTED_SERVICES so the
        // catalog stays the single source of truth; each service is enabled
        // by default and the user can opt out of any one.
        <div className="setting-row setting-row-external-services" data-testid="settings-external-services"><p className="setting-section-title">{t("settings.externalServices")}</p><p className="setting-section-note">{t("settings.externalServices.note")}</p>{EXTERNAL_TRUSTED_SERVICES.map(service => <label className="setting-check" key={service.key}><input type="checkbox" checked={externalServices[service.key] !== false} data-testid={`setting-${service.key}`} onChange={event => setExternalService(service.key, event.target.checked)} /><span>{t(service.label)}</span></label>)}</div>}<label className="setting-row"><span>{t("settings.language")}</span><select data-testid="setting-ui-language" value={uiLanguagePreference} onChange={event => setUiLanguagePreference(normalizeUiLanguagePreference(event.target.value))}><option value="auto">{t("settings.language.auto")}</option><option value="en">{"English"}</option><option value="ru">{"Русский"}</option><option value="zh">{"中文"}</option><option value="hi">{"हिन्दी"}</option></select></label><label className="setting-row"><span>{t("settings.responseLanguage")}</span><select data-testid="setting-response-language" value={responseLanguage} onChange={event => setResponseLanguage(normalizeResponseLanguageMode(event.target.value))}><option value="last_message">{t("settings.responseLanguage.lastMessage")}</option><option value="preferred">{t("settings.responseLanguage.preferred")}</option><option value="ui">{t("settings.responseLanguage.ui")}</option></select></label>{responseLanguage === "preferred" ? <label className="setting-row"><span>{t("settings.preferredLanguage")}</span><select data-testid="setting-preferred-language" value={preferredLanguage} onChange={event => setPreferredLanguage(normalizePreferredLanguage(event.target.value))}><option value="en">{"English"}</option><option value="ru">{"Русский"}</option><option value="zh">{"中文"}</option><option value="hi">{"हिन्दी"}</option></select></label> : null}<label className="setting-row"><span>{t("settings.theme")}</span><select data-testid="setting-theme" value={themePreference} onChange={event => setThemePreference(normalizeThemePreference(event.target.value))}><option value="auto">{t("settings.theme.auto")}</option><option value="light">{t("settings.theme.light")}</option><option value="dark">{t("settings.theme.dark")}</option></select></label><label className="setting-row"><span>{t("settings.uiSkin")}</span><select data-testid="setting-ui-skin" value={uiSkin} onChange={event => setUiSkin(normalizeUiSkin(event.target.value))}><option value="flat">{t("settings.uiSkin.flat")}</option><option value="glass">{t("settings.uiSkin.glass")}</option><option value="contrast">{t("settings.uiSkin.contrast")}</option></select></label><label className="setting-row"><span>{t("settings.toolbarIconPack")}</span><select data-testid="setting-toolbar-icon-pack" value={toolbarIconPack} onChange={event => setToolbarIconPack(normalizeToolbarIconPack(event.target.value))}><option value="fontawesome">{t("settings.toolbarIconPack.fontawesome")}</option><option value="material-symbols">{t("settings.toolbarIconPack.materialSymbols")}</option><option value="bootstrap-icons">{t("settings.toolbarIconPack.bootstrapIcons")}</option><option value="ionicons">{t("settings.toolbarIconPack.ionicons")}</option><option value="remix-icon">{t("settings.toolbarIconPack.remixIcon")}</option><option value="tabler-icons">{t("settings.toolbarIconPack.tablerIcons")}</option><option value="names">{t("settings.toolbarIconPack.names")}</option></select></label><label className="setting-row"><span>{t("settings.chatStyle")}</span><select data-testid="setting-chat-style" value={chatStyle} onChange={event => setChatStyle(normalizeChatStyle(event.target.value))}><option value="cards">{t("settings.chatStyle.cards")}</option><option value="compact">{t("settings.chatStyle.compact")}</option><option value="bubbles">{t("settings.chatStyle.bubbles")}</option></select></label><label className="setting-row"><span>{t("settings.composerStyle")}</span><select data-testid="setting-composer-style" value={composerStyle} onChange={event => setComposerStyle(normalizeComposerStyle(event.target.value))}><option value="flat">{t("settings.composerStyle.flat")}</option><option value="glass-soft">{t("settings.composerStyle.glassSoft")}</option><option value="glass-clear">{t("settings.composerStyle.glassClear")}</option><option value="bubble">{t("settings.composerStyle.bubble")}</option></select></label><label className="setting-row"><span>{t("settings.composerAction")}</span><select data-testid="setting-composer-action" value={composerAction} onChange={event => setComposerAction(normalizeComposerAction(event.target.value))}><option value="attach">{t("settings.composerAction.attach")}</option><option value="plus">{t("settings.composerAction.plus")}</option></select></label><label className="setting-row"><span>{t("settings.assistantName")}</span><input data-testid="setting-assistant-name" type="text" value={assistantName} maxLength={64} placeholder={t("settings.assistantName.placeholder")} onChange={event => setAssistantName(sanitizeAssistantNameInput(event.target.value))} /></label><label className="setting-row"><span>{t("settings.location")}</span><input data-testid="setting-location" type="text" value={locationPreference} placeholder={t("settings.location.placeholder")} onChange={event => setLocationPreference(event.target.value.slice(0, 80))} /></label></div>} /><SidebarSection title={t("sidebar.examplePrompts")} testId="sidebar-prompts" collapsed={sidebarPromptsCollapsed} onToggle={() => setSidebarPromptsCollapsed(value => !value)} children={<div className="prompt-list" data-testid="example-prompts">{EXAMPLE_PROMPTS.map(entry => <button key={entry.text} type="button" data-prompt-label={entry.label} data-prompt-text={entry.text} onClick={() => {
          setDemoMode(false);
          setPrompt(entry.text);
        }} title={entry.label}>{entry.text}</button>)}</div>} />{seed.tools && seed.tools.length > 0 ? <SidebarSection title={t("sidebar.tools")} testId="sidebar-tools" collapsed={sidebarToolsCollapsed} onToggle={() => setSidebarToolsCollapsed(value => !value)} children={<div className="tool-registry" data-testid="tool-registry"><ul className="tool-list">{seed.tools.map(tool => {
            const displayTool = localizeTool(tool, uiLanguage);
            return <li key={tool.id} className={`tool tool-mode-${tool.mode || "thinking"}`} data-testid="tool-entry" data-tool-id={tool.id} data-tool-mode={tool.mode || "thinking"}><div className="tool-head"><strong>{displayTool.name || tool.id}</strong><span className="tool-mode">{tool.mode === "agent" ? t("toolMode.agent") : t("toolMode.thinking")}</span></div>{displayTool.description ? <p className="tool-desc">{displayTool.description}</p> : null}</li>;
          })}</ul></div>} /> : null}{diagnosticsMode ? <SidebarSection title={t("sidebar.trace")} testId="sidebar-trace" collapsed={sidebarTraceCollapsed} onToggle={() => setSidebarTraceCollapsed(value => !value)} children={<dl className="trace-list"><div><dt>{t("trace.model")}</dt><dd>{"formal-ai"}</dd></div><div><dt>{t("trace.mode")}</dt><dd>{demoStatus}</dd></div><div><dt>{t("trace.intent")}</dt><dd>{lastAssistant?.intent ?? "none"}</dd></div><div><dt>{t("trace.data")}</dt><dd>{"data/source-index.lino"}</dd></div><div><dt>{t("trace.seedFiles")}</dt><dd>{Object.keys(seed.raw || {}).join(", ") || "(loading)"}</dd></div><div><dt>{t("trace.toolsLoaded")}</dt><dd>{String((seed.tools || []).length)}</dd></div><div><dt>{t("trace.conceptsLoaded")}</dt><dd>{String((seed.concepts || []).length)}</dd></div></dl>} /> : null}</aside><div className="context-resizer" data-testid="context-resizer" role="separator" aria-orientation="vertical" aria-label={t("titles.resizeSidebar")} aria-valuemin={CONTEXT_PANEL_MIN_WIDTH} aria-valuemax={contextPanelMaxWidth()} aria-valuenow={contextPanelWidth} tabIndex={0} title={t("titles.resizeSidebar")} onPointerDown={handleContextResizePointerDown} onKeyDown={handleContextResizeKeyDown} /><section className="chat-panel"><section className="messages" aria-live="polite" data-testid="message-list">{messages.map(message => <Message key={message.id} message={message} diagnosticsMode={diagnosticsMode} thinkingDetailLevel={thinkingDetailLevel} minMessageAnimationMs={minMessageAnimationMs} renderPermissionPanel={renderDesktopPermissionPanel} commandApprovals={commandApprovals} onApproveCommand={approveDesktopCommand} onDenyCommand={denyDesktopCommand} t={t} reportIssueUrl={shouldOfferMessageReport(message) ? createIssueUrl({
          ...reportContext,
          focusMessage: message
        }) : null} />)}{pending ? <PendingAssistantBubble t={t} /> : null}<div ref={transcriptEndRef} /></section><form className="composer" onSubmit={event => {
        event.preventDefault();
        send();
      }}><input ref={attachmentInputRef} type="file" multiple={true} style={{
          display: "none"
        }} data-testid="composer-attachment-input" onChange={handleAttachFiles} />{demoMode ? <p className="composer-demo-hint" data-testid="composer-demo-hint">{t("composer.demoHint.before")}<ToolbarIcon action="demo" pack={toolbarIconPack} className="composer-demo-hint-icon" />{t("composer.demoHint.after")}</p> : null}{composerMenuOpen ? <div className="composer-menu" data-testid="composer-menu"><button type="button" className="composer-menu-item" onClick={triggerAttachFiles}>{t("buttons.attachFiles")}</button><button type="button" className="composer-menu-item" onClick={handleExportMemory}>{t("buttons.exportMemory")}</button><button type="button" className="composer-menu-item" onClick={triggerImportMemory}>{t("buttons.importMemory")}</button><a className="composer-menu-item" href={currentReportUrl} target="_blank" rel="noopener noreferrer">{t("buttons.reportIssue")}</a></div> : null}<div className="composer-grid"><button type="button" className="composer-action-button" data-testid="composer-menu-toggle" aria-expanded={composerMenuOpen} aria-label={t("buttons.composerMenu")} title={t("titles.composerMenu")} onClick={() => setComposerMenuOpen(value => !value)}>{composerActionIcon}</button><textarea ref={composerInputRef} value={prompt} rows={1} placeholder={agentMode ? t("composer.placeholder.agent") : t("composer.placeholder.chat")} autoComplete="off" autoCorrect="off" autoCapitalize="sentences" enterKeyHint="send" inputMode="text" spellCheck={true} onChange={event => setPrompt(event.target.value)} onKeyDown={handleKeyDown} disabled={demoMode} data-testid="chat-composer-input" /><button className="send-button" type="submit" disabled={pending || demoMode || !prompt.trim() && attachments.length === 0} data-testid="chat-composer-submit">{pending ? <span className="send-spinner" aria-hidden="true" data-testid="send-spinner" /> : <span className="send-icon" aria-hidden="true">{"↑"}</span>}<span className="send-label">{pending ? t("composer.sending") : t("composer.send")}</span></button></div>{attachmentStatus ? <p className="composer-attachment-status" data-testid="composer-attachment-status">{attachmentStatus}</p> : null}</form></section></section></main>;
}

function wait(milliseconds) {
  return new Promise((resolve) => {
    window.setTimeout(resolve, milliseconds);
  });
}

createRoot(document.getElementById("root")).render(
  <ChakraProvider value={chakraSystem}>
    <App />
  </ChakraProvider>,
);
