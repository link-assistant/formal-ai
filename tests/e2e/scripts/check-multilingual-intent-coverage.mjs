// CI guard for language coverage in symbolic conversational intents.
//
// This intentionally checks seed files directly, not only runtime behavior, so
// reviewers get a precise failure when a new phrase is added for one language
// without the matching supported-language matrix entries.

import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import { fileURLToPath } from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, '../../..');

function readRepoFile(relativePath) {
  return fs.readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

function parseSupportedLanguages() {
  const agentInfo = readRepoFile('data/seed/agent-info.lino');
  const match = agentInfo.match(/field "supported_languages"\s*\n\s+value "([^"]+)"/);
  if (!match) {
    throw new Error('data/seed/agent-info.lino is missing supported_languages');
  }
  return match[1].split('|').filter(Boolean);
}

function parseIntentRouting() {
  const routes = new Map();
  let current = null;

  for (const line of readRepoFile('data/seed/intent-routing.lino').split(/\r?\n/)) {
    const intent = line.match(/^  intent "([^"]+)"/);
    if (intent) {
      current = {
        id: intent[1],
        slug: '',
        keywords: [],
        phrases: [],
        tokens: [],
        combos: [],
      };
      routes.set(current.id, current);
      continue;
    }

    if (!current) continue;

    const slug = line.match(/^    slug "([^"]+)"/);
    if (slug) {
      current.slug = slug[1];
      continue;
    }

    const entry = line.match(/^    (keyword|phrase|token|combo) "([^"]+)"/);
    if (entry) {
      const collection = `${entry[1]}s`;
      current[collection].push(entry[2]);
    }
  }

  return routes;
}

function parsePromptPatterns() {
  const patterns = [];
  let current = null;

  for (const line of readRepoFile('data/seed/prompt-patterns.lino').split(/\r?\n/)) {
    const pattern = line.match(/^  pattern "([^"]+)"/);
    if (pattern) {
      if (current) patterns.push(current);
      current = { id: pattern[1] };
      continue;
    }

    if (!current) continue;

    const field = line.match(/^    ([a-z_]+) "([^"]*)"/);
    if (field) {
      current[field[1]] = field[2];
    }
  }

  if (current) patterns.push(current);
  return patterns;
}

function parseResponseRecords() {
  const responses = [];
  let current = null;

  for (const line of readRepoFile('data/seed/multilingual-responses.lino').split(/\r?\n/)) {
    const response = line.match(/^  response "([^"]+)"/);
    if (response) {
      if (current) responses.push(current);
      current = { id: response[1] };
      continue;
    }

    if (!current) continue;

    const field = line.match(/^    ([a-z_]+) "([^"]*)"/);
    if (field && field[1] !== 'variant') {
      current[field[1]] = field[2];
    }
  }

  if (current) responses.push(current);
  return responses;
}

function parseConceptRecords() {
  const concepts = [];
  let current = null;
  let localized = null;

  for (const line of readRepoFile('data/seed/concepts.lino').split(/\r?\n/)) {
    const concept = line.match(/^(concept_[a-z0-9_]+)$/);
    if (concept) {
      if (current) concepts.push(current);
      current = { id: concept[1], localized: [] };
      localized = null;
      continue;
    }

    if (!current) continue;

    const localizedHeader = line.match(/^  localized "([^"]+)"/);
    if (localizedHeader) {
      localized = { language: localizedHeader[1] };
      current.localized.push(localized);
      continue;
    }

    const localizedField = line.match(/^    ([a-z_]+) "([^"]*)"/);
    if (localized && localizedField) {
      localized[localizedField[1]] = localizedField[2];
      continue;
    }

    const field = line.match(/^  ([a-z_]+) "([^"]*)"/);
    if (field) {
      current[field[1]] = field[2];
      localized = null;
    }
  }

  if (current) concepts.push(current);
  return concepts;
}

