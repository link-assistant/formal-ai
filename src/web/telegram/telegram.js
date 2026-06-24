// formal-ai Telegram bot landing page (issue #554).
//
// The Telegram bot is driven by the formal-ai CLI: install the CLI, create a
// bot token with @BotFather, then run `formal-ai telegram`. Long polling is the
// default (no public URL needed); a webhook server is opt-in via `--mode=webhook`
// (or the `formal-ai serve` command, which also exposes `/telegram/webhook`).
// All rendering/theming/locale machinery is the shared site-chrome.js; this file
// is just the page config. `window.FormalAiTelegram` is published for the e2e suite.

(function (global) {
  "use strict";

  var chrome = global.FormalAiSiteChrome;
  if (!chrome || typeof chrome.createChooser !== "function") {
    return;
  }

  var REPO = "https://github.com/link-assistant/formal-ai";
  var RAW_SH = "https://raw.githubusercontent.com/link-assistant/formal-ai/main/scripts/install.sh";
  var RAW_PS1 = "https://raw.githubusercontent.com/link-assistant/formal-ai/main/scripts/install.ps1";
  var BOTFATHER = "https://t.me/BotFather";

  var CARGO_CMD = "cargo install formal-ai";
  var CURL_CMD = "curl -fsSL " + RAW_SH + " | sh -s -- telegram";
  var PS_CMD = "$env:FORMAL_AI_INSTALL_TARGET='telegram'; irm " + RAW_PS1 + " | iex";
  var RUN_CMD = "TELEGRAM_BOT_TOKEN=<token> formal-ai telegram";
  var WEBHOOK_CMD = "FORMAL_AI_TELEGRAM_MODE=webhook formal-ai telegram";

  chrome.createChooser({
    rootId: "telegram-app",
    topbarClass: "landing-topbar",
    brandHref: "../",
    repoUrl: REPO,
    exposeAs: "FormalAiTelegram",
    sections: [
      {
        id: "install",
        titleKey: "installTitle",
        introKey: "installIntro",
        commands: [
          { command: CARGO_CMD, labelKey: "cargoLabel", testid: "telegram-cargo" },
          { command: CURL_CMD, labelKey: "curlLabel", testid: "telegram-curl" },
          { command: PS_CMD, labelKey: "psLabel", testid: "telegram-ps" },
        ],
        links: [{ href: RAW_SH, labelKey: "rawScriptLabel", external: true, testid: "telegram-raw" }],
        noteKey: "installNote",
      },
      {
        id: "run",
        titleKey: "runTitle",
        introKey: "runIntro",
        steps: ["runStep1", "runStep2", "runStep3"],
        commands: [{ command: RUN_CMD, labelKey: "runLabel", testid: "telegram-run" }],
        noteKey: "runNote",
      },
      {
        id: "webhook",
        titleKey: "webhookTitle",
        introKey: "webhookIntro",
        commands: [{ command: WEBHOOK_CMD, labelKey: "webhookLabel", testid: "telegram-webhook" }],
        noteKey: "webhookNote",
      },
    ],
    destinations: [
      { id: "cli", href: "../cli/", icon: "⌨️", titleKey: "navCliTitle", descKey: "navCliDesc", actionKey: "navCliAction" },
      { id: "download", href: "../download/", icon: "⬇️", titleKey: "navDownloadTitle", descKey: "navDownloadDesc", actionKey: "navDownloadAction" },
      { id: "docs", href: "../docs/", icon: "📚", titleKey: "navDocsTitle", descKey: "navDocsDesc", actionKey: "navDocsAction" },
    ],
    copy: {
      en: {
        heading: "formal-ai Telegram bot",
        eyebrow: "Telegram bot",
        summary:
          "Chat with the symbolic agent from Telegram. The bot is driven by the formal-ai CLI: install it, create a bot token with @BotFather, then run one command. Long polling needs no public URL.",
        installTitle: "1. Install the CLI",
        installIntro:
          "The Telegram bot ships inside the formal-ai command-line tool. Install it with Cargo, or with the universal one-line installer's `telegram` target.",
        cargoLabel: "Any OS with Rust",
        curlLabel: "macOS / Linux (terminal)",
        psLabel: "Windows (PowerShell)",
        rawScriptLabel: "View install.sh (raw)",
        installNote:
          "Cargo needs the Rust toolchain (https://rustup.rs). The installer's `telegram` target installs the CLI that powers the bot for you.",
        runTitle: "2. Create a bot and run it",
        runIntro:
          "Get a bot token from Telegram, then start the bot in long-polling mode — no webhook or public URL required.",
        runStep1: "Open @BotFather in Telegram and send /newbot to create a bot and copy its token.",
        runStep2: "Export the token as TELEGRAM_BOT_TOKEN (or pass it with --token).",
        runStep3: "Run the command below; the bot stays running and answers messages via long polling.",
        runLabel: "Start the bot (long polling)",
        runNote:
          "Long polling is the default mode. Optional env: FORMAL_AI_TELEGRAM_TIMEOUT (30s), FORMAL_AI_TELEGRAM_LIMIT (100).",
        webhookTitle: "Webhook mode (optional)",
        webhookIntro:
          "Hosting behind a public HTTPS endpoint? Run the bot as a webhook server instead of polling. It listens on 127.0.0.1:8080 by default and exposes POST /telegram/webhook.",
        webhookLabel: "Run as a webhook server",
        webhookNote:
          "Set FORMAL_AI_HOST / FORMAL_AI_PORT to change the bind address. `formal-ai serve` also exposes /telegram/webhook alongside the OpenAI-compatible API.",
        navCliTitle: "formal-ai CLI",
        navCliDesc: "The command-line tool that powers the bot and the local server.",
        navCliAction: "Install the CLI",
        navDownloadTitle: "All downloads",
        navDownloadDesc: "Desktop app, checksums, and every release asset in one place.",
        navDownloadAction: "Open downloads",
        navDocsTitle: "Documentation",
        navDocsDesc: "Guides, the API reference, and how the project fits together.",
        navDocsAction: "Read the docs",
      },
      ru: {
        heading: "Telegram-бот formal-ai",
        eyebrow: "Telegram-бот",
        summary:
          "Общайтесь с символьным агентом из Telegram. Бот работает на CLI formal-ai: установите его, создайте токен бота через @BotFather и выполните одну команду. Для long polling публичный URL не нужен.",
        installTitle: "1. Установите CLI",
        installIntro:
          "Telegram-бот входит в инструмент командной строки formal-ai. Установите его через Cargo или целью `telegram` универсального однострочного установщика.",
        cargoLabel: "Любая ОС с Rust",
        curlLabel: "macOS / Linux (терминал)",
        psLabel: "Windows (PowerShell)",
        rawScriptLabel: "Посмотреть install.sh (raw)",
        installNote:
          "Для Cargo нужен набор инструментов Rust (https://rustup.rs). Цель `telegram` установщика установит CLI, на котором работает бот.",
        runTitle: "2. Создайте бота и запустите его",
        runIntro:
          "Получите токен бота в Telegram, затем запустите бот в режиме long polling — без вебхука и публичного URL.",
        runStep1: "Откройте @BotFather в Telegram и отправьте /newbot, чтобы создать бота и скопировать токен.",
        runStep2: "Экспортируйте токен как TELEGRAM_BOT_TOKEN (или передайте через --token).",
        runStep3: "Выполните команду ниже; бот продолжит работать и отвечать на сообщения через long polling.",
        runLabel: "Запустить бота (long polling)",
        runNote:
          "Long polling — режим по умолчанию. Доп. переменные: FORMAL_AI_TELEGRAM_TIMEOUT (30 с), FORMAL_AI_TELEGRAM_LIMIT (100).",
        webhookTitle: "Режим вебхука (опционально)",
        webhookIntro:
          "Хостинг за публичным HTTPS-эндпоинтом? Запустите бот как webhook-сервер вместо опроса. По умолчанию он слушает 127.0.0.1:8080 и предоставляет POST /telegram/webhook.",
        webhookLabel: "Запустить как webhook-сервер",
        webhookNote:
          "Задайте FORMAL_AI_HOST / FORMAL_AI_PORT, чтобы изменить адрес. `formal-ai serve` также предоставляет /telegram/webhook вместе с API, совместимым с OpenAI.",
        navCliTitle: "formal-ai CLI",
        navCliDesc: "Инструмент командной строки, на котором работают бот и локальный сервер.",
        navCliAction: "Установить CLI",
        navDownloadTitle: "Все загрузки",
        navDownloadDesc: "Настольное приложение, контрольные суммы и все файлы релиза в одном месте.",
        navDownloadAction: "Открыть загрузки",
        navDocsTitle: "Документация",
        navDocsDesc: "Руководства, справочник API и устройство проекта.",
        navDocsAction: "Читать документацию",
      },
      zh: {
        heading: "formal-ai Telegram 机器人",
        eyebrow: "Telegram 机器人",
        summary:
          "在 Telegram 中与符号化代理对话。机器人由 formal-ai 命令行驱动：安装它，用 @BotFather 创建机器人令牌，然后运行一条命令。长轮询无需公网 URL。",
        installTitle: "1. 安装 CLI",
        installIntro:
          "Telegram 机器人内置于 formal-ai 命令行工具中。可用 Cargo 安装，或使用通用一行安装器的 `telegram` 目标。",
        cargoLabel: "任何装有 Rust 的系统",
        curlLabel: "macOS / Linux（终端）",
        psLabel: "Windows（PowerShell）",
        rawScriptLabel: "查看 install.sh（原始文件）",
        installNote:
          "Cargo 需要 Rust 工具链（https://rustup.rs）。安装器的 `telegram` 目标会为你安装驱动机器人的 CLI。",
        runTitle: "2. 创建机器人并运行",
        runIntro:
          "从 Telegram 获取机器人令牌，然后以长轮询模式启动机器人 —— 无需 webhook 或公网 URL。",
        runStep1: "在 Telegram 中打开 @BotFather，发送 /newbot 创建机器人并复制其令牌。",
        runStep2: "将令牌导出为 TELEGRAM_BOT_TOKEN（或用 --token 传入）。",
        runStep3: "运行下面的命令；机器人会持续运行并通过长轮询回复消息。",
        runLabel: "启动机器人（长轮询）",
        runNote:
          "长轮询是默认模式。可选环境变量：FORMAL_AI_TELEGRAM_TIMEOUT（30 秒）、FORMAL_AI_TELEGRAM_LIMIT（100）。",
        webhookTitle: "Webhook 模式（可选）",
        webhookIntro:
          "部署在公网 HTTPS 端点后面？用 webhook 服务器代替轮询运行机器人。它默认监听 127.0.0.1:8080，并提供 POST /telegram/webhook。",
        webhookLabel: "作为 webhook 服务器运行",
        webhookNote:
          "设置 FORMAL_AI_HOST / FORMAL_AI_PORT 可更改绑定地址。`formal-ai serve` 也会在 OpenAI 兼容 API 之外提供 /telegram/webhook。",
        navCliTitle: "formal-ai CLI",
        navCliDesc: "驱动机器人和本地服务器的命令行工具。",
        navCliAction: "安装 CLI",
        navDownloadTitle: "全部下载",
        navDownloadDesc: "桌面应用、校验和以及每个发布资源，集中在一处。",
        navDownloadAction: "打开下载",
        navDocsTitle: "文档",
        navDocsDesc: "指南、API 参考以及整体架构说明。",
        navDocsAction: "阅读文档",
      },
      hi: {
        heading: "formal-ai Telegram बॉट",
        eyebrow: "Telegram बॉट",
        summary:
          "Telegram से सिंबॉलिक एजेंट के साथ चैट करें। बॉट formal-ai CLI से चलता है: इसे इंस्टॉल करें, @BotFather से बॉट टोकन बनाएँ, फिर एक कमांड चलाएँ। लॉन्ग पोलिंग के लिए सार्वजनिक URL नहीं चाहिए।",
        installTitle: "1. CLI इंस्टॉल करें",
        installIntro:
          "Telegram बॉट formal-ai कमांड-लाइन टूल में शामिल है। इसे Cargo से, या यूनिवर्सल एक-पंक्ति इंस्टॉलर के `telegram` टारगेट से इंस्टॉल करें।",
        cargoLabel: "Rust वाला कोई भी OS",
        curlLabel: "macOS / Linux (टर्मिनल)",
        psLabel: "Windows (PowerShell)",
        rawScriptLabel: "install.sh देखें (raw)",
        installNote:
          "Cargo को Rust टूलचेन चाहिए (https://rustup.rs)। इंस्टॉलर का `telegram` टारगेट आपके लिए बॉट चलाने वाला CLI इंस्टॉल करता है।",
        runTitle: "2. बॉट बनाएँ और चलाएँ",
        runIntro:
          "Telegram से बॉट टोकन लें, फिर बॉट को लॉन्ग-पोलिंग मोड में शुरू करें — किसी webhook या सार्वजनिक URL की ज़रूरत नहीं।",
        runStep1: "Telegram में @BotFather खोलें और /newbot भेजकर बॉट बनाएँ व उसका टोकन कॉपी करें।",
        runStep2: "टोकन को TELEGRAM_BOT_TOKEN के रूप में एक्सपोर्ट करें (या --token से पास करें)।",
        runStep3: "नीचे दी कमांड चलाएँ; बॉट चलता रहेगा और लॉन्ग पोलिंग से संदेशों का उत्तर देगा।",
        runLabel: "बॉट शुरू करें (लॉन्ग पोलिंग)",
        runNote:
          "लॉन्ग पोलिंग डिफ़ॉल्ट मोड है। वैकल्पिक env: FORMAL_AI_TELEGRAM_TIMEOUT (30 सेकंड), FORMAL_AI_TELEGRAM_LIMIT (100)।",
        webhookTitle: "Webhook मोड (वैकल्पिक)",
        webhookIntro:
          "सार्वजनिक HTTPS एंडपॉइंट के पीछे होस्ट कर रहे हैं? पोलिंग के बजाय बॉट को webhook सर्वर के रूप में चलाएँ। यह डिफ़ॉल्ट रूप से 127.0.0.1:8080 पर सुनता है और POST /telegram/webhook देता है।",
        webhookLabel: "webhook सर्वर के रूप में चलाएँ",
        webhookNote:
          "बाइंड पता बदलने के लिए FORMAL_AI_HOST / FORMAL_AI_PORT सेट करें। `formal-ai serve` OpenAI-संगत API के साथ /telegram/webhook भी देता है।",
        navCliTitle: "formal-ai CLI",
        navCliDesc: "बॉट और लोकल सर्वर को चलाने वाला कमांड-लाइन टूल।",
        navCliAction: "CLI इंस्टॉल करें",
        navDownloadTitle: "सभी डाउनलोड",
        navDownloadDesc: "डेस्कटॉप ऐप, चेकसम और हर रिलीज़ एसेट एक ही जगह।",
        navDownloadAction: "डाउनलोड खोलें",
        navDocsTitle: "दस्तावेज़",
        navDocsDesc: "गाइड, API संदर्भ और पूरी संरचना की जानकारी।",
        navDocsAction: "दस्तावेज़ पढ़ें",
      },
    },
  });
})(typeof window !== "undefined" ? window : globalThis);
