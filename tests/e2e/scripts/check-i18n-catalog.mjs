import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { createI18n, parseLinoCatalogs } from 'lino-i18n';

const EXPECTED_LOCALES = ['en', 'ru', 'zh', 'hi'];
const REQUIRED_KEYS = [
  'buttons.reportIssue',
  'buttons.reportMissingRule',
  'buttons.exportMemory',
  'buttons.importMemory',
  'buttons.diagnostics',
  'buttons.diagnosticsOn',
  'buttons.agent',
  'buttons.chat',
  'buttons.demo',
  'buttons.demoOn',
  'buttons.openMenu',
  'buttons.closeMenu',
  'buttons.attachFiles',
  'buttons.composerMenu',
  'titles.reportIssue',
  'titles.exportMemory',
  'titles.importMemory',
  'titles.diagnosticsShow',
  'titles.diagnosticsHide',
  'titles.agentOn',
  'titles.agentOff',
  'titles.demoOn',
  'titles.demoOff',
  'titles.menuOpen',
  'titles.menuClose',
  'titles.composerMenu',
  'titles.resizeSidebar',
  'composer.placeholder.chat',
  'composer.placeholder.agent',
  'composer.demoHint.before',
  'composer.demoHint.after',
  'composer.send',
  'composer.attachments',
  'conversation.new',
  'conversation.empty',
  'conversation.deletedEmpty',
  'conversation.emptyTitle',
  'conversation.messageCount',
  'conversation.showDeleted',
  'conversation.delete',
  'message.author.user',
  'message.thinking',
  'message.diagnosticsSteps',
  'message.diagnosticsTools',
  'message.toolInputs',
  'message.toolOutputs',
  'message.toolReasoning',
  'fetch.collapse',
  'fetch.expand',
  'fetch.fullscreen',
  'fetch.minimize',
  'fetch.openInNewTab',
  'fetch.frameTitle',
  'memory.exportTriggered',
  'memory.importTriggered',
  'sidebar.conversations',
  'sidebar.menu',
  'sidebar.examplePrompts',
  'sidebar.tools',
  'sidebar.trace',
  'sidebar.settings',
  'settings.ambiguity',
  'settings.moreQuestions',
  'settings.moreGuessing',
  'settings.temperature',
  'settings.deterministic',
  'settings.varied',
  'settings.variations',
  'settings.definitionFusion',
  'settings.definitionFusion.explicit',
  'settings.definitionFusion.auto',
  'settings.language',
  'settings.language.auto',
  'settings.theme',
  'settings.theme.auto',
  'settings.theme.light',
  'settings.theme.dark',
  'settings.uiSkin',
  'settings.uiSkin.flat',
  'settings.uiSkin.glass',
  'settings.uiSkin.contrast',
  'settings.chatStyle',
  'settings.chatStyle.cards',
  'settings.chatStyle.compact',
  'settings.chatStyle.bubbles',
  'settings.composerStyle',
  'settings.composerStyle.flat',
  'settings.composerStyle.glassSoft',
  'settings.composerStyle.glassClear',
  'settings.composerStyle.bubble',
  'settings.composerAction',
  'settings.composerAction.attach',
  'settings.composerAction.plus',
  'settings.location',
  'settings.location.placeholder',
  'status.demoPlaying',
  'status.manual',
  'status.nextDialogIn',
  'status.memoryUnavailable',
  'status.memoryExported',
  'status.memoryImportedBundle',
  'status.memoryImportedEvents',
  'status.migration',
  'status.exportFailed',
  'status.importFailed',
  'status.working',
  'toolMode.agent',
  'toolMode.thinking',
  'trace.model',
  'trace.mode',
  'trace.intent',
  'trace.data',
  'trace.seedFiles',
  'trace.toolsLoaded',
  'trace.conceptsLoaded',
];

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, '../../..');
const catalogPath = path.join(repoRoot, 'src/web/i18n-catalog.lino');
const text = fs.readFileSync(catalogPath, 'utf8');
const failures = [];

if (!text.includes('"""')) {
  failures.push('catalog must use lino-i18n multiline quoted strings');
}

if (!/\n  buttons\n    reportIssue /.test(text)) {
  failures.push('catalog must keep related messages in nested blocks');
}

const parsed = parseLinoCatalogs(text);
const catalogs = new Map(
  parsed.map(({ locale, translations }) => [locale, translations]),
);
const actualLocales = [...catalogs.keys()].sort();

for (const locale of EXPECTED_LOCALES) {
  if (!catalogs.has(locale)) {
    failures.push(`missing locale block: ${locale}`);
  }
}

for (const locale of actualLocales) {
  if (!EXPECTED_LOCALES.includes(locale)) {
    failures.push(`unexpected locale block: ${locale}`);
  }
}

const canonicalKeys = new Set(REQUIRED_KEYS);
const localesObject = {};
const isGeneratedLabelKey = (key) =>
  key.endsWith('.label') && canonicalKeys.has(key.slice(0, -'.label'.length));

for (const locale of EXPECTED_LOCALES) {
  const translations = catalogs.get(locale) || {};
  localesObject[locale] = translations;
  const keys = new Set(Object.keys(translations));

  for (const requiredKey of REQUIRED_KEYS) {
    if (!keys.has(requiredKey)) {
      failures.push(`${locale} is missing required key: ${requiredKey}`);
    }
  }

  for (const key of keys) {
    if (!canonicalKeys.has(key) && !isGeneratedLabelKey(key)) {
      failures.push(`${locale} has unexpected key: ${key}`);
    }
  }

  for (const [key, value] of Object.entries(translations)) {
    if (typeof value !== 'string' || value.trim().length === 0) {
      failures.push(`${locale}.${key} has an empty translation`);
    }
  }
}

const engine = createI18n({
  locales: localesObject,
  defaultLocale: 'en',
  fallback: ['en'],
});

const runtimeChecks = [
  ['en', 'buttons.reportIssue', 'Report issue'],
  ['ru', 'buttons.reportIssue', 'Сообщить о проблеме'],
  ['zh', 'buttons.reportIssue', '报告问题'],
  ['hi', 'buttons.reportIssue', 'समस्या रिपोर्ट करें'],
  ['zz', 'buttons.reportIssue', 'Report issue'],
  ['en', 'settings.language', 'Language'],
  ['en', 'status.nextDialogIn', 'Next dialog in 5s', { seconds: 5 }],
];

for (const [locale, key, expected, params = {}] of runtimeChecks) {
  const actual = engine.t(key, params, { locale, defaultValue: key });
  if (actual !== expected) {
    failures.push(
      `runtime check failed for ${locale}.${key}: expected ${JSON.stringify(
        expected,
      )}, got ${JSON.stringify(actual)}`,
    );
  }
}

if (failures.length > 0) {
  console.error('i18n catalog check failed:');
  for (const failure of failures) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}

console.log(
  `i18n catalog check passed for ${EXPECTED_LOCALES.length} locales and ${REQUIRED_KEYS.length} keys`,
);
