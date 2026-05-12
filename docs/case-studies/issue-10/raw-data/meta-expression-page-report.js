import { serializeLinksNotation } from '../src/index.js';
import { formatAppVersion, loadAppVersionInfo } from './app-version.js';

const repositoryUrl = 'https://github.com/link-assistant/meta-expression';
const defaultIssueUrl = `${repositoryUrl}/issues/new`;
const pageLabels = {
  analyse: 'Analyse',
  compare: 'Compare',
  check: 'Check',
  uniqueness: 'Uniqueness',
  formalize: 'Formalize',
  translate: 'Translate',
  preferences: 'Preferences',
};
const pageAliases = { 'fact-check': 'check', uniquness: 'uniqueness' };

export function setupPageIssueReporting(options = {}) {
  const links = [
    ...new Set([
      ...document.querySelectorAll('[data-issue-report-link]'),
      document.querySelector('#report-issue'),
    ]),
  ].filter(Boolean);
  const versionBadge = document.querySelector('#app-version');
  let versionInfo = {};

  const updateLinks = () => {
    const href = createCurrentPageIssueUrl({ ...options, versionInfo });
    for (const link of links) {
      link.href = href;
    }
  };

  for (const link of links) {
    link.href = defaultIssueUrl;
    link.addEventListener('pointerdown', updateLinks);
    link.addEventListener('focus', updateLinks);
    link.addEventListener('click', updateLinks);
  }

  loadAppVersionInfo().then((info) => {
    versionInfo = info;
    if (versionBadge) {
      versionBadge.textContent = formatAppVersion(info);
      versionBadge.title = createVersionTitle(info);
    }
    updateLinks();
  });

  updateLinks();
  return { updateLinks, getVersionInfo: () => versionInfo };
}

export function createCurrentPageIssueUrl(options = {}) {
  const pageId = options.getPageId?.() ?? pageFromLocation();
  const report = createCurrentPageReport(pageId, options);
  return createPageIssueReportUrl(report, options);
}

export function createPageIssueReportUrl(report, options = {}) {
  const params = new URLSearchParams({
    title: options.title ?? createIssueTitle(report),
    body: createPageIssueReport(report),
    labels: options.labels ?? 'bug',
  });
  return `${(options.repositoryUrl ?? repositoryUrl).replace(
    /\/$/,
    ''
  )}/issues/new?${params.toString()}`;
}

export function createPageIssueReport(report) {
  const lines = [];
  lines.push('## Environment', '');
  for (const [label, value] of Object.entries(report.environment ?? {})) {
    lines.push(`- **${label}**: ${formatReportValue(value)}`);
  }
  for (const section of report.sections ?? []) {
    appendSection(lines, section);
  }
  if (report.reproductionSteps?.length) {
    lines.push('', '## Reproduction Steps', '');
    for (const [index, step] of report.reproductionSteps.entries()) {
      lines.push(`${index + 1}. ${step}`);
    }
  }
  lines.push('', '## Description', '');
  lines.push('<!-- Please describe what looked wrong or incomplete. -->', '');
  return lines.join('\n');
}

function createCurrentPageReport(pageId, options) {
  const normalizedPageId = normalizePageId(pageId);
  const pageLabel = pageLabels[normalizedPageId] ?? pageLabels.analyse;
  return {
    pageId: normalizedPageId,
    pageLabel,
    summary: getPageSummary(normalizedPageId),
    environment: collectEnvironment(pageLabel, options),
    sections: collectPageSections(normalizedPageId, options),
    reproductionSteps: createReproductionSteps(pageLabel),
  };
}

function collectEnvironment(pageLabel, options) {
  const versionInfo = options.versionInfo ?? {};
  return {
    Page: pageLabel,
    Version: formatAppVersion(versionInfo),
    Commit: formatCommit(versionInfo.commitSha),
    Ref: versionInfo.ref,
    'Build time': versionInfo.buildTime,
    'Build source': versionInfo.source,
    URL: globalThis.location?.href,
    Locale: options.getLocale?.(),
    Theme: options.getTheme?.(),
    'User Agent': globalThis.navigator?.userAgent,
    Timestamp: new Date().toISOString(),
  };
}

