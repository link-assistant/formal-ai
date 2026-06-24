import { useEffect, useMemo, useState } from 'react';
import {
  RELEASE_API,
  RELEASES_URL,
  assetNameFor,
  assetsByName,
  downloadFamilies,
  primaryOptionFor,
  releaseVersion,
  resolveDownloadAsset,
  resolveChecksumHref,
  resolveDownloadHref,
  resolveProvenanceHref,
} from './downloads.js';

const copy = {
  en: {
    eyebrow: 'Local VK automation',
    title: 'VK Bot Desktop',
    summary:
      'Run the VK bot from a signed desktop app with local and SSH server modes.',
    release: 'Latest release',
    checksum: 'Checksums',
    primaryUnknown: 'Choose your operating system',
    primaryAction: 'Download',
    otherSystems: 'Other downloads',
    macos: 'macOS',
    windows: 'Windows',
    linux: 'Linux',
    allReleases: 'All releases',
    statusReady: 'Release assets ready',
    statusLoading: 'Checking latest release',
    statusFallback: 'Open latest release to download',
    downloadChecking: 'Checking release assets',
    downloadUnavailable: 'Not available in latest release',
    macArm: 'macOS Apple silicon',
    macArmZip: 'macOS Apple silicon zip',
    macIntel: 'macOS Intel',
    macIntelZip: 'macOS Intel zip',
    winInstaller: 'Windows installer',
    winInstallerArm: 'Windows ARM installer',
    winPortable: 'Windows portable',
    winPortableArm: 'Windows ARM portable',
    linuxAppImage: 'Linux AppImage',
    linuxAppImageArm: 'Linux ARM AppImage',
    linuxDeb: 'Linux .deb',
    linuxDebArm: 'Linux ARM .deb',
    linuxTar: 'Linux tar.gz',
    linuxTarArm: 'Linux ARM tar.gz',
    previewAlt: 'VK Bot Desktop application interface preview',
    verify: 'Verify downloads with SHA256SUMS.txt from the same release.',
    provenance: 'Build provenance',
    verifyTitle: 'Verify your download',
    verifyRegular: 'UI check',
    verifyAdvanced: 'Command-line check',
    verifyUiIntro:
      'Select the downloaded file and SHA256SUMS.txt. The check runs locally in this browser.',
    verifyFile: 'Downloaded file',
    verifyChecksumFile: 'SHA256SUMS.txt',
    verifyResultIdle: 'Choose both files to compare the SHA-256 checksum.',
    verifyResultWorking: 'Calculating checksum...',
    verifyResultMissing: 'No matching line was found for this filename.',
    verifyResultMatch: 'Checksum matches.',
    verifyResultMismatch: 'Checksum does not match.',
    regularStepOne:
      'Download the app and SHA256SUMS.txt from the same release.',
    regularStepTwo:
      'Use the form below to select both files. They stay on your device.',
    regularStepThree:
      'Install only when the page reports that the checksum matches.',
    windowsCommand: 'Windows PowerShell',
    macosCommand: 'macOS Terminal',
    linuxCommand: 'Linux Terminal',
    advancedStepOne:
      'Check BUILD-PROVENANCE.txt for the repository, workflow run, tag, commit, and builder OS.',
    advancedStepTwo:
      'When release attestations are available, verify the artifact with GitHub CLI.',
    reproducibleNote:
      'Byte-for-byte reproducible desktop builds need a pinned rebuild environment; this release records provenance now and leaves that stronger guarantee explicit.',
    installMacosTitle: 'Open the app on macOS',
    installMacosWhy:
      'Builds are ad-hoc signed without an Apple Developer ID, so macOS Gatekeeper blocks the first launch with "Apple could not verify VK Bot Desktop is free of malware." After verifying the SHA-256 checksum above, use either of the workflows below to allow the app once.',
    installMacosTerminalTitle: 'Terminal one-liner',
    installMacosTerminalStep:
      'After dragging VK Bot Desktop.app into /Applications, remove the quarantine attribute from a Terminal:',
    installMacosSettingsTitle: 'System Settings (macOS 15 Sequoia)',
    installMacosSettingsStep1:
      'Double-click VK Bot Desktop, then click Done when "Apple could not verify..." appears.',
    installMacosSettingsStep2:
      'Open System Settings → Privacy & Security and scroll to the Security section.',
    installMacosSettingsStep3:
      'Click "Open Anyway" next to VK Bot Desktop, confirm, and authenticate with Touch ID or your admin password.',
    installMacosFooter:
      'Subsequent launches do not show the warning. Only run these steps for VK Bot Desktop release artifacts whose SHA-256 matches SHA256SUMS.txt from the same GitHub release.',
  },
  ru: {
    eyebrow: 'Локальная автоматизация VK',
    title: 'VK Bot Desktop',
    summary:
      'Запускайте VK-бота из подписанного desktop-приложения с локальным и SSH-режимами.',
    release: 'Последний релиз',
    checksum: 'Контрольные суммы',
    primaryUnknown: 'Выберите операционную систему',
    primaryAction: 'Скачать',
    otherSystems: 'Другие загрузки',
    macos: 'macOS',
    windows: 'Windows',
    linux: 'Linux',
    allReleases: 'Все релизы',
    statusReady: 'Файлы релиза готовы',
    statusLoading: 'Проверяем последний релиз',
    statusFallback: 'Откройте последний релиз для загрузки',
    downloadChecking: 'Проверяем файлы релиза',
    downloadUnavailable: 'Нет в последнем релизе',
    macArm: 'macOS Apple silicon',
    macArmZip: 'macOS Apple silicon zip',
    macIntel: 'macOS Intel',
    macIntelZip: 'macOS Intel zip',
    winInstaller: 'Windows installer',
    winInstallerArm: 'Windows ARM installer',
    winPortable: 'Windows portable',
    winPortableArm: 'Windows ARM portable',
    linuxAppImage: 'Linux AppImage',
    linuxAppImageArm: 'Linux ARM AppImage',
    linuxDeb: 'Linux .deb',
    linuxDebArm: 'Linux ARM .deb',
    linuxTar: 'Linux tar.gz',
    linuxTarArm: 'Linux ARM tar.gz',
    previewAlt: 'Интерфейс приложения VK Bot Desktop',
    verify: 'Проверяйте загрузки через SHA256SUMS.txt из того же релиза.',
    provenance: 'Происхождение сборки',
    verifyTitle: 'Проверка загрузки',
    verifyRegular: 'Проверка в интерфейсе',
    verifyAdvanced: 'Проверка в командной строке',
    verifyUiIntro:
      'Выберите скачанный файл и SHA256SUMS.txt. Проверка выполняется локально в браузере.',
    verifyFile: 'Скачанный файл',
    verifyChecksumFile: 'SHA256SUMS.txt',
    verifyResultIdle: 'Выберите оба файла, чтобы сравнить SHA-256.',
    verifyResultWorking: 'Считаем контрольную сумму...',
    verifyResultMissing: 'Для этого имени файла нет строки в SHA256SUMS.txt.',
    verifyResultMatch: 'Контрольная сумма совпадает.',
    verifyResultMismatch: 'Контрольная сумма не совпадает.',
    regularStepOne:
      'Скачайте приложение и SHA256SUMS.txt из одного и того же релиза.',
    regularStepTwo:
      'Выберите оба файла в форме ниже. Они остаются на вашем устройстве.',
    regularStepThree:
      'Устанавливайте файл только если страница сообщает о совпадении.',
    windowsCommand: 'Windows PowerShell',
    macosCommand: 'macOS Terminal',
    linuxCommand: 'Linux Terminal',
    advancedStepOne:
      'Проверьте BUILD-PROVENANCE.txt: репозиторий, workflow run, тег, коммит и OS сборщика.',
    advancedStepTwo:
      'Когда attestation доступен в релизе, проверьте файл через GitHub CLI.',
    reproducibleNote:
      'Побайтово воспроизводимые desktop-сборки требуют зафиксированной среды пересборки; текущий релиз уже записывает provenance и явно отделяет это от более строгой гарантии.',
    installMacosTitle: 'Открытие приложения на macOS',
    installMacosWhy:
      'Сборки подписаны ad-hoc, без Apple Developer ID, поэтому Gatekeeper блокирует первый запуск сообщением «Не удалось проверить, что приложение «VK Bot Desktop» не содержит вредоносного ПО». Сначала сверьте SHA-256 выше, а затем выполните один из вариантов ниже, чтобы открыть приложение.',
    installMacosTerminalTitle: 'Команда в Терминале',
    installMacosTerminalStep:
      'Перетащите VK Bot Desktop.app в /Applications и снимите карантин в Терминале:',
    installMacosSettingsTitle: 'Системные настройки (macOS 15 Sequoia)',
    installMacosSettingsStep1:
      'Откройте VK Bot Desktop двойным щелчком и нажмите «Готово», когда появится предупреждение «Apple не удалось проверить...».',
    installMacosSettingsStep2:
      'Откройте Системные настройки → Конфиденциальность и безопасность и пролистайте до раздела «Безопасность».',
    installMacosSettingsStep3:
      'Нажмите «Открыть всё равно» рядом с VK Bot Desktop, подтвердите и введите пароль администратора или Touch ID.',
    installMacosFooter:
      'При последующих запусках предупреждение не появляется. Используйте эти шаги только для релизных файлов VK Bot Desktop, чья контрольная сумма SHA-256 совпала с SHA256SUMS.txt из того же релиза GitHub.',
  },
};

