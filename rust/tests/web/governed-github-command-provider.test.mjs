import test from 'node:test';
import assert from 'node:assert/strict';
import { requireRegisteredTreeEntry, createGovernedGithubCommandProvider } from '../../../scripts/governed-github-command-provider.mjs';
test('registered immutable tree source and bounded provider reject ambiguous authority', () => {
  const tree = {
    sha: 'a'.repeat(40),
    truncated: false,
    tree: [{
      path: 'release.yml',
      type: 'blob',
      mode: '100644',
      sha: 'b'.repeat(40)
    }]
  };
  assert.equal(requireRegisteredTreeEntry(tree, 'a'.repeat(40), 'release.yml', 'blob', '100644').sha, 'b'.repeat(40));
  let count = 0;
  for (const mutate of [t => t.sha = 'c'.repeat(40), t => t.truncated = true, t => t.tree.push({
    ...t.tree[0]
  }), t => t.tree[0].mode = '120000', t => t.tree[0].mode = '160000', t => t.tree[0].type = 'commit', t => t.tree[0].sha = 'unknown', t => t.tree[0].path = 'nested/release.yml']) {
    const changed = structuredClone(tree);
    mutate(changed);
    assert.throws(() => requireRegisteredTreeEntry(changed, 'a'.repeat(40), 'release.yml', 'blob', '100644'));
    count++;
  }
  for (const name of ['nested/release.yml', '..', '.']) {
    assert.throws(() => requireRegisteredTreeEntry(tree, 'a'.repeat(40), name, 'blob', '100644'));
    count++;
  }
  const expected = {
    repository: 'owner/repo',
    head: 'a'.repeat(40),
    run: '123',
    attempt: '1',
    workflowPath: '.github/workflows/release.yml'
  };
  for (const paths of [['.github/workflows/release.yml', '.github/workflows/release.yml'], ['.github/workflows/release.yml', '.github/workflows/nested/release-staged.yml'], ['.github/workflows/release.yml', '../release-staged.yml']]) {
    assert.throws(() => createGovernedGithubCommandProvider({
      expected,
      sourcePaths: paths,
      writerGroup: 'declared-writer'
    }));
    count++;
  }
  assert.equal(count, 14);
});

test('hosted governed provider refuses absent or blank explicit credential before transport',async()=>{
 const {spawnSync}=await import('node:child_process');
 const {fileURLToPath}=await import('node:url');const {dirname,resolve}=await import('node:path');
 const root=resolve(dirname(fileURLToPath(import.meta.url)),'../../..');
 const source="import {createGovernedGithubCommandProvider} from './scripts/governed-github-command-provider.mjs';createGovernedGithubCommandProvider({expected:{repository:'owner/repo',head:'a'.repeat(40),run:'1',attempt:'1',workflowPath:'.github/workflows/release.yml'},sourcePaths:['.github/workflows/release.yml','.github/workflows/release-staged.yml'],writerGroup:'writer'});";
 for(const credential of [undefined,'','   ']) {
  const environment={...process.env,GITHUB_ACTIONS:'true'};if(credential===undefined)delete environment.GH_TOKEN;else environment.GH_TOKEN=credential;
  const result=spawnSync(process.execPath,['--input-type=module','-e',source],{cwd:root,env:environment,encoding:'utf8',timeout:5000});
  assert.notEqual(result.status,0);assert.match(result.stderr,/explicit hosted GitHub read credential required/);assert.equal(result.stdout,'');
 }
});
