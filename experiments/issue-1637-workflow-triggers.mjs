// Prints, per workflow, whether pull_request/push are filtered by paths.
import fs from 'node:fs';
import YAML from 'yaml';
const dir = '.github/workflows';
for (const f of fs.readdirSync(dir).filter((n) => n.endsWith('.yml')).sort()) {
  const d = YAML.parse(fs.readFileSync(`${dir}/${f}`, 'utf8'));
  const on = d.on ?? d[true];
  const show = (ev) => {
    if (!on || typeof on !== 'object' || !(ev in on)) return '-';
    const v = on[ev] ?? {};
    return v.paths ? `paths(${v.paths.length})` : v['paths-ignore'] ? `ignore(${v['paths-ignore'].length})` : 'ALL';
  };
  console.log(`${f}\tpr=${show('pull_request')}\tpush=${show('push')}`);
}
