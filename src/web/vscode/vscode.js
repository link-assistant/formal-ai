// formal-ai VS Code extension install page (issue #554).
//
// A dedicated landing page for the VS Code extension. Because the extension is
// not published to the Marketplace yet, the *only* install methods are manual:
//
//   • a one-line installer (curl|sh on macOS/Linux, irm|iex on Windows) that
//     downloads the published .vsix and runs `code --install-extension`, and
//   • a fully manual "VS Code Extension only" flow: download the .vsix from the
//     latest GitHub release and install it from VS Code or the `code` CLI.
//
// The page also points at the one-click install from the desktop app and notes
// the vscode.dev / github.dev web-host caveat. All rendering/theming/locale
// machinery is the shared site-chrome.js; this file is just the page config.
// `window.FormalAiVsCode` is published for the e2e suite.

(function (global) {
  "use strict";

  var chrome = global.FormalAiSiteChrome;
  if (!chrome || typeof chrome.createChooser !== "function") {
    return;
  }

  var REPO = "https://github.com/link-assistant/formal-ai";
  var RAW_SH = "https://raw.githubusercontent.com/link-assistant/formal-ai/main/scripts/install.sh";
  var RAW_PS1 = "https://raw.githubusercontent.com/link-assistant/formal-ai/main/scripts/install.ps1";
  var LATEST_RELEASE = REPO + "/releases/latest";

  var CURL_CMD = "curl -fsSL " + RAW_SH + " | sh -s -- vscode";
  var PS_CMD =
    "$env:FORMAL_AI_INSTALL_TARGET='vscode'; irm " + RAW_PS1 + " | iex";
  var MANUAL_CMD = "code --install-extension formal-ai-vscode-<version>.vsix";

  chrome.createChooser({
    rootId: "vscode-app",
    topbarClass: "landing-topbar",
    brandHref: "../",
    repoUrl: REPO,
    exposeAs: "FormalAiVsCode",
    sections: [
      {
        id: "quick",
        titleKey: "quickTitle",
        introKey: "quickIntro",
        commands: [
          { command: CURL_CMD, labelKey: "curlLabel", testid: "vscode-curl" },
          { command: PS_CMD, labelKey: "psLabel", testid: "vscode-ps" },
        ],
        noteKey: "quickNote",
      },
      {
        id: "manual",
        titleKey: "manualTitle",
        introKey: "manualIntro",
        steps: ["manualStep1", "manualStep2", "manualStep3", "manualStep4"],
        commands: [
          { command: MANUAL_CMD, labelKey: "manualCmdLabel", testid: "vscode-manual" },
        ],
        links: [
          { href: LATEST_RELEASE, labelKey: "latestReleaseLabel", external: true, testid: "vscode-release" },
          { href: RAW_SH, labelKey: "rawScriptLabel", external: true, testid: "vscode-raw" },
        ],
        noteKey: "onlyModeNote",
      },
      {
        id: "desktop",
        titleKey: "desktopTitle",
        introKey: "desktopIntro",
        links: [
          { href: "../download/", labelKey: "desktopLinkLabel", testid: "vscode-desktop-link" },
        ],
      },
      {
        id: "web",
        titleKey: "webTitle",
        introKey: "webIntro",
      },
    ],
    destinations: [
      { id: "download", href: "../download/", icon: "⬇️", titleKey: "navDownloadTitle", descKey: "navDownloadDesc", actionKey: "navDownloadAction" },
      { id: "docs", href: "../docs/", icon: "📚", titleKey: "navDocsTitle", descKey: "navDocsDesc", actionKey: "navDocsAction" },
    ],
    copy: {
      en: {
        heading: "formal-ai for VS Code",
        eyebrow: "VS Code extension",
        summary:
          "The same symbolic chat UI as the web app, embedded in VS Code. It is not on the Marketplace yet, so install it manually with the one-liner below, a downloaded .vsix, or one click from the desktop app.",
        quickTitle: "Quick install (one command)",
        quickIntro:
          "Run one line in a terminal. It downloads the published .vsix and installs it with `code --install-extension`, verifying the download against the release checksums.",
        curlLabel: "macOS / Linux (terminal)",
        psLabel: "Windows (PowerShell)",
        quickNote:
          "Requires the `code` command on your PATH. In VS Code run “Shell Command: Install 'code' command in PATH” first if it is missing.",
        manualTitle: "VS Code Extension only (manual .vsix)",
        manualIntro:
          "Prefer to install just the extension by hand — no scripts, nothing else? Download the .vsix and add it to VS Code:",
        manualStep1: "Open the latest GitHub release and download formal-ai-vscode-<version>.vsix.",
        manualStep2: "In VS Code, open the Extensions view (Ctrl/Cmd+Shift+X).",
        manualStep3: "Click the ··· menu at the top of the view and choose “Install from VSIX…”.",
        manualStep4: "Pick the downloaded .vsix — or, from a terminal, run the command below.",
        manualCmdLabel: "Install from the terminal",
        latestReleaseLabel: "Open the latest release",
        rawScriptLabel: "View install.sh (raw)",
        onlyModeNote:
          "This installs only the VS Code extension — it does not install the desktop app or the CLI.",
        desktopTitle: "One click from the desktop app",
        desktopIntro:
          "Already running formal-ai Desktop? Open Settings → “Install VS Code extension” to download and install it in one click — the app finds your `code` CLI and runs the install for you.",
        desktopLinkLabel: "Get the desktop app",
        webTitle: "Using vscode.dev or github.dev?",
        webIntro:
          "The browser-based VS Code hosts cannot install a .vsix from disk or run a local server, so manual install does not apply there. Use a desktop VS Code window for the steps above; the web host still runs the in-process symbolic engine once the extension is installed on a synced profile.",
        navDownloadTitle: "All downloads",
        navDownloadDesc: "Desktop app, checksums, and every release asset in one place.",
        navDownloadAction: "Open downloads",
        navDocsTitle: "Documentation",
        navDocsDesc: "How the extension works — the dual-host design and settings.",
        navDocsAction: "Read the docs",
      },
      ru: {
        heading: "formal-ai для VS Code",
        eyebrow: "Расширение VS Code",
        summary:
          "Тот же символьный чат, что и в веб-приложении, встроенный в VS Code. Его пока нет в Marketplace, поэтому установите его вручную: однострочной командой ниже, скачанным .vsix или одним кликом из настольного приложения.",
        quickTitle: "Быстрая установка (одна команда)",
        quickIntro:
          "Выполните одну строку в терминале. Она скачивает опубликованный .vsix и устанавливает его через `code --install-extension`, проверяя загрузку по контрольным суммам релиза.",
        curlLabel: "macOS / Linux (терминал)",
        psLabel: "Windows (PowerShell)",
        quickNote:
          "Требуется команда `code` в PATH. Если её нет, сначала выполните в VS Code «Shell Command: Install 'code' command in PATH».",
        manualTitle: "Только расширение VS Code (вручную, .vsix)",
        manualIntro:
          "Хотите установить только расширение вручную — без скриптов и лишнего? Скачайте .vsix и добавьте его в VS Code:",
        manualStep1: "Откройте последний релиз на GitHub и скачайте formal-ai-vscode-<версия>.vsix.",
        manualStep2: "В VS Code откройте панель расширений (Ctrl/Cmd+Shift+X).",
        manualStep3: "Нажмите меню ··· вверху панели и выберите «Install from VSIX…».",
        manualStep4: "Выберите скачанный .vsix — или выполните команду ниже в терминале.",
        manualCmdLabel: "Установить из терминала",
        latestReleaseLabel: "Открыть последний релиз",
        rawScriptLabel: "Посмотреть install.sh (raw)",
        onlyModeNote:
          "Так устанавливается только расширение VS Code — настольное приложение и CLI не устанавливаются.",
        desktopTitle: "Один клик из настольного приложения",
        desktopIntro:
          "Уже запущен formal-ai Desktop? Откройте «Настройки» → «Установить расширение VS Code», чтобы скачать и установить его одним кликом — приложение само найдёт `code` и выполнит установку.",
        desktopLinkLabel: "Скачать настольное приложение",
        webTitle: "Используете vscode.dev или github.dev?",
        webIntro:
          "Браузерные версии VS Code не могут установить .vsix с диска или запустить локальный сервер, поэтому ручная установка там недоступна. Выполните шаги выше в настольном VS Code; веб-версия запускает встроенный символьный движок, как только расширение установлено в синхронизированном профиле.",
        navDownloadTitle: "Все загрузки",
        navDownloadDesc: "Настольное приложение, контрольные суммы и все файлы релиза в одном месте.",
        navDownloadAction: "Открыть загрузки",
        navDocsTitle: "Документация",
        navDocsDesc: "Как работает расширение — двойной хост и настройки.",
        navDocsAction: "Читать документацию",
      },
      zh: {
        heading: "VS Code 版 formal-ai",
        eyebrow: "VS Code 扩展",
        summary:
          "与网页应用相同的符号化聊天界面，内嵌到 VS Code 中。它尚未上架 Marketplace，因此请手动安装：使用下面的一行命令、下载的 .vsix，或在桌面应用中一键安装。",
        quickTitle: "快速安装（一条命令）",
        quickIntro:
          "在终端中运行一行命令。它会下载已发布的 .vsix 并通过 `code --install-extension` 安装，同时按发布校验和验证下载内容。",
        curlLabel: "macOS / Linux（终端）",
        psLabel: "Windows（PowerShell）",
        quickNote:
          "需要 PATH 中有 `code` 命令。若缺失，请先在 VS Code 中运行 “Shell Command: Install 'code' command in PATH”。",
        manualTitle: "仅安装 VS Code 扩展（手动 .vsix）",
        manualIntro:
          "想只手动安装扩展，不用脚本、不装其它东西？下载 .vsix 并添加到 VS Code：",
        manualStep1: "打开最新的 GitHub 发布页，下载 formal-ai-vscode-<version>.vsix。",
        manualStep2: "在 VS Code 中打开扩展视图（Ctrl/Cmd+Shift+X）。",
        manualStep3: "点击视图顶部的 ··· 菜单，选择 “Install from VSIX…”。",
        manualStep4: "选择下载的 .vsix —— 或在终端运行下面的命令。",
        manualCmdLabel: "从终端安装",
        latestReleaseLabel: "打开最新发布",
        rawScriptLabel: "查看 install.sh（原始文件）",
        onlyModeNote:
          "这只会安装 VS Code 扩展 —— 不会安装桌面应用或 CLI。",
        desktopTitle: "在桌面应用中一键安装",
        desktopIntro:
          "已经在运行 formal-ai 桌面版？打开“设置”→“安装 VS Code 扩展”，即可一键下载并安装 —— 应用会找到你的 `code` 命令并为你完成安装。",
        desktopLinkLabel: "获取桌面应用",
        webTitle: "在使用 vscode.dev 或 github.dev？",
        webIntro:
          "基于浏览器的 VS Code 无法从磁盘安装 .vsix，也无法运行本地服务器，因此那里不适用手动安装。请在桌面版 VS Code 中执行上述步骤；当扩展安装在已同步的配置文件中后，网页版仍会运行进程内符号引擎。",
        navDownloadTitle: "全部下载",
        navDownloadDesc: "桌面应用、校验和以及每个发布资源，集中在一处。",
        navDownloadAction: "打开下载",
        navDocsTitle: "文档",
        navDocsDesc: "扩展的工作方式 —— 双宿主设计与设置项。",
        navDocsAction: "阅读文档",
      },
      hi: {
        heading: "VS Code के लिए formal-ai",
        eyebrow: "VS Code एक्सटेंशन",
        summary:
          "वेब ऐप जैसा ही सिंबॉलिक चैट UI, VS Code में एम्बेडेड। यह अभी Marketplace पर नहीं है, इसलिए इसे मैन्युअल रूप से इंस्टॉल करें: नीचे दी एक-पंक्ति कमांड से, डाउनलोड किए .vsix से, या डेस्कटॉप ऐप से एक क्लिक में।",
        quickTitle: "त्वरित इंस्टॉल (एक कमांड)",
        quickIntro:
          "टर्मिनल में एक पंक्ति चलाएँ। यह प्रकाशित .vsix डाउनलोड करके `code --install-extension` से इंस्टॉल करती है और रिलीज़ चेकसम के विरुद्ध डाउनलोड को सत्यापित करती है।",
        curlLabel: "macOS / Linux (टर्मिनल)",
        psLabel: "Windows (PowerShell)",
        quickNote:
          "PATH में `code` कमांड चाहिए। यदि नहीं है, तो पहले VS Code में “Shell Command: Install 'code' command in PATH” चलाएँ।",
        manualTitle: "केवल VS Code एक्सटेंशन (मैन्युअल .vsix)",
        manualIntro:
          "बिना स्क्रिप्ट, बिना कुछ और, केवल एक्सटेंशन हाथ से इंस्टॉल करना चाहते हैं? .vsix डाउनलोड करें और VS Code में जोड़ें:",
        manualStep1: "नवीनतम GitHub रिलीज़ खोलें और formal-ai-vscode-<version>.vsix डाउनलोड करें।",
        manualStep2: "VS Code में Extensions व्यू खोलें (Ctrl/Cmd+Shift+X)।",
        manualStep3: "व्यू के ऊपर ··· मेनू पर क्लिक करें और “Install from VSIX…” चुनें।",
        manualStep4: "डाउनलोड किया .vsix चुनें — या टर्मिनल से नीचे दी कमांड चलाएँ।",
        manualCmdLabel: "टर्मिनल से इंस्टॉल करें",
        latestReleaseLabel: "नवीनतम रिलीज़ खोलें",
        rawScriptLabel: "install.sh देखें (raw)",
        onlyModeNote:
          "इससे केवल VS Code एक्सटेंशन इंस्टॉल होता है — डेस्कटॉप ऐप या CLI नहीं।",
        desktopTitle: "डेस्कटॉप ऐप से एक क्लिक",
        desktopIntro:
          "क्या formal-ai Desktop पहले से चल रहा है? Settings → “Install VS Code extension” खोलें और एक क्लिक में डाउनलोड व इंस्टॉल करें — ऐप आपका `code` ढूँढकर इंस्टॉल कर देता है।",
        desktopLinkLabel: "डेस्कटॉप ऐप लें",
        webTitle: "vscode.dev या github.dev का उपयोग कर रहे हैं?",
        webIntro:
          "ब्राउज़र-आधारित VS Code डिस्क से .vsix इंस्टॉल नहीं कर सकता और न लोकल सर्वर चला सकता है, इसलिए वहाँ मैन्युअल इंस्टॉल लागू नहीं होता। ऊपर दिए चरण डेस्कटॉप VS Code में करें; एक्सटेंशन के सिंक्ड प्रोफ़ाइल पर इंस्टॉल हो जाने पर वेब-होस्ट भी इन-प्रोसेस सिंबॉलिक इंजन चलाता है।",
        navDownloadTitle: "सभी डाउनलोड",
        navDownloadDesc: "डेस्कटॉप ऐप, चेकसम और हर रिलीज़ एसेट एक ही जगह।",
        navDownloadAction: "डाउनलोड खोलें",
        navDocsTitle: "दस्तावेज़",
        navDocsDesc: "एक्सटेंशन कैसे काम करता है — ड्यूल-होस्ट डिज़ाइन और सेटिंग्स।",
        navDocsAction: "दस्तावेज़ पढ़ें",
      },
    },
  });
})(typeof window !== "undefined" ? window : globalThis);
