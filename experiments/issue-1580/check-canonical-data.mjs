// Replay canonical parsing for every data file without compiling Rust.
import {readdirSync, readFileSync} from 'node:fs';
import {join} from 'node:path';
import {Parser} from 'links-notation';
function* files(directory) {
  for (const entry of readdirSync(directory, {withFileTypes: true})) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) yield* files(path);
    else if (entry.isFile() && path.endsWith('.lino')) yield path;
  }
}
let checked = 0;
const failures = [];
for (const file of files('data')) {
  const source = readFileSync(file, 'utf8');
  checked += 1;
  try { new Parser().parse(source.trim()); }
  catch (error) { failures.push({file, error: String(error.message).slice(0, 1200)}); }
}
console.log(JSON.stringify({checked, failures, nativeCompilationOrExecution: false}, null, 2));
if (failures.length) process.exitCode = 1;
