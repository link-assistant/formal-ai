// formal-ai CLI install page (issue #554).
//
// A dedicated landing page for the command-line interface. The CLI is published
// to crates.io, so the primary install is `cargo install formal-ai`; the same
// universal installer used by every other interface also has a `cli` target.
// All rendering/theming/locale machinery is the shared site-chrome.js; this file
// is just the page config. `window.FormalAiCli` is published for the e2e suite.

(function (global) {
  "use strict";

  var chrome = global.FormalAiSiteChrome;
  if (!chrome || typeof chrome.createChooser !== "function") {
    return;
  }

  var REPO = "https://github.com/link-assistant/formal-ai";
  var RAW_SH = "https://raw.githubusercontent.com/link-assistant/formal-ai/main/scripts/install.sh";
  var RAW_PS1 = "https://raw.githubusercontent.com/link-assistant/formal-ai/main/scripts/install.ps1";

  var CARGO_CMD = "cargo install formal-ai";
  var CURL_CMD = "curl -fsSL " + RAW_SH + " | sh -s -- cli";
  var PS_CMD = "$env:FORMAL_AI_INSTALL_TARGET='cli'; irm " + RAW_PS1 + " | iex";
  var HELP_CMD = "formal-ai --help";
  var SERVE_CMD = "formal-ai serve";

  chrome.createChooser({
    rootId: "cli-app",
    topbarClass: "landing-topbar",
    brandHref: "../",
    repoUrl: REPO,
    exposeAs: "FormalAiCli",
    sections: [
      {
        id: "cargo",
        titleKey: "cargoTitle",
        introKey: "cargoIntro",
        commands: [{ command: CARGO_CMD, labelKey: "cargoLabel", testid: "cli-cargo" }],
        noteKey: "cargoNote",
      },
      {
        id: "installer",
        titleKey: "installerTitle",
        introKey: "installerIntro",
        commands: [
          { command: CURL_CMD, labelKey: "curlLabel", testid: "cli-curl" },
          { command: PS_CMD, labelKey: "psLabel", testid: "cli-ps" },
        ],
        links: [{ href: RAW_SH, labelKey: "rawScriptLabel", external: true, testid: "cli-raw" }],
      },
      {
        id: "usage",
        titleKey: "usageTitle",
        introKey: "usageIntro",
        commands: [
          { command: HELP_CMD, labelKey: "helpLabel", testid: "cli-help" },
          { command: SERVE_CMD, labelKey: "serveLabel", testid: "cli-serve" },
        ],
        noteKey: "usageNote",
      },
    ],
    destinations: [
      { id: "download", href: "../download/", icon: "⬇️", titleKey: "navDownloadTitle", descKey: "navDownloadDesc", actionKey: "navDownloadAction" },
      { id: "vscode", href: "../vscode/", icon: "🧩", titleKey: "navVscodeTitle", descKey: "navVscodeDesc", actionKey: "navVscodeAction" },
      { id: "docs", href: "../docs/", icon: "📚", titleKey: "navDocsTitle", descKey: "navDocsDesc", actionKey: "navDocsAction" },
    ],
    copy: {
      en: {
        heading: "formal-ai CLI",
        eyebrow: "Command-line interface",
        summary:
          "The formal-ai command-line tool: run the symbolic agent from your shell and start an OpenAI-compatible local server. Install it with Cargo or the universal one-line installer.",
        cargoTitle: "Install with Cargo (recommended)",
        cargoIntro:
          "The CLI is published on crates.io. With a Rust toolchain installed, one command builds and installs the latest formal-ai binary onto your PATH.",
        cargoLabel: "Any OS with Rust",
        cargoNote:
          "Needs the Rust toolchain. Install it from https://rustup.rs if `cargo` is not already available.",
        installerTitle: "One-line installer",
        installerIntro:
          "No Rust yet, or prefer the same installer as the other interfaces? The universal installer's `cli` target runs `cargo install formal-ai` for you (and tells you how to get Rust if it is missing).",
        curlLabel: "macOS / Linux (terminal)",
        psLabel: "Windows (PowerShell)",
        rawScriptLabel: "View install.sh (raw)",
        usageTitle: "First steps",
        usageIntro:
          "Verify the install and explore the commands, then start the local OpenAI-compatible server that the desktop app and the VS Code extension can talk to.",
        helpLabel: "Show every command",
        serveLabel: "Start the local server",
        usageNote:
          "`formal-ai serve` listens on 127.0.0.1:8080 by default and exposes POST /v1/chat/completions.",
        navDownloadTitle: "All downloads",
        navDownloadDesc: "Desktop app, checksums, and every release asset in one place.",
        navDownloadAction: "Open downloads",
        navVscodeTitle: "VS Code extension",
        navVscodeDesc: "Install the same symbolic chat UI inside VS Code.",
        navVscodeAction: "Install for VS Code",
        navDocsTitle: "Documentation",
        navDocsDesc: "Guides, the API reference, and how the project fits together.",
        navDocsAction: "Read the docs",
      },
      ru: {
        heading: "formal-ai CLI",
        eyebrow: "Интерфейс командной строки",
        summary:
          "Инструмент командной строки formal-ai: запускайте символьный агент из терминала и поднимайте локальный сервер, совместимый с OpenAI. Установите его через Cargo или универсальным однострочным установщиком.",
        cargoTitle: "Установка через Cargo (рекомендуется)",
        cargoIntro:
          "CLI опубликован на crates.io. С установленным набором инструментов Rust одна команда соберёт и установит последнюю версию formal-ai в ваш PATH.",
        cargoLabel: "Любая ОС с Rust",
        cargoNote:
          "Требуется набор инструментов Rust. Установите его с https://rustup.rs, если `cargo` ещё недоступен.",
        installerTitle: "Однострочный установщик",
        installerIntro:
          "Ещё нет Rust или предпочитаете тот же установщик, что и для других интерфейсов? Цель `cli` универсального установщика сама выполнит `cargo install formal-ai` (и подскажет, как получить Rust, если его нет).",
        curlLabel: "macOS / Linux (терминал)",
        psLabel: "Windows (PowerShell)",
        rawScriptLabel: "Посмотреть install.sh (raw)",
        usageTitle: "Первые шаги",
        usageIntro:
          "Проверьте установку и изучите команды, затем запустите локальный сервер, совместимый с OpenAI, к которому могут обращаться настольное приложение и расширение VS Code.",
        helpLabel: "Показать все команды",
        serveLabel: "Запустить локальный сервер",
        usageNote:
          "`formal-ai serve` по умолчанию слушает 127.0.0.1:8080 и предоставляет POST /v1/chat/completions.",
        navDownloadTitle: "Все загрузки",
        navDownloadDesc: "Настольное приложение, контрольные суммы и все файлы релиза в одном месте.",
        navDownloadAction: "Открыть загрузки",
        navVscodeTitle: "Расширение VS Code",
        navVscodeDesc: "Установите тот же символьный чат прямо в VS Code.",
        navVscodeAction: "Установить для VS Code",
        navDocsTitle: "Документация",
        navDocsDesc: "Руководства, справочник API и устройство проекта.",
        navDocsAction: "Читать документацию",
      },
      zh: {
        heading: "formal-ai 命令行",
        eyebrow: "命令行界面",
        summary:
          "formal-ai 命令行工具：从终端运行符号化代理，并启动兼容 OpenAI 的本地服务器。可用 Cargo 或通用一行安装器进行安装。",
        cargoTitle: "用 Cargo 安装（推荐）",
        cargoIntro:
          "该 CLI 已发布到 crates.io。装好 Rust 工具链后，一条命令即可构建并把最新的 formal-ai 可执行文件安装到 PATH。",
        cargoLabel: "任何装有 Rust 的系统",
        cargoNote:
          "需要 Rust 工具链。若尚无 `cargo`，请从 https://rustup.rs 安装。",
        installerTitle: "一行安装器",
        installerIntro:
          "还没有 Rust，或想用与其它界面相同的安装器？通用安装器的 `cli` 目标会为你运行 `cargo install formal-ai`（若缺少 Rust 也会提示如何获取）。",
        curlLabel: "macOS / Linux（终端）",
        psLabel: "Windows（PowerShell）",
        rawScriptLabel: "查看 install.sh（原始文件）",
        usageTitle: "第一步",
        usageIntro:
          "验证安装并浏览命令，然后启动桌面应用和 VS Code 扩展都能连接的本地 OpenAI 兼容服务器。",
        helpLabel: "显示全部命令",
        serveLabel: "启动本地服务器",
        usageNote:
          "`formal-ai serve` 默认监听 127.0.0.1:8080，并提供 POST /v1/chat/completions。",
        navDownloadTitle: "全部下载",
        navDownloadDesc: "桌面应用、校验和以及每个发布资源，集中在一处。",
        navDownloadAction: "打开下载",
        navVscodeTitle: "VS Code 扩展",
        navVscodeDesc: "在 VS Code 中安装相同的符号化聊天界面。",
        navVscodeAction: "为 VS Code 安装",
        navDocsTitle: "文档",
        navDocsDesc: "指南、API 参考以及整体架构说明。",
        navDocsAction: "阅读文档",
      },
      hi: {
        heading: "formal-ai CLI",
        eyebrow: "कमांड-लाइन इंटरफ़ेस",
        summary:
          "formal-ai कमांड-लाइन टूल: अपने शेल से सिंबॉलिक एजेंट चलाएँ और OpenAI-संगत लोकल सर्वर शुरू करें। इसे Cargo या यूनिवर्सल एक-पंक्ति इंस्टॉलर से इंस्टॉल करें।",
        cargoTitle: "Cargo से इंस्टॉल करें (अनुशंसित)",
        cargoIntro:
          "CLI crates.io पर प्रकाशित है। Rust टूलचेन इंस्टॉल होने पर, एक कमांड नवीनतम formal-ai बाइनरी बनाकर आपके PATH में इंस्टॉल कर देती है।",
        cargoLabel: "Rust वाला कोई भी OS",
        cargoNote:
          "Rust टूलचेन चाहिए। यदि `cargo` उपलब्ध नहीं है तो उसे https://rustup.rs से इंस्टॉल करें।",
        installerTitle: "एक-पंक्ति इंस्टॉलर",
        installerIntro:
          "अभी Rust नहीं है, या वही इंस्टॉलर चाहते हैं जो अन्य इंटरफ़ेस के लिए है? यूनिवर्सल इंस्टॉलर का `cli` टारगेट आपके लिए `cargo install formal-ai` चलाता है (और Rust न होने पर बताता है कि कैसे लाएँ)।",
        curlLabel: "macOS / Linux (टर्मिनल)",
        psLabel: "Windows (PowerShell)",
        rawScriptLabel: "install.sh देखें (raw)",
        usageTitle: "पहले कदम",
        usageIntro:
          "इंस्टॉल जाँचें और कमांड देखें, फिर वह लोकल OpenAI-संगत सर्वर शुरू करें जिससे डेस्कटॉप ऐप और VS Code एक्सटेंशन बात कर सकते हैं।",
        helpLabel: "सभी कमांड दिखाएँ",
        serveLabel: "लोकल सर्वर शुरू करें",
        usageNote:
          "`formal-ai serve` डिफ़ॉल्ट रूप से 127.0.0.1:8080 पर सुनता है और POST /v1/chat/completions देता है।",
        navDownloadTitle: "सभी डाउनलोड",
        navDownloadDesc: "डेस्कटॉप ऐप, चेकसम और हर रिलीज़ एसेट एक ही जगह।",
        navDownloadAction: "डाउनलोड खोलें",
        navVscodeTitle: "VS Code एक्सटेंशन",
        navVscodeDesc: "वही सिंबॉलिक चैट UI VS Code में इंस्टॉल करें।",
        navVscodeAction: "VS Code के लिए इंस्टॉल करें",
        navDocsTitle: "दस्तावेज़",
        navDocsDesc: "गाइड, API संदर्भ और पूरी संरचना की जानकारी।",
        navDocsAction: "दस्तावेज़ पढ़ें",
      },
    },
  });
})(typeof window !== "undefined" ? window : globalThis);
