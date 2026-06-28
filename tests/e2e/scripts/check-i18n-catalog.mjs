import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { createI18n, parseLinoCatalogs } from 'lino-i18n';

const EXPECTED_LOCALES = ['en', 'ru', 'zh', 'hi'];
const REQUIRED_KEYS = [
  'buttons.reportIssue',
  'buttons.reportMissingRule',
  'buttons.sourceCode',
  'buttons.download',
  'buttons.exportMemory',
  'buttons.importMemory',
  'buttons.resetMemory',
  'buttons.diagnostics',
  'buttons.diagnosticsOn',
  'buttons.agent',
  'buttons.chat',
  'buttons.fullAuto',
  'buttons.demo',
  'buttons.demoOn',
  'buttons.openMenu',
  'buttons.closeMenu',
  'buttons.attachFiles',
  'buttons.composerMenu',
  'buttons.collapseSidebar',
  'buttons.expandSidebar',
  'buttons.expandOnlySection',
  'titles.reportIssue',
  'titles.sourceCode',
  'titles.download',
  'titles.exportMemory',
  'titles.importMemory',
  'titles.resetMemory',
  'titles.diagnosticsShow',
  'titles.diagnosticsHide',
  'titles.agentOn',
  'titles.agentOff',
  'titles.fullAuto',
  'titles.modeGroup',
  'titles.demoOn',
  'titles.demoOff',
  'titles.menuOpen',
  'titles.menuClose',
  'titles.composerMenu',
  'titles.resizeSidebar',
  'titles.collapseSidebar',
  'titles.expandSidebar',
  'titles.expandOnlySection',
  'composer.placeholder.chat',
  'composer.placeholder.agent',
  'composer.demoHint.before',
  'composer.demoHint.after',
  'composer.send',
  'composer.sending',
  'composer.attachments',
  'conversation.new',
  'conversation.empty',
  'conversation.deletedEmpty',
  'conversation.emptyTitle',
  'conversation.messageCount',
  'conversation.showDeleted',
  'conversation.delete',
  'conversation.deletePermanent',
  'conversation.purgeDeleted',
  'conversation.purgeDeletedTitle',
  'conversation.copyMarkdown',
  'conversation.copyMarkdownDone',
  'conversation.copyMarkdownTitle',
  'message.author.user',
  'message.copyCode',
  'message.copyCodeDone',
  'message.copyCodeTitle',
  'message.copyMarkdown',
  'message.copyMarkdownDone',
  'message.copyMarkdownTitle',
  'message.thinking',
  'message.thinkingExpand',
  'message.thinkingCollapse',
  'message.thinkingPrevious',
  'message.thinkingCurrent',
  'message.thinkingLanguage.en',
  'message.thinkingLanguage.ru',
  'message.thinkingLanguage.zh',
  'message.thinkingLanguage.hi',
  'message.thinkingLanguage.unknown',
  'message.thinkingRoute.reply',
  'message.thinkingRoute.greeting',
  'message.thinkingRoute.farewell',
  'message.thinkingRoute.unknown',
  'message.thinkingRoute.generic',
  'message.thinkingRule.selected',
  'message.thinkingRule.greeting',
  'message.thinkingRule.farewell',
  'message.thinkingRule.unknown',
  'message.thinkingStep.impulse',
  'message.thinkingStep.formalize',
  'message.thinkingStep.formalizeResolved',
  'message.thinkingStep.formalizeOpGreet',
  'message.thinkingStep.formalizeOpFarewell',
  'message.thinkingStep.formalizeOpExpress',
  'message.thinkingStep.formalizeOpCompute',
  'message.thinkingStep.formalizeOpDefine',
  'message.thinkingStep.formalizeOpLookup',
  'message.thinkingStep.formalizeOpSearch',
  'message.thinkingStep.formalizeOpProcedure',
  'message.thinkingStep.formalizeOpIdentify',
  'message.thinkingStep.detectLanguage',
  'message.thinkingStep.resolveResponseLanguage',
  'message.thinkingStep.clarifyFormalization',
  'message.thinkingStep.dispatchHandler',
  'message.thinkingStep.matchRule',
  'message.thinkingStep.invokeTool',
  'message.thinkingStep.fallback',
  'message.thinkingStep.userContext',
  'message.thinkingStep.deformalize',
  'message.thinkingStep.routeAttempt',
  'message.thinkingStep.coreferenceBinding',
  'message.thinkingStep.modifierDetection',
  'message.thinkingStep.ruleConstruction',
  'message.thinkingStep.ruleVerification',
  'message.thinkingStep.programPlan',
  'message.thinkingStep.desktopShell',
  'message.thinkingStep.httpChat',
  'message.thinkingStep.memory',
  'message.thinkingStep.agentPlan',
  'message.thinkingStep.agentSubstep',
  'message.thinkingStep.triggerButton',
  'message.thinkingStep.applyMessageCommand',
  'message.thinkingStep.triggerMessageAction',
  'message.thinkingStep.extractTerm',
  'message.thinkingStep.scanMemory',
  'message.thinkingStep.groupByConversation',
  'message.thinkingStep.generic',
  'message.thinkingStep.working',
  'message.thinkingStep.pendingReading',
  'message.thinkingStep.pendingFormalizing',
  'message.thinkingStep.pendingDispatching',
  'message.thinkingStep.pendingComposing',
  'message.thinkingStep.fallbackNormalize',
  'message.thinkingStep.fallbackIntent',
  'message.thinkingStep.fallbackRender',
  'message.thinkingStep.impulsePlain',
  'message.thinkingStep.formalizePlain',
  'message.thinkingStep.formalizeTuple',
  'message.thinkingStep.formalizeResolvedPlain',
  'message.thinkingStep.formalizeResolvedTuple',
  'message.thinkingStep.clarifyFormalizationPlain',
  'message.thinkingStep.dispatchHandlerPlain',
  'message.thinkingStep.matchRulePlain',
  'message.thinkingStep.routeAttemptPlain',
  'message.thinkingStep.compute',
  'message.thinkingStep.computePlain',
  'message.thinkingStep.computeEngine',
  'message.thinkingStep.computeEnginePlain',
  'message.thinkingStep.computeExpression',
  'message.thinkingStep.computeSteps',
  'message.thinkingStep.lookupFact',
  'message.thinkingStep.lookupFactPlain',
  'message.thinkingStep.invokeToolPlain',
  'message.thinkingStep.ruleVerificationPlain',
  'message.thinkingStep.policyRefusal',
  'message.thinkingStep.policyRefusalPlain',
  'message.thinkingStep.programPlanPlain',
  'message.thinkingStep.scanMemoryPlain',
  'message.thinkingStep.deformalizePlain',
  'message.thinkingStep.agentPlanPlain',
  'message.thinkingStep.userContextDefault',
  'message.diagnosticsSteps',
  'message.diagnosticsTools',
  'message.diagnosticsHttp',
  'message.diagnosticsHttpRequest',
  'message.diagnosticsHttpResponse',
  'message.diagnosticsHttpUnified',
  'message.diagnosticsHttpStatus',
  'message.diagnosticsHttpEmpty',
  'message.diagnosticsProviders',
  'message.diagnosticsProviderRow',
  'message.diagnosticsProviderOk',
  'message.diagnosticsProviderError',
  'message.toolInputs',
  'message.toolOutputs',
  'message.toolReasoning',
  'message.formalization',
  'message.formalizationSubjectVerbObject',
  'message.otherSources',
  'message.sourceCounts',
  'fetch.collapse',
  'fetch.expand',
  'fetch.fullscreen',
  'fetch.minimize',
  'fetch.openInNewTab',
  'fetch.frameTitle',
  'memory.exportTriggered',
  'memory.importTriggered',
  'memory.resetCancelled',
  'confirm.resetMemoryExportFirst',
  'confirm.resetMemory',
  'confirm.purgeDeletedExportFirst',
  'confirm.purgeDeleted',
  'confirm.deleteConversationPermanentExportFirst',
  'confirm.deleteConversationPermanent',
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
  'settings.followUpInitiative',
  'settings.userInitiative',
  'settings.assistantInitiative',
  'settings.variations',
  'settings.definitionFusion',
  'settings.definitionFusion.explicit',
  'settings.definitionFusion.auto',
  'settings.blueprintComposition',
  'settings.blueprintComposition.composed',
  'settings.blueprintComposition.documented',
  'settings.thinkingDetail',
  'settings.thinkingDetail.brief',
  'settings.thinkingDetail.standard',
  'settings.thinkingDetail.detailed',
  'settings.minMessageAnimation',
  'settings.animationImmediate',
  'settings.animationRelaxed',
  'settings.animationSeconds',
  'settings.experimentalOcr',
  'settings.experimentalOcr.warning',
  'settings.externalServices',
  'settings.externalServices.note',
  'settings.externalServiceWikihow',
  'settings.externalServiceStackExchange',
  'settings.externalServiceMediawikiFamily',
  'settings.externalServiceGithub',
  'settings.language',
  'settings.language.auto',
  'settings.responseLanguage',
  'settings.responseLanguage.lastMessage',
  'settings.responseLanguage.preferred',
  'settings.responseLanguage.ui',
  'settings.preferredLanguage',
  'settings.theme',
  'settings.theme.auto',
  'settings.theme.light',
  'settings.theme.dark',
  'settings.uiSkin',
  'settings.uiSkin.flat',
  'settings.uiSkin.glass',
  'settings.uiSkin.contrast',
  'settings.toolbarIconPack',
  'settings.toolbarIconPack.fontawesome',
  'settings.toolbarIconPack.materialSymbols',
  'settings.toolbarIconPack.bootstrapIcons',
  'settings.toolbarIconPack.ionicons',
  'settings.toolbarIconPack.remixIcon',
  'settings.toolbarIconPack.tablerIcons',
  'settings.toolbarIconPack.names',
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
  'settings.assistantName',
  'settings.assistantName.placeholder',
  'settings.location',
  'settings.location.placeholder',
  'settings.resetHeading',
  'settings.resetAll',
  'settings.resetOne',
  'settings.resetNone',
  'status.demoPlaying',
  'status.manual',
  'status.mode',
  'status.nextDialogIn',
  'status.memoryUnavailable',
  'status.memoryExported',
  'status.memoryImportedBundle',
  'status.memoryImportedEvents',
  'status.memoryReset',
  'status.deletedConversationsPurged',
  'status.conversationPurged',
  'status.migration',
  'status.exportFailed',
  'status.importFailed',
  'status.memoryResetFailed',
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
  // Issue #514 / #511: desktop tool permission UI and command-approval prose.
  // These were hardcoded in src/web/app.js; they must live in the catalog so the
  // permission panel, command approval, and shell messages translate per language.
  // The lino-i18n parser sets each tool's bare key to its `label` child, so the
  // `.label` variant is auto-allowed by isGeneratedLabelKey; `.description` is explicit.
  'permissions.tool.http_fetch',
  'permissions.tool.http_fetch.description',
  'permissions.tool.url_navigate',
  'permissions.tool.url_navigate.description',
  'permissions.tool.eval_js',
  'permissions.tool.eval_js.description',
  'permissions.tool.read_local_file',
  'permissions.tool.read_local_file.description',
  'permissions.tool.code_exec',
  'permissions.tool.code_exec.description',
  'permissions.tool.shell',
  'permissions.tool.shell.description',
  'permissions.panel.title',
  'permissions.panel.active',
  'permissions.panel.saved',
  'permissions.panel.rowLabel',
  'permissions.state.granted',
  'permissions.state.declined',
  'permissions.state.undecided',
  'permissions.action.grant',
  'permissions.action.decline',
  // Issue #541 (R9): one-click affordance on the permission panel — grant every
  // tool, opt in to Agent mode, and (if a task was deferred) run it.
  'permissions.action.grantAll',
  'permissions.action.grantAllAndRun',
  'permissions.toolCount',
  'permissions.command.title',
  'permissions.command.approve',
  'permissions.command.deny',
  'permissions.command.status.pending',
  'permissions.command.status.running',
  'permissions.command.status.approved',
  'permissions.command.status.denied',
  'permissions.onboarding.intro',
  'permissions.onboarding.perTool',
  'permissions.onboarding.modes',
  'permissions.message.shellRan',
  'permissions.message.shellNotRun',
  'permissions.message.shellNotGranted',
  'permissions.message.approvalPrompt',
  'permissions.message.commandDeclined',
  'permissions.message.noOutput',
  'permissions.message.reasonNoResult',
  'permissions.message.reasonRefused',
  // Issue #511: desktop Services panel labels, also moved out of hardcoded prose.
  'services.title',
  'services.telegram',
  'services.telegram.label',
  'services.server',
  'services.server.label',
  'services.agent',
  'services.agent.label',
  'services.dockerMissing',
  'services.installAgent',
  'services.installing',
  'services.start',
  'services.starting',
  'services.stop',
  'services.stopping',
  'services.state.ready',
  'services.state.running',
  'services.state.stopped',
  'services.state.needsToken',
  'services.state.dockerUnavailable',
  'services.state.error',
  'services.state.unknown',
  // Issue #548: desktop auto-update notification and user-triggered install UI.
  'updates.title',
  'updates.currentVersion',
  'updates.check',
  'updates.checking',
  'updates.update',
  'updates.updating',
  'updates.progress',
  'updates.state.idle',
  'updates.state.checking',
  'updates.state.available',
  'updates.state.notAvailable',
  'updates.state.downloading',
  'updates.state.downloaded',
  'updates.state.installing',
  'updates.state.disabled',
  'updates.state.error',

  'vscodeInstall.title',
  'vscodeInstall.summary',
  'vscodeInstall.install',
  'vscodeInstall.installing',
  'vscodeInstall.installed',
  'vscodeInstall.noCli',
  'vscodeInstall.noAsset',
  'vscodeInstall.lookupFailed',
  'vscodeInstall.downloadFailed',
  'vscodeInstall.installFailed',
  'vscodeInstall.error',
];

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, '../../..');
// The catalog is split across files so each stays under the Links Notation line
// limit (see scripts/check-file-size.rs). The loader (src/web/i18n.js) fetches
// each file and merges their per-locale keys, so this checker does the same.
const catalogFiles = [
  'src/web/i18n-catalog.lino',
  'src/web/i18n-catalog-permissions.lino',
];
const catalogTexts = catalogFiles.map((relativePath) =>
  fs.readFileSync(path.join(repoRoot, relativePath), 'utf8'),
);
const text = catalogTexts.join('\n');
const failures = [];

