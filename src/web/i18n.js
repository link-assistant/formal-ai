(function (global) {
  "use strict";

  var DEFAULT_LANGUAGE = "en";
  var SUPPORTED_LANGUAGES = ["en", "ru", "zh", "hi"];

  // Local browser catalog while link-foundation/lino-i18n is still a package
  // template. Keys follow a Links-Notation-style namespace so the catalog can
  // migrate to a parser-backed package without changing callers.
  var CATALOG = {
    en: {
      "buttons.reportIssue": "Report issue",
      "buttons.reportMissingRule": "Report missing rule",
      "buttons.exportMemory": "Export memory",
      "buttons.importMemory": "Import memory",
      "buttons.diagnostics": "Diagnostics",
      "buttons.diagnosticsOn": "Diagnostics on",
      "buttons.agent": "Agent",
      "buttons.chat": "Chat",
      "buttons.demo": "Demo",
      "buttons.demoOn": "Demo on",
      "buttons.openMenu": "Open menu",
      "buttons.closeMenu": "Close menu",
      "composer.placeholder.chat": "Message formal-ai",
      "composer.placeholder.agent":
        "Describe a multi-step task (separate steps with ; or 'then')",
      "composer.demoHint.before": "Demo is running - tap ",
      "composer.demoHint.after": " to stop and type your own message.",
      "composer.send": "Send",
      "conversation.new": "+ New conversation",
      "conversation.empty": "Start a new conversation.",
      "conversation.emptyTitle": "(empty)",
      "conversation.messageCount": "{count} msg",
      "sidebar.conversations": "Conversations",
      "sidebar.examplePrompts": "Example prompts",
      "sidebar.tools": "Tools",
      "sidebar.trace": "Trace",
      "status.demoPlaying": "Demo playing",
      "status.manual": "Manual mode",
      "status.nextDialogIn": "Next dialog in {seconds}s",
      "status.memoryUnavailable": "Memory unavailable",
      "status.exportFailed": "Export failed",
      "status.importFailed": "Import failed",
      "status.working": "Working",
      "trace.model": "Model",
      "trace.mode": "Mode",
      "trace.intent": "Intent",
      "trace.data": "Data",
      "trace.seedFiles": "Seed files",
      "trace.toolsLoaded": "Tools loaded",
      "trace.conceptsLoaded": "Concepts loaded",
    },
    ru: {
      "buttons.reportIssue": "Сообщить о проблеме",
      "buttons.reportMissingRule": "Сообщить о недостающем правиле",
      "buttons.exportMemory": "Экспорт памяти",
      "buttons.importMemory": "Импорт памяти",
      "buttons.diagnostics": "Диагностика",
      "buttons.diagnosticsOn": "Диагностика включена",
      "buttons.agent": "Агент",
      "buttons.chat": "Чат",
      "buttons.demo": "Демо",
      "buttons.demoOn": "Демо включено",
      "buttons.openMenu": "Открыть меню",
      "buttons.closeMenu": "Закрыть меню",
      "composer.placeholder.chat": "Сообщение formal-ai",
      "composer.placeholder.agent":
        "Опишите задачу из нескольких шагов (разделяйте шаги ; или «затем»)",
      "composer.demoHint.before": "Демо выполняется - нажмите ",
      "composer.demoHint.after": ", чтобы остановить и написать свое сообщение.",
      "composer.send": "Отправить",
      "conversation.new": "+ Новый разговор",
      "conversation.empty": "Начните новый разговор.",
      "conversation.emptyTitle": "(пусто)",
      "conversation.messageCount": "{count} сообщ.",
      "sidebar.conversations": "Разговоры",
      "sidebar.examplePrompts": "Примеры запросов",
      "sidebar.tools": "Инструменты",
      "sidebar.trace": "Трассировка",
      "status.demoPlaying": "Демо выполняется",
      "status.manual": "Ручной режим",
      "status.nextDialogIn": "Следующий диалог через {seconds} с",
      "status.memoryUnavailable": "Память недоступна",
      "status.exportFailed": "Экспорт не удался",
      "status.importFailed": "Импорт не удался",
      "status.working": "В работе",
      "trace.model": "Модель",
      "trace.mode": "Режим",
      "trace.intent": "Намерение",
      "trace.data": "Данные",
      "trace.seedFiles": "Файлы seed",
      "trace.toolsLoaded": "Инструментов загружено",
      "trace.conceptsLoaded": "Понятий загружено",
    },
    zh: {
      "buttons.reportIssue": "报告问题",
      "buttons.reportMissingRule": "报告缺失规则",
      "buttons.exportMemory": "导出记忆",
      "buttons.importMemory": "导入记忆",
      "buttons.diagnostics": "诊断",
      "buttons.diagnosticsOn": "诊断开启",
      "buttons.agent": "代理",
      "buttons.chat": "聊天",
      "buttons.demo": "演示",
      "buttons.demoOn": "演示开启",
      "buttons.openMenu": "打开菜单",
      "buttons.closeMenu": "关闭菜单",
      "composer.placeholder.chat": "给 formal-ai 发消息",
      "composer.placeholder.agent": "描述多步骤任务（用 ; 或“然后”分隔步骤）",
      "composer.demoHint.before": "演示正在运行 - 点按 ",
      "composer.demoHint.after": " 可停止并输入自己的消息。",
      "composer.send": "发送",
      "conversation.new": "+ 新对话",
      "conversation.empty": "开始一个新对话。",
      "conversation.emptyTitle": "（空）",
      "conversation.messageCount": "{count} 条消息",
      "sidebar.conversations": "对话",
      "sidebar.examplePrompts": "示例提示",
      "sidebar.tools": "工具",
      "sidebar.trace": "跟踪",
      "status.demoPlaying": "演示播放中",
      "status.manual": "手动模式",
      "status.nextDialogIn": "{seconds} 秒后下一个对话",
      "status.memoryUnavailable": "记忆不可用",
      "status.exportFailed": "导出失败",
      "status.importFailed": "导入失败",
      "status.working": "工作中",
      "trace.model": "模型",
      "trace.mode": "模式",
      "trace.intent": "意图",
      "trace.data": "数据",
      "trace.seedFiles": "Seed 文件",
      "trace.toolsLoaded": "已加载工具",
      "trace.conceptsLoaded": "已加载概念",
    },
    hi: {
      "buttons.reportIssue": "समस्या रिपोर्ट करें",
      "buttons.reportMissingRule": "छूटा नियम रिपोर्ट करें",
      "buttons.exportMemory": "स्मृति निर्यात करें",
      "buttons.importMemory": "स्मृति आयात करें",
      "buttons.diagnostics": "निदान",
      "buttons.diagnosticsOn": "निदान चालू",
      "buttons.agent": "एजेंट",
      "buttons.chat": "चैट",
      "buttons.demo": "डेमो",
      "buttons.demoOn": "डेमो चालू",
      "buttons.openMenu": "मेन्यू खोलें",
      "buttons.closeMenu": "मेन्यू बंद करें",
      "composer.placeholder.chat": "formal-ai को संदेश",
      "composer.placeholder.agent":
        "कई चरणों वाला कार्य लिखें (चरणों को ; या 'then' से अलग करें)",
      "composer.demoHint.before": "डेमो चल रहा है - ",
      "composer.demoHint.after": " दबाकर रोकें और अपना संदेश लिखें।",
      "composer.send": "भेजें",
      "conversation.new": "+ नई बातचीत",
      "conversation.empty": "नई बातचीत शुरू करें।",
      "conversation.emptyTitle": "(खाली)",
      "conversation.messageCount": "{count} संदेश",
      "sidebar.conversations": "बातचीत",
      "sidebar.examplePrompts": "उदाहरण प्रॉम्प्ट",
      "sidebar.tools": "टूल",
      "sidebar.trace": "ट्रेस",
      "status.demoPlaying": "डेमो चल रहा है",
      "status.manual": "मैनुअल मोड",
      "status.nextDialogIn": "{seconds}s में अगला संवाद",
      "status.memoryUnavailable": "स्मृति उपलब्ध नहीं",
      "status.exportFailed": "निर्यात विफल",
      "status.importFailed": "आयात विफल",
      "status.working": "काम जारी है",
      "trace.model": "मॉडल",
      "trace.mode": "मोड",
      "trace.intent": "इरादा",
      "trace.data": "डेटा",
      "trace.seedFiles": "Seed फाइलें",
      "trace.toolsLoaded": "लोड किए गए टूल",
      "trace.conceptsLoaded": "लोड किए गए कॉन्सेप्ट",
    },
  };

  function normalizeLanguageTag(value) {
    var raw = String(value || "").toLowerCase().trim();
    if (!raw || raw === "auto") return "";
    var base = raw.split(/[-_]/)[0];
    return SUPPORTED_LANGUAGES.indexOf(base) >= 0 ? base : "";
  }

  function resolveLanguage(preference, candidates) {
    var explicit = normalizeLanguageTag(preference);
    if (explicit) return explicit;
    var list = Array.isArray(candidates) ? candidates : [candidates];
    for (var index = 0; index < list.length; index += 1) {
      var normalized = normalizeLanguageTag(list[index]);
      if (normalized) return normalized;
    }
    return DEFAULT_LANGUAGE;
  }

  function browserLanguages() {
    var nav = global.navigator || {};
    if (Array.isArray(nav.languages) && nav.languages.length > 0) {
      return nav.languages.slice();
    }
    return nav.language ? [nav.language] : [];
  }

  function detectLanguage(preference) {
    return resolveLanguage(preference, browserLanguages());
  }

  function interpolate(template, params) {
    var values = params || {};
    return String(template).replace(/\{([a-zA-Z0-9_]+)\}/g, function (_, key) {
      return values[key] === undefined || values[key] === null
        ? ""
        : String(values[key]);
    });
  }

  function t(key, language, params) {
    var lang = normalizeLanguageTag(language) || DEFAULT_LANGUAGE;
    var table = CATALOG[lang] || CATALOG[DEFAULT_LANGUAGE];
    var fallback = CATALOG[DEFAULT_LANGUAGE] || {};
    var value =
      Object.prototype.hasOwnProperty.call(table, key)
        ? table[key]
        : fallback[key];
    return interpolate(value || key, params);
  }

  global.FormalAiI18n = {
    DEFAULT_LANGUAGE: DEFAULT_LANGUAGE,
    SUPPORTED_LANGUAGES: SUPPORTED_LANGUAGES.slice(),
    CATALOG: CATALOG,
    browserLanguages: browserLanguages,
    detectLanguage: detectLanguage,
    normalizeLanguageTag: normalizeLanguageTag,
    resolveLanguage: resolveLanguage,
    t: t,
  };
})(typeof window !== "undefined" ? window : globalThis);