function text(locale, key) {
  return copy[locale]?.[key] || copy.en[key];
}

function detectLocale() {
  const languages =
    typeof navigator !== 'undefined'
      ? navigator.languages || [navigator.language]
      : ['en'];

  return languages.some((language) =>
    String(language || '')
      .toLowerCase()
      .startsWith('ru')
  )
    ? 'ru'
    : 'en';
}

function detectTheme() {
  if (
    typeof window !== 'undefined' &&
    typeof window.matchMedia === 'function' &&
    window.matchMedia('(prefers-color-scheme: dark)').matches
  ) {
    return 'dark';
  }

  return 'light';
}

function detectOperatingSystem() {
  const userAgentData =
    typeof navigator !== 'undefined' ? navigator.userAgentData : undefined;
  const platform = String(
    userAgentData?.platform ||
      (typeof navigator !== 'undefined' ? navigator.platform : '') ||
      ''
  ).toLowerCase();
  const userAgent = String(
    typeof navigator !== 'undefined' ? navigator.userAgent : ''
  ).toLowerCase();
  const signal = `${platform} ${userAgent}`;

  if (signal.includes('mac')) {
    return 'macos';
  }

  if (signal.includes('win')) {
    return 'windows';
  }

  if (signal.includes('linux') || signal.includes('x11')) {
    return 'linux';
  }

  return 'unknown';
}