function collectPageSections(pageId, options) {
  if (pageId === 'compare') {
    return collectCompareSections();
  }
  if (pageId === 'formalize') {
    return collectFormalizeSections(options);
  }
  if (pageId === 'check') {
    return collectCheckSections(options);
  }
  if (pageId === 'uniqueness') {
    return collectUniquenessSections(options);
  }
  if (pageId === 'translate') {
    return collectTranslateSections();
  }
  if (pageId === 'preferences') {
    return collectPreferencesSections(options);
  }
  return collectAnalyseSections(options);
}

function collectAnalyseSections(options) {
  const analysis = options.getAnalysis?.();
  if (!analysis) {
    return [
      { heading: 'Statement', code: valueOf('#statement-input') },
      {
        heading: 'Page State',
        lines: ['No analysis result is available yet.'],
      },
    ];
  }
  const result = analysis.result;
  return [
    { heading: 'Statement', code: analysis.statement.value.text },
    {
      heading: 'Selected Interpretation',
      lines: [
        `- **Id**: ${analysis.selectedInterpretation.id}`,
        `- **Kind**: ${analysis.selectedInterpretation.kind}`,
        `- **Paraphrase**: ${analysis.selectedInterpretation.paraphrase}`,
        `- **Strategy**: ${options.getStrategyId?.() ?? ''}`,
      ],
    },
    {
      heading: 'Result',
      lines: [
        `- **Value**: ${result.value}`,
        `- **Correctness**: ${textOf('#correctness-value')}`,
        `- **Confidence**: ${textOf('#confidence-value')}`,
        `- **Level**: ${textOf('#formalization-level')} ${textOf(
          '#formalization-level-name'
        )}`,
        `- **Explanation**: ${result.explanation}`,
      ],
    },
    listSection(
      'Candidate Interpretations',
      analysis.interpretations,
      (item) => `- **${item.id}** (${item.kind}): ${item.paraphrase}`
    ),
    listSection(
      'Alternatives',
      analysis.alternatives,
      (item) => `- ${item.text}: ${item.reason}`
    ),
    listSection('Dependencies', analysis.dependencies, (item) => `- ${item}`),
    listSection(
      'Definitions',
      analysis.definitions,
      (item) => `- **${item.phrase}**: ${item.label} (${item.wikidataId})`
    ),
    listSection('Confirmations', analysis.confirmations, evidenceLine),
    listSection('Refutations', analysis.refutations, evidenceLine),
    {
      heading: 'Reasoning Trace',
      lines: analysis.linksNetwork.links.map(
        (link) => `- **${link.role}**: ${reportValue(link.value)}`
      ),
    },
    {
      heading: 'Links Notation',
      code: serializeLinksNotation(analysis.linksNetwork),
    },
  ];
}

function collectCompareSections() {
  const claims = [
    ...document.querySelectorAll('#compare-rows .compare-row'),
  ].map((row, index) => {
    const claim = row.querySelector('.compare-claim')?.value.trim() ?? '';
    const correctness = row.querySelector('.compare-correctness')?.textContent;
    const confidence = row.querySelector('.compare-confidence')?.textContent;
    return `${index + 1}. ${claim || '(empty)'} | correctness ${
      correctness || '-'
    } | confidence ${confidence || '-'}`;
  });
  return [{ heading: 'Compared Claims', lines: claims }];
}

