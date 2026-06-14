// formal-ai landing page (issue #479).
//
// The site root (/) is a lightweight chooser that points visitors at the three
// things the project ships:
//
//   • the interactive web app           → /app/
//   • the documentation (incl. API ref) → /docs/
//   • the desktop download page         → /download/
//
// All of the rendering, theming and locale machinery lives in the shared
// site-chrome.js (the same `data-theme` + `formal-ai.preferences.v1` contract
// download.js and the chat app use), so this file is just the page's config.
// `window.FormalAiLanding` is published for the e2e suite.

(function (global) {
  "use strict";

  var chrome = global.FormalAiSiteChrome;
  if (!chrome || typeof chrome.createChooser !== "function") {
    return;
  }

  chrome.createChooser({
    rootId: "landing-app",
    topbarClass: "landing-topbar",
    brandHref: "./",
    repoUrl: "https://github.com/link-assistant/formal-ai",
    exposeAs: "FormalAiLanding",
    destinations: [
      { id: "app", href: "app/", icon: "🌐", titleKey: "navAppTitle", descKey: "navAppDesc", actionKey: "navAppAction" },
      { id: "docs", href: "docs/", icon: "📚", titleKey: "navDocsTitle", descKey: "navDocsDesc", actionKey: "navDocsAction" },
      { id: "download", href: "download/", icon: "⬇️", titleKey: "navDownloadTitle", descKey: "navDownloadDesc", actionKey: "navDownloadAction" },
    ],
    copy: {
      en: {
        heading: "formal-ai",
        eyebrow: "Local-first reasoning agent",
        summary:
          "A local, in-process formal-reasoning agent. Try it in your browser, read the documentation, or install the desktop app for macOS, Windows, and Linux.",
        navAppTitle: "Web app",
        navAppDesc: "Open the interactive formal-ai demo right in your browser.",
        navAppAction: "Open the app",
        navDocsTitle: "Documentation",
        navDocsDesc: "Guides, the API reference, and how the project fits together.",
        navDocsAction: "Read the docs",
        navDownloadTitle: "Desktop app",
        navDownloadDesc: "Download formal-ai Desktop for macOS, Windows, and Linux.",
        navDownloadAction: "Get the desktop app",
      },
      ru: {
        heading: "formal-ai",
        eyebrow: "Локальный агент рассуждений",
        summary:
          "Локальный агент формальных рассуждений, работающий в одном процессе. Откройте его в браузере, прочитайте документацию или установите настольное приложение для macOS, Windows и Linux.",
        navAppTitle: "Веб-приложение",
        navAppDesc: "Откройте интерактивную демонстрацию formal-ai прямо в браузере.",
        navAppAction: "Открыть приложение",
        navDocsTitle: "Документация",
        navDocsDesc: "Руководства, справочник API и устройство проекта.",
        navDocsAction: "Читать документацию",
        navDownloadTitle: "Настольное приложение",
        navDownloadDesc: "Скачайте formal-ai Desktop для macOS, Windows и Linux.",
        navDownloadAction: "Скачать приложение",
      },
      zh: {
        heading: "formal-ai",
        eyebrow: "本地优先的推理代理",
        summary:
          "一个在本地进程内运行的形式化推理代理。在浏览器中体验、阅读文档，或为 macOS、Windows 和 Linux 安装桌面应用。",
        navAppTitle: "网页应用",
        navAppDesc: "直接在浏览器中打开 formal-ai 交互式演示。",
        navAppAction: "打开应用",
        navDocsTitle: "文档",
        navDocsDesc: "指南、API 参考以及整体架构说明。",
        navDocsAction: "阅读文档",
        navDownloadTitle: "桌面应用",
        navDownloadDesc: "为 macOS、Windows 和 Linux 下载 formal-ai 桌面版。",
        navDownloadAction: "获取桌面应用",
      },
      hi: {
        heading: "formal-ai",
        eyebrow: "स्थानीय तर्क एजेंट",
        summary:
          "एक स्थानीय, इन-प्रोसेस फ़ॉर्मल-रीज़निंग एजेंट। इसे ब्राउज़र में आज़माएँ, दस्तावेज़ पढ़ें, या macOS, Windows और Linux के लिए डेस्कटॉप ऐप इंस्टॉल करें।",
        navAppTitle: "वेब ऐप",
        navAppDesc: "formal-ai का इंटरैक्टिव डेमो सीधे अपने ब्राउज़र में खोलें।",
        navAppAction: "ऐप खोलें",
        navDocsTitle: "दस्तावेज़",
        navDocsDesc: "गाइड, API संदर्भ और पूरी संरचना की जानकारी।",
        navDocsAction: "दस्तावेज़ पढ़ें",
        navDownloadTitle: "डेस्कटॉप ऐप",
        navDownloadDesc: "macOS, Windows और Linux के लिए formal-ai डेस्कटॉप डाउनलोड करें।",
        navDownloadAction: "डेस्कटॉप ऐप लें",
      },
    },
  });
})(typeof window !== "undefined" ? window : globalThis);