function verificationCommands(release) {
  const version = releaseVersion(release) || '0.9.9';

  return [
    {
      key: 'windowsCommand',
      command: `Get-FileHash .\\vk-bot-desktop-windows-installer-x64-${version}.exe -Algorithm SHA256`,
    },
    {
      key: 'macosCommand',
      command: `shasum -a 256 vk-bot-desktop-macos-arm64-${version}.dmg`,
    },
    {
      key: 'linuxCommand',
      command: 'sha256sum -c SHA256SUMS.txt --ignore-missing',
    },
  ];
}

function previewImageFor(locale, theme) {
  return `assets/app-preview-${locale}-${theme}.png`;
}

async function sha256Hex(file) {
  const buffer = await file.arrayBuffer();
  const hash = await crypto.subtle.digest('SHA-256', buffer);
  return Array.from(new Uint8Array(hash))
    .map((byte) => byte.toString(16).padStart(2, '0'))
    .join('');
}

function checksumForFile(textValue, fileName) {
  const lines = String(textValue || '').split(/\r?\n/);
  for (const line of lines) {
    const match = line.trim().match(/^([a-fA-F0-9]{64})\s+\*?(.+)$/);
    if (match && match[2].trim() === fileName) {
      return match[1].toLowerCase();
    }
  }
  return undefined;
}