function collectFormalizeSections(options) {
  const result = options.getFormalizeResult?.();
  const sources = options.getFormalizeSourcesSpec?.() ?? '';
  const selectedContextIds = checkedLabels('#formalize-big-contexts input');
  return [
    { heading: 'Text', code: valueOf('#formalize-input') },
    {
      heading: 'Options',
      lines: [
        `- **Max n-gram size**: ${valueOf('#formalize-ngram-size')}`,
        `- **Link target**: ${options.getFormalizeLinkTargetMode?.() ?? ''}`,
        `- **Sources**: ${sources}`,
        `- **Interpretation display**: ${
          options.getInterpretationDisplayMode?.() ?? ''
        }`,
        `- **Selected big contexts**: ${selectedContextIds.join(', ')}`,
      ],
    },
    { heading: 'Status', lines: [`- ${textOf('#formalize-status')}`] },
    { heading: 'Rendered Result', lines: linesFrom('#formalize-output') },
    { heading: 'Contexts', lines: linesFrom('#formalize-contexts > *') },
    {
      heading: 'Big Contexts',
      lines: linesFrom('#formalize-big-contexts > *'),
    },
    {
      heading: 'Top Interpretations',
      lines: linesFrom('#formalize-interpretations > *'),
    },
    {
      heading: 'Markdown',
      code: result?.markdown ?? textOf('#formalize-markdown'),
    },
    {
      heading: 'Links Notation',
      code: result?.linksNotation ?? textOf('#formalize-lino'),
    },
    { heading: 'Overrides', code: valueOf('#formalize-overrides') },
  ];
}

function collectCheckSections(options) {
  const result = options.getCheckResult?.();
  return [
    { heading: 'Text', code: valueOf('#check-input') },
    {
      heading: 'Options',
      lines: [
        `- **Live Wikimedia evidence**: ${
          document.querySelector('#check-live')?.checked ? 'on' : 'off'
        }`,
      ],
    },
    { heading: 'Status', lines: [`- ${textOf('#check-status')}`] },
    { heading: 'Summary', lines: linesFrom('#check-summary > *') },
    { heading: 'Checked Result', lines: linesFrom('#check-output') },
    {
      heading: 'Markdown',
      code: result?.markdown ?? textOf('#check-markdown'),
    },
    {
      heading: 'Links Notation',
      code: result?.linksNotation ?? textOf('#check-lino'),
    },
  ];
}

function collectUniquenessSections(options) {
  const result = options.getUniquenessResult?.();
  return [
    { heading: 'Text', code: valueOf('#uniqueness-input') },
    { heading: 'Status', lines: [`- ${textOf('#uniqueness-status')}`] },
    { heading: 'Summary', lines: linesFrom('#uniqueness-summary > *') },
    { heading: 'Rendered Result', lines: linesFrom('#uniqueness-output') },
    { heading: 'Matches', lines: linesFrom('#uniqueness-matches > *') },
    {
      heading: 'Markdown',
      code: result?.markdown ?? textOf('#uniqueness-markdown'),
    },
    {
      heading: 'Links Notation',
      code: result?.linksNotation ?? textOf('#uniqueness-lino'),
    },
  ];
}

function collectTranslateSections() {
  return [
    { heading: 'Text', code: valueOf('#translate-input') },
    {
      heading: 'Options',
      lines: [
        `- **From**: ${valueOf('#translate-source-language')}`,
        `- **To**: ${valueOf('#translate-target-language')}`,
      ],
    },
    { heading: 'Status', lines: [`- ${textOf('#translate-status')}`] },
    { heading: 'Formalized Input', lines: linesFrom('#translate-formalized') },
    { heading: 'Translated Result', lines: linesFrom('#translate-output') },
    { heading: 'Questions', lines: linesFrom('#translate-questions > *') },
    { heading: 'Markdown', code: textOf('#translate-markdown') },
    { heading: 'Links Notation', code: textOf('#translate-lino') },
    { heading: 'Translation CST', code: textOf('#translate-cst') },
    { heading: 'Translation Steps', lines: linesFrom('#translate-steps > *') },
  ];
}

function collectPreferencesSections(options) {
  const profile = options.getPreferenceProfile?.();
  return [
    { heading: 'Worldview', lines: linesFrom('#preferences-worldview label') },
    { heading: 'Religions', lines: linesFrom('#preferences-religions label') },
    { heading: 'Context', lines: linesFrom('#preferences-contexts label') },
    { heading: 'Links Notation', code: valueOf('#preferences-lino') },
    {
      heading: 'Profile JSON',
      code: profile ? JSON.stringify(profile, null, 2) : '',
    },
  ];
}

