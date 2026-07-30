// Issue #699 batch 3: verify the browser worker fails an underivable
// `write_program` request with a named, seed-driven skill gap instead of
// reciting the template catalogue — the same behavior as
// `src/program_skill_gap.rs`.
//
// Run: node experiments/check_worker_program_skill_gap.mjs
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

// The gap identity is English and names both parameters.
const name = sandbox.programSkillGapName(null, 'rust', 'en');
check('gap name reports the language', name.includes('rust'));
check('gap name reports the missing task', name.includes('missing'));

// Every supported response language renders a localized reply that names the
// gap and the routes, and never recites the catalogue.
for (const language of ['en', 'ru', 'hi', 'zh']) {
  const answer = sandbox.programSkillGapAnswer(null, 'rust', language);
  check(`${language}: reply is rendered`, answer.length > 0);
  check(`${language}: reply names the routes`, answer.includes('seed_idiom_composer'));
  check(`${language}: reply does not recite tasks`, !/hello_world|count_to_three/.test(answer));
}

// The retired dead end is gone from the worker sources.
const RETIRED_RECITATION = ['Supported', 'tasks:'].join(' ');
const workerText = parts
  .map((part) => fs.readFileSync(path.join(WORKER_DIR, part), 'utf8'))
  .join('\n');
check('no catalogue recitation', !workerText.includes(RETIRED_RECITATION));
check('no unsupported intent', !workerText.includes('"write_program_unsupported"'));

process.exitCode = failures ? 1 : 0;