function parseToolRecords() {
  const tools = [];
  let current = null;
  let localized = null;

  for (const line of readRepoFile('data/seed/tools.lino').split(/\r?\n/)) {
    const tool = line.match(/^  tool "([^"]+)"/);
    if (tool) {
      if (current) tools.push(current);
      current = { id: tool[1], localized: [] };
      localized = null;
      continue;
    }

    if (!current) continue;

    const localizedHeader = line.match(/^    localized "([^"]+)"/);
    if (localizedHeader) {
      localized = { language: localizedHeader[1] };
      current.localized.push(localized);
      continue;
    }

    const localizedField = line.match(/^      ([a-z_]+) "([^"]*)"/);
    if (localized && localizedField) {
      localized[localizedField[1]] = localizedField[2];
      continue;
    }

    const field = line.match(/^    ([a-z_]+) "([^"]*)"/);
    if (field) {
      current[field[1]] = field[2];
      localized = null;
    }
  }

  if (current) tools.push(current);
  return tools;
}

function parseContextRecords() {
  const contexts = [];
  let current = null;
  let label = null;

  for (const line of readRepoFile('data/seed/concept-contexts.lino').split(/\r?\n/)) {
    const context = line.match(/^  context "([^"]+)"/);
    if (context) {
      if (current) contexts.push(current);
      current = { id: context[1], labels: [] };
      label = null;
      continue;
    }

    if (!current) continue;

    const labelHeader = line.match(/^    label "([^"]+)"/);
    if (labelHeader) {
      label = { language: labelHeader[1] };
      current.labels.push(label);
      continue;
    }

    const labelText = line.match(/^      text "([^"]*)"/);
    if (label && labelText) {
      label.text = labelText[1];
      continue;
    }

    if (line.match(/^    [a-z_]+ /) || line.match(/^  [a-z_]+ /)) {
      label = null;
    }
  }

  if (current) contexts.push(current);
  return contexts;
}

function parseFeatureCapabilitySlugs() {
  return [
    ...readRepoFile('src/solver_handlers/feature_capability.rs').matchAll(
      /slug:\s*"([^"]+)"/g,
    ),
  ].map((match) => match[1]);
}

