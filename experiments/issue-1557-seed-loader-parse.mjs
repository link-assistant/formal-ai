// Issue #1557: parse data/seed/language-detection.lino with the browser seed
// loader and print the language rules it extracts, to confirm the new
// `prose_syntax` record is ignored by rule extraction and parses cleanly.
// Usage: node experiments/issue-1557-seed-loader-parse.mjs
import { readFileSync } from 'node:fs';
import vm from 'node:vm';

const root = new URL('../', import.meta.url);
const context = vm.createContext({ TextEncoder, console });
vm.runInContext(readFileSync(new URL('js/parser-cache-primordials.js', root), 'utf8'), context);
vm.runInContext(readFileSync(new URL('js/seed_loader.js', root), 'utf8'), context);
const seed = context.FormalAiSeed;
const text = readFileSync(new URL('data/seed/language-detection.lino', root), 'utf8');
const tree = seed.parse(text);
const top = Array.isArray(tree) ? tree[0] : tree;
console.log('children:', (top.children || []).map((child) => `${child.name} ${child.id ?? ''}`.trim()).join(', '));
const syntax = (top.children || []).find((child) => child.name === 'prose_syntax');
console.log('prose_syntax:', JSON.stringify(syntax?.children?.map((child) => [child.name, child.id])));
const rules = seed.extractLanguageRules(top);
console.log('rules:', rules.map((rule) => `${rule.language}:${rule.script}:${rule.markers.length}`).join(' '));
