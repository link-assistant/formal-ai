// Issue #1550: the cases of the shared meta-executor parity fixture
// (rust/tests/fixtures/meta-executor-parity.json). Every case runs the
// procedure over a table host: `answers[text]` is the direct derivation of a
// task text (absent means the method gives no draft), `cached[text]` is a
// stored row. The native test rust/tests/unit/meta_executor_parity.rs builds
// the same host from the same table.

import {
  decompositionShape,
  executeProcedure,
  taskLeaves,
} from '../../../../js/agentic/crate/meta_executor.mjs';

const answered = (content) => ({ intent: 'answered', content, confidence: 1 });
const unknown = { intent: 'unknown', content: '', confidence: 0 };

const TWO_LEVEL = {
  en: ['Translate hello to French, then count the letters and report the vowels',
    'Translate hello to French', 'count the letters and report the vowels', 'count the letters', 'report the vowels'],
  ru: ['Переведи привет на французский, затем посчитай буквы и назови гласные',
    'Переведи привет на французский', 'посчитай буквы и назови гласные', 'посчитай буквы', 'назови гласные'],
  hi: ['हैलो का फ्रेंच में अनुवाद करो, फिर अक्षर गिनो और स्वर बताओ',
    'हैलो का फ्रेंच में अनुवाद करो', 'अक्षर गिनो और स्वर बताओ', 'अक्षर गिनो', 'स्वर बताओ'],
  zh: ['把你好翻译成法语，然后数字母并且列出元音',
    '把你好翻译成法语', '数字母并且列出元音', '数字母', '列出元音'],
  es: ['Traduce hola al francés, luego cuenta las letras y nombra las vocales',
    'Traduce hola al francés', 'cuenta las letras y nombra las vocales', 'cuenta las letras', 'nombra las vocales'],
};

function twoLevel(language, overrides = {}) {
  const [root, first, second, third, fourth] = TWO_LEVEL[language];
  return {
    name: `two-level-${language}`,
    text: root,
    language,
    answers: {
      [root]: unknown,
      [first]: answered('bonjour'),
      [second]: unknown,
      [third]: answered('5'),
      [fourth]: answered('e, o'),
    },
    cached: {},
    withheld: [],
    bounds: {},
    ...overrides,
  };
}

/** Literal quotation controls shared with the native fixture reader. */
export const QUOTATION_CASES = [
  ['ascii-double-aggregation', 'Count "salt and pepper" and report the size', 'aggregation', ['Count "salt and pepper"', 'report the size']],
  ['ascii-double-sequence', 'Read "first then second" then report the length', 'sequence', ['Read "first then second"', 'report the length']],
  ['ascii-single-aggregation', "Count 'salt and pepper' and report the size", 'aggregation', ["Count 'salt and pepper'", 'report the size']],
  ['unquoted-contraction', "Don't count and report the size", 'aggregation', ["Don't count", 'report the size']],
  ['quoted-contraction', "Read 'Don't split and join' and report the length", 'aggregation', ["Read 'Don't split and join'", 'report the length']],
  ['plural-possessive', "Count the dogs' coats and report the size", 'aggregation', ["Count the dogs' coats", 'report the size']],
  ['escaped-double-quote', 'Read "a \\"quote and join\\" stays" and report the length', 'aggregation', ['Read "a \\"quote and join\\" stays"', 'report the length']],
  ['adjacent-cjk-single-quote', "统计'盐并且胡椒'并且报告大小", 'aggregation', ["统计'盐并且胡椒'", '报告大小'], 'zh'],
  ['adjacent-cjk-double-quote', '统计"盐并且胡椒"并且报告大小', 'aggregation', ['统计"盐并且胡椒"', '报告大小'], 'zh'],
  ['russian-double-quote', 'Посчитай "соль и перец" и назови размер', 'aggregation', ['Посчитай "соль и перец"', 'назови размер'], 'ru'],
];