function appendSection(lines, section) {
  lines.push('', `## ${section.heading}`, '');
  if (section.lines?.length) {
    lines.push(...section.lines.map((line) => line || '- (empty)'));
  }
  if (section.code !== undefined) {
    lines.push('```', section.code || '', '```');
  }
}

function createIssueTitle(report) {
  const summary = report.summary ? `: ${report.summary}` : '';
  return `Issue on ${report.pageLabel} page${summary}`.slice(0, 120);
}

function getPageSummary(pageId) {
  if (pageId === 'formalize') {
    return shorten(valueOf('#formalize-input'));
  }
  if (pageId === 'translate') {
    return shorten(valueOf('#translate-input'));
  }
  if (pageId === 'check') {
    return shorten(valueOf('#check-input'));
  }
  if (pageId === 'uniqueness') {
    return shorten(valueOf('#uniqueness-input'));
  }
  if (pageId === 'compare') {
    return shorten(valueOf('#compare-rows .compare-claim'));
  }
  return pageId === 'preferences'
    ? 'preferences'
    : shorten(valueOf('#statement-input'));
}

function createReproductionSteps(pageLabel) {
  return [
    `Open ${globalThis.location?.href ?? 'the web app'}`,
    `Switch to the ${pageLabel} page`,
    'Use the page until the issue occurs',
    'Click Report Issue',
  ];
}

function listSection(heading, items, formatter) {
  return {
    heading,
    lines: items?.length ? items.map(formatter) : ['None.'],
  };
}

function evidenceLine(item) {
  return `- ${item.sourceType}: ${item.quote}`;
}

function reportValue(value) {
  if (value === null || value === undefined) {
    return '';
  }
  if (typeof value !== 'object') {
    return String(value);
  }
  return value.text ?? value.paraphrase ?? value.claim ?? value.kind ?? '';
}

function linesFrom(selector) {
  const nodes = [...document.querySelectorAll(selector)];
  if (nodes.length === 0) {
    const single = document.querySelector(selector);
    return single ? [`- ${normalizeText(single.textContent)}`] : ['None.'];
  }
  return nodes.map((node) => `- ${normalizeText(node.textContent)}`);
}

function checkedLabels(selector) {
  return [...document.querySelectorAll(selector)]
    .filter((input) => input.checked)
    .map((input) => normalizeText(input.parentElement?.textContent));
}

function valueOf(selector) {
  const element = document.querySelector(selector);
  return element?.value ?? element?.textContent ?? '';
}

function textOf(selector) {
  return normalizeText(document.querySelector(selector)?.textContent);
}

function normalizeText(value) {
  return String(value ?? '')
    .replace(/\s+/g, ' ')
    .trim();
}

function shorten(value) {
  const normalized = normalizeText(value);
  return normalized.length > 50 ? `${normalized.slice(0, 50)}...` : normalized;
}

function formatCommit(sha) {
  return sha
    ? `[${sha.slice(0, 7)}](${repositoryUrl}/commit/${sha})`
    : 'unknown';
}

function formatReportValue(value) {
  if (value === undefined || value === null || value === '') {
    return 'unknown';
  }
  return String(value);
}

function createVersionTitle(info) {
  const parts = [
    info.packageVersion ? `version ${info.packageVersion}` : '',
    info.commitSha ? `commit ${info.commitSha}` : '',
    info.buildTime ? `built ${info.buildTime}` : '',
  ].filter(Boolean);
  return parts.join(' | ') || 'Version unknown';
}

function pageFromLocation() {
  const fragment = globalThis.location?.hash?.replace('#/', '') ?? '';
  return pageLabels[normalizePageId(fragment)]
    ? normalizePageId(fragment)
    : 'analyse';
}

function normalizePageId(pageId) {
  return pageAliases[pageId] ?? pageId;
}
