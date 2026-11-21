import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import YAML from 'yaml';

test('included string observation retains unavailable expansions and leaves security checks authoritative', () => {
  const workflow = YAML.parse(readFileSync('.github/workflows/security.yml', 'utf8'));
  const steps = workflow.jobs.codeql.steps;
  const probe = steps.find(step => step.name === 'Observe included Rust string payloads');
  assert.equal(probe.if, "${{ github.job == 'codeql-readonly' && matrix.language == 'rust' && inputs.macro-contexts }}");
  assert.equal(probe['timeout-minutes'], 4);
  assert.equal(probe.env.CODEQL_BINARY, '${{ steps.codeql-init.outputs.codeql-path }}');
  assert.equal(probe.env.CODEQL_DATABASES, '${{ steps.codeql-analyze.outputs.db-locations }}');
  assert.match(probe.run, /--profile included-string-payloads --trace-evaluation/);
  assert.match(probe.run, /codeql-results\/included-string-payloads/);
  assert.ok(steps.indexOf(probe) > steps.findIndex(step => step.id === 'codeql-analyze'));
  const analysis = steps.find(step => step.id === 'codeql-analyze');
  const cacheOptions = analysis.env.CODEQL_ACTION_EXTRA_OPTIONS;
  assert.match(cacheOptions, /github.job == 'codeql-readonly'/);
  assert.match(cacheOptions, /matrix.language == 'rust' && inputs.macro-contexts/);
  const options = JSON.parse(cacheOptions.match(/'(\{"database".*?\})'/u)[1]);
  assert.deepEqual(options.database['run-queries'], [
    '--keep-full-cache', '--max-disk-cache=4096',
  ]);
  assert.match(cacheOptions, /\|\| '\{\}'/u);
  const gate = steps.find(step => step.name === 'Reject unresolved CodeQL findings');
  assert.ok(steps.indexOf(gate) > steps.indexOf(probe));
  assert.equal(gate.if, '${{ !cancelled() }}');
  assert.equal(workflow.jobs['codeql-readonly'].steps, steps);
});

test('producer cache arguments compose with the action without duplicate boolean options', () => {
  const workflow = YAML.parse(readFileSync('.github/workflows/security.yml', 'utf8'));
  const analysis = workflow.jobs.codeql.steps.find(step => step.id === 'codeql-analyze');
  const options = JSON.parse(analysis.env.CODEQL_ACTION_EXTRA_OPTIONS.match(/'(\{"database".*?\})'/u)[1]);
  // The retained action supplies this option before appending extra options.
  // CodeQL treats its positive and negative spellings as the same option.
  const arguments_ = ['--expect-discarded-cache', ...options.database['run-queries']];
  const spellings = arguments_.filter(value => /^--(?:no-)?expect-discarded-cache$/u.test(value));
  assert.equal(spellings.length, 1, 'CodeQL rejects duplicate positive/negative boolean options');
});

