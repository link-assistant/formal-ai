// Guard native diagnostic boundaries without invoking a native toolchain.
import fs from 'node:fs';
import path from 'node:path';
const root = path.resolve(import.meta.dirname, '..');
function walk(directory) { return fs.readdirSync(directory, {withFileTypes:true}).flatMap(entry => entry.isDirectory() ? walk(path.join(directory, entry.name)) : [path.join(directory, entry.name)]); }
const failures = [];
for (const file of walk(path.join(root, 'rust/src')).filter(file => file.endsWith('.rs'))) {
  const source = fs.readFileSync(file, 'utf8');
  if (source.includes('eprintln!')) failures.push(path.relative(root, file) + ': bypasses shared native logger');
}
if (failures.length) { console.error(failures.join('\n')); process.exitCode = 1; }
else console.log('Native diagnostic bypass guard passed');
