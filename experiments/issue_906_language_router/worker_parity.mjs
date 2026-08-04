// Issue #906: run the Rust regression corpus
// (`tests/unit/issue_906_language_router.rs`) against the *browser worker's*
// mirror of the implementation-language router, so the two copies of the rule
// cannot drift apart.
//
// Run: node experiments/issue_906_language_router/worker_parity.mjs
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

// The corpus, kept in the same order as the Rust table.
const CORPUS = [
  ['Write me hello world program in Rust', 'rust'],
  ['hello world in python', 'python'],
  ['write a hello world program in JavaScript', 'javascript'],
  ['hello world in py', 'python'],
  ['hello world in golang', 'go'],
  ['hello world in node', 'javascript'],
  ['напиши программу hello world на python', 'python'],
  ['Напиши хелло ворлд на питоне', 'python'],
  ['count to three in rust', 'rust'],
  ['hello world in elvish', 'elvish'],
  ['hello world in the elvish language', 'elvish'],
  ['write a hello world program in language elvish', 'elvish'],
  [
    'Create a file named hello.txt in the current directory whose entire content is the single line: Hello World.',
    null,
  ],
  ['Fix the failing CI job in the current directory.', null],
  ['run the tests in the background', null],
  ['Write a program that prints hello world.', null],
  ['write a program', null],
  ['hello world', null],
  ['hello world in 3 steps', null],
  ['print the numbers in reverse order', null],
];

let failures = 0;
for (const [prompt, expected] of CORPUS) {
  const normalized = sandbox.normalizePrompt(prompt);
  const resolved = sandbox.programLanguageFromPrompt(normalized) ?? null;
  const ok = resolved === expected;
  if (!ok) failures += 1;
  console.log(
    `${ok ? 'PASS' : 'FAIL'}  ${JSON.stringify(prompt)} -> ${JSON.stringify(resolved)} (expected ${JSON.stringify(expected)})`,
  );
}

console.log(failures === 0 ? '\nworker parity: OK' : `\nworker parity: ${failures} mismatch(es)`);
process.exit(failures === 0 ? 0 : 1);