test('included string adapter preserves complete literal text and rejects incomplete observations', { timeout: 20000 }, () => {
  const adapter = new URL('./codeql-macro-contexts/observe.py', import.meta.url);
  const code = String.raw`
import importlib.util, json, pathlib, sys, tempfile
sys.dont_write_bytecode = True
spec=importlib.util.spec_from_file_location('probe',sys.argv[1])
a=importlib.util.module_from_spec(spec)
spec.loader.exec_module(a)
with tempfile.TemporaryDirectory(prefix='included-string-observation-') as temporary:
    root=pathlib.Path(temporary)
    database=root/'database with spaces'
    (database/'db-rust').mkdir(parents=True)
    (database/'codeql-database.yml').write_text('primaryLanguage: rust\n')
    pack=root/'qlpacks/codeql/rust-queries/0.1.47'
    pack.mkdir(parents=True)
    (pack/'qlpack.yml').write_text('name: codeql/rust-queries\nversion: 0.1.47\n')
    expected='r###"first line\nРусский 中文 हिंदी\n'+'unabridged '*90+'"###'
    (root/'expected.json').write_text(json.dumps(expected))
    query=pathlib.Path(sys.argv[1]).with_name('IncludedStringPayloads.ql').read_bytes()
    (root/'query.ql').write_bytes(query)
    executable=root/'codeql'
    executable.write_text('''#!/usr/bin/env python3
import json,pathlib,sys
root=pathlib.Path(__file__).parent
mode=(root/'mode').read_text()
args=sys.argv[1:]
with (root/'calls.jsonl').open('a') as stream:
    print(json.dumps(args),file=stream)
if args[0]=='version':
    print('{"version":"fixture-only"}')
elif args[:2]==['resolve','ram']:
    assert '--ram=4096' in args and '--off-heap-ram=0' in args
    print('["-J-Xmx2400M"]')
elif args[:2]==['query','run']:
    cache=root/'completed-producer-relations'
    if '--keep-full-cache' not in args:
        cache.unlink(missing_ok=True)
    assert '--max-disk-cache=4096' in args
    assert '--warnings=error' in args
    trace=[value for value in args if value.startswith('--evaluator-log=')]
    assert bool(trace)==(mode=='traced-success')
    if trace:
        pathlib.Path(trace[0].split('=',1)[1]).write_text('fixture structured evaluator log\\n')
    assert pathlib.Path(args[-1]).read_bytes()==(root/'query.ql').read_bytes()
    assert pathlib.Path(args[-1]).name=='IncludedStringPayloads.ql'
    if mode=='query-failure':
        print('actual fixture query failed',file=sys.stderr)
        sys.exit(7)
    if mode=='query-warning':
        print('WARNING: fixture payload unavailable',file=sys.stderr)
    output=pathlib.Path(next(value.removeprefix('--output=') for value in args if value.startswith('--output=')))
    assert output.name=='included-string-payloads.bqrs'
    output.write_bytes(b'fixture BQRS')
elif args[:2]==['bqrs','decode']:
    expected=json.loads((root/'expected.json').read_text())
    rows=[[{'id':1},'rust/src/included.rs',4,1,4,12,'literal-expansion',expected,'expansion-has-cfg-node'],
          [{'id':2},'rust/src/missing.rs',8,1,8,12,'unexpanded','','no-expansion-cfg-node']]
    table={'columns':[{}]*9,'tuples':rows}
    if mode=='empty':
        table['tuples']=[]
    if mode=='wrong-columns':
        table['columns']=[{}]*8
    if mode=='short-row':
        table['tuples'][0]=rows[0][:-1]
    output=pathlib.Path(next(value.removeprefix('--output=') for value in args if value.startswith('--output=')))
    assert output.name=='included-string-payloads.json'
    output.write_text(json.dumps({'#select':table}))
else:
    sys.exit(10)
''')
    executable.chmod(0o755)
    before={p.relative_to(pack):p.read_bytes() for p in pack.rglob('*') if p.is_file()}
    for mode in ['success','traced-success','empty','wrong-columns','short-row','query-failure','query-warning','unknown-profile']:
        (root/'mode').write_text(mode)
        (root/'calls.jsonl').write_text('')
        cache=root/'completed-producer-relations'
        cache.write_bytes(b'original producer cache fixture')
        output=root/('output-'+mode)
        try:
            a.observe(executable,{'rust':str(database)},output,
                      'unknown' if mode=='unknown-profile' else 'included-string-payloads',
                      trace_evaluation=mode=='traced-success')
            assert mode in ['success','traced-success'],mode
        except (ValueError,RuntimeError):
            assert mode not in ['success','traced-success'],mode
        assert cache.read_bytes()==b'original producer cache fixture',mode
        manifest=json.loads((output/'manifest.json').read_text())
        assert manifest['complete']==(mode in ['success','traced-success']),mode
        assert manifest['semanticCoverageProven'] is False
        assert manifest['independentAuthorshipCredit']==0
        assert before=={p.relative_to(pack):p.read_bytes() for p in pack.rglob('*') if p.is_file()},mode
        if mode in ['success','traced-success']:
            assert manifest['traceEvaluation']==(mode=='traced-success')
            assert (output/'evaluation.jsonl').exists()==(mode=='traced-success')
            assert manifest['profile']=='included-string-payloads'
            assert manifest['rows']==2
            rows=json.loads((output/'included-string-payloads.json').read_text())['#select']['tuples']
            assert rows[0][7]==expected
            assert rows[1][6:]==['unexpanded','','no-expansion-cfg-node']
            assert (output/'IncludedStringPayloads.ql').read_bytes()==query
        if mode=='query-failure':
            assert manifest['commands'][-1]['exitCode']==7
            assert (output/'query.stderr').read_text()=='actual fixture query failed\n'
        if mode=='query-warning':
            assert manifest['queryRuntimeDiagnostics']==['WARNING: fixture payload unavailable']
            assert manifest['commands'][-1]['name']=='query'
        if mode=='unknown-profile':
            assert manifest['commands']==[]
            assert (root/'calls.jsonl').read_text()==''
`;
  const result = spawnSync('python3', ['-c', code, adapter.pathname], { encoding: 'utf8', timeout: 15000 });
  assert.equal(result.signal, null, result.stderr);
  assert.equal(result.status, 0, result.stdout + result.stderr);
});
