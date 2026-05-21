// CI guard for language coverage in symbolic conversational intents.
//
// This intentionally checks seed files directly, not only runtime behavior, so
// reviewers get a precise failure when a new phrase is added for one language
// without the matching supported-language matrix entries.

import fs from 'node:fs';
import path from 'node:path';
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

const supportedLanguages = parseSupportedLanguages();
const routes = parseIntentRouting();
const patterns = parsePromptPatterns();
const responses = parseResponseRecords();
const concepts = parseConceptRecords();
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

const requiredLocalizedResponseIntents = [
  'greeting',
  'farewell',
  'courtesy_response',
  'test_status',
  'identity',
  'clarification',
  'unknown',
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

if (errors.length > 0) {
  console.error('Multilingual intent coverage check failed:');
  for (const error of errors) {
    console.error(`- ${error}`);
  }
  process.exit(1);
}

console.log(
  `Multilingual intent coverage OK for ${supportedLanguages.join(', ')} (${requiredLocalizedResponseIntents.length} localized response intents, ${concepts.filter((record) => record.localized.length > 0).length} localized concept records).`,
);
