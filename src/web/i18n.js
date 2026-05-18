(function (global) {
  "use strict";

  var DEFAULT_LANGUAGE = "en";
  var SUPPORTED_LANGUAGES = ["en", "ru", "zh", "hi"];
  var PUBLISHED_RUNTIME_SOURCE = "lino-i18n@0.0.1";
  var LOCAL_RUNTIME_SOURCE = "local-fallback";
  var runtimeEngine = null;

  // Flat catalog keys match lino-i18n's runtime lookup API and keep the web
  // build usable when the CDN dependency is unavailable.
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
      "buttons.attachFiles": "Attach files",
      "buttons.composerMenu": "Composer menu",
      "titles.reportIssue":
        "Report issue - open a pre-filled GitHub issue with the current session transcript. See docs/upload-memory.md for how to attach the full memory export (Gist or .zip).",
      "titles.exportMemory":
        "Export memory - save the full agent state to formal-ai-memory.lino: the entire seed, UI preferences, environment metadata, and the append-only event log. See docs/upload-memory.md for attaching it to a GitHub issue (Gist or .zip).",
      "titles.importMemory":
        "Import memory - load a previous export. Accepts both the new full-memory bundle and the legacy demo_memory event-only log. Migration hints are shown next to this bar.",
      "titles.diagnosticsShow":
        "Show reasoning trace, intent, evidence, and thinking-steps panels.",
      "titles.diagnosticsHide":
        "Hide reasoning trace, intent, evidence, and thinking-steps panels.",
      "titles.agentOn": "Agent mode is on - switch back to single-turn chat.",
      "titles.agentOff":
        "Chat mode - switch to agent mode and each message will be decomposed into sequential steps and executed as a plan.",
      "titles.demoOn":
        "Demo is on - stop the scripted dialog and resume manual chat.",
      "titles.demoOff": "Start the scripted demo dialog.",
      "titles.menuOpen": "Open the side panel (conversations, prompts, tools).",
      "titles.menuClose":
        "Close the side panel (conversations, prompts, tools).",
      "titles.composerMenu":
        "Open input actions for attachments, memory, and reporting.",
      "composer.placeholder.chat": "Message formal-ai",
      "composer.placeholder.agent":
        "Describe a multi-step task (separate steps with ; or 'then')",
      "composer.demoHint.before": "Demo is running - tap ",
      "composer.demoHint.after": " to stop and type your own message.",
      "composer.send": "Send",
      "composer.attachments": "{count} attached",
      "conversation.new": "+ New conversation",
      "conversation.empty": "Start a new conversation.",
      "conversation.emptyTitle": "(empty)",
      "conversation.messageCount": "{count} msg",
      "message.author.user": "You",
      "message.thinking": "Thinking",
      "fetch.collapse": "Collapse",
      "fetch.expand": "Expand",
      "fetch.openInNewTab": "Open in new tab",
      "fetch.frameTitle": "Fetched page: {url}",
      "memory.exportTriggered":
        "Triggered Export memory. Your browser is downloading `formal-ai-memory.lino`.",
      "memory.importTriggered":
        "Triggered Import memory. Pick a `.lino` file in the open dialog to restore the saved memory.",
      "sidebar.conversations": "Conversations",
      "sidebar.examplePrompts": "Example prompts",
      "sidebar.tools": "Tools",
      "sidebar.trace": "Trace",
      "sidebar.settings": "Settings",
      "settings.ambiguity": "Ambiguity",
      "settings.moreQuestions": "More questions",
      "settings.moreGuessing": "More guessing",
      "settings.temperature": "Temperature",
      "settings.deterministic": "Deterministic",
      "settings.varied": "Varied",
      "settings.variations": "Greeting variations",
      "settings.language": "Language",
      "settings.language.auto": "Auto",
      "settings.theme": "Theme",
      "settings.theme.auto": "Auto",
      "settings.theme.light": "Light",
      "settings.theme.dark": "Dark",
      "settings.uiSkin": "UI skin",
      "settings.uiSkin.flat": "Flat",
      "settings.uiSkin.glass": "Glass",
      "settings.uiSkin.contrast": "Contrast",
      "settings.chatStyle": "Chat style",
      "settings.chatStyle.cards": "Cards",
      "settings.chatStyle.compact": "Compact",
      "settings.chatStyle.bubbles": "Bubbles",
      "settings.composerStyle": "Input style",
      "settings.composerStyle.flat": "Flat",
      "settings.composerStyle.glassSoft": "Glass soft",
      "settings.composerStyle.glassClear": "Glass clear",
      "settings.composerStyle.bubble": "Bubble",
      "settings.composerAction": "Input action",
      "settings.composerAction.attach": "Attach",
      "settings.composerAction.plus": "Plus",
      "settings.location": "Location",
      "settings.location.placeholder": "City or region",
      "status.demoPlaying": "Demo playing",
      "status.manual": "Manual mode",
      "status.nextDialogIn": "Next dialog in {seconds}s",
      "status.memoryUnavailable": "Memory unavailable",
      "status.memoryExported":
        "Exported full memory: {events} event(s) + {seedFiles} seed file(s)",
      "status.memoryImportedBundle":
        "Imported {inserted} event(s) from full bundle",
      "status.memoryImportedEvents": "Imported {inserted} events",
      "status.migration": "{headline}. Migration: {suggestions}",
      "status.exportFailed": "Export failed",
      "status.importFailed": "Import failed",
      "status.working": "Working",
      "toolMode.agent": "agent",
      "toolMode.thinking": "thinking",
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
      "buttons.attachFiles": "Прикрепить файлы",
      "buttons.composerMenu": "Меню ввода",
      "titles.reportIssue":
        "Сообщить о проблеме - откроет заранее заполненный GitHub issue с текущим диалогом. См. docs/upload-memory.md, чтобы прикрепить полный экспорт памяти (Gist или .zip).",
      "titles.exportMemory":
        "Экспорт памяти - сохраняет полное состояние агента в formal-ai-memory.lino: весь seed, настройки UI, метаданные окружения и append-only журнал событий. См. docs/upload-memory.md, чтобы прикрепить экспорт к GitHub issue (Gist или .zip).",
      "titles.importMemory":
        "Импорт памяти - загружает предыдущий экспорт. Поддерживает новый полный bundle памяти и устаревший журнал demo_memory только с событиями. Подсказки миграции появятся рядом с панелью.",
      "titles.diagnosticsShow":
        "Показать диагностическую трассировку, намерение, доказательства и шаги рассуждения.",
      "titles.diagnosticsHide":
        "Скрыть диагностическую трассировку, намерение, доказательства и шаги рассуждения.",
      "titles.agentOn": "Режим агента включен - вернуться к одиночному чату.",
      "titles.agentOff":
        "Режим чата - перейти в режим агента, где каждое сообщение разбивается на последовательные шаги и выполняется как план.",
      "titles.demoOn":
        "Демо включено - остановить сценарный диалог и вернуться к ручному чату.",
      "titles.demoOff": "Запустить сценарный демо-диалог.",
      "titles.menuOpen": "Открыть боковую панель (разговоры, запросы, инструменты).",
      "titles.menuClose": "Закрыть боковую панель (разговоры, запросы, инструменты).",
      "titles.composerMenu":
        "Открыть действия ввода: вложения, память и отчеты.",
      "composer.placeholder.chat": "Сообщение formal-ai",
      "composer.placeholder.agent":
        "Опишите задачу из нескольких шагов (разделяйте шаги ; или «затем»)",
      "composer.demoHint.before": "Демо выполняется - нажмите ",
      "composer.demoHint.after": ", чтобы остановить и написать свое сообщение.",
      "composer.send": "Отправить",
      "composer.attachments": "Прикреплено: {count}",
      "conversation.new": "+ Новый разговор",
      "conversation.empty": "Начните новый разговор.",
      "conversation.emptyTitle": "(пусто)",
      "conversation.messageCount": "{count} сообщ.",
      "message.author.user": "Вы",
      "message.thinking": "Мышление",
      "fetch.collapse": "Свернуть",
      "fetch.expand": "Развернуть",
      "fetch.openInNewTab": "Открыть в новой вкладке",
      "fetch.frameTitle": "Полученная страница: {url}",
      "memory.exportTriggered":
        "Запущен экспорт памяти. Браузер скачивает `formal-ai-memory.lino`.",
      "memory.importTriggered":
        "Запущен импорт памяти. Выберите файл `.lino` в открытом диалоге, чтобы восстановить сохраненную память.",
      "sidebar.conversations": "Разговоры",
      "sidebar.examplePrompts": "Примеры запросов",
      "sidebar.tools": "Инструменты",
      "sidebar.trace": "Трассировка",
      "sidebar.settings": "Настройки",
      "settings.ambiguity": "Неясность",
      "settings.moreQuestions": "Больше вопросов",
      "settings.moreGuessing": "Больше догадок",
      "settings.temperature": "Температура",
      "settings.deterministic": "Детерминированно",
      "settings.varied": "Разнообразнее",
      "settings.variations": "Варианты приветствий",
      "settings.language": "Язык",
      "settings.language.auto": "Авто",
      "settings.theme": "Тема",
      "settings.theme.auto": "Авто",
      "settings.theme.light": "Светлая",
      "settings.theme.dark": "Темная",
      "settings.uiSkin": "Скин UI",
      "settings.uiSkin.flat": "Плоский",
      "settings.uiSkin.glass": "Стекло",
      "settings.uiSkin.contrast": "Контраст",
      "settings.chatStyle": "Стиль чата",
      "settings.chatStyle.cards": "Карточки",
      "settings.chatStyle.compact": "Компактный",
      "settings.chatStyle.bubbles": "Пузыри",
      "settings.composerStyle": "Стиль ввода",
      "settings.composerStyle.flat": "Плоский",
      "settings.composerStyle.glassSoft": "Мягкое стекло",
      "settings.composerStyle.glassClear": "Прозрачное стекло",
      "settings.composerStyle.bubble": "Пузырь",
      "settings.composerAction": "Действие ввода",
      "settings.composerAction.attach": "Скрепка",
      "settings.composerAction.plus": "Плюс",
      "settings.location": "Местоположение",
      "settings.location.placeholder": "Город или регион",
      "status.demoPlaying": "Демо выполняется",
      "status.manual": "Ручной режим",
      "status.nextDialogIn": "Следующий диалог через {seconds} с",
      "status.memoryUnavailable": "Память недоступна",
      "status.memoryExported":
        "Полный экспорт памяти: {events} событ. + {seedFiles} seed-файл(ов)",
      "status.memoryImportedBundle":
        "Импортировано {inserted} событ. из полного bundle",
      "status.memoryImportedEvents": "Импортировано событий: {inserted}",
      "status.migration": "{headline}. Миграция: {suggestions}",
      "status.exportFailed": "Экспорт не удался",
      "status.importFailed": "Импорт не удался",
      "status.working": "В работе",
      "toolMode.agent": "агент",
      "toolMode.thinking": "мышление",
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
      "buttons.attachFiles": "附加文件",
      "buttons.composerMenu": "输入菜单",
      "titles.reportIssue":
        "报告问题 - 打开已预填当前会话记录的 GitHub issue。请参阅 docs/upload-memory.md，了解如何附加完整记忆导出（Gist 或 .zip）。",
      "titles.exportMemory":
        "导出记忆 - 将完整代理状态保存为 formal-ai-memory.lino：全部 seed、UI 偏好、环境元数据和追加式事件日志。请参阅 docs/upload-memory.md，了解如何附加到 GitHub issue（Gist 或 .zip）。",
      "titles.importMemory":
        "导入记忆 - 加载以前的导出。支持新的完整记忆 bundle 和旧版 demo_memory 事件日志。迁移提示会显示在此栏旁边。",
      "titles.diagnosticsShow": "显示推理跟踪、意图、证据和思考步骤面板。",
      "titles.diagnosticsHide": "隐藏推理跟踪、意图、证据和思考步骤面板。",
      "titles.agentOn": "代理模式已开启 - 切回单轮聊天。",
      "titles.agentOff":
        "聊天模式 - 切换到代理模式，每条消息会被拆成连续步骤并作为计划执行。",
      "titles.demoOn": "演示已开启 - 停止脚本对话并恢复手动聊天。",
      "titles.demoOff": "启动脚本演示对话。",
      "titles.menuOpen": "打开侧边面板（对话、提示、工具）。",
      "titles.menuClose": "关闭侧边面板（对话、提示、工具）。",
      "titles.composerMenu": "打开输入操作：附件、记忆和报告。",
      "composer.placeholder.chat": "给 formal-ai 发消息",
      "composer.placeholder.agent": "描述多步骤任务（用 ; 或“然后”分隔步骤）",
      "composer.demoHint.before": "演示正在运行 - 点按 ",
      "composer.demoHint.after": " 可停止并输入自己的消息。",
      "composer.send": "发送",
      "composer.attachments": "已附加 {count} 个",
      "conversation.new": "+ 新对话",
      "conversation.empty": "开始一个新对话。",
      "conversation.emptyTitle": "（空）",
      "conversation.messageCount": "{count} 条消息",
      "message.author.user": "你",
      "message.thinking": "思考",
      "fetch.collapse": "折叠",
      "fetch.expand": "展开",
      "fetch.openInNewTab": "在新标签页打开",
      "fetch.frameTitle": "已获取页面：{url}",
      "memory.exportTriggered":
        "已触发导出记忆。浏览器正在下载 `formal-ai-memory.lino`。",
      "memory.importTriggered":
        "已触发导入记忆。请在打开的对话框中选择 `.lino` 文件来恢复保存的记忆。",
      "sidebar.conversations": "对话",
      "sidebar.examplePrompts": "示例提示",
      "sidebar.tools": "工具",
      "sidebar.trace": "跟踪",
      "sidebar.settings": "设置",
      "settings.ambiguity": "歧义",
      "settings.moreQuestions": "多提问",
      "settings.moreGuessing": "多猜测",
      "settings.temperature": "温度",
      "settings.deterministic": "确定",
      "settings.varied": "多样",
      "settings.variations": "问候变化",
      "settings.language": "语言",
      "settings.language.auto": "自动",
      "settings.theme": "主题",
      "settings.theme.auto": "自动",
      "settings.theme.light": "浅色",
      "settings.theme.dark": "深色",
      "settings.uiSkin": "界面皮肤",
      "settings.uiSkin.flat": "扁平",
      "settings.uiSkin.glass": "玻璃",
      "settings.uiSkin.contrast": "高对比",
      "settings.chatStyle": "聊天样式",
      "settings.chatStyle.cards": "卡片",
      "settings.chatStyle.compact": "紧凑",
      "settings.chatStyle.bubbles": "气泡",
      "settings.composerStyle": "输入样式",
      "settings.composerStyle.flat": "扁平",
      "settings.composerStyle.glassSoft": "柔和玻璃",
      "settings.composerStyle.glassClear": "透明玻璃",
      "settings.composerStyle.bubble": "气泡",
      "settings.composerAction": "输入操作",
      "settings.composerAction.attach": "附件",
      "settings.composerAction.plus": "加号",
      "settings.location": "位置",
      "settings.location.placeholder": "城市或地区",
      "status.demoPlaying": "演示播放中",
      "status.manual": "手动模式",
      "status.nextDialogIn": "{seconds} 秒后下一个对话",
      "status.memoryUnavailable": "记忆不可用",
      "status.memoryExported":
        "已导出完整记忆：{events} 个事件 + {seedFiles} 个 seed 文件",
      "status.memoryImportedBundle":
        "已从完整 bundle 导入 {inserted} 个事件",
      "status.memoryImportedEvents": "已导入 {inserted} 个事件",
      "status.migration": "{headline}。迁移：{suggestions}",
      "status.exportFailed": "导出失败",
      "status.importFailed": "导入失败",
      "status.working": "工作中",
      "toolMode.agent": "代理",
      "toolMode.thinking": "思考",
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
      "buttons.attachFiles": "फाइलें जोड़ें",
      "buttons.composerMenu": "इनपुट मेन्यू",
      "titles.reportIssue":
        "समस्या रिपोर्ट करें - वर्तमान सत्र transcript के साथ पहले से भरा GitHub issue खोलें। पूरा memory export जोड़ने के लिए docs/upload-memory.md देखें (Gist या .zip)।",
      "titles.exportMemory":
        "स्मृति निर्यात करें - पूरे agent state को formal-ai-memory.lino में सेव करें: पूरा seed, UI preferences, environment metadata, और append-only event log. GitHub issue में जोड़ने के लिए docs/upload-memory.md देखें (Gist या .zip)।",
      "titles.importMemory":
        "स्मृति आयात करें - पिछला export load करें। नया full-memory bundle और legacy demo_memory event-only log दोनों स्वीकार हैं। Migration hints इसी bar के पास दिखेंगे।",
      "titles.diagnosticsShow":
        "reasoning trace, intent, evidence, और thinking-steps panels दिखाएं।",
      "titles.diagnosticsHide":
        "reasoning trace, intent, evidence, और thinking-steps panels छिपाएं।",
      "titles.agentOn": "एजेंट मोड चालू है - single-turn chat पर लौटें।",
      "titles.agentOff":
        "चैट मोड - एजेंट मोड पर जाएं, जहां हर संदेश sequential steps में टूटकर plan की तरह चलेगा।",
      "titles.demoOn":
        "डेमो चालू है - scripted dialog रोकें और manual chat फिर शुरू करें।",
      "titles.demoOff": "scripted demo dialog शुरू करें।",
      "titles.menuOpen": "side panel खोलें (बातचीत, prompts, tools)।",
      "titles.menuClose": "side panel बंद करें (बातचीत, prompts, tools)।",
      "titles.composerMenu":
        "attachments, memory, और reporting के input actions खोलें।",
      "composer.placeholder.chat": "formal-ai को संदेश",
      "composer.placeholder.agent":
        "कई चरणों वाला कार्य लिखें (चरणों को ; या 'then' से अलग करें)",
      "composer.demoHint.before": "डेमो चल रहा है - ",
      "composer.demoHint.after": " दबाकर रोकें और अपना संदेश लिखें।",
      "composer.send": "भेजें",
      "composer.attachments": "{count} जुड़ी हुई",
      "conversation.new": "+ नई बातचीत",
      "conversation.empty": "नई बातचीत शुरू करें।",
      "conversation.emptyTitle": "(खाली)",
      "conversation.messageCount": "{count} संदेश",
      "message.author.user": "आप",
      "message.thinking": "सोच",
      "fetch.collapse": "समेटें",
      "fetch.expand": "फैलाएं",
      "fetch.openInNewTab": "नए टैब में खोलें",
      "fetch.frameTitle": "लाई गई पेज: {url}",
      "memory.exportTriggered":
        "स्मृति निर्यात शुरू हुआ। आपका browser `formal-ai-memory.lino` डाउनलोड कर रहा है।",
      "memory.importTriggered":
        "स्मृति आयात शुरू हुआ। saved memory restore करने के लिए खुले dialog में `.lino` file चुनें।",
      "sidebar.conversations": "बातचीत",
      "sidebar.examplePrompts": "उदाहरण प्रॉम्प्ट",
      "sidebar.tools": "टूल",
      "sidebar.trace": "ट्रेस",
      "sidebar.settings": "सेटिंग्स",
      "settings.ambiguity": "अस्पष्टता",
      "settings.moreQuestions": "अधिक सवाल",
      "settings.moreGuessing": "अधिक अनुमान",
      "settings.temperature": "तापमान",
      "settings.deterministic": "निश्चित",
      "settings.varied": "विविध",
      "settings.variations": "ग्रीटिंग विविधता",
      "settings.language": "भाषा",
      "settings.language.auto": "ऑटो",
      "settings.theme": "थीम",
      "settings.theme.auto": "ऑटो",
      "settings.theme.light": "लाइट",
      "settings.theme.dark": "डार्क",
      "settings.uiSkin": "UI स्किन",
      "settings.uiSkin.flat": "फ्लैट",
      "settings.uiSkin.glass": "ग्लास",
      "settings.uiSkin.contrast": "कॉन्ट्रास्ट",
      "settings.chatStyle": "चैट शैली",
      "settings.chatStyle.cards": "कार्ड",
      "settings.chatStyle.compact": "कॉम्पैक्ट",
      "settings.chatStyle.bubbles": "बबल",
      "settings.composerStyle": "इनपुट शैली",
      "settings.composerStyle.flat": "फ्लैट",
      "settings.composerStyle.glassSoft": "सॉफ्ट ग्लास",
      "settings.composerStyle.glassClear": "क्लियर ग्लास",
      "settings.composerStyle.bubble": "बबल",
      "settings.composerAction": "इनपुट action",
      "settings.composerAction.attach": "अटैच",
      "settings.composerAction.plus": "प्लस",
      "settings.location": "स्थान",
      "settings.location.placeholder": "शहर या क्षेत्र",
      "status.demoPlaying": "डेमो चल रहा है",
      "status.manual": "मैनुअल मोड",
      "status.nextDialogIn": "{seconds}s में अगला संवाद",
      "status.memoryUnavailable": "स्मृति उपलब्ध नहीं",
      "status.memoryExported":
        "पूरी स्मृति निर्यात हुई: {events} event(s) + {seedFiles} seed file(s)",
      "status.memoryImportedBundle":
        "full bundle से {inserted} event(s) आयात हुए",
      "status.memoryImportedEvents": "{inserted} events आयात हुए",
      "status.migration": "{headline}. Migration: {suggestions}",
      "status.exportFailed": "निर्यात विफल",
      "status.importFailed": "आयात विफल",
      "status.working": "काम जारी है",
      "toolMode.agent": "एजेंट",
      "toolMode.thinking": "सोच",
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

  function localTranslate(key, language, params) {
    var lang = normalizeLanguageTag(language) || DEFAULT_LANGUAGE;
    var table = CATALOG[lang] || CATALOG[DEFAULT_LANGUAGE];
    var fallback = CATALOG[DEFAULT_LANGUAGE] || {};
    var value =
      Object.prototype.hasOwnProperty.call(table, key)
        ? table[key]
        : fallback[key];
    return interpolate(value || key, params);
  }

  function t(key, language, params) {
    var lang = normalizeLanguageTag(language) || DEFAULT_LANGUAGE;
    if (runtimeEngine && typeof runtimeEngine.t === "function") {
      try {
        return runtimeEngine.t(key, params || {}, {
          locale: lang,
          defaultValue: localTranslate(key, lang, params),
        });
      } catch (_error) {
        return localTranslate(key, lang, params);
      }
    }
    return localTranslate(key, lang, params);
  }

  function dispatchReady() {
    if (typeof global.dispatchEvent !== "function") return;
    try {
      if (typeof global.CustomEvent === "function") {
        global.dispatchEvent(
          new global.CustomEvent("formal-ai:i18n-ready", {
            detail: { source: api.ENGINE_SOURCE },
          }),
        );
      } else {
        global.dispatchEvent({ type: "formal-ai:i18n-ready" });
      }
    } catch (_error) {
      // Event dispatch is best-effort; the fallback translator is already live.
    }
  }

  function importModule(specifier) {
    try {
      return new Function("specifier", "return import(specifier);")(specifier);
    } catch (error) {
      return Promise.reject(error);
    }
  }

  function loadPublishedRuntime() {
    return importModule("lino-i18n")
      .then(function (module) {
        if (!module || typeof module.createI18n !== "function") {
          throw new Error("lino-i18n did not export createI18n");
        }
        runtimeEngine = module.createI18n({
          locales: CATALOG,
          defaultLocale: DEFAULT_LANGUAGE,
          fallback: [DEFAULT_LANGUAGE],
        });
        api.ENGINE_SOURCE = PUBLISHED_RUNTIME_SOURCE;
        api.lastError = null;
        dispatchReady();
        return api;
      })
      .catch(function (error) {
        runtimeEngine = null;
        api.ENGINE_SOURCE = LOCAL_RUNTIME_SOURCE;
        api.lastError = error && error.message ? error.message : String(error);
        dispatchReady();
        return api;
      });
  }

  var api = {
    DEFAULT_LANGUAGE: DEFAULT_LANGUAGE,
    SUPPORTED_LANGUAGES: SUPPORTED_LANGUAGES.slice(),
    CATALOG: CATALOG,
    ENGINE_SOURCE: LOCAL_RUNTIME_SOURCE,
    PUBLISHED_RUNTIME_SOURCE: PUBLISHED_RUNTIME_SOURCE,
    lastError: null,
    browserLanguages: browserLanguages,
    detectLanguage: detectLanguage,
    normalizeLanguageTag: normalizeLanguageTag,
    resolveLanguage: resolveLanguage,
    t: t,
    ready: Promise.resolve(null),
  };

  global.FormalAiI18n = api;
  api.ready = loadPublishedRuntime();
})(typeof window !== "undefined" ? window : globalThis);
