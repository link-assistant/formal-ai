// Read-only, SHA-pinned template comparison inputs. No builds or installs.
import { execFileSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
const gh = (...args) => execFileSync('gh', args, { encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 });
const root = '/tmp/issue-1580-research/templates';
mkdirSync(root, { recursive: true });
const records = [];
for (const language of ['js', 'rust', 'python', 'php', 'csharp', 'java', 'cpp', 'go']) {
  const repo = `link-foundation/${language}-ai-driven-development-pipeline-template`;
  const info = JSON.parse(gh('api', `repos/${repo}`));
  const commit = JSON.parse(gh('api', `repos/${repo}/commits/${info.default_branch}`));
  const tree = JSON.parse(gh('api', `repos/${repo}/git/trees/${commit.sha}?recursive=1`));
  if (tree.truncated) throw new Error(`${repo}: truncated tree`);
  writeFileSync(`${root}/${language}-tree.json`, JSON.stringify(tree, null, 2) + '\n');
  const directory = `${root}/${language}`;
  mkdirSync(directory, { recursive: true });
  const candidates = tree.tree.filter(row => row.type === 'blob' && (
    row.path.startsWith('.github/') || row.path.startsWith('scripts/') ||
    /(?:eslint|prettier|pre-commit|Makefile|Justfile|package\.json|pyproject\.toml|Cargo\.toml|composer\.json|pom\.xml|\.csproj$|\.goreleaser)/u.test(row.path)));
  const sources = [];
  for (const row of candidates) {
    const source = gh('api', `repos/${repo}/git/blobs/${row.sha}`, '--jq', '.content');
    const content = Buffer.from(source.replaceAll('\n', ''), 'base64');
    const path = `${directory}/${row.path}`;
    mkdirSync(path.slice(0, path.lastIndexOf('/')), { recursive: true });
    writeFileSync(path, content);
    sources.push({ path: row.path, sha: row.sha, bytes: content.length });
  }
  records.push({ repo, sha: commit.sha, date: commit.commit.committer.date, treeEntries: tree.tree.length, sources });
  writeFileSync(`${root}/manifest.json`, JSON.stringify(records, null, 2) + '\n');
  process.stdout.write(`${language}: ${tree.tree.length} tree entries, ${sources.length} CI/configuration sources\n`);
}