if (!text.includes('"""')) {
  failures.push('catalog must use lino-i18n multiline quoted strings');
}

if (!/\n  buttons\n    reportIssue /.test(text)) {
  failures.push('catalog must keep related messages in nested blocks');
}

const catalogs = new Map();
for (const catalogText of catalogTexts) {
  for (const { locale, translations } of parseLinoCatalogs(catalogText)) {
    const merged = catalogs.get(locale) || {};
    Object.assign(merged, translations);
    catalogs.set(locale, merged);
  }
}
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
  ['en', 'status.mode', 'Mode: Agent', { mode: 'Agent' }],
  // Issue #511/#514: desktop permission strings must resolve per UI language and
  // interpolate placeholders rather than render hardcoded English.
  ['en', 'permissions.toolCount', '0/6 tools granted', { granted: 0, total: 6 }],
  ['ru', 'permissions.toolCount', 'Предоставлено инструментов: 1/6', { granted: 1, total: 6 }],
  ['en', 'permissions.panel.title', 'Desktop tool permissions'],
  ['ru', 'permissions.panel.title', 'Разрешения инструментов рабочего стола'],
  ['zh', 'permissions.state.granted', '已授予'],
  ['hi', 'permissions.action.grant', 'प्रदान करें'],
  ['en', 'updates.state.available', 'Update 0.213.0 available', { version: '0.213.0' }],
  // Issue #554: the one-click VS Code extension install strings resolve per UI
  // language and interpolate the detected CLI name.
  ['en', 'vscodeInstall.installed', 'Installed into code. Reload VS Code to start using it.', { cli: 'code' }],
  ['ru', 'vscodeInstall.install', 'Установить в VS Code'],
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