/** Status controls use a second shared fixture to keep both files bounded. */
export const STATUS_CASES = [
  {
    name: 'unverified-program-leaf',
    text: 'Produce source',
    language: 'en',
    answers: { 'Produce source': { intent: 'meta_reasoned_program_unverified', content: 'A nonempty program', confidence: 1 } },
    cached: {},
    withheld: [],
    bounds: {},
  },
  {
    name: 'unverified-program-child',
    text: 'First task and second task',
    language: 'en',
    answers: {
      'First task': { intent: 'meta_reasoned_program_unverified', content: 'A nonempty program', confidence: 1 },
      'second task': answered('Second result'),
    },
    cached: {},
    withheld: [],
    bounds: {},
  },
  {
    name: 'unverified-program-cache',
    text: 'Produce source',
    language: 'en',
    answers: {},
    cached: { 'Produce source': { intent: 'meta_reasoned_program_unverified', content: 'A nonempty program', confidence: 1 } },
    withheld: [],
    bounds: {},
  },
  {
    name: 'verified-program-child',
    text: 'First task and second task',
    language: 'en',
    answers: {
      'First task': { intent: 'meta_reasoned_program', content: 'A nonempty program', confidence: 1 },
      'second task': answered('Second result'),
    },
    cached: {},
    withheld: [],
    bounds: {},
  },
];

/** Every case of the fixture, in order. */
export const CASES = [
  twoLevel('en'),
  twoLevel('ru'),
  twoLevel('hi'),
  twoLevel('zh'),
  twoLevel('es'),
  { ...twoLevel('en'), name: 'withheld-sub-task-test', withheld: ['t0.2.1'] },
  { ...twoLevel('zh'), name: 'withheld-sub-task-test-zh', withheld: ['t0.1'] },
  {
    ...twoLevel('en'),
    name: 'direct-derivation-wins-when-it-passes',
    answers: { ...twoLevel('en').answers, [TWO_LEVEL.en[0]]: answered('bonjour; 5; e, o') },
  },
  {
    ...twoLevel('en'),
    name: 'cache-row-is-the-least-action',
    cached: { 'count the letters': answered('5') },
  },
  { ...twoLevel('en'), name: 'depth-bound', bounds: { depth: 1 } },
  { ...twoLevel('en'), name: 'breadth-bound', bounds: { breadth: 1 } },
  { ...twoLevel('en'), name: 'budget-bound', bounds: { budget: 2 } },
  {
    name: 'alternative-takes-the-first-passing-sub-task',
    text: 'Answer in Klingon or answer in French',
    language: 'en',
    answers: { 'answer in Klingon': unknown, 'answer in French': answered('Bonjour') },
    cached: {},
    withheld: [],
    bounds: {},
  },
  {
    name: 'quoted-connective-is-not-split',
    text: 'Переведи «соль и перец» на английский',
    language: 'ru',
    answers: { 'Переведи «соль и перец» на английский': answered('salt and pepper') },
    cached: {},
    withheld: [],
    bounds: {},
  },
  {
    name: 'atomic-request',
    text: 'What is the capital of France?',
    language: 'en',
    answers: { 'What is the capital of France?': answered('Paris') },
    cached: {},
    withheld: [],
    bounds: {},
  },
  ...QUOTATION_CASES.map(([name, text, , parts, language = 'en']) => ({
    name,
    text,
    language,
    answers: Object.fromEntries(parts.map((part, index) => [part, answered(String(index + 1))])),
    cached: {},
    withheld: [],
    bounds: {},
  })),
  ...STATUS_CASES,
];

/** The table host of one case. */
export function tableHost(entry) {
  const own = (table, text) => (Object.hasOwn(table, text) ? table[text] : null);
  return {
    draft: async (text) => own(entry.answers, text),
    cached: async (text) => own(entry.cached, text),
  };
}

/** Replay one case: the observable result both runtimes must agree on. */
export async function replayCase(entry) {
  const run = await executeProcedure({ text: entry.text, language: entry.language }, tableHost(entry), {
    bounds: entry.bounds,
    withheldTests: entry.withheld,
  });
  return {
    shape: decompositionShape(run.root),
    leaves: taskLeaves(run.root),
    status: run.outcome.status,
    content: run.outcome.candidate?.content ?? null,
    events: run.events.map((event) => `${event.kind} ${event.payload}`),
  };
}
