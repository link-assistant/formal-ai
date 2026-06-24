// formal-ai Desktop — download page (issue #347).
//
// A self-contained, dependency-free port of the konard/vk-bot-desktop download
// page, retargeted to formal-ai's release assets and integrated with the chat
// app's theme + locale system. It reuses the same `data-theme` contract and the
// same Links-Notation-backed `formal-ai.preferences.v1` storage (via
// ../preferences.js), so a visitor's theme/locale choice is shared with the main
// app. The only REST call it makes is to the GitHub Releases API (issue #347 R7:
// OpenAI-/GitHub-shaped REST only; internal state stays in Links Notation).
//
// Pure helpers are exported on `window.FormalAiDownload` for the e2e suite, and
// a release payload can be injected via `window.__FORMAL_AI_DOWNLOAD_RELEASE__`
// (and the OS forced via `window.__FORMAL_AI_DOWNLOAD_OS__` or `?os=`) so the
// page renders deterministically in tests and CI screenshots without network.

(function (global) {
  "use strict";

  // ---------------------------------------------------------------------------
  // Release + asset configuration
  // ---------------------------------------------------------------------------

  var REPO = "link-assistant/formal-ai";
  var RELEASE_API =
    "https://api.github.com/repos/link-assistant/formal-ai/releases/latest";
  var RELEASES_URL =
    "https://github.com/link-assistant/formal-ai/releases/latest";
  var CHECKSUM_ASSET_NAME = "SHA256SUMS.txt";
  var PROVENANCE_ASSET_NAME = "BUILD-PROVENANCE.txt";

  // The asset prefixes here MUST match the electron-builder `artifactName`
  // templates in desktop/package.json (formal-ai-desktop-<os>-<arch>).
  var downloadOptions = [
    { id: "macos-arm64", os: "macos", labelKey: "macArm", assetPrefix: "formal-ai-desktop-macos-arm64", extension: "dmg" },
    { id: "macos-arm64-zip", os: "macos", labelKey: "macArmZip", assetPrefix: "formal-ai-desktop-macos-arm64", extension: "zip" },
    { id: "macos-x64", os: "macos", labelKey: "macIntel", assetPrefix: "formal-ai-desktop-macos-x64", extension: "dmg" },
    { id: "macos-x64-zip", os: "macos", labelKey: "macIntelZip", assetPrefix: "formal-ai-desktop-macos-x64", extension: "zip" },
    { id: "windows-x64", os: "windows", labelKey: "winInstaller", assetPrefix: "formal-ai-desktop-windows-installer-x64", extension: "exe" },
    { id: "windows-arm64", os: "windows", labelKey: "winInstallerArm", assetPrefix: "formal-ai-desktop-windows-installer-arm64", extension: "exe" },
    { id: "windows-portable-x64", os: "windows", labelKey: "winPortable", assetPrefix: "formal-ai-desktop-windows-portable-x64", extension: "exe" },
    { id: "windows-portable-arm64", os: "windows", labelKey: "winPortableArm", assetPrefix: "formal-ai-desktop-windows-portable-arm64", extension: "exe" },
    { id: "linux-appimage-x64", os: "linux", labelKey: "linuxAppImage", assetPrefix: "formal-ai-desktop-linux-x64", extension: "AppImage" },
    { id: "linux-appimage-arm64", os: "linux", labelKey: "linuxAppImageArm", assetPrefix: "formal-ai-desktop-linux-arm64", extension: "AppImage" },
    { id: "linux-deb-x64", os: "linux", labelKey: "linuxDeb", assetPrefix: "formal-ai-desktop-linux-x64", extension: "deb" },
    { id: "linux-deb-arm64", os: "linux", labelKey: "linuxDebArm", assetPrefix: "formal-ai-desktop-linux-arm64", extension: "deb" },
    { id: "linux-tar-x64", os: "linux", labelKey: "linuxTar", assetPrefix: "formal-ai-desktop-linux-x64", extension: "tar.gz" },
    { id: "linux-tar-arm64", os: "linux", labelKey: "linuxTarArm", assetPrefix: "formal-ai-desktop-linux-arm64", extension: "tar.gz" },
  ];

  function optionById(id) {
    return downloadOptions.find(function (option) {
      return option.id === id;
    });
  }

  function primaryOptionFor(os) {
    if (os === "macos") return optionById("macos-arm64");
    if (os === "windows") return optionById("windows-x64");
    if (os === "linux") return optionById("linux-appimage-x64");
    return undefined;
  }

  function assetsByName(release) {
    var map = {};
    var assets = (release && release.assets) || [];
    for (var i = 0; i < assets.length; i += 1) {
      map[assets[i].name] = assets[i];
    }
    return map;
  }

  function releaseVersion(release) {
    var tag = String(
      (release && (release.tag_name || release.tagName || release.name)) || "",
    );
    var match = tag.match(/(?:^|-)v?(\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?)/);
    return match ? match[1] : undefined;
  }

  function assetNameFor(option, release) {
    if (!option) return undefined;
    var version = releaseVersion(release) || "version";
    return option.assetPrefix + "-" + version + "." + option.extension;
  }

  function legacyAssetNameFor(option) {
    return option.assetPrefix + "." + option.extension;
  }

  function candidateAssetNames(option, release) {
    if (!option) return [];
    return [assetNameFor(option, release), legacyAssetNameFor(option)].filter(
      function (name, index, names) {
        return name && names.indexOf(name) === index;
      },
    );
  }

  function resolveDownloadAsset(option, releaseAssets, release) {
    var names = candidateAssetNames(option, release);
    for (var i = 0; i < names.length; i += 1) {
      if (releaseAssets[names[i]]) return releaseAssets[names[i]];
    }
    return undefined;
  }

  function resolveDownloadHref(option, releaseAssets, release) {
    var asset = resolveDownloadAsset(option, releaseAssets, release);
    return asset ? asset.browser_download_url : undefined;
  }

  function resolveChecksumHref(releaseAssets) {
    var asset = releaseAssets[CHECKSUM_ASSET_NAME];
    return (asset && asset.browser_download_url) || RELEASES_URL;
  }

  function resolveProvenanceHref(releaseAssets) {
    var asset = releaseAssets[PROVENANCE_ASSET_NAME];
    return (asset && asset.browser_download_url) || RELEASES_URL;
  }

  function downloadFamilies() {
    return [
      {
        os: "macos",
        families: [
          { id: "macos-arm64", primary: optionById("macos-arm64"), secondary: [optionById("macos-arm64-zip")] },
          { id: "macos-x64", primary: optionById("macos-x64"), secondary: [optionById("macos-x64-zip")] },
        ],
      },
      {
        os: "windows",
        families: [
          { id: "windows-installer", primary: optionById("windows-x64"), secondary: [optionById("windows-arm64")] },
          { id: "windows-portable", primary: optionById("windows-portable-x64"), secondary: [optionById("windows-portable-arm64")] },
        ],
      },
      {
        os: "linux",
        families: [
          { id: "linux-x64", primary: optionById("linux-appimage-x64"), secondary: [optionById("linux-deb-x64"), optionById("linux-tar-x64")] },
          { id: "linux-arm64", primary: optionById("linux-appimage-arm64"), secondary: [optionById("linux-deb-arm64"), optionById("linux-tar-arm64")] },
        ],
      },
    ];
  }

  // ---------------------------------------------------------------------------
  // Localized copy (en / ru / zh / hi — formal-ai's four UI languages)
  // ---------------------------------------------------------------------------

  var SUPPORTED_LOCALES = ["en", "ru", "zh", "hi"];

  var copy = {
    en: {
      eyebrow: "Local reasoning, on your machine",
      title: "formal-ai Desktop",
      summary:
        "Run formal-ai locally with an in-process reasoning agent and an optional, off-by-default OpenAI-compatible server. Available for macOS, Windows, and Linux.",
      release: "Latest release",
      checksum: "Checksums",
      provenance: "Build provenance",
      primaryUnknown: "Choose your operating system",
      primaryAction: "Download",
      otherSystems: "Other downloads",
      macos: "macOS",
      windows: "Windows",
      linux: "Linux",
      allReleases: "All releases",
      sourceCode: "Source code",
      backToApp: "Open the web app",
      language: "Language",
      theme: "Theme",
      themeAuto: "Auto",
      themeLight: "Light",
      themeDark: "Dark",
      statusReady: "Release assets ready",
      statusLoading: "Checking latest release",
      statusFallback: "Open latest release to download",
      downloadChecking: "Checking release assets",
      downloadUnavailable: "Not available in latest release",
      macArm: "macOS Apple silicon", macArmZip: "macOS Apple silicon zip",
      macIntel: "macOS Intel", macIntelZip: "macOS Intel zip",
      winInstaller: "Windows installer", winInstallerArm: "Windows ARM installer",
      winPortable: "Windows portable", winPortableArm: "Windows ARM portable",
      linuxAppImage: "Linux AppImage", linuxAppImageArm: "Linux ARM AppImage",
      linuxDeb: "Linux .deb", linuxDebArm: "Linux ARM .deb",
      linuxTar: "Linux tar.gz", linuxTarArm: "Linux ARM tar.gz",
      previewAlt: "formal-ai application interface preview",
      verify: "Verify downloads with SHA256SUMS.txt from the same release.",
      verifyTitle: "Verify your download",
      verifyRegular: "UI check",
      verifyAdvanced: "Command-line check",
      verifyUiIntro:
        "Select the downloaded file and SHA256SUMS.txt. The check runs locally in this browser.",
      verifyFile: "Downloaded file",
      verifyChecksumFile: "SHA256SUMS.txt",
      verifyResultIdle: "Choose both files to compare the SHA-256 checksum.",
      verifyResultWorking: "Calculating checksum...",
      verifyResultMissing: "No matching line was found for this filename.",
      verifyResultMatch: "Checksum matches.",
      verifyResultMismatch: "Checksum does not match.",
      regularStepOne: "Download the app and SHA256SUMS.txt from the same release.",
      regularStepTwo: "Use the form below to select both files. They stay on your device.",
      regularStepThree: "Install only when the page reports that the checksum matches.",
      windowsCommand: "Windows PowerShell",
      macosCommand: "macOS Terminal",
      linuxCommand: "Linux Terminal",
      advancedStepOne:
        "Check BUILD-PROVENANCE.txt for the repository, workflow run, tag, commit, and builder OS.",
      advancedStepTwo:
        "When release attestations are available, verify the artifact with GitHub CLI.",
      reproducibleNote:
        "Byte-for-byte reproducible desktop builds need a pinned rebuild environment; this release records provenance now and leaves that stronger guarantee explicit.",
      installMacosTitle: "Open the app on macOS",
      installMacosWhy:
        "Builds are ad-hoc signed without an Apple Developer ID, so macOS Gatekeeper blocks the first launch. After verifying the SHA-256 checksum above, use either workflow below to allow the app once.",
      installMacosTerminalTitle: "Terminal one-liner",
      installMacosTerminalStep:
        "After dragging formal-ai Desktop.app into /Applications, remove the quarantine attribute from a Terminal:",
      installMacosSettingsTitle: "System Settings (macOS 15 Sequoia)",
      installMacosSettingsStep1:
        'Double-click formal-ai Desktop, then click Done when "Apple could not verify..." appears.',
      installMacosSettingsStep2:
        "Open System Settings → Privacy & Security and scroll to the Security section.",
      installMacosSettingsStep3:
        'Click "Open Anyway" next to formal-ai Desktop, confirm, and authenticate.',
      installMacosFooter:
        "Subsequent launches do not show the warning. Only run these steps for formal-ai release artifacts whose SHA-256 matches SHA256SUMS.txt from the same GitHub release.",
      installMacosShotsCaption:
        "Real macOS 15 (Sequoia) Gatekeeper prompts captured from our sibling app VK Bot Desktop, which ships with the same ad-hoc signing — formal-ai Desktop shows identical dialogs with its own name.",
      installMacosShot1Alt:
        'macOS warning dialog: "VK Bot Desktop" Not Opened, with Done and Move to Trash buttons.',
      installMacosShot2Alt:
        "macOS System Settings, Privacy & Security, with the Open Anyway button for VK Bot Desktop.",
      installMacosShot3Alt:
        'macOS confirmation dialog: Open "VK Bot Desktop"? with Open Anyway, Move to Trash and Done buttons.',
      agentTitle: "In-process by default",
      agentBody:
        "formal-ai Desktop runs an in-process reasoning agent — no server required. You can optionally enable a local OpenAI-compatible server and point the claude, codex, or agent CLIs at it.",
      agentDocs: "Server API & CLI setup",
    },
    ru: {
      eyebrow: "Локальные рассуждения на вашем устройстве",
      title: "formal-ai Desktop",
      summary:
        "Запускайте formal-ai локально: встроенный агент рассуждений и опциональный, по умолчанию выключенный, OpenAI-совместимый сервер. Доступно для macOS, Windows и Linux.",
      release: "Последний релиз",
      checksum: "Контрольные суммы",
      provenance: "Происхождение сборки",
      primaryUnknown: "Выберите операционную систему",
      primaryAction: "Скачать",
      otherSystems: "Другие загрузки",
      macos: "macOS", windows: "Windows", linux: "Linux",
      allReleases: "Все релизы",
      sourceCode: "Исходный код",
      backToApp: "Открыть веб-приложение",
      language: "Язык",
      theme: "Тема",
      themeAuto: "Авто", themeLight: "Светлая", themeDark: "Тёмная",
      statusReady: "Файлы релиза готовы",
      statusLoading: "Проверяем последний релиз",
      statusFallback: "Откройте последний релиз для загрузки",
      downloadChecking: "Проверяем файлы релиза",
      downloadUnavailable: "Нет в последнем релизе",
      macArm: "macOS Apple silicon", macArmZip: "macOS Apple silicon zip",
      macIntel: "macOS Intel", macIntelZip: "macOS Intel zip",
      winInstaller: "Windows installer", winInstallerArm: "Windows ARM installer",
      winPortable: "Windows portable", winPortableArm: "Windows ARM portable",
      linuxAppImage: "Linux AppImage", linuxAppImageArm: "Linux ARM AppImage",
      linuxDeb: "Linux .deb", linuxDebArm: "Linux ARM .deb",
      linuxTar: "Linux tar.gz", linuxTarArm: "Linux ARM tar.gz",
      previewAlt: "Интерфейс приложения formal-ai",
      verify: "Проверяйте загрузки через SHA256SUMS.txt из того же релиза.",
      verifyTitle: "Проверка загрузки",
      verifyRegular: "Проверка в интерфейсе",
      verifyAdvanced: "Проверка в командной строке",
      verifyUiIntro:
        "Выберите скачанный файл и SHA256SUMS.txt. Проверка выполняется локально в браузере.",
      verifyFile: "Скачанный файл",
      verifyChecksumFile: "SHA256SUMS.txt",
      verifyResultIdle: "Выберите оба файла, чтобы сравнить SHA-256.",
      verifyResultWorking: "Считаем контрольную сумму...",
      verifyResultMissing: "Для этого имени файла нет строки в SHA256SUMS.txt.",
      verifyResultMatch: "Контрольная сумма совпадает.",
      verifyResultMismatch: "Контрольная сумма не совпадает.",
      regularStepOne: "Скачайте приложение и SHA256SUMS.txt из одного и того же релиза.",
      regularStepTwo: "Выберите оба файла в форме ниже. Они остаются на вашем устройстве.",
      regularStepThree: "Устанавливайте файл только если страница сообщает о совпадении.",
      windowsCommand: "Windows PowerShell",
      macosCommand: "macOS Terminal",
      linuxCommand: "Linux Terminal",
      advancedStepOne:
        "Проверьте BUILD-PROVENANCE.txt: репозиторий, workflow run, тег, коммит и OS сборщика.",
      advancedStepTwo:
        "Когда attestation доступен в релизе, проверьте файл через GitHub CLI.",
      reproducibleNote:
        "Побайтово воспроизводимые desktop-сборки требуют зафиксированной среды пересборки; текущий релиз уже записывает provenance и явно отделяет это от более строгой гарантии.",
      installMacosTitle: "Открытие приложения на macOS",
      installMacosWhy:
        "Сборки подписаны ad-hoc, без Apple Developer ID, поэтому Gatekeeper блокирует первый запуск. Сначала сверьте SHA-256 выше, затем выполните один из вариантов ниже, чтобы открыть приложение.",
      installMacosTerminalTitle: "Команда в Терминале",
      installMacosTerminalStep:
        "Перетащите formal-ai Desktop.app в /Applications и снимите карантин в Терминале:",
      installMacosSettingsTitle: "Системные настройки (macOS 15 Sequoia)",
      installMacosSettingsStep1:
        "Откройте formal-ai Desktop двойным щелчком и нажмите «Готово», когда появится предупреждение.",
      installMacosSettingsStep2:
        "Откройте Системные настройки → Конфиденциальность и безопасность и пролистайте до раздела «Безопасность».",
      installMacosSettingsStep3:
        "Нажмите «Открыть всё равно» рядом с formal-ai Desktop, подтвердите и пройдите аутентификацию.",
      installMacosFooter:
        "При последующих запусках предупреждение не появляется. Используйте эти шаги только для релизных файлов formal-ai, чья контрольная сумма SHA-256 совпала с SHA256SUMS.txt из того же релиза GitHub.",
      installMacosShotsCaption:
        "Реальные диалоги Gatekeeper в macOS 15 (Sequoia), снятые в нашем родственном приложении VK Bot Desktop, которое подписано так же (ad-hoc). formal-ai Desktop показывает те же диалоги со своим именем.",
      installMacosShot1Alt:
        "Предупреждение macOS: «VK Bot Desktop» не открыт, с кнопками «Готово» и «Переместить в Корзину».",
      installMacosShot2Alt:
        "Системные настройки macOS, «Конфиденциальность и безопасность», с кнопкой «Всё равно открыть» для VK Bot Desktop.",
      installMacosShot3Alt:
        "Диалог подтверждения macOS: открыть «VK Bot Desktop»? с кнопками «Всё равно открыть», «Переместить в Корзину» и «Готово».",
      agentTitle: "По умолчанию встроенный агент",
      agentBody:
        "formal-ai Desktop запускает встроенный агент рассуждений — сервер не нужен. При желании можно включить локальный OpenAI-совместимый сервер и направить на него CLI claude, codex или agent.",
      agentDocs: "Серверный API и настройка CLI",
    },
    zh: {
      eyebrow: "本地推理，运行在你的设备上",
      title: "formal-ai 桌面版",
      summary:
        "在本地运行 formal-ai：内置推理代理，并提供可选的、默认关闭的 OpenAI 兼容服务器。支持 macOS、Windows 和 Linux。",
      release: "最新版本",
      checksum: "校验和",
      provenance: "构建溯源",
      primaryUnknown: "选择你的操作系统",
      primaryAction: "下载",
      otherSystems: "其他下载",
      macos: "macOS", windows: "Windows", linux: "Linux",
      allReleases: "所有版本",
      sourceCode: "源代码",
      backToApp: "打开网页应用",
      language: "语言",
      theme: "主题",
      themeAuto: "自动", themeLight: "浅色", themeDark: "深色",
      statusReady: "发布文件已就绪",
      statusLoading: "正在检查最新版本",
      statusFallback: "打开最新版本以下载",
      downloadChecking: "正在检查发布文件",
      downloadUnavailable: "最新版本中不可用",
      macArm: "macOS Apple 芯片", macArmZip: "macOS Apple 芯片 zip",
      macIntel: "macOS Intel", macIntelZip: "macOS Intel zip",
      winInstaller: "Windows 安装程序", winInstallerArm: "Windows ARM 安装程序",
      winPortable: "Windows 便携版", winPortableArm: "Windows ARM 便携版",
      linuxAppImage: "Linux AppImage", linuxAppImageArm: "Linux ARM AppImage",
      linuxDeb: "Linux .deb", linuxDebArm: "Linux ARM .deb",
      linuxTar: "Linux tar.gz", linuxTarArm: "Linux ARM tar.gz",
      previewAlt: "formal-ai 应用界面预览",
      verify: "请使用同一版本的 SHA256SUMS.txt 校验下载文件。",
      verifyTitle: "校验你的下载",
      verifyRegular: "界面校验",
      verifyAdvanced: "命令行校验",
      verifyUiIntro:
        "选择下载的文件和 SHA256SUMS.txt。校验在本浏览器本地完成。",
      verifyFile: "下载的文件",
      verifyChecksumFile: "SHA256SUMS.txt",
      verifyResultIdle: "请选择两个文件以比较 SHA-256 校验和。",
      verifyResultWorking: "正在计算校验和……",
      verifyResultMissing: "在 SHA256SUMS.txt 中未找到该文件名对应的行。",
      verifyResultMatch: "校验和匹配。",
      verifyResultMismatch: "校验和不匹配。",
      regularStepOne: "从同一版本下载应用和 SHA256SUMS.txt。",
      regularStepTwo: "在下面的表单中选择两个文件。它们保留在你的设备上。",
      regularStepThree: "仅在页面提示校验和匹配时才安装。",
      windowsCommand: "Windows PowerShell",
      macosCommand: "macOS 终端",
      linuxCommand: "Linux 终端",
      advancedStepOne:
        "检查 BUILD-PROVENANCE.txt 中的仓库、workflow run、标签、提交和构建系统。",
      advancedStepTwo: "当版本提供 attestation 时，使用 GitHub CLI 校验文件。",
      reproducibleNote:
        "逐字节可复现的桌面构建需要固定的重建环境；当前版本已记录溯源信息，并将更强的保证明确区分开来。",
      installMacosTitle: "在 macOS 上打开应用",
      installMacosWhy:
        "构建使用 ad-hoc 签名，没有 Apple Developer ID，因此 macOS Gatekeeper 会阻止首次启动。请先校验上面的 SHA-256，然后使用下面任一方式放行一次。",
      installMacosTerminalTitle: "终端单行命令",
      installMacosTerminalStep:
        "将 formal-ai Desktop.app 拖入 /Applications 后，在终端中移除隔离属性：",
      installMacosSettingsTitle: "系统设置（macOS 15 Sequoia）",
      installMacosSettingsStep1:
        "双击 formal-ai Desktop，出现“无法验证”提示时点击“完成”。",
      installMacosSettingsStep2: "打开 系统设置 → 隐私与安全性，滚动到“安全性”部分。",
      installMacosSettingsStep3:
        "在 formal-ai Desktop 旁点击“仍要打开”，确认并完成身份验证。",
      installMacosFooter:
        "后续启动不再显示警告。仅对 SHA-256 与同一 GitHub 版本的 SHA256SUMS.txt 匹配的 formal-ai 发布文件执行这些步骤。",
      installMacosShotsCaption:
        "在我们的姊妹应用 VK Bot Desktop 中拍摄的真实 macOS 15 (Sequoia) Gatekeeper 对话框；它采用相同的 ad-hoc 签名。formal-ai Desktop 显示相同的对话框，只是名称不同。",
      installMacosShot1Alt:
        "macOS 警告对话框：未打开“VK Bot Desktop”，带“完成”和“移到废纸篓”按钮。",
      installMacosShot2Alt:
        "macOS 系统设置 → 隐私与安全性，显示 VK Bot Desktop 的“仍要打开”按钮。",
      installMacosShot3Alt:
        "macOS 确认对话框：打开“VK Bot Desktop”？带“仍要打开”、“移到废纸篓”和“完成”按钮。",
      agentTitle: "默认内置代理",
      agentBody:
        "formal-ai 桌面版运行内置推理代理——无需服务器。你可以选择启用本地 OpenAI 兼容服务器，并让 claude、codex 或 agent CLI 指向它。",
      agentDocs: "服务器 API 与 CLI 配置",
    },
    hi: {
      eyebrow: "आपके डिवाइस पर लोकल रीज़निंग",
      title: "formal-ai डेस्कटॉप",
      summary:
        "formal-ai को लोकली चलाएँ: एक इन-प्रोसेस रीज़निंग एजेंट और एक वैकल्पिक, डिफ़ॉल्ट रूप से बंद OpenAI-संगत सर्वर। macOS, Windows और Linux के लिए उपलब्ध।",
      release: "नवीनतम रिलीज़",
      checksum: "चेकसम",
      provenance: "बिल्ड प्रोवेनेंस",
      primaryUnknown: "अपना ऑपरेटिंग सिस्टम चुनें",
      primaryAction: "डाउनलोड",
      otherSystems: "अन्य डाउनलोड",
      macos: "macOS", windows: "Windows", linux: "Linux",
      allReleases: "सभी रिलीज़",
      sourceCode: "सोर्स कोड",
      backToApp: "वेब ऐप खोलें",
      language: "भाषा",
      theme: "थीम",
      themeAuto: "ऑटो", themeLight: "लाइट", themeDark: "डार्क",
      statusReady: "रिलीज़ फ़ाइलें तैयार हैं",
      statusLoading: "नवीनतम रिलीज़ जाँची जा रही है",
      statusFallback: "डाउनलोड के लिए नवीनतम रिलीज़ खोलें",
      downloadChecking: "रिलीज़ फ़ाइलें जाँची जा रही हैं",
      downloadUnavailable: "नवीनतम रिलीज़ में उपलब्ध नहीं",
      macArm: "macOS Apple silicon", macArmZip: "macOS Apple silicon zip",
      macIntel: "macOS Intel", macIntelZip: "macOS Intel zip",
      winInstaller: "Windows इंस्टॉलर", winInstallerArm: "Windows ARM इंस्टॉलर",
      winPortable: "Windows पोर्टेबल", winPortableArm: "Windows ARM पोर्टेबल",
      linuxAppImage: "Linux AppImage", linuxAppImageArm: "Linux ARM AppImage",
      linuxDeb: "Linux .deb", linuxDebArm: "Linux ARM .deb",
      linuxTar: "Linux tar.gz", linuxTarArm: "Linux ARM tar.gz",
      previewAlt: "formal-ai एप्लिकेशन इंटरफ़ेस पूर्वावलोकन",
      verify: "उसी रिलीज़ की SHA256SUMS.txt से डाउनलोड सत्यापित करें।",
      verifyTitle: "अपना डाउनलोड सत्यापित करें",
      verifyRegular: "UI जाँच",
      verifyAdvanced: "कमांड-लाइन जाँच",
      verifyUiIntro:
        "डाउनलोड की गई फ़ाइल और SHA256SUMS.txt चुनें। जाँच इसी ब्राउज़र में लोकली चलती है।",
      verifyFile: "डाउनलोड की गई फ़ाइल",
      verifyChecksumFile: "SHA256SUMS.txt",
      verifyResultIdle: "SHA-256 चेकसम की तुलना के लिए दोनों फ़ाइलें चुनें।",
      verifyResultWorking: "चेकसम की गणना हो रही है...",
      verifyResultMissing: "इस फ़ाइलनाम के लिए कोई मिलती-जुलती पंक्ति नहीं मिली।",
      verifyResultMatch: "चेकसम मेल खाता है।",
      verifyResultMismatch: "चेकसम मेल नहीं खाता।",
      regularStepOne: "उसी रिलीज़ से ऐप और SHA256SUMS.txt डाउनलोड करें।",
      regularStepTwo: "नीचे दिए फ़ॉर्म में दोनों फ़ाइलें चुनें। वे आपके डिवाइस पर ही रहती हैं।",
      regularStepThree: "तभी इंस्टॉल करें जब पेज बताए कि चेकसम मेल खाता है।",
      windowsCommand: "Windows PowerShell",
      macosCommand: "macOS Terminal",
      linuxCommand: "Linux Terminal",
      advancedStepOne:
        "रिपॉज़िटरी, workflow run, टैग, कमिट और बिल्डर OS के लिए BUILD-PROVENANCE.txt जाँचें।",
      advancedStepTwo:
        "जब रिलीज़ attestation उपलब्ध हों, तो GitHub CLI से आर्टिफ़ैक्ट सत्यापित करें।",
      reproducibleNote:
        "बाइट-दर-बाइट पुनरुत्पादनीय डेस्कटॉप बिल्ड के लिए एक पिन किया हुआ रीबिल्ड वातावरण चाहिए; यह रिलीज़ अभी प्रोवेनेंस दर्ज करती है और उस मज़बूत गारंटी को स्पष्ट रखती है।",
      installMacosTitle: "macOS पर ऐप खोलें",
      installMacosWhy:
        "बिल्ड ad-hoc साइन किए गए हैं, बिना Apple Developer ID के, इसलिए macOS Gatekeeper पहली बार लॉन्च रोकता है। ऊपर SHA-256 सत्यापित करने के बाद, ऐप को एक बार अनुमति देने के लिए नीचे का कोई भी तरीका अपनाएँ।",
      installMacosTerminalTitle: "टर्मिनल एक-पंक्ति कमांड",
      installMacosTerminalStep:
        "formal-ai Desktop.app को /Applications में खींचने के बाद, टर्मिनल से quarantine विशेषता हटाएँ:",
      installMacosSettingsTitle: "सिस्टम सेटिंग्स (macOS 15 Sequoia)",
      installMacosSettingsStep1:
        'formal-ai Desktop पर डबल-क्लिक करें, फिर "Apple could not verify..." आने पर Done क्लिक करें।',
      installMacosSettingsStep2:
        "सिस्टम सेटिंग्स → Privacy & Security खोलें और Security खंड तक स्क्रॉल करें।",
      installMacosSettingsStep3:
        'formal-ai Desktop के पास "Open Anyway" क्लिक करें, पुष्टि करें और प्रमाणित करें।',
      installMacosFooter:
        "बाद के लॉन्च में चेतावनी नहीं दिखती। ये चरण केवल उन formal-ai रिलीज़ आर्टिफ़ैक्ट्स के लिए चलाएँ जिनका SHA-256 उसी GitHub रिलीज़ की SHA256SUMS.txt से मेल खाता है।",
      installMacosShotsCaption:
        "हमारे सहयोगी ऐप VK Bot Desktop में लिए गए असली macOS 15 (Sequoia) Gatekeeper संवाद, जो वही ad-hoc साइनिंग इस्तेमाल करता है — formal-ai Desktop अपने नाम के साथ वही संवाद दिखाता है।",
      installMacosShot1Alt:
        'macOS चेतावनी संवाद: "VK Bot Desktop" नहीं खुला, "Done" और "Move to Trash" बटनों के साथ।',
      installMacosShot2Alt:
        'macOS सिस्टम सेटिंग्स, प्राइवेसी और सुरक्षा, VK Bot Desktop के लिए "Open Anyway" बटन के साथ।',
      installMacosShot3Alt:
        'macOS पुष्टिकरण संवाद: "VK Bot Desktop" खोलें? "Open Anyway", "Move to Trash" और "Done" बटनों के साथ।',
      agentTitle: "डिफ़ॉल्ट रूप से इन-प्रोसेस",
      agentBody:
        "formal-ai डेस्कटॉप एक इन-प्रोसेस रीज़निंग एजेंट चलाता है — किसी सर्वर की ज़रूरत नहीं। आप चाहें तो एक लोकल OpenAI-संगत सर्वर सक्षम कर सकते हैं और claude, codex या agent CLI को उस पर निर्देशित कर सकते हैं।",
      agentDocs: "सर्वर API और CLI सेटअप",
    },
  };

  function text(locale, key) {
    return (copy[locale] && copy[locale][key]) || copy.en[key] || key;
  }

  // ---------------------------------------------------------------------------
  // Detection
  // ---------------------------------------------------------------------------

  function detectOperatingSystem() {
    if (global.__FORMAL_AI_DOWNLOAD_OS__) {
      return global.__FORMAL_AI_DOWNLOAD_OS__;
    }
    try {
      var params = new URLSearchParams(global.location ? global.location.search : "");
      var forced = params.get("os");
      if (forced) return forced;
    } catch (_error) {
      /* ignore */
    }
    var nav = typeof navigator !== "undefined" ? navigator : {};
    var uaData = nav.userAgentData;
    var platform = String(
      (uaData && uaData.platform) || nav.platform || "",
    ).toLowerCase();
    var userAgent = String(nav.userAgent || "").toLowerCase();
    var signal = platform + " " + userAgent;
    if (signal.indexOf("mac") !== -1) return "macos";
    if (signal.indexOf("win") !== -1) return "windows";
    if (signal.indexOf("linux") !== -1 || signal.indexOf("x11") !== -1) return "linux";
    return "unknown";
  }

  function normalizeLocale(tag) {
    var lower = String(tag || "").toLowerCase();
    for (var i = 0; i < SUPPORTED_LOCALES.length; i += 1) {
      if (lower === SUPPORTED_LOCALES[i] || lower.indexOf(SUPPORTED_LOCALES[i] + "-") === 0) {
        return SUPPORTED_LOCALES[i];
      }
    }
    if (lower.indexOf("zh") === 0) return "zh";
    return undefined;
  }

  function detectLocaleFromBrowser() {
    var nav = typeof navigator !== "undefined" ? navigator : {};
    var languages = nav.languages || (nav.language ? [nav.language] : ["en"]);
    for (var i = 0; i < languages.length; i += 1) {
      var match = normalizeLocale(languages[i]);
      if (match) return match;
    }
    return "en";
  }

  // ---------------------------------------------------------------------------
  // Preference round-trip (shared with the chat app via ../preferences.js)
  // ---------------------------------------------------------------------------

  function readPreferences() {
    if (global.FormalAiPreferences && typeof global.FormalAiPreferences.load === "function") {
      return global.FormalAiPreferences.load({});
    }
    return {};
  }

  function writePreference(key, value) {
    if (!global.FormalAiPreferences || typeof global.FormalAiPreferences.save !== "function") {
      return;
    }
    // load() returns exactly the stored keys (defaults={}), so merging here
    // preserves every other key the chat app persisted.
    var current = global.FormalAiPreferences.load({});
    current[key] = value;
    global.FormalAiPreferences.save(current);
  }

  function resolveTheme(themePreference) {
    if (themePreference === "dark") return "dark";
    if (themePreference === "light") return "light";
    if (
      typeof global.matchMedia === "function" &&
      global.matchMedia("(prefers-color-scheme: dark)").matches
    ) {
      return "dark";
    }
    return "light";
  }

  function resolveLocale(localePreference) {
    var normalized = normalizeLocale(localePreference);
    if (normalized) return normalized;
    return detectLocaleFromBrowser();
  }

  // ---------------------------------------------------------------------------
  // Checksum helpers (client-side SHA-256, identical contract to the reference)
  // ---------------------------------------------------------------------------

  function sha256Hex(file) {
    return file.arrayBuffer().then(function (buffer) {
      return crypto.subtle.digest("SHA-256", buffer).then(function (hash) {
        var bytes = new Uint8Array(hash);
        var out = "";
        for (var i = 0; i < bytes.length; i += 1) {
          out += bytes[i].toString(16).padStart(2, "0");
        }
        return out;
      });
    });
  }

  function checksumForFile(textValue, fileName) {
    var lines = String(textValue || "").split(/\r?\n/);
    for (var i = 0; i < lines.length; i += 1) {
      var match = lines[i].trim().match(/^([a-fA-F0-9]{64})\s+\*?(.+)$/);
      if (match && match[2].trim() === fileName) {
        return match[1].toLowerCase();
      }
    }
    return undefined;
  }

  function verificationCommands(release) {
    var version = releaseVersion(release) || "0.0.0";
    return [
      {
        key: "windowsCommand",
        command:
          "Get-FileHash .\\formal-ai-desktop-windows-installer-x64-" +
          version +
          ".exe -Algorithm SHA256",
      },
      {
        key: "macosCommand",
        command: "shasum -a 256 formal-ai-desktop-macos-arm64-" + version + ".dmg",
      },
      { key: "linuxCommand", command: "sha256sum -c SHA256SUMS.txt --ignore-missing" },
    ];
  }

  var MACOS_INSTALL_COMMAND =
    'sudo xattr -dr com.apple.quarantine "/Applications/formal-ai Desktop.app"';

  // Real macOS 15 (Sequoia) Gatekeeper dialogs, mapped 1:1 to the System Settings
  // steps below (installMacosSettingsStep1/2/3). Issue #479 asked for macOS
  // screenshots like konard.github.io/vk-bot-desktop, and the maintainer was
  // explicit that synthetic/drawn images are not acceptable -- they must be
  // "copied from our code" at https://github.com/konard/vk-bot-desktop. Gatekeeper
  // cannot be triggered on a hosted macOS CI runner, so we reuse the genuine
  // captures from our sibling desktop app VK Bot Desktop, which ships with the
  // same explicit electron-builder ad-hoc signing hook (identity "-") when no
  // Apple Developer ID secrets are available.
  // The dialog wording, layout and buttons are byte-identical for formal-ai
  // Desktop; only the app name shown in the prompt differs ("VK Bot Desktop" vs
  // "formal-ai Desktop"). Provenance is documented in
  // src/web/download/assets/screenshots/README.md.
  var MACOS_GATEKEEPER_SHOTS = [
    { src: "assets/screenshots/macos-gatekeeper-not-opened.png", altKey: "installMacosShot1Alt" },
    { src: "assets/screenshots/macos-gatekeeper-open-anyway.png", altKey: "installMacosShot2Alt" },
    { src: "assets/screenshots/macos-gatekeeper-confirm.png", altKey: "installMacosShot3Alt" },
  ];

  // ---------------------------------------------------------------------------
  // Tiny hyperscript helper (avoids inline handlers/styles for CSP compliance)
  // ---------------------------------------------------------------------------

  function h(tag, props) {
    var el = document.createElement(tag);
    if (props) {
      Object.keys(props).forEach(function (key) {
        var value = props[key];
        if (value == null || value === false) return;
        if (key === "class") {
          el.className = value;
        } else if (key === "text") {
          el.textContent = value;
        } else if (key === "dataset") {
          Object.keys(value).forEach(function (dataKey) {
            el.dataset[dataKey] = value[dataKey];
          });
        } else if (key.indexOf("on") === 0 && typeof value === "function") {
          el.addEventListener(key.slice(2).toLowerCase(), value);
        } else if (value === true) {
          el.setAttribute(key, "");
        } else {
          el.setAttribute(key, String(value));
        }
      });
    }
    for (var i = 2; i < arguments.length; i += 1) {
      appendChild(el, arguments[i]);
    }
    return el;
  }

  function appendChild(parent, child) {
    if (child == null || child === false) return;
    if (Array.isArray(child)) {
      child.forEach(function (item) {
        appendChild(parent, item);
      });
      return;
    }
    if (typeof child === "string" || typeof child === "number") {
      parent.appendChild(document.createTextNode(String(child)));
      return;
    }
    parent.appendChild(child);
  }

  // ---------------------------------------------------------------------------
  // Rendering
  // ---------------------------------------------------------------------------

  var state = {
    locale: "en",
    themePreference: "auto",
    selectedOs: "unknown",
    release: null,
    releaseStatus: "loading",
  };

  function downloadOptionLink(option, releaseAssets, release, locale, compact) {
    var asset = resolveDownloadAsset(option, releaseAssets, release);
    var href = asset ? asset.browser_download_url : undefined;
    var displayName = (asset && asset.name) || assetNameFor(option, release);
    var className = compact ? "download-chip" : "download-primary-card";
    if (href) {
      return h(
        "a",
        { class: className, href: href, "data-testid": "download-" + option.id },
        h("span", { text: text(locale, option.labelKey) }),
        compact ? null : h("code", { text: displayName }),
      );
    }
    return h(
      "div",
      { class: className + " unavailable", "aria-disabled": "true", "data-testid": "download-" + option.id },
      h("span", { text: text(locale, option.labelKey) }),
      compact ? null : h("code", { text: displayName }),
    );
  }

  function downloadFamily(family, releaseAssets, release, locale) {
    return h(
      "div",
      { class: "download-family" },
      downloadOptionLink(family.primary, releaseAssets, release, locale, false),
      h(
        "div",
        { class: "download-secondary-list" },
        family.secondary.map(function (option) {
          return downloadOptionLink(option, releaseAssets, release, locale, true);
        }),
      ),
    );
  }

  function segmentedControl(options, activeValue, onSelect, ariaLabel, className) {
    return h(
      "div",
      { class: className, role: "group", "aria-label": ariaLabel },
      options.map(function (option) {
        return h("button", {
          type: "button",
          class: activeValue === option.value ? "active" : "",
          "aria-pressed": activeValue === option.value ? "true" : "false",
          "data-value": option.value,
          text: option.label,
          onClick: function () {
            onSelect(option.value);
          },
        });
      }),
    );
  }

  function verificationTool(locale) {
    var fileInput = h("input", { type: "file", "data-testid": "verify-file" });
    var sumInput = h("input", {
      type: "file",
      accept: ".txt,text/plain",
      "data-testid": "verify-sums",
    });
    var resultEl = h("div", {
      class: "verification-result idle",
      role: "status",
      "data-testid": "verify-result",
      text: text(locale, "verifyResultIdle"),
    });

    function setResult(stateName, key) {
      resultEl.className = "verification-result " + stateName;
      resultEl.textContent = text(locale, key);
    }

    function run() {
      var file = fileInput.files && fileInput.files[0];
      var sums = sumInput.files && sumInput.files[0];
      if (!file || !sums) {
        setResult("idle", "verifyResultIdle");
        return;
      }
      setResult("working", "verifyResultWorking");
      Promise.all([sha256Hex(file), sums.text()])
        .then(function (results) {
          var actual = results[0];
          var expected = checksumForFile(results[1], file.name);
          if (!expected) {
            setResult("missing", "verifyResultMissing");
            return;
          }
          setResult(
            actual === expected ? "match" : "mismatch",
            actual === expected ? "verifyResultMatch" : "verifyResultMismatch",
          );
        })
        .catch(function () {
          setResult("missing", "verifyResultMissing");
        });
    }

    fileInput.addEventListener("change", run);
    sumInput.addEventListener("change", run);

    return h(
      "div",
      { class: "verification-tool" },
      h("p", { text: text(locale, "verifyUiIntro") }),
      h(
        "div",
        { class: "verification-inputs" },
        h("label", null, text(locale, "verifyFile"), fileInput),
        h("label", null, text(locale, "verifyChecksumFile"), sumInput),
      ),
      resultEl,
    );
  }

  function render() {
    var root = document.getElementById("download-app");
    if (!root) return;
    var locale = state.locale;
    var release = state.release;
    var releaseAssets = assetsByName(release);
    var primaryOption = primaryOptionFor(state.selectedOs);
    var primaryHref = resolveDownloadHref(primaryOption, releaseAssets, release);
    var statusKey =
      state.releaseStatus === "ready"
        ? "statusReady"
        : state.releaseStatus === "loading"
          ? "statusLoading"
          : "statusFallback";

    root.textContent = "";

    // ---- Top bar: language + theme switchers + nav -------------------------
    var topbar = h(
      "header",
      { class: "download-topbar" },
      h(
        "a",
        { class: "brand", href: "../app/", "data-testid": "back-to-app" },
        h("span", { class: "brand-mark", "aria-hidden": "true", text: "◆" }),
        h("span", { text: "formal-ai" }),
      ),
      h(
        "div",
        { class: "topbar-controls" },
        segmentedControl(
          SUPPORTED_LOCALES.map(function (value) {
            return { value: value, label: value.toUpperCase() };
          }),
          locale,
          function (value) {
            state.locale = value;
            writePreference("uiLanguage", value);
            applyDocumentChrome();
            render();
          },
          text(locale, "language"),
          "locale-switch",
        ),
        segmentedControl(
          [
            { value: "auto", label: text(locale, "themeAuto") },
            { value: "light", label: text(locale, "themeLight") },
            { value: "dark", label: text(locale, "themeDark") },
          ],
          state.themePreference,
          function (value) {
            state.themePreference = value;
            writePreference("theme", value);
            applyDocumentChrome();
            render();
          },
          text(locale, "theme"),
          "theme-switch",
        ),
      ),
    );

    // ---- Hero ---------------------------------------------------------------
    var primaryCard;
    if (primaryOption && primaryHref) {
      primaryCard = h(
        "a",
        { class: "primary-download", href: primaryHref, "data-testid": "primary-download" },
        h("span", { text: text(locale, "primaryAction") }),
        h("strong", { text: text(locale, primaryOption.labelKey) }),
      );
    } else if (primaryOption) {
      primaryCard = h(
        "div",
        { class: "primary-download empty", "aria-disabled": "true", "data-testid": "primary-download" },
        h("span", { text: text(locale, "primaryAction") }),
        h("strong", { text: text(locale, primaryOption.labelKey) }),
        h("em", {
          text: text(
            locale,
            state.releaseStatus === "loading" ? "downloadChecking" : "downloadUnavailable",
          ),
        }),
      );
    } else {
      primaryCard = h(
        "div",
        { class: "primary-download empty", "data-testid": "primary-download" },
        h("span", { text: text(locale, "primaryUnknown") }),
      );
    }

    var previewOs = state.selectedOs === "unknown" ? "macos" : state.selectedOs;
    var hero = h(
      "section",
      { class: "hero", "aria-labelledby": "site-title" },
      h(
        "div",
        { class: "hero-copy" },
        h("p", { class: "eyebrow", text: text(locale, "eyebrow") }),
        h("h1", { id: "site-title", text: text(locale, "title") }),
        h("p", { class: "summary", text: text(locale, "summary") }),
        h(
          "div",
          { class: "status-row", role: "status", "data-testid": "release-status" },
          h("span", { text: text(locale, statusKey) }),
          release && release.tag_name ? h("strong", { text: release.tag_name }) : null,
        ),
        h(
          "div",
          { class: "download-panel" },
          primaryCard,
          segmentedControl(
            ["macos", "windows", "linux"].map(function (os) {
              return { value: os, label: text(locale, os) };
            }),
            state.selectedOs,
            function (value) {
              state.selectedOs = value;
              render();
            },
            text(locale, "otherSystems"),
            "os-tabs",
          ),
        ),
        h(
          "nav",
          { class: "support-links", "aria-label": text(locale, "release") },
          h("a", { href: resolveChecksumHref(releaseAssets), text: text(locale, "checksum") }),
          h("a", { href: RELEASES_URL, text: text(locale, "allReleases") }),
          h("a", { href: "https://github.com/" + REPO, text: text(locale, "sourceCode") }),
        ),
      ),
      heroMedia(previewOs, locale),
    );

    // ---- All downloads ------------------------------------------------------
    var downloadsSection = h(
      "section",
      { class: "downloads", "aria-labelledby": "downloads-title" },
      h(
        "div",
        null,
        h("p", { class: "eyebrow", text: text(locale, "otherSystems") }),
        h("h2", { id: "downloads-title", text: text(locale, "release") }),
      ),
      h(
        "div",
        { class: "download-grid" },
        downloadFamilies().map(function (group) {
          return h(
            "div",
            { class: "download-group", "data-os": group.os },
            h("h3", { text: text(locale, group.os) }),
            group.families.map(function (family) {
              return downloadFamily(family, releaseAssets, release, locale);
            }),
          );
        }),
      ),
      h("p", { class: "verify-note", text: text(locale, "verify") }),
    );

    // ---- In-process agent callout ------------------------------------------
    var agentSection = h(
      "section",
      { class: "agent-callout", "aria-labelledby": "agent-title" },
      h("h2", { id: "agent-title", text: text(locale, "agentTitle") }),
      h("p", { text: text(locale, "agentBody") }),
      h(
        "nav",
        { class: "support-links", "aria-label": text(locale, "agentDocs") },
        h(
          "a",
          {
            href: "https://github.com/" + REPO + "/blob/main/docs/desktop/server-api.md",
            text: text(locale, "agentDocs"),
          },
        ),
      ),
    );

    // ---- macOS Gatekeeper ---------------------------------------------------
    var installMacos = h(
      "section",
      { class: "install-macos", "aria-labelledby": "install-macos-title" },
      h(
        "div",
        null,
        h("p", { class: "eyebrow", text: text(locale, "macos") }),
        h("h2", { id: "install-macos-title", text: text(locale, "installMacosTitle") }),
      ),
      h("p", { class: "install-macos-why", text: text(locale, "installMacosWhy") }),
      h(
        "div",
        { class: "install-macos-grid" },
        h(
          "details",
          { open: true },
          h("summary", { text: text(locale, "installMacosSettingsTitle") }),
          h(
            "ol",
            null,
            h("li", { text: text(locale, "installMacosSettingsStep1") }),
            h("li", { text: text(locale, "installMacosSettingsStep2") }),
            h("li", { text: text(locale, "installMacosSettingsStep3") }),
          ),
          h(
            "figure",
            { class: "install-macos-screenshots" },
            MACOS_GATEKEEPER_SHOTS.map(function (shot) {
              return h("img", {
                src: shot.src,
                alt: text(locale, shot.altKey),
                loading: "lazy",
                decoding: "async",
              });
            }),
            h("figcaption", { text: text(locale, "installMacosShotsCaption") }),
          ),
        ),
        h(
          "details",
          null,
          h("summary", { text: text(locale, "installMacosTerminalTitle") }),
          h("ol", null, h("li", { text: text(locale, "installMacosTerminalStep") })),
          h(
            "div",
            { class: "command-list" },
            h(
              "div",
              null,
              h("strong", { text: text(locale, "macosCommand") }),
              h("code", { text: MACOS_INSTALL_COMMAND }),
            ),
          ),
        ),
      ),
      h("p", { class: "install-macos-footer", text: text(locale, "installMacosFooter") }),
    );

    // ---- Verification -------------------------------------------------------
    var verification = h(
      "section",
      { class: "verification", "aria-labelledby": "verification-title" },
      h(
        "div",
        null,
        h("p", { class: "eyebrow", text: text(locale, "checksum") }),
        h("h2", { id: "verification-title", text: text(locale, "verifyTitle") }),
      ),
      h(
        "div",
        { class: "verification-grid" },
        h(
          "details",
          { open: true },
          h("summary", { text: text(locale, "verifyRegular") }),
          h(
            "ol",
            null,
            h("li", { text: text(locale, "regularStepOne") }),
            h("li", { text: text(locale, "regularStepTwo") }),
            h("li", { text: text(locale, "regularStepThree") }),
          ),
          verificationTool(locale),
        ),
        h(
          "details",
          null,
          h("summary", { text: text(locale, "verifyAdvanced") }),
          h(
            "ol",
            null,
            h("li", { text: text(locale, "advancedStepOne") }),
            h("li", { text: text(locale, "advancedStepTwo") }),
          ),
          h(
            "div",
            { class: "command-list" },
            verificationCommands(release)
              .map(function (item) {
                return h(
                  "div",
                  null,
                  h("strong", { text: text(locale, item.key) }),
                  h("code", { text: item.command }),
                );
              })
              .concat([
                h(
                  "div",
                  null,
                  h("strong", { text: "GitHub CLI" }),
                  h("code", {
                    text: "gh attestation verify ./downloaded-file --repo " + REPO,
                  }),
                ),
              ]),
          ),
          h("p", { text: text(locale, "reproducibleNote") }),
        ),
      ),
      h(
        "nav",
        { class: "support-links", "aria-label": text(locale, "checksum") },
        h("a", { href: resolveChecksumHref(releaseAssets), text: text(locale, "checksum") }),
        h("a", { href: resolveProvenanceHref(releaseAssets), text: text(locale, "provenance") }),
      ),
    );

    appendChild(root, [topbar, hero, downloadsSection, agentSection, installMacos, verification]);
  }

  function heroMedia(previewOs, locale) {
    var frame = h(
      "div",
      { class: "window-frame" },
      h(
        "div",
        { class: "window-titlebar", "aria-hidden": "true" },
        h(
          "span",
          { class: "traffic-lights" },
          h("span", null),
          h("span", null),
          h("span", null),
        ),
        h("span", { class: "window-title", text: "formal-ai Desktop" }),
        h(
          "span",
          { class: "window-actions" },
          h("span", null),
          h("span", null),
          h("span", null),
        ),
      ),
    );

    // A CSS faux-transcript placeholder that always looks intentional. When the
    // generated preview screenshot exists it is layered on top; on error we keep
    // the placeholder (no broken-image icon).
    var placeholder = h(
      "div",
      { class: "window-placeholder", "aria-hidden": "true" },
      h("div", { class: "ph-line ph-user" }),
      h("div", { class: "ph-line ph-assistant" }),
      h("div", { class: "ph-line ph-assistant short" }),
      h("div", { class: "ph-line ph-user short" }),
      h("div", { class: "ph-line ph-assistant" }),
    );
    frame.appendChild(placeholder);

    var theme = resolveTheme(state.themePreference);
    var img = h("img", {
      class: "window-preview",
      alt: text(locale, "previewAlt"),
      src: "assets/app-preview-" + locale + "-" + theme + ".png",
      loading: "eager",
    });
    img.addEventListener("load", function () {
      frame.classList.add("has-preview");
    });
    img.addEventListener("error", function () {
      if (img.getAttribute("src") !== "assets/app-preview.png") {
        img.setAttribute("src", "assets/app-preview.png");
      } else {
        frame.classList.remove("has-preview");
      }
    });
    frame.appendChild(img);

    return h(
      "div",
      { class: "hero-media " + previewOs, "aria-label": text(locale, "previewAlt") },
      frame,
    );
  }

  // ---------------------------------------------------------------------------
  // Document chrome (theme attribute + lang) — shared with the chat app
  // ---------------------------------------------------------------------------

  function applyDocumentChrome() {
    if (typeof document === "undefined") return;
    var theme = resolveTheme(state.themePreference);
    document.documentElement.setAttribute("data-theme", theme);
    document.documentElement.lang = state.locale;
  }

  // ---------------------------------------------------------------------------
  // Release fetch
  // ---------------------------------------------------------------------------

  function loadRelease() {
    // Deterministic injection for tests / CI screenshots — skips the network.
    if (global.__FORMAL_AI_DOWNLOAD_RELEASE__) {
      state.release = global.__FORMAL_AI_DOWNLOAD_RELEASE__;
      state.releaseStatus = "ready";
      render();
      return;
    }
    if (typeof fetch !== "function") {
      state.releaseStatus = "fallback";
      render();
      return;
    }
    var controller = typeof AbortController === "function" ? new AbortController() : undefined;
    fetch(RELEASE_API, controller ? { signal: controller.signal } : undefined)
      .then(function (response) {
        if (!response.ok) throw new Error("Release request failed: " + response.status);
        return response.json();
      })
      .then(function (data) {
        state.release = data;
        state.releaseStatus = "ready";
        render();
      })
      .catch(function (error) {
        if (!error || error.name !== "AbortError") {
          state.releaseStatus = "fallback";
          render();
        }
      });
  }

  // ---------------------------------------------------------------------------
  // Init
  // ---------------------------------------------------------------------------

  function init() {
    var prefs = readPreferences();
    state.themePreference =
      prefs.theme === "dark" || prefs.theme === "light" ? prefs.theme : "auto";
    state.locale = resolveLocale(prefs.uiLanguage);
    state.selectedOs = detectOperatingSystem();
    state.releaseStatus = "loading";

    applyDocumentChrome();
    render();
    loadRelease();

    // Follow OS theme changes while in "auto".
    if (typeof global.matchMedia === "function") {
      var media = global.matchMedia("(prefers-color-scheme: dark)");
      var onChange = function () {
        if (state.themePreference === "auto") {
          applyDocumentChrome();
          render();
        }
      };
      if (typeof media.addEventListener === "function") {
        media.addEventListener("change", onChange);
      } else if (typeof media.addListener === "function") {
        media.addListener(onChange);
      }
    }
  }

  // Exposed for the e2e suite and for reuse.
  global.FormalAiDownload = {
    REPO: REPO,
    RELEASE_API: RELEASE_API,
    RELEASES_URL: RELEASES_URL,
    downloadOptions: downloadOptions,
    downloadFamilies: downloadFamilies,
    optionById: optionById,
    primaryOptionFor: primaryOptionFor,
    assetsByName: assetsByName,
    releaseVersion: releaseVersion,
    assetNameFor: assetNameFor,
    candidateAssetNames: candidateAssetNames,
    resolveDownloadAsset: resolveDownloadAsset,
    resolveDownloadHref: resolveDownloadHref,
    resolveChecksumHref: resolveChecksumHref,
    resolveProvenanceHref: resolveProvenanceHref,
    detectOperatingSystem: detectOperatingSystem,
    normalizeLocale: normalizeLocale,
    resolveLocale: resolveLocale,
    resolveTheme: resolveTheme,
    checksumForFile: checksumForFile,
    verificationCommands: verificationCommands,
    text: text,
    copy: copy,
    SUPPORTED_LOCALES: SUPPORTED_LOCALES,
    render: render,
    init: init,
    state: state,
  };

  if (typeof document !== "undefined") {
    if (document.readyState === "loading") {
      document.addEventListener("DOMContentLoaded", init);
    } else {
      init();
    }
  }
})(typeof window !== "undefined" ? window : globalThis);
