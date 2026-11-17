// Lists every named import of a test file (rust/tests/web, scripts, experiments)
// that its js/ or ts/ module no longer exports. The pull request lane runs a
// fast test profile, so a renamed export can break a suite only main runs.
// Usage: node experiments/issue-1162/check-test-imports.mjs [glob roots...]
import { readFileSync, readdirSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

const roots = process.argv.slice(2).length ? process.argv.slice(2) : ['rust/tests/web'];
const files = roots.flatMap((root) => readdirSync(root, { recursive: true })
  .map(String).filter((name) => name.endsWith('.mjs')).map((name) => join(root, name)));
const importPattern = /import\s*\{([^}]*)\}\s*from\s*'([^']+)'/gsu;
const exportsOf = new Map();
let missing = 0;
for (const file of files) {
  const text = readFileSync(file, 'utf8');
  for (const [, names, specifier] of text.matchAll(importPattern)) {
    if (!specifier.startsWith('.')) continue;
    const target = resolve(dirname(file), specifier);
    if (!/\/(js|ts)\//u.test(target)) continue;
    if (!exportsOf.has(target)) {
      try {
        exportsOf.set(target, new Set(Object.keys(await import(pathToFileURL(target).href))));
      } catch (error) {
        exportsOf.set(target, error);
      }
    }
    const available = exportsOf.get(target);
    if (available instanceof Error) continue;
    for (const raw of names.split(',')) {
      const name = raw.trim().split(/\s+as\s+/u)[0].replace(/^type\s+/u, '').trim();
      if (name && !available.has(name)) {
        missing += 1;
        console.log(`${file}: ${name} is not exported by ${specifier}`);
      }
    }
  }
}
console.log(`${files.length} files, ${missing} missing named imports`);
process.exitCode = missing ? 1 : 0;
