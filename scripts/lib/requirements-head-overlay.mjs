// A scratch copy of the repository whose requirement-pipeline inputs and
// outputs are exactly the committed ones (`HEAD`), for proving the JavaScript
// twins reproduce the committed bytes while other work edits the tree.
//
// - `committed` paths (files or directories) are extracted from `HEAD` with
//   `git archive`.
// - `listed` directories are materialized as empty files named after every file
//   `HEAD` holds there: the pipeline only asks whether a test file exists.
// - Every other entry is a symbolic link to the working tree, so a shard's
//   relative link resolves as it does in the repository.
//
// Writes go to the extracted copies, never through a link.
import { spawnSync } from 'node:child_process';
import { mkdirSync, readdirSync, symlinkSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';

/** The inputs and outputs of the four requirement-pipeline scripts. */
export const PIPELINE_COMMITTED = [
  'REQUIREMENTS.md',
  'README.md',
  'docs/requirements',
  'docs/requirements-traceability.md',
  'docs/status.md',
  'docs/benchmarks.md',
  'data/meta/requirement-status-ledger.lino',
  'data/meta/requirement-status-ledger',
  'data/meta/self-hosting-ledger.lino',
  'data/meta/debt-ratchet.lino',
  'data/meta/core-boundary-ledger.lino',
  'data/meta/handler-migration-ledger.lino',
  'data/meta/ladder-ratchet.lino',
  'data/meta/worker-line-budget',
  'data/seed/languages.lino',
  'data/benchmarks/external-results.lino',
];

/** Where the pipeline looks for the test file a requirement row names. */
export const PIPELINE_LISTED = ['rust/tests', 'tests', 'desktop/scripts'];

function git(repo, args, options = {}) {
  const result = spawnSync('git', ['-C', repo, ...args], { encoding: 'utf8', maxBuffer: 256 << 20, ...options });
  if (result.status !== 0) throw new Error(`git ${args.join(' ')} failed: ${result.stderr}`);
  return result.stdout;
}

/** Build the overlay in the empty directory `target`; returns `target`. */
export function headOverlay(repo, target, committed = PIPELINE_COMMITTED, listed = PIPELINE_LISTED, revision = 'HEAD') {
  mkdirSync(target, { recursive: true });
  const present = git(repo, ['ls-tree', '--name-only', revision, '--', ...committed]).split('\n').filter(Boolean);
  if (present.length) {
    const archive = spawnSync('git', ['-C', repo, 'archive', revision, '--', ...present], { maxBuffer: 512 << 20 });
    if (archive.status !== 0) throw new Error(`git archive failed: ${archive.stderr}`);
    const untar = spawnSync('tar', ['-x', '-C', target], { input: archive.stdout });
    if (untar.status !== 0) throw new Error(`tar failed: ${untar.stderr}`);
  }
  const files = git(repo, ['ls-tree', '-r', '--name-only', revision, '--', ...listed]).split('\n').filter(Boolean);
  for (const file of files) {
    mkdirSync(dirname(join(target, file)), { recursive: true });
    writeFileSync(join(target, file), '');
  }
  const owned = [...committed, ...listed];
  const link = (relative) => {
    for (const name of readdirSync(join(repo, relative))) {
      const path = relative ? `${relative}/${name}` : name;
      if (owned.includes(path)) continue;
      if (owned.some((entry) => entry.startsWith(`${path}/`))) {
        mkdirSync(join(target, path), { recursive: true });
        link(path);
      } else {
        symlinkSync(join(repo, path), join(target, path));
      }
    }
  };
  link('');
  return target;
}
