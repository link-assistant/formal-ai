// Conservative static reference graph: unresolved/dynamic references stay explicit.
// This is a review aid, not a claim that source behavior has been verified.
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {gzipSync} from 'node:zlib';
import {createRequire} from 'node:module';
const {parse} = createRequire(import.meta.url)('yaml');

export function traceTree(directory, inventory) {
  const files = new Set(inventory.map(item => item.path));
  const blobs = new Map(inventory.map(item => [item.path, item.gitBlob]));
  const queue = [...files].filter(file => /^\.github\/workflows\/[^/]+\.ya?ml$/u.test(file));
  const roots = [...queue];
  const visited = new Set(), edges = [], commands = [], unresolved = [], edgeKeys = new Set();
  const add = (from, target, reason) => {
    if (!files.has(target)) return false;
    const key = JSON.stringify([from,target,reason]);
    if (!edgeKeys.has(key)) {
      edgeKeys.add(key);
      edges.push({from, to: target, reason});
      if (!visited.has(target)) queue.push(target);
    }
    return true;
  };
  const resolve = (from, value, reason, relative = false) => {
    if (value.includes('://') || value.startsWith('node:')) return;
    const base = relative ? path.posix.dirname(from) : '.';
    const target = path.posix.normalize(path.posix.join(base, value));
    if (target.startsWith('../')) {
      unresolved.push({from, value, reason: 'reference leaves the source tree'});
      return;
    }
    const variants = [target, ...['.mjs','.js','.cjs','.ts','.tsx','.py','.rs','.php','.sh'].map(extension => target + extension),
      ...['action.yml','action.yaml','index.mjs','index.js','mod.rs','__init__.py'].map(file => path.posix.join(target,file))];
    let matched = false;
    for (const candidate of variants) matched = add(from,candidate,reason) || matched;
    if (matched) return;
    if (files.has(target.replace(/^\.\//u,''))) return;
    const prefix = target.replace(/\/$/u,'') + '/';
    const children = [...files].filter(file => file.startsWith(prefix));
    if (children.length) {
      for (const child of children) add(from,child,reason + ' (conservative directory expansion)');
    } else if (value.startsWith('.') || /^(?:scripts|tests?|src|\.github|config|rust|js)\//u.test(value)) {
      unresolved.push({from,value,reason:'no static file match: '+reason});
    }
  };
  const scan = (from, text, reason) => {
    for (const match of text.matchAll(/(?:^|[\s"'`(=])((?:\.{1,2}\/)?(?:[\w@.-]+\/)+[\w@.*?{}$-]+)/gmu)) {
      const value = match[1];
      if (/[$*?{}]/u.test(value)) {
        unresolved.push({from,value,reason:'dynamic/glob path in '+reason});
      } else resolve(from,value,reason);
    }
    // Quoted static JS/PHP/Rust/shell relative paths include helpers outside scripts/.
    for (const match of text.matchAll(/["']((?:\.\.?\/)[^"'\n]+)["']/gu)) {
      if (/[$*?{}]/u.test(match[1])) unresolved.push({from,value:match[1],reason:'dynamic relative path'});
      else resolve(from,match[1],'relative literal',true);
    }
    // Python local imports and Rust out-of-line modules; installed modules remain external.
    if (from.endsWith('.py')) for (const match of text.matchAll(/^\s*(?:from|import)\s+([\w.]+)/gmu)) {
      const value = match[1].replaceAll('.','/');
      if ([...files].some(file => file.startsWith(value + '/') || file === value + '.py')) resolve(from,value,'local Python module');
      else if (files.has(path.posix.join(path.posix.dirname(from),value + '.py'))) resolve(from,'./'+value,'local Python sibling',true);
    }
    if (from.endsWith('.rs')) for (const match of text.matchAll(/^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+)\s*;/gmu)) {
      resolve(from,'./'+match[1],'Rust module',true);
    }
    if (/\b(?:npm|pnpm|yarn|bun)\s+(?:run\s+|test\b|install\b|ci\b)/u.test(text)) resolve(from,'package.json','package command manifest');
    if (/\bcomposer\s/u.test(text)) resolve(from,'composer.json','Composer command manifest');
    if (/\b(?:uv|pytest|ruff|mypy)\s/u.test(text)) resolve(from,'pyproject.toml','Python command configuration');
    if (/\bpre-commit\s/u.test(text)) resolve(from,'.pre-commit-config.yaml','pre-commit hooks');
    if (/\b(?:g?make)\b/u.test(text)) resolve(from,'Makefile','make command definitions');
    if (/\bcargo\s/u.test(text)) resolve(from,'Cargo.toml','Cargo command manifest');
  };
  for (let index = 0; index < queue.length; index++) {
    const file = queue[index];
    if (visited.has(file)) continue;
    visited.add(file);
    const bytes = fs.readFileSync(path.join(directory,file));
    const identity = createHash('sha1').update(`blob ${bytes.length}\0`).update(bytes).digest('hex');
    if (identity !== blobs.get(file)) throw Error('source drift: '+file);
    if (bytes.includes(0)) continue;
    const text = bytes.toString('utf8');
    if (/^\.github\/(?:workflows\/[^/]+|actions\/.+\/action)\.ya?ml$/u.test(file)) {
      const document = parse(text);
      for (const field of ['main','pre','post']) if (typeof document.runs?.[field] === 'string') resolve(file,'./'+document.runs[field],'JavaScript action '+field,true);
      const jobs = document.jobs ?? {composite: document.runs};
      for (const [jobId,job] of Object.entries(jobs)) {
        if (!job) continue;
        if (job.uses?.startsWith('./')) resolve(file,job.uses,'local reusable workflow');
        const workingDirectory = job.defaults?.run?.['working-directory'] ?? document.defaults?.run?.['working-directory'] ?? '.';
        for (const [stepIndex,step] of (job.steps ?? []).entries()) {
          if (step.uses?.startsWith('./')) resolve(file,step.uses,'local action');
          if (typeof step.run === 'string') {
            const directory = step['working-directory'] ?? workingDirectory;
            commands.push({file,job:jobId,step:step.id ?? step.name ?? stepIndex,shell:step.shell ?? job.defaults?.run?.shell ?? document.defaults?.run?.shell ?? null,workingDirectory:directory,if:step.if ?? null,run:step.run});
            scan(file,step.run,'workflow run block');
            if (directory !== '.') unresolved.push({from:file,value:directory,reason:'working-directory requires command-context review'});
          }
        }
      }
    }
    if (file.endsWith('package.json')) {
      const manifest = JSON.parse(text);
      for (const [name,command] of Object.entries(manifest.scripts ?? {})) {
        commands.push({file,script:name,run:command,scope:'conservative: every package script, even unused scripts'});
        scan(file,command,'package script '+name);
      }
    }
    scan(file,text,'source/configuration text');
    // PHP's scripts/bootstrap.php autoloads script classes through dynamic paths.
    // Include the complete class directory, retaining that conservative assumption.
    if (file === 'scripts/bootstrap.php') resolve(file,'scripts/src','declared PHP script autoload prefix');
  }
  const unique = values => [...new Map(values.map(value => [JSON.stringify(value),value])).values()];
  return {coverageClaim:'lexical reference inventory, not complete execution closure; dynamic, implicit and semantic behavior remain unverified',roots,files:[...visited].sort(),edges:unique(edges),commands,unresolved:unique(unresolved)};
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const input = process.argv[2] ?? '/tmp/issue-1580-research/templates';
  const output = process.argv[3] ?? 'docs/case-studies/issue-1580/templates';
  const summaries = [];
  for (const source of JSON.parse(fs.readFileSync(path.join(input,'manifest.json'),'utf8'))) {
    const language = source.repo.split('/')[1].split('-')[0];
    const graph = traceTree(path.join(input,language+'-complete'),JSON.parse(fs.readFileSync(path.join(input,language+'-complete-inventory.json'),'utf8')));
    const file=language+'-ci-reference-graph.json.gz';
    const content=Buffer.from(JSON.stringify({repository:source.repo,commit:source.sha,...graph},null,2)+'\n');
    const packed=gzipSync(content);
    fs.writeFileSync(path.join(output,file),packed);
    const sha256=bytes=>createHash('sha256').update(bytes).digest('hex');
    summaries.push({language,repository:source.repo,commit:source.sha,file,
      sha256:sha256(packed),uncompressedSha256:sha256(content),
      roots:graph.roots.length,files:graph.files.length,commands:graph.commands.length,unresolved:graph.unresolved.length});
  }
  fs.writeFileSync(path.join(output,'ci-reference-graphs.json'),JSON.stringify(summaries,null,2)+'\n');
  console.log(JSON.stringify(summaries));
}