function DownloadOptionLink({
  option,
  releaseAssets,
  release,
  locale,
  compact,
}) {
  const asset = resolveDownloadAsset(option, releaseAssets, release);
  const href = asset?.browser_download_url;
  const displayName = asset?.name || assetNameFor(option, release);
  const className = compact ? 'download-chip' : 'download-primary-card';

  return href ? (
    <a href={href} className={className}>
      <span>{text(locale, option.labelKey)}</span>
      {!compact ? <code>{displayName}</code> : null}
    </a>
  ) : (
    <div className={`${className} unavailable`} aria-disabled="true">
      <span>{text(locale, option.labelKey)}</span>
      {!compact ? <code>{displayName}</code> : null}
    </div>
  );
}

function DownloadFamily({ family, releaseAssets, release, locale }) {
  return (
    <div className="download-family">
      <DownloadOptionLink
        option={family.primary}
        releaseAssets={releaseAssets}
        release={release}
        locale={locale}
      />
      <div className="download-secondary-list">
        {family.secondary.map((option) => (
          <DownloadOptionLink
            key={option.id}
            option={option}
            releaseAssets={releaseAssets}
            release={release}
            locale={locale}
            compact
          />
        ))}
      </div>
    </div>
  );
}

function VerificationTool({ locale }) {
  const [file, setFile] = useState(null);
  const [checksumFile, setChecksumFile] = useState(null);
  const [result, setResult] = useState({ state: 'idle' });

  useEffect(() => {
    if (!file || !checksumFile) {
      setResult({ state: 'idle' });
      return undefined;
    }

    let active = true;
    setResult({ state: 'working' });
    Promise.all([sha256Hex(file), checksumFile.text()])
      .then(([actualHash, checksumText]) => {
        if (!active) {
          return;
        }
        const expectedHash = checksumForFile(checksumText, file.name);
        if (!expectedHash) {
          setResult({ state: 'missing' });
          return;
        }
        setResult({
          state: actualHash === expectedHash ? 'match' : 'mismatch',
          actualHash,
          expectedHash,
        });
      })
      .catch(() => {
        if (active) {
          setResult({ state: 'missing' });
        }
      });

    return () => {
      active = false;
    };
  }, [file, checksumFile]);

  const resultKey =
    result.state === 'working'
      ? 'verifyResultWorking'
      : result.state === 'missing'
        ? 'verifyResultMissing'
        : result.state === 'match'
          ? 'verifyResultMatch'
          : result.state === 'mismatch'
            ? 'verifyResultMismatch'
            : 'verifyResultIdle';

  return (
    <div className="verification-tool">
      <p>{text(locale, 'verifyUiIntro')}</p>
      <div className="verification-inputs">
        <label>
          {text(locale, 'verifyFile')}
          <input
            type="file"
            onChange={(event) => setFile(event.target.files?.[0] || null)}
          />
        </label>
        <label>
          {text(locale, 'verifyChecksumFile')}
          <input
            type="file"
            accept=".txt,text/plain"
            onChange={(event) =>
              setChecksumFile(event.target.files?.[0] || null)
            }
          />
        </label>
      </div>
      <div className={`verification-result ${result.state}`} role="status">
        {text(locale, resultKey)}
      </div>
    </div>
  );
}

export const MACOS_INSTALL_COMMAND =
  'sudo xattr -dr com.apple.quarantine "/Applications/VK Bot Desktop.app"';

const MACOS_GATEKEEPER_SCREENSHOTS = [
  {
    src: 'assets/screenshots/issue-31-macos-done.png',
    alt: 'macOS warning dialog with Done button',
  },
  {
    src: 'assets/screenshots/issue-31-macos-open-anyway-settings.png',
    alt: 'macOS Privacy & Security settings with Open Anyway button',
  },
  {
    src: 'assets/screenshots/issue-31-macos-open-anyway-confirm.png',
    alt: 'macOS confirmation dialog with Open Anyway button',
  },
];

