// highlight-demo.mjs — runnable demo of the dependency-free highlighter that
// the formal-ai chat UI uses (issue #330).
//
// Run command:
//   node examples/issue-330-code-highlighting/highlight-demo.mjs
//
// It loads src/web/syntax-highlight.js (a plain script that attaches
// `FormalAiHighlight` to the global object), highlights one snippet per
// language, and prints the resolved grammar plus the safe HTML token spans.
// This is the exact code path the browser runs — no browser, no dependencies.

import { fileURLToPath } from 'node:url';
import path from 'node:path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, '../..');

// Importing the script for its side effect: it assigns globalThis.FormalAiHighlight.
await import(path.join(repoRoot, 'src/web/syntax-highlight.js'));
const { highlight, listLanguages } = globalThis.FormalAiHighlight;

const samples = [
  ['rust', 'fn main() {\n    let names = read_dir(".").unwrap();\n}'],
  ['python', 'def main():\n    print(sorted(os.listdir(".")))'],
  ['go', 'func main() {\n    entries, _ := os.ReadDir(".")\n}'],
  ['ruby', 'Dir.entries(".").sort.each { |name| puts name }'],
];

console.log(`Supported languages: ${listLanguages().join(', ')}\n`);

for (const [lang, source] of samples) {
  const { value, language } = highlight(source, lang);
  console.log(`# ${lang} (resolved: ${language})`);
  console.log(value);
  console.log('');
}

// A tiny assertion so the demo doubles as a smoke test.
const { value } = highlight('fn main() {}', 'rust');
if (!value.includes('hljs-keyword')) {
  console.error('FAIL: expected a hljs-keyword span for Rust `fn`');
  process.exit(1);
}
console.log('OK: highlighter produced hljs-* token spans.');