function parseFeatureCapabilityTestMatrix() {
  const matrix = new Map();
  const source = readRepoFile('tests/unit/specification/capabilities.rs');
  const cases = source.matchAll(
    /FeatureCapabilityLanguageCase\s*\{[\s\S]*?feature:\s*"([^"]+)"[\s\S]*?language:\s*"([^"]+)"/g,
  );
  for (const match of cases) {
    const feature = match[1];
    const language = match[2];
    if (!matrix.has(feature)) matrix.set(feature, new Set());
    matrix.get(feature).add(language);
  }
  return matrix;
}

function parseBrowserTranslationRegistry() {
  const source = readRepoFile('src/web/formal_ai_worker.js');
  const match = source.match(
    /const TRANSLATION_MEANING_REGISTRY = (\[[\s\S]*?\n\]);/,
  );
  if (!match) {
    throw new Error('src/web/formal_ai_worker.js is missing TRANSLATION_MEANING_REGISTRY');
  }
  return vm.runInNewContext(`(${match[1]})`);
}

const supportedLanguages = parseSupportedLanguages();
const routes = parseIntentRouting();
const patterns = parsePromptPatterns();
const responses = parseResponseRecords();
const concepts = parseConceptRecords();
const tools = parseToolRecords();
const contextRecords = parseContextRecords();
const featureCapabilitySlugs = parseFeatureCapabilitySlugs();
const featureCapabilityTestMatrix = parseFeatureCapabilityTestMatrix();
const browserTranslationRegistry = parseBrowserTranslationRegistry();
const errors = [];

function assert(condition, message) {
  if (!condition) errors.push(message);
}

function assertMatrixMatchesSupportedLanguages(name, matrix) {
  const matrixLanguages = Object.keys(matrix).sort();
  const supported = [...supportedLanguages].sort();
  assert(
    matrixLanguages.join('|') === supported.join('|'),
    `${name} must cover every supported language: expected ${supported.join(', ')}, got ${matrixLanguages.join(', ')}`,
  );
}

function assertBalancedLanguageCaseCounts(name, matrix) {
  const counts = Object.entries(matrix).map(([language, entries]) => [
    language,
    entries.length,
  ]);
  const expected = counts[0]?.[1] ?? 0;
  for (const [language, count] of counts) {
    assert(
      count === expected,
      `${name} must add the same number of cases for every supported language: expected ${expected} for ${language}, got ${count}`,
    );
  }
}

function assertPromptPatternCoverageGroups(intent) {
  const intentPatterns = patterns.filter((pattern) => pattern.intent === intent);
  assert(intentPatterns.length > 0, `prompt-patterns.lino must define patterns for ${intent}`);

  const groups = new Map();
  for (const pattern of intentPatterns) {
    assert(
      pattern.coverage_group,
      `prompt-patterns.lino ${pattern.id} must define coverage_group so ${intent} additions stay multilingual`,
    );
    if (!pattern.coverage_group) continue;
    if (!groups.has(pattern.coverage_group)) groups.set(pattern.coverage_group, {});
    const group = groups.get(pattern.coverage_group);
    assert(
      !group[pattern.language],
      `prompt-patterns.lino ${intent} coverage_group ${pattern.coverage_group} must not duplicate ${pattern.language}`,
    );
    group[pattern.language] = pattern;
  }

  for (const [group, matrix] of groups.entries()) {
    assertMatrixMatchesSupportedLanguages(
      `prompt-patterns.lino ${intent} coverage_group ${group}`,
      matrix,
    );
  }
}

const howAreYouGreetingPhrases = {
  en: ['how are you', 'how are you doing'],
  ru: ['как дела', 'как твои дела'],
  hi: ['कैसे हो', 'आप कैसे हैं'],
  zh: ['你好吗', '你怎么样'],
};

assertMatrixMatchesSupportedLanguages('howAreYouGreetingPhrases', howAreYouGreetingPhrases);

const greetingRoute = routes.get('intent_greeting');
assert(greetingRoute, 'intent-routing.lino must define intent_greeting');

for (const [language, phrases] of Object.entries(howAreYouGreetingPhrases)) {
  for (const phrase of phrases) {
    assert(
      greetingRoute?.phrases.includes(phrase),
      `intent_greeting must route ${language} how-are-you phrase ${JSON.stringify(phrase)}`,
    );
    assert(
      patterns.some(
        (pattern) =>
          pattern.intent === 'greeting' &&
          pattern.language === language &&
          pattern.kind === 'phrase' &&
          pattern.text === phrase,
      ),
      `prompt-patterns.lino must document ${language} greeting phrase ${JSON.stringify(phrase)}`,
    );
  }
}

const testStatusPatterns = {
  en: [
    { kind: 'keyword', text: 'test' },
    { kind: 'phrase', text: 'test passed' },
    { kind: 'phrase', text: 'i am here' },
  ],
  ru: [
    { kind: 'keyword', text: 'тест' },
    { kind: 'phrase', text: 'тест пройден' },
    { kind: 'phrase', text: 'я здесь' },
  ],
  hi: [
    { kind: 'keyword', text: 'टेस्ट' },
    { kind: 'phrase', text: 'परीक्षण सफल रहा' },
    { kind: 'phrase', text: 'मैं यहाँ हूँ' },
  ],
  zh: [
    { kind: 'keyword', text: '测试' },
    { kind: 'phrase', text: '测试通过' },
    { kind: 'phrase', text: '我在这里' },
  ],
};

assertMatrixMatchesSupportedLanguages('testStatusPatterns', testStatusPatterns);

const testStatusRoute = routes.get('intent_test_status');
assert(testStatusRoute, 'intent-routing.lino must define intent_test_status');

for (const [language, entries] of Object.entries(testStatusPatterns)) {
  for (const entry of entries) {
    const routeCollection = entry.kind === 'keyword' ? 'keywords' : 'phrases';
    assert(
      testStatusRoute?.[routeCollection].includes(entry.text),
      `intent_test_status must route ${language} ${entry.kind} ${JSON.stringify(entry.text)}`,
    );
    assert(
      patterns.some(
        (pattern) =>
          pattern.intent === 'test_status' &&
          pattern.language === language &&
          pattern.kind === entry.kind &&
          pattern.text === entry.text,
      ),
      `prompt-patterns.lino must document ${language} test_status ${entry.kind} ${JSON.stringify(entry.text)}`,
    );
  }
}

const behaviorRulesListPatterns = {
  en: ['show behavior rules', 'show list of your rules', 'list your rules'],
  ru: ['покажи правила поведения', 'покажи список своих правил', 'перечисли свои правила'],
  hi: ['व्यवहार के नियम सूचीबद्ध करें', 'अपने नियमों की सूची दिखाओ', 'अपने नियम गिनाओ'],
  zh: ['列出行为规则', '显示你的规则列表', '列出你的规则'],
};

assertMatrixMatchesSupportedLanguages('behaviorRulesListPatterns', behaviorRulesListPatterns);
assertPromptPatternCoverageGroups('behavior_rules_list');

for (const [language, phrases] of Object.entries(behaviorRulesListPatterns)) {
  for (const phrase of phrases) {
    assert(
      patterns.some(
        (pattern) =>
          pattern.intent === 'behavior_rules_list' &&
          pattern.language === language &&
          pattern.kind === 'phrase' &&
          pattern.text === phrase,
      ),
      `prompt-patterns.lino must document ${language} behavior_rules_list phrase ${JSON.stringify(phrase)}`,
    );
  }
}

const wikipediaArticleQuestionCases = {
  en: [
    {
      prompt: 'does wikipedia have an article about Agreement (linguistics)',
      expectedTitle: 'Agreement (linguistics)',
      coverageGroup: 'exact_title',
    },
    {
      prompt: 'agreement in a sentence - is there a wikipedia article',
      expectedTitle: 'Agreement (linguistics)',
      coverageGroup: 'grammar_context',
    },
  ],
  ru: [
    {
      prompt: 'есть ли в википедии статья о Согласование (грамматика)',
      expectedTitle: 'Согласование (грамматика)',
      coverageGroup: 'exact_title',
    },
    {
      prompt: 'согласованность в предложении - есть такая статья в википедии',
      expectedTitle: 'Согласование (грамматика)',
      coverageGroup: 'grammar_context',
    },
  ],
  hi: [
    {
      prompt: 'क्या विकिपीडिया पर व्याकरणिक सहमति लेख है',
      expectedTitle: 'व्याकरणिक सहमति',
      coverageGroup: 'exact_title',
    },
    {
      prompt: 'वाक्य में सहमति - क्या विकिपीडिया पर ऐसा लेख है',
      expectedTitle: 'व्याकरणिक सहमति',
      coverageGroup: 'grammar_context',
    },
  ],
  zh: [
    {
      prompt: '维基百科有一致 (语言学)条目吗',
      expectedTitle: '一致 (语言学)',
      coverageGroup: 'exact_title',
    },
    {
      prompt: '句子中的一致 - 维基百科有这样的条目吗',
      expectedTitle: '一致 (语言学)',
      coverageGroup: 'grammar_context',
    },
  ],
};

assertMatrixMatchesSupportedLanguages(
  'wikipediaArticleQuestionCases',
  wikipediaArticleQuestionCases,
);
assertBalancedLanguageCaseCounts(
  'wikipediaArticleQuestionCases',
  wikipediaArticleQuestionCases,
);
assertPromptPatternCoverageGroups('wikipedia_article_question');

{
  const browserMultilingualTests = readRepoFile('tests/e2e/tests/multilingual.spec.js');
  for (const [language, entries] of Object.entries(wikipediaArticleQuestionCases)) {
    for (const entry of entries) {
      assert(
        patterns.some(
          (pattern) =>
            pattern.intent === 'wikipedia_article_question' &&
            pattern.language === language &&
            pattern.kind === 'phrase' &&
            pattern.coverage_group === entry.coverageGroup &&
            pattern.text === entry.prompt,
        ),
        `prompt-patterns.lino must document ${language} wikipedia_article_question ${entry.coverageGroup} prompt ${JSON.stringify(entry.prompt)}`,
      );
      assert(
        browserMultilingualTests.includes(entry.prompt) &&
          browserMultilingualTests.includes(entry.expectedTitle),
        `tests/e2e/tests/multilingual.spec.js must cover ${language} wikipedia_article_question prompt ${JSON.stringify(entry.prompt)} and expected title ${JSON.stringify(entry.expectedTitle)}`,
      );
    }
  }
}

const definitionStyleDisambiguationCases = {
  en: [
    {
      prompt: 'What is creature?',
      expectedTitle: 'Creature',
      expectedText: 'Creature — a living being or organism.',
      expectedHost: 'en.wikipedia.org',
      rejectedText: 'Animalia',
    },
  ],
  ru: [
    {
      prompt: 'Что такое существо?',
      expectedTitle: 'Существо',
      expectedText: 'Существо — живой организм, живая особь, животное, человек.',
      expectedHost: 'ru.wikipedia.org',
      rejectedText: 'Animalia',
    },
  ],
  hi: [
    {
      prompt: 'प्राणी क्या है?',
      expectedTitle: 'प्राणी',
      expectedText: 'प्राणी — जीवित जीव या व्यक्ति।',
      expectedHost: 'hi.wikipedia.org',
      rejectedText: 'Animalia',
    },
  ],
  zh: [
    {
      prompt: '生物是什么?',
      expectedTitle: '生物',
      expectedText: '生物 — 有生命的个体或有机体。',
      expectedHost: 'zh.wikipedia.org',
      rejectedText: 'Animalia',
    },
  ],
};

assertMatrixMatchesSupportedLanguages(
  'definitionStyleDisambiguationCases',
  definitionStyleDisambiguationCases,
);
assertBalancedLanguageCaseCounts(
  'definitionStyleDisambiguationCases',
  definitionStyleDisambiguationCases,
);

{
  const browserMultilingualTests = readRepoFile('tests/e2e/tests/multilingual.spec.js');
  for (const [language, entries] of Object.entries(definitionStyleDisambiguationCases)) {
    for (const entry of entries) {
      assert(
        browserMultilingualTests.includes(entry.prompt) &&
          browserMultilingualTests.includes(entry.expectedTitle) &&
          browserMultilingualTests.includes(entry.expectedText) &&
          browserMultilingualTests.includes(entry.expectedHost) &&
          browserMultilingualTests.includes(entry.rejectedText),
        `tests/e2e/tests/multilingual.spec.js must cover ${language} definition-style disambiguation prompt ${JSON.stringify(entry.prompt)} with expected Wikipedia title ${JSON.stringify(entry.expectedTitle)} before Wikidata fallback`,
      );
    }
  }
}

const uiLanguageCommandCases = {
  en: [{ prompt: 'set ui language to english', expectedValue: 'en' }],
  ru: [{ prompt: 'переключи язык на русский', expectedValue: 'ru' }],
  hi: [{ prompt: 'भाषा हिंदी सेट करें', expectedValue: 'hi' }],
  zh: [{ prompt: '设置界面语言为中文', expectedValue: 'zh' }],
};

assertMatrixMatchesSupportedLanguages(
  'uiLanguageCommandCases',
  uiLanguageCommandCases,
);
assertBalancedLanguageCaseCounts(
  'uiLanguageCommandCases',
  uiLanguageCommandCases,
);

{
  const browserDemoTests = readRepoFile('tests/e2e/tests/demo.spec.js');
  for (const [language, entries] of Object.entries(uiLanguageCommandCases)) {
    for (const entry of entries) {
      assert(
        browserDemoTests.includes(entry.prompt) &&
          browserDemoTests.includes(entry.expectedValue),
        `tests/e2e/tests/demo.spec.js must cover ${language} UI-language command ${JSON.stringify(entry.prompt)} -> ${entry.expectedValue}`,
      );
    }
  }
}

const webSearchSourceMarkerCases = {
  en: [{ prompt: 'Find apple on the internet', query: 'apple' }],
  ru: [{ prompt: 'Найди яблоко в интернете', query: 'яблоко' }],
  hi: [{ prompt: 'सेब के बारे में इंटरनेट पर खोजो', query: 'सेब' }],
  zh: [{ prompt: '查找苹果网上信息', query: '苹果' }],
};

assertMatrixMatchesSupportedLanguages(
  'webSearchSourceMarkerCases',
  webSearchSourceMarkerCases,
);
assertBalancedLanguageCaseCounts(
  'webSearchSourceMarkerCases',
  webSearchSourceMarkerCases,
);

for (const [language, entries] of Object.entries(webSearchSourceMarkerCases)) {
  const rustWebRequestTests = readRepoFile('tests/unit/web_requests.rs');
  const browserSearchTests = readRepoFile('tests/e2e/tests/issue-153.spec.js');
  for (const entry of entries) {
    assert(
      entry.prompt.trim() && entry.query.trim(),
      `webSearchSourceMarkerCases ${language} entries must define prompt and query`,
    );
    assert(
      rustWebRequestTests.includes(entry.prompt) &&
        rustWebRequestTests.includes(entry.query),
      `tests/unit/web_requests.rs must cover ${language} web-search source-marker prompt ${JSON.stringify(entry.prompt)}`,
    );
    assert(
      browserSearchTests.includes(entry.prompt) &&
        browserSearchTests.includes(entry.query),
      `tests/e2e/tests/issue-153.spec.js must cover ${language} web-search source-marker prompt ${JSON.stringify(entry.prompt)}`,
    );
  }
}

const webSearchEnumerationResearchCases = {
  en: [
    {
      prompt: 'list all genshin characters with off-field DMG',
      rustQuery: 'genshin characters with off field dmg',
      browserRequest: 'genshin characters with off-field DMG',
    },
  ],
  ru: [
    {
      prompt: 'перечисли всех персонажей genshin с уроном вне поля',
      rustQuery: 'персонажей genshin с уроном вне поля',
      browserRequest: 'персонажей genshin с уроном вне поля',
    },
  ],
  hi: [
    {
      prompt: 'सभी Genshin पात्र जिनके पास off-field DMG है',
      rustQuery: 'genshin पात्र जिनके पास off field dmg है',
      browserRequest: 'Genshin पात्र जिनके पास off-field DMG है',
    },
  ],
  zh: [
    {
      prompt: '列出所有 Genshin 角色 具有 off-field DMG',
      rustQuery: 'genshin 角色 具有 off field dmg',
      browserRequest: 'Genshin 角色 具有 off-field DMG',
    },
  ],
};

assertMatrixMatchesSupportedLanguages(
  'webSearchEnumerationResearchCases',
  webSearchEnumerationResearchCases,
);
assertBalancedLanguageCaseCounts(
  'webSearchEnumerationResearchCases',
  webSearchEnumerationResearchCases,
);

{
  const rustWebRequestTests = readRepoFile('tests/unit/web_requests.rs');
  const browserIssue228Tests = readRepoFile('tests/e2e/tests/issue-228.spec.js');
  for (const [language, entries] of Object.entries(webSearchEnumerationResearchCases)) {
    for (const entry of entries) {
      assert(
        entry.prompt.trim() &&
          entry.rustQuery.trim() &&
          entry.browserRequest.trim(),
        `webSearchEnumerationResearchCases ${language} entries must define prompt, rustQuery, and browserRequest`,
      );
      assert(
        rustWebRequestTests.includes(entry.prompt) &&
          rustWebRequestTests.includes(entry.rustQuery),
        `tests/unit/web_requests.rs must cover ${language} enumeration-research prompt ${JSON.stringify(entry.prompt)}`,
      );
      assert(
        browserIssue228Tests.includes(entry.prompt) &&
          browserIssue228Tests.includes(entry.browserRequest),
        `tests/e2e/tests/issue-228.spec.js must cover ${language} enumeration-research prompt ${JSON.stringify(entry.prompt)}`,
      );
    }
  }
}

const currentDayCalendarCases = {
  en: ['What day is today?'],
  ru: ['Какой сегодня день?'],
  hi: ['आज कौन सा दिन है?'],
  zh: ['今天是星期几?'],
};

assertMatrixMatchesSupportedLanguages(
  'currentDayCalendarCases',
  currentDayCalendarCases,
);
assertBalancedLanguageCaseCounts(
  'currentDayCalendarCases',
  currentDayCalendarCases,
);

{
  const rustReasoningTests = readRepoFile('tests/unit/specification/reasoning_paths.rs');
  const browserMultilingualTests = readRepoFile('tests/e2e/tests/multilingual.spec.js');
  for (const [language, prompts] of Object.entries(currentDayCalendarCases)) {
    for (const prompt of prompts) {
      assert(
        rustReasoningTests.includes(prompt),
        `tests/unit/specification/reasoning_paths.rs must cover ${language} current-day calendar prompt ${JSON.stringify(prompt)}`,
      );
      assert(
        browserMultilingualTests.includes(prompt),
        `tests/e2e/tests/multilingual.spec.js must cover ${language} current-day calendar prompt ${JSON.stringify(prompt)}`,
      );
    }
  }
}

const primeInfinitudeProofCases = {
  en: [
    {
      prompt: 'Hello. Prove that there are infinitely many prime numbers',
      expectedStatement: 'There are infinitely many prime numbers',
    },
  ],
  ru: [
    {
      prompt: 'привет. докажи что простых бесконечно',
      expectedStatement: 'Простых чисел бесконечно много',
    },
  ],
  hi: [
    {
      prompt: 'नमस्ते. साबित करो कि अभाज्य संख्याएँ अनंत हैं',
      expectedStatement: 'अभाज्य संख्याएँ अनंत हैं',
    },
  ],
  zh: [
    {
      prompt: '你好。证明素数有无穷多个',
      expectedStatement: '素数有无穷多个',
    },
  ],
};

assertMatrixMatchesSupportedLanguages(
  'primeInfinitudeProofCases',
  primeInfinitudeProofCases,
);
assertBalancedLanguageCaseCounts(
  'primeInfinitudeProofCases',
  primeInfinitudeProofCases,
);

{
  const rustProofTests = readRepoFile('tests/unit/proof_request.rs');
  const browserIssue209Tests = readRepoFile('tests/e2e/tests/issue-209.spec.js');
  for (const [language, entries] of Object.entries(primeInfinitudeProofCases)) {
    for (const entry of entries) {
      assert(
        rustProofTests.includes(entry.prompt) &&
          rustProofTests.includes(entry.expectedStatement),
        `tests/unit/proof_request.rs must cover ${language} prime-infinitude proof prompt ${JSON.stringify(entry.prompt)}`,
      );
      assert(
        browserIssue209Tests.includes(entry.prompt) &&
          browserIssue209Tests.includes(entry.expectedStatement),
        `tests/e2e/tests/issue-209.spec.js must cover ${language} prime-infinitude proof prompt ${JSON.stringify(entry.prompt)}`,
      );
    }
  }
}

const requiredLocalizedResponseIntents = [
  'greeting',
  'farewell',
  'courtesy_response',
  'test_status',
  'identity',
  'assistant_name',
  'clarification',
  'capabilities',
  'capabilities_more',
  'unknown',
  'unknown_reasoning_question',
  'unknown_reasoning_trace',
  'meta_explanation',
  'inappropriate_content',
];

for (const intent of requiredLocalizedResponseIntents) {
  for (const language of supportedLanguages) {
    assert(
      responses.some((response) => response.intent === intent && response.language === language),
      `multilingual-responses.lino must provide ${intent} response for ${language}`,
    );
  }
}

for (const concept of concepts.filter((record) => record.localized.length > 0)) {
  const languages = concept.localized.map((localized) => localized.language);
  assert(
    new Set(languages).size === languages.length,
    `concepts.lino ${concept.id} must not duplicate localized language records`,
  );
  assertMatrixMatchesSupportedLanguages(
    `concepts.lino ${concept.id} localized records`,
    Object.fromEntries(concept.localized.map((localized) => [localized.language, localized])),
  );

  for (const language of supportedLanguages) {
    const localized = concept.localized.find((entry) => entry.language === language);
    for (const field of ['term', 'aliases', 'summary', 'source', 'source_kind']) {
      assert(
        localized?.[field]?.trim(),
        `concepts.lino ${concept.id} localized ${language} must define ${field}`,
      );
    }
  }
}

for (const tool of tools) {
  const languageRecords = {
    en: { name: tool.name, description: tool.description },
    ...Object.fromEntries(tool.localized.map((localized) => [localized.language, localized])),
  };
  assertMatrixMatchesSupportedLanguages(
    `tools.lino ${tool.id} localized records`,
    languageRecords,
  );
  for (const language of supportedLanguages) {
    const localized = languageRecords[language];
    for (const field of ['name', 'description']) {
      assert(
        localized?.[field]?.trim(),
        `tools.lino ${tool.id} ${language} must define ${field}`,
      );
    }
  }
}

const knownFeatureCapabilities = new Set(featureCapabilitySlugs);
for (const [feature, languages] of featureCapabilityTestMatrix) {
  assert(
    knownFeatureCapabilities.has(feature),
    `tests/unit/specification/capabilities.rs covers unknown feature capability ${feature}`,
  );
  assertMatrixMatchesSupportedLanguages(
    `feature capability unit-test matrix for ${feature}`,
    Object.fromEntries([...languages].map((language) => [language, true])),
  );
}

for (const feature of featureCapabilitySlugs) {
  assert(
    featureCapabilityTestMatrix.has(feature),
    `tests/unit/specification/capabilities.rs must cover feature capability ${feature} for every supported language`,
  );
}

for (const entry of browserTranslationRegistry) {
  assert(entry.token, 'TRANSLATION_MEANING_REGISTRY entries must define token');
  assertMatrixMatchesSupportedLanguages(
    `TRANSLATION_MEANING_REGISTRY ${entry.token} primary`,
    entry.primary || {},
  );
  assertMatrixMatchesSupportedLanguages(
    `TRANSLATION_MEANING_REGISTRY ${entry.token} aliases`,
    entry.aliases || {},
  );

  for (const language of supportedLanguages) {
    assert(
      entry.primary?.[language]?.trim(),
      `TRANSLATION_MEANING_REGISTRY ${entry.token} primary.${language} must be non-empty`,
    );
    assert(
      Array.isArray(entry.aliases?.[language]) && entry.aliases[language].length > 0,
      `TRANSLATION_MEANING_REGISTRY ${entry.token} aliases.${language} must be a non-empty array`,
    );
  }
}

for (const context of contextRecords) {
  const languages = context.labels.map((label) => label.language);
  assert(
    new Set(languages).size === languages.length,
    `concept-contexts.lino ${context.id} must not duplicate label languages`,
  );
  assertMatrixMatchesSupportedLanguages(
    `concept-contexts.lino ${context.id} labels`,
    Object.fromEntries(context.labels.map((label) => [label.language, label])),
  );

  for (const language of supportedLanguages) {
    const label = context.labels.find((entry) => entry.language === language);
    assert(
      label?.text?.trim(),
      `concept-contexts.lino ${context.id} label ${language} must define text`,
    );
  }
}

if (errors.length > 0) {
  console.error('Multilingual intent coverage check failed:');
  for (const error of errors) {
    console.error(`- ${error}`);
  }
  process.exit(1);
}

console.log(
  `Multilingual intent coverage OK for ${supportedLanguages.join(', ')} (${requiredLocalizedResponseIntents.length} localized response intents, ${concepts.filter((record) => record.localized.length > 0).length} localized concept records, ${contextRecords.length} concept context records, ${tools.length} localized tools, ${featureCapabilitySlugs.length} feature capabilities, ${browserTranslationRegistry.length} translation meanings).`,
);