export default function App() {
  const [locale, setLocale] = useState(() => detectLocale());
  const [theme, setTheme] = useState(() => detectTheme());
  const [selectedOs, setSelectedOs] = useState(() => detectOperatingSystem());
  const [release, setRelease] = useState(null);
  const [releaseStatus, setReleaseStatus] = useState('loading');

  useEffect(() => {
    document.documentElement.lang = locale;
    document.documentElement.dataset.theme = theme;
  }, [locale, theme]);

  useEffect(() => {
    if (typeof window.matchMedia !== 'function') {
      return undefined;
    }

    const media = window.matchMedia('(prefers-color-scheme: dark)');
    const onChange = () => setTheme(detectTheme());

    if (typeof media.addEventListener === 'function') {
      media.addEventListener('change', onChange);
      return () => media.removeEventListener('change', onChange);
    }

    media.addListener(onChange);
    return () => media.removeListener(onChange);
  }, []);

  useEffect(() => {
    const controller = new AbortController();

    fetch(RELEASE_API, { signal: controller.signal })
      .then((response) => {
        if (!response.ok) {
          throw new Error(`Release request failed: ${response.status}`);
        }
        return response.json();
      })
      .then((data) => {
        setRelease(data);
        setReleaseStatus('ready');
      })
      .catch((error) => {
        if (error.name !== 'AbortError') {
          setReleaseStatus('fallback');
        }
      });

    return () => controller.abort();
  }, []);

  const releaseAssets = useMemo(() => assetsByName(release), [release]);
  const primaryOption = primaryOptionFor(selectedOs);
  const primaryHref = resolveDownloadHref(
    primaryOption,
    releaseAssets,
    release
  );
  const previewOs = selectedOs === 'unknown' ? 'macos' : selectedOs;
  const statusKey =
    releaseStatus === 'ready'
      ? 'statusReady'
      : releaseStatus === 'loading'
        ? 'statusLoading'
        : 'statusFallback';

  return (
    <main className="page-shell">
      <section className="hero" aria-labelledby="site-title">
        <div className="hero-copy">
          <div className="locale-switch" aria-label="Language">
            {['en', 'ru'].map((value) => (
              <button
                key={value}
                type="button"
                className={locale === value ? 'active' : ''}
                onClick={() => setLocale(value)}
              >
                {value.toUpperCase()}
              </button>
            ))}
          </div>
          <p className="eyebrow">{text(locale, 'eyebrow')}</p>
          <h1 id="site-title">{text(locale, 'title')}</h1>
          <p className="summary">{text(locale, 'summary')}</p>
          <div className="status-row" role="status">
            <span>{text(locale, statusKey)}</span>
            {release?.tag_name ? <strong>{release.tag_name}</strong> : null}
          </div>
          <div className="download-panel">
            {primaryOption && primaryHref ? (
              <a className="primary-download" href={primaryHref}>
                <span>{text(locale, 'primaryAction')}</span>
                <strong>{text(locale, primaryOption.labelKey)}</strong>
              </a>
            ) : primaryOption ? (
              <div className="primary-download empty" aria-disabled="true">
                <span>{text(locale, 'primaryAction')}</span>
                <strong>{text(locale, primaryOption.labelKey)}</strong>
                <em>
                  {text(
                    locale,
                    releaseStatus === 'loading'
                      ? 'downloadChecking'
                      : 'downloadUnavailable'
                  )}
                </em>
              </div>
            ) : (
              <div className="primary-download empty">
                <span>{text(locale, 'primaryUnknown')}</span>
              </div>
            )}
            <div className="os-tabs" aria-label={text(locale, 'otherSystems')}>
              {['macos', 'windows', 'linux'].map((os) => (
                <button
                  key={os}
                  type="button"
                  className={selectedOs === os ? 'active' : ''}
                  onClick={() => setSelectedOs(os)}
                >
                  {text(locale, os)}
                </button>
              ))}
            </div>
          </div>
          <nav className="support-links" aria-label="Release links">
            <a href={resolveChecksumHref(releaseAssets)}>
              {text(locale, 'checksum')}
            </a>
            <a href={RELEASES_URL}>{text(locale, 'allReleases')}</a>
          </nav>
        </div>
        <div
          className={`hero-media ${previewOs}`}
          aria-label={text(locale, 'previewAlt')}
        >
          <div className="window-frame">
            <div className="window-titlebar" aria-hidden="true">
              <span className="traffic-lights">
                <span />
                <span />
                <span />
              </span>
              <span className="window-title">VK Bot Desktop</span>
              <span className="window-actions">
                <span />
                <span />
                <span />
              </span>
            </div>
            <img
              src={previewImageFor(locale, theme)}
              alt={text(locale, 'previewAlt')}
              onError={(event) => {
                event.currentTarget.src = 'assets/app-preview.png';
              }}
            />
          </div>
        </div>
      </section>

      <section className="downloads" aria-labelledby="downloads-title">
        <div>
          <p className="eyebrow">{text(locale, 'otherSystems')}</p>
          <h2 id="downloads-title">{text(locale, 'release')}</h2>
        </div>
        <div className="download-grid">
          {downloadFamilies().map((group) => (
            <div className="download-group" key={group.os}>
              <h3>{text(locale, group.os)}</h3>
              {group.families.map((family) => (
                <DownloadFamily
                  key={family.id}
                  family={family}
                  releaseAssets={releaseAssets}
                  release={release}
                  locale={locale}
                />
              ))}
            </div>
          ))}
        </div>
        <p className="verify-note">{text(locale, 'verify')}</p>
      </section>

      <section className="install-macos" aria-labelledby="install-macos-title">
        <div>
          <p className="eyebrow">{text(locale, 'macos')}</p>
          <h2 id="install-macos-title">{text(locale, 'installMacosTitle')}</h2>
        </div>
        <p className="install-macos-why">{text(locale, 'installMacosWhy')}</p>
        <div className="install-macos-grid">
          <details open>
            <summary>{text(locale, 'installMacosSettingsTitle')}</summary>
            <ol>
              <li>{text(locale, 'installMacosSettingsStep1')}</li>
              <li>{text(locale, 'installMacosSettingsStep2')}</li>
              <li>{text(locale, 'installMacosSettingsStep3')}</li>
            </ol>
            <div className="install-macos-screenshots">
              {MACOS_GATEKEEPER_SCREENSHOTS.map((screenshot) => (
                <img
                  key={screenshot.src}
                  src={screenshot.src}
                  alt={screenshot.alt}
                  loading="lazy"
                />
              ))}
            </div>
          </details>
          <details>
            <summary>{text(locale, 'installMacosTerminalTitle')}</summary>
            <ol>
              <li>{text(locale, 'installMacosTerminalStep')}</li>
            </ol>
            <div className="command-list">
              <div>
                <strong>{text(locale, 'macosCommand')}</strong>
                <code>{MACOS_INSTALL_COMMAND}</code>
              </div>
            </div>
          </details>
        </div>
        <p className="install-macos-footer">
          {text(locale, 'installMacosFooter')}
        </p>
      </section>

      <section className="verification" aria-labelledby="verification-title">
        <div>
          <p className="eyebrow">{text(locale, 'checksum')}</p>
          <h2 id="verification-title">{text(locale, 'verifyTitle')}</h2>
        </div>
        <div className="verification-grid">
          <details open>
            <summary>{text(locale, 'verifyRegular')}</summary>
            <ol>
              <li>{text(locale, 'regularStepOne')}</li>
              <li>{text(locale, 'regularStepTwo')}</li>
              <li>{text(locale, 'regularStepThree')}</li>
            </ol>
            <VerificationTool locale={locale} />
          </details>
          <details>
            <summary>{text(locale, 'verifyAdvanced')}</summary>
            <ol>
              <li>{text(locale, 'advancedStepOne')}</li>
              <li>{text(locale, 'advancedStepTwo')}</li>
            </ol>
            <div className="command-list">
              {verificationCommands(release).map((item) => (
                <div key={item.key}>
                  <strong>{text(locale, item.key)}</strong>
                  <code>{item.command}</code>
                </div>
              ))}
              <div>
                <strong>GitHub CLI</strong>
                <code>
                  gh attestation verify ./downloaded-file --repo
                  konard/vk-bot-desktop
                </code>
              </div>
            </div>
            <p>{text(locale, 'reproducibleNote')}</p>
          </details>
        </div>
        <nav className="support-links" aria-label="Verification links">
          <a href={resolveChecksumHref(releaseAssets)}>
            {text(locale, 'checksum')}
          </a>
          <a href={resolveProvenanceHref(releaseAssets)}>
            {text(locale, 'provenance')}
          </a>
        </nav>
      </section>
    </main>
  );
}
