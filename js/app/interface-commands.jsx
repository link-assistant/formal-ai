// Chat-driven interface commands ("switch to dark theme", "rename yourself
// to ...", memory export/import phrases) recognised in the user's own words.

import { clampNumber } from "./preferences.jsx";

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

export function normalizeMemoryPrompt(text) {
  return String(text || "")
    .toLowerCase()
    .replace(/[\s  -​]+/g, " ")
    .replace(/[!?.,;:。!?,;:、]+$/g, "")
    .trim();
}

export function recognizeMemoryAction(text) {
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

function containsThemeObject(normalized) {
  return matchesAnyPattern(normalized, [
    /(?:^|[^\p{L}\p{N}])theme(?:$|[^\p{L}\p{N}])/u,
    /(?:^|[^\p{L}\p{N}])dark mode(?:$|[^\p{L}\p{N}])/u,
    /(?:^|[^\p{L}\p{N}])light mode(?:$|[^\p{L}\p{N}])/u,
    /(?:^|[^\p{L}\p{N}])тема(?:$|[^\p{L}\p{N}])/u,
    /主题/u,
  ]);
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

export function sanitizeAssistantNameInput(value) {
  return String(value || "")
    .replace(/[\r\n\t]+/g, " ")
    .slice(0, 64);
}

export function normalizeAssistantName(value) {
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
    const value = normalizeAssistantName(match[1].replace(/[.!?。！？]+$/u, ""));
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

export function commandValueLabel(command) {
  if (command.kind === "report_issue") return command.label;
  if (command.kind === "trigger") return command.label;
  if (command.key === "assistantName" && !command.value) return "not set";
  if (typeof command.value === "boolean") return command.value ? "on" : "off";
  if (typeof command.value === "number") return command.value.toFixed(2);
  return String(command.value);
}

export function interfaceCommandResponse(command, reportIssueUrl) {
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

export function recognizeInterfaceCommand(text, capabilities = []) {
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

  // Match the preference object as a complete word. In particular, `тема`
  // must not match inside Russian words such as `система`; otherwise ordinary
  // prose can be intercepted before it reaches the solver (issue #776).
  if (containsThemeObject(normalized)) {
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
