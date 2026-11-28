// List download-artifact wrapper calls that select by pattern (or every
// artifact) without merge-multiple: their layout depends on how many artifacts
// match, since actions/download-artifact fff8c148a (v5) extracts a single
// match straight into `path` (issue #1639, ladder run 45659315559).
import fs from 'node:fs';
import path from 'node:path';
import {parse} from 'yaml';

const root = process.argv[2] ?? '.';
const files = [
  ...fs.readdirSync(path.join(root, '.github/workflows')).filter(name => name.endsWith('.yml')).map(name => `.github/workflows/${name}`),
  ...fs.readdirSync(path.join(root, '.github/actions')).map(name => `.github/actions/${name}/action.yml`)
    .filter(file => fs.existsSync(path.join(root, file))),
];
for (const file of files.sort()) {
  const document = parse(fs.readFileSync(path.join(root, file), 'utf8')) ?? {};
  const steps = [...Object.values(document.jobs ?? {}).flatMap(job => job.steps ?? []), ...(document.runs?.steps ?? [])];
  for (const step of steps) {
    if (!String(step.uses ?? '').includes('download-artifact')) continue;
    const options = step.with ?? {};
    if (options.name || options['artifact-ids'] || String(options['merge-multiple'] ?? 'false') === 'true') continue;
    console.log([file, step.uses, options.pattern ?? '<all>', options.path, step.name ?? ''].join('\t'));
  }
}
