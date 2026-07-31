// Issue #699 batch 2: verify the browser worker resolves misspelled names from
// remembered surfaces (no hardcoded person table) and localizes the merged
// definition headings, exactly like `src/entity_resolution.rs` /
// `src/definition_merge.rs`.
//
// Run: node experiments/check_worker_entity_resolution.mjs
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';

const WORKER_DIR = 'src/web/worker';
const parts = fs
  .readdirSync(WORKER_DIR)
  .filter((name) => name.endsWith('.js'))
  .sort();

const sandbox = {
  self: { location: { search: '' } },
  console,
  postMessage: () => {},
  TextEncoder,
  TextDecoder,
};
sandbox.globalThis = sandbox;
vm.createContext(sandbox);
for (const part of parts) {
  vm.runInContext(fs.readFileSync(path.join(WORKER_DIR, part), 'utf8'), sandbox, {
    filename: part,
  });
}

// Hydrate the seed text the same way loadSeed() does, straight from data/seed.
const raw = {};
for (const file of fs.readdirSync('data/seed').filter((n) => n.endsWith('.lino'))) {
  raw[`seed/${file}`] = fs.readFileSync(path.join('data/seed', file), 'utf8');
}
sandbox.hydrateLinoSeedText(raw);
// `answerFor` reads MULTILINGUAL_ANSWERS, which `loadSeed()` merges from the
// seed bundle in the browser; build the same table straight from data/seed.
const answers = {};
for (const [name, text] of Object.entries(raw)) {
  if (!name.includes('multilingual-responses')) continue;
  const root = sandbox.parseLinoTree(text);
  const container = root.children.find((child) => child.name === 'multilingual_responses') || root;
  for (const record of container.children) {
    if (record.name !== 'response') continue;
    const value = (child) => (record.children.find((c) => c.name === child) || {}).value || '';
    const intent = value('intent');
    if (!intent) continue;
    answers[intent] = answers[intent] || {};
    answers[intent][value('language')] = value('text');
  }
}
// A top-level `let` is not a property of the context object, so assign inside.
sandbox.__answers = answers;
vm.runInContext('MULTILINGUAL_ANSWERS = __answers;', sandbox);

let failures = 0;
function check(name, condition) {
  console.log(`${condition ? 'PASS' : 'FAIL'}  ${name}`);
  if (!condition) failures += 1;
}

// Held-out typos: none of these names or spellings were in the retired table.
check('ada lovlace -> Ada Lovelace', sandbox.suggestNameCorrection('ada lovlace') === 'Ada Lovelace');
check('alan turring -> Alan Turing', sandbox.suggestNameCorrection('alan turring') === 'Alan Turing');
check(
  'альберт эйнштеин -> Альберт Эйнштейн',
  sandbox.suggestNameCorrection('альберт эйнштеин') === 'Альберт Эйнштейн',
);
check('निकोला टेस्ल -> निकोला टेस्ला', sandbox.suggestNameCorrection('निकोला टेस्ल') === 'निकोला टेस्ला');
// Pinned behavior from before the migration.
check('elon mask -> Elon Musk', sandbox.suggestNameCorrection('elon mask') === 'Elon Musk');
// Correct spellings are not "corrected".
check('Ada Lovelace has no correction', sandbox.suggestNameCorrection('Ada Lovelace') === null);
// No misspelling is stored anywhere in the registry.
check(
  'registry stores no misspellings',
  !/mask|tramp|tromp|bidan|einstien|issac|vladmir|puting/i.test(raw['seed/entity-names.lino']),
);
// Localized headings come from seed data, not from JavaScript literals.
const worker09 = fs.readFileSync(path.join(WORKER_DIR, 'formal_ai_worker_09.js'), 'utf8');
check('no hardcoded merge headings', !worker09.includes('Merged definition of'));
check('no hardcoded person table', !worker09.includes('KNOWN_PERSON_VARIANTS'));

process.exitCode = failures ? 1 : 0;
