import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,rmSync,existsSync,readdirSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,resolve} from 'node:path';
import {spawnSync,execFileSync} from 'node:child_process';
const script=resolve('scripts/ci-pull-request-admission.mjs');

test('large real Git path inventories retain every NUL-delimited change', {timeout:30000}, () => {
  const directory=mkdtempSync(join(tmpdir(),'formal-ai-large-admission-'));
  const environment={...process.env,GIT_AUTHOR_NAME:'Admission fixture',GIT_AUTHOR_EMAIL:'fixture@example.invalid',
    GIT_COMMITTER_NAME:'Admission fixture',GIT_COMMITTER_EMAIL:'fixture@example.invalid'};
  const git=(args,input='')=>execFileSync('git',args,{cwd:directory,env:environment,input,encoding:'utf8'}).trim();
  try {
    git(['init','--bare','--quiet']);
    const blob=git(['hash-object','-w','--stdin']);
    const empty=git(['mktree']);
    const base=git(['commit-tree',empty], 'Base fixture\n');
    const paths=Array.from({length:24000},(_,index)=>'guide-'+String(index).padStart(5,'0')+'-'+ 'x'.repeat(180)+'.md');
    paths.push('space name.md','line\nbreak.md','unicode-🙂.md','zz-final.rs');
    const inventory=paths.map(path=>'100644 blob '+blob+'\t'+path+'\0').join('');
    assert(Buffer.byteLength(paths.join('\0'))>4*1024*1024,'exceed the original Git stdout buffer');
    const tree=git(['mktree','-z'],inventory);
    const head=git(['commit-tree',tree,'-p',base],'Head fixture\n');
    const merge=git(['commit-tree',tree,'-p',base,'-p',head],'Synthetic pull request merge\n');
    git(['update-ref','HEAD',merge]);git(['update-ref','refs/remotes/origin/main',base]);
    const output=spawnSync(process.execPath,['--max-old-space-size=128','--input-type=module','-e',
      "import {changedPaths,admission} from "+JSON.stringify(new URL('file://'+script).href)+";"+
      "const paths=changedPaths('pull_request');"+
      "console.log(JSON.stringify({paths,native:admission(paths)['native-type-check']}));"],
      {cwd:directory,encoding:'utf8',env:environment,maxBuffer:8*1024*1024});
    assert.equal(output.status,0,output.stderr);
    const observed=JSON.parse(output.stdout);
    assert.equal(observed.paths.length,paths.length);
    assert.deepEqual(new Set(observed.paths),new Set(paths));
    assert.equal(observed.native,true,'the final source path must select native checks');
    for(const event of ['pull_request','workflow_dispatch']) {
      const result=spawnSync(process.execPath,['--max-old-space-size=128',script,'detect'],
        {cwd:directory,encoding:'utf8',env:{...environment,GITHUB_EVENT_NAME:event,GITHUB_OUTPUT:''},maxBuffer:128*1024});
      assert.equal(result.status,0,result.stderr);
      assert.match(result.stdout,/native-type-check=true/u);
    }
    git(['update-ref','-d','refs/remotes/origin/main']);
    const refused=spawnSync(process.execPath,['--max-old-space-size=128',script,'detect'],
      {cwd:directory,encoding:'utf8',env:{...environment,GITHUB_EVENT_NAME:'workflow_dispatch',GITHUB_OUTPUT:''}});
    assert.notEqual(refused.status,0,'failed Git inventory cannot authorize an empty change set');
    assert.doesNotMatch(refused.stdout,/native-type-check=false/u);
  } finally {rmSync(directory,{recursive:true,force:true});}
});

function withGitInventory(paths, check) {
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-byte-admission-'));
  const environment = {...process.env, TMPDIR: directory,
    GIT_AUTHOR_NAME: 'Admission fixture', GIT_AUTHOR_EMAIL: 'fixture@example.invalid',
    GIT_COMMITTER_NAME: 'Admission fixture', GIT_COMMITTER_EMAIL: 'fixture@example.invalid'};
  delete environment.NODE_OPTIONS;
  const git = (args, input = '') => execFileSync('git', args,
    {cwd: directory, env: environment, input, encoding: 'utf8'}).trim();
  try {
    git(['init', '--bare', '--quiet']);
    const blob = git(['hash-object', '-w', '--stdin']);
    const base = git(['commit-tree', git(['mktree'])], 'Base fixture\n');
    const tree = git(['mktree', '-z'], Buffer.concat(paths.flatMap(path =>
      [Buffer.from('100644 blob ' + blob + '\t'), Buffer.from(path), Buffer.from([0])])));
    const head = git(['commit-tree', tree, '-p', base], 'Head fixture\n');
    const merge = git(['commit-tree', tree, '-p', base, '-p', head], 'Synthetic merge\n');
    git(['update-ref', 'HEAD', merge]);
    git(['update-ref', 'refs/remotes/origin/main', base]);
    check({directory, environment, base});
  } finally { rmSync(directory, {recursive: true, force: true}); }
}

test('real Git admission rejects invalid UTF-8 before emitting outputs', {timeout: 30000}, async t => {
  for (const bytes of [[0xff], [0xe2, 0x82], [0xf0, 0x28, 0x8c, 0x28]]) {
    await t.test(Buffer.from(bytes).toString('hex'), () => {
      withGitInventory([Buffer.concat([Buffer.from('guide-'), Buffer.from(bytes), Buffer.from('.md')])],
        ({directory, environment, base}) => {
          for (const event of ['pull_request', 'workflow_dispatch', 'push']) {
            const output = join(directory, 'admission-output');
            const result = spawnSync(process.execPath, ['--max-old-space-size=128', script, 'detect'],
              {cwd: directory, encoding: 'utf8', maxBuffer: 128 * 1024, env: {...environment,
                GITHUB_EVENT_NAME: event, GITHUB_EVENT_BEFORE: base, GITHUB_OUTPUT: output}});
            assert.notEqual(result.status, 0, event + ': invalid Git path bytes must be refused');
            assert.match(result.stderr, /encoded data was not valid/u);
            assert.equal(result.stdout, '', 'no partial admission flags');
            assert.equal(existsSync(output), false, 'no GitHub output before complete validation');
            assert(!readdirSync(directory).some(name => name.startsWith('formal-ai-admission-inventory-')),
              'temporary inventory removed after decoder failure');
          }
        });
    });
  }
});

test('real Git admission preserves BOM paths and multibyte characters across read boundaries', {timeout: 30000}, () => {
  for (const paths of [['\uFEFFsource.rs'],
    ['a'.repeat(32767) + '🙂.md', 'line\nbreak.md', 'unicode-漢字.rs']]) {
    withGitInventory(paths, ({directory, environment}) => {
      const result = spawnSync(process.execPath, ['--max-old-space-size=128', '--input-type=module', '-e',
        'import {changedPaths} from ' + JSON.stringify(new URL('file://' + script).href) + ';' +
        'console.log(JSON.stringify(changedPaths("pull_request")));'],
        {cwd: directory, env: environment, encoding: 'utf8', maxBuffer: 256 * 1024});
      assert.equal(result.status, 0, result.stderr);
      assert.deepEqual(new Set(JSON.parse(result.stdout)), new Set(paths));
    });
  }
});
