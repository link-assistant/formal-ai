// Functional check for the JS worker mirror of the issue #341 follow-up fix.
// Loads src/web/formal_ai_worker.js in a vm sandbox with the browser globals
// shimmed, then drives the two-step scraper dialogue through `solve()`.
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import { fileURLToPath } from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, '..');
const source = fs.readFileSync(
  path.join(repoRoot, 'src/web/formal_ai_worker.js'),
  'utf8',
);

const sandbox = {
  self: { location: { search: '' } },
  importScripts() {},
  postMessage() {},
  TextEncoder,
  TextDecoder,
  console,
};
sandbox.self.onmessage = null;
sandbox.globalThis = sandbox;
vm.createContext(sandbox);
// Expose solve() after the worker body defines it.
vm.runInContext(source + '\n;globalThis.__solve = solve;', sandbox, {
  filename: 'formal_ai_worker.js',
});

const step1 = [
  'Design a simple web scraper in Python that:',
  '1. Fetches a webpage',
  '2. Extracts all headings (h1, h2, h3)',
  '3. Counts word frequency',
  '4. Generates a markdown summary',
].join('\n');
const step2 =
  'test it by scraping wikipedia.org and show me the top 10 most frequent words.';

const prefs = {};
const plan = await sandbox.__solve(step1, [], prefs, {});
console.log('=== STEP 1 intent:', plan.intent, '===');

const history = [
  { role: 'user', content: step1 },
  { role: 'assistant', content: plan.content },
];
const follow = await sandbox.__solve(step2, history, prefs, {});
console.log('=== STEP 2 intent:', follow.intent, '===');
console.log('--- STEP 2 content ---');
console.log(follow.content);

const ok =
  follow.intent === 'software_project_followup' &&
  follow.content.includes('wikipedia.org') &&
  follow.content.includes('the top 10 most frequent words') &&
  !follow.content.toLowerCase().includes('encyclopedia');

// Multilingual parity with the Rust solver: design in English, then exercise
// the artifact in ru / hi / zh.
const multilingual = [
  ['ru русский', 'теперь протестируй его на wikipedia.org'],
  ['hi हिंदी', 'अब इसका परीक्षण करो'],
  ['zh 中文', '现在测试它'],
];
let multilingualOk = true;
for (const [language, prompt] of multilingual) {
  const result = await sandbox.__solve(prompt, history, prefs, {});
  const pass = result.intent === 'software_project_followup';
  console.log(`  ${language}: ${result.intent} -> ${pass ? 'PASS' : 'FAIL'}`);
  multilingualOk = multilingualOk && pass;
}

console.log('\nRESULT:', ok && multilingualOk ? 'PASS' : 'FAIL');
process.exit(ok && multilingualOk ? 0 : 1);
