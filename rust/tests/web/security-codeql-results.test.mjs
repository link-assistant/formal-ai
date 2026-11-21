import '../../../experiments/issue-1580/codeql-included-string-observation.test.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, rmSync, readFileSync, chmodSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';
import YAML from 'yaml';
import { Script } from 'node:vm';

const cli = new URL('../../../scripts/check-codeql-results.mjs', import.meta.url);
const run = (results = []) => ({ tool: { driver: { name: 'CodeQL' } },
  invocations: [{ executionSuccessful: true }], results });
const sarif = (...runs) => ({ version: '2.1.0', runs });

test('SARIF gate rejects missing/invalid reports and checks every file/run/severity', { timeout: 20000 }, () => {
  const dir = mkdtempSync(join(tmpdir(), 'codeql-results-'));
  const check = (expected) => {
    const result = spawnSync(process.execPath, [cli.pathname, dir], { encoding: 'utf8', timeout: 5000 });
    assert.equal(result.status === 0, expected, result.stdout + result.stderr);
    return result;
  };
  const write = (value, name = 'rust.sarif') => writeFileSync(join(dir, name),
    typeof value === 'string' ? value : JSON.stringify(value));
  try {
    check(false);
    for (const invalid of ['{', {}, sarif(), sarif({}), sarif({ ...run(), results: null }),
      sarif({ ...run(), invocations: [{ executionSuccessful: false }] }),
      sarif({ ...run(), invocations: [] }), sarif({ ...run(), results: [null] })]) {
      write(invalid); check(false);
    }
    write(sarif(run())); check(true);
    for (const level of ['error', 'warning', 'note', 'none', undefined]) {
      const finding = { ruleId: 'test/rule', message: { text: 'unresolved' }, level };
      write(sarif(run(), run([finding])));
      assert.match(check(false).stderr, /test\/rule/);
      write(sarif(run([{ ...finding, suppressions: [{ kind: 'inSource' }] }]))); check(false);
      write(sarif(run([{ ...finding, baselineState: 'absent' }]))); check(true);
    }
    write(sarif(run()));
    mkdirSync(join(dir, 'nested'));
    write(sarif(run([{ ruleId: 'test/second', message: { text: 'second file' } }])), 'nested/actions.sarif');
    check(false);
  } finally { rmSync(dir, { recursive: true, force: true }); }
});

test('SARIF invocation diagnostics gate extraction even with no findings', { timeout: 20000 }, () => {
  const dir = mkdtempSync(join(tmpdir(), 'codeql-notifications-'));
  try {
    const check = (invocations) => {
      writeFileSync(join(dir, 'analysis.sarif'), JSON.stringify(sarif({ ...run(), invocations })));
      return spawnSync(process.execPath, [cli.pathname, dir], { encoding: 'utf8', timeout: 5000 });
    };
    for (const field of ['toolExecutionNotifications', 'toolConfigurationNotifications']) {
      const diagnostic = { descriptor: { id: 'extract/failure' }, message: { text: 'Extraction failed' },
        locations: [{ physicalLocation: { artifactLocation: { uri: 'src/lib.rs' }, region: { startLine: 42 } } }] };
      for (const level of ['error', 'warning', undefined]) {
        const result = check([{ executionSuccessful: true },
          { executionSuccessful: true, [field]: [{ ...diagnostic, level }] }]);
        assert.equal(result.status, 1);
        assert.match(result.stderr, /extract\/failure/);
        assert.match(result.stderr, /"src\/lib.rs":"42"/);
        assert.match(result.stderr, /Extraction failed/);
      }
      for (const value of [null, {}, 'invalid', [null], [1], [{}], [{ level: 'note' }],
        [{ message: { text: 'bad level' }, level: 'unknown' }],
        [{ message: 'invalid' }], [{ message: { text: 12 } }], [{ message: { text: 'null level' }, level: null }]]) {
        const result = check([{ executionSuccessful: true, [field]: value }]);
        assert.equal(result.status, 1, JSON.stringify(value));
        assert.match(result.stderr, /Malformed SARIF notification/);
      }
      for (const level of ['note', 'none']) {
        const result = check([{ executionSuccessful: true, [field]: [{ ...diagnostic, level }] }]);
        assert.equal(result.status, 0, result.stderr);
      }
      assert.equal(check([{ executionSuccessful: true, [field]: [] }]).status, 0);
    }
  } finally { rmSync(dir, { recursive: true, force: true }); }
});

test('SARIF collects malformed items, diagnostics and findings across files without workflow commands', { timeout: 10000 }, () => {
  const dir = mkdtempSync(join(tmpdir(), 'codeql-all-problems-'));
  const payload = 'x\n::error title=spoof::fake\r::warning::fake';
  try {
    writeFileSync(join(dir, 'a.sarif'), '{');
    writeFileSync(join(dir, 'b.sarif'), JSON.stringify(sarif({ ...run([
      null, { ruleId: payload, level: payload, message: { text: payload },
        locations: [{ physicalLocation: { artifactLocation: { uri: payload }, region: { startLine: payload } } }] },
    ]), invocations: [{ executionSuccessful: false, toolExecutionNotifications: [null,
      { descriptor: { id: payload }, level: 'error', message: { text: 'diagnostic' } }],
      toolConfigurationNotifications: {} }] })));
    writeFileSync(join(dir, 'c.sarif'), JSON.stringify(sarif(run([
      { ruleId: 'malformed/location', message: { text: 'malformed object' },
        locations: [{ physicalLocation: { artifactLocation: { uri: { toString: null } },
          region: { startLine: { toString: null } } } }] },
      { ruleId: 'last/finding', message: { text: 'last file' } },
    ]))));
    const result = spawnSync(process.execPath, [cli.pathname, dir], { encoding: 'utf8', timeout: 5000 });
    assert.equal(result.status, 1);
    for (const text of ['Invalid SARIF report', 'Malformed SARIF notification:',
      'Malformed SARIF notification array', 'Malformed SARIF result', 'diagnostic', 'last/finding',
      'Missing or unsuccessful', JSON.stringify(payload), JSON.stringify(JSON.stringify({ toString: null }))]) assert.ok(result.stderr.includes(text), text);
    assert.doesNotMatch(result.stderr, /^::/m);
  } finally { rmSync(dir, { recursive: true, force: true }); }
});

const security = () => YAML.parse(readFileSync('.github/workflows/security.yml', 'utf8')).jobs.codeql;

test('the CodeQL job checks and preserves the exact reports produced by analysis', () => {
  const job = security();
  const analysis = job.steps.find(step => step.uses?.startsWith('github/codeql-action/analyze@'));
  const gate = job.steps.find(step => step.name === 'Reject unresolved CodeQL findings');
  const archive = job.steps.find(step => step.name === 'Preserve CodeQL reports');
  assert.ok(analysis?.with?.output, 'analysis must write an explicit report directory');
  assert.ok(gate, 'successful upload alone does not establish absence of findings');
  assert.equal(gate.if, '${{ !cancelled() }}');
  assert.equal(gate.run, `node scripts/check-codeql-results.mjs "${analysis.with.output}"`);
  assert.ok(job.steps.indexOf(gate) > job.steps.indexOf(analysis));
  assert.equal(archive.if, '${{ !cancelled() }}');
  assert.equal(archive.with.path, analysis.with.output + '/');
  assert.equal(archive.with['if-no-files-found'], 'error');
  assert.ok(job.steps.indexOf(archive) > job.steps.indexOf(gate));
  assert.ok(job.steps.some(step => step.uses?.startsWith('actions/setup-node@')));
});

for (const [label, installStatus, sysrootStatus, sourcePresent, success] of [
  ['unavailable toolchain', 1, 0, true, false],
  ['failed sysroot discovery', 0, 1, true, false],
  ['missing standard-library sources', 0, 0, false, false],
  ['verified sysroot and sources', 0, 0, true, true],
]) {
  test('the actual CodeQL pin shell refuses incomplete coverage: ' + label, { timeout: 10000 }, () => {
    const dir = mkdtempSync(join(tmpdir(), 'codeql-pin-'));
    try {
      const sysroot = join(dir, 'sysroot'), environment = join(dir, 'environment');
      const source = join(sysroot, 'lib/rustlib/src/rust/library');
      if (sourcePresent) mkdirSync(source, { recursive: true });
      writeFileSync(environment, '');
      const rustup = join(dir, 'rustup');
      writeFileSync(rustup, '#!/bin/sh\nif [ "$1" = toolchain ]; then exit "$FIXTURE_INSTALL_STATUS"; fi\nprintf "%s\\n" "$FIXTURE_SYSROOT"\nexit "$FIXTURE_SYSROOT_STATUS"\n');
      chmodSync(rustup, 0o755);
      const step = security().steps.find(item => item.name === 'Pin the extractor sysroot to a std the CodeQL bundle can parse');
      const result = spawnSync('bash', ['-c', step.run], { encoding: 'utf8', timeout: 5000,
        env: { ...process.env, PATH: dir + ':' + process.env.PATH, GITHUB_ENV: environment,
          CODEQL_RUST_SYSROOT_TOOLCHAIN: 'fixture', FIXTURE_INSTALL_STATUS: String(installStatus),
          FIXTURE_SYSROOT_STATUS: String(sysrootStatus), FIXTURE_SYSROOT: sysroot } });
      assert.equal(result.signal, null, result.stderr);
      assert.equal(result.status === 0, success, result.stdout + result.stderr);
      const exported = readFileSync(environment, 'utf8');
      if (success) assert.equal(exported, `CODEQL_EXTRACTOR_RUST_OPTION_SYSROOT=${sysroot}\nCODEQL_EXTRACTOR_RUST_OPTION_SYSROOT_SRC=${source}\n`);
      else assert.equal(exported, '', 'failed prerequisites must never export a partial sysroot');
    } finally { rmSync(dir, { recursive: true, force: true }); }
  });
}


test('manual producer verification is explicit, read-only and shares the production analysis', () => {
  const workflow = YAML.parse(readFileSync('.github/workflows/security.yml', 'utf8'));
  const production = workflow.jobs.codeql;
  const verification = workflow.jobs['codeql-readonly'];
  assert.ok(verification, 'original producer reports need an opt-in branch verification path');
  assert.deepEqual(workflow.on.workflow_dispatch.inputs.codeql,
    { description: 'Run read-only CodeQL producer verification on this branch', required: false,
      default: false, type: 'boolean' });
  assert.deepEqual(verification.permissions, { actions: 'read', contents: 'read', 'security-events': 'read' });
  assert.deepEqual(verification.strategy, production.strategy);
  assert.deepEqual(verification.env, production.env);
  assert.deepEqual(verification.steps, production.steps,
    'the producer pin, analysis, result gate and report archive must be the same implementation');
  assert.equal(verification['timeout-minutes'], production['timeout-minutes']);
  const evaluate = (expression, github, inputs = {}) =>
    new Script(expression.trim().slice(3, -2)).runInNewContext({github, inputs}, {timeout: 100});
  for (const [event, ref, optedIn, expected] of [
    ['pull_request', 'refs/pull/1581/merge', true, false],
    ['pull_request_target', 'refs/heads/main', true, false],
    ['push', 'refs/heads/main', true, false],
    ['workflow_dispatch', 'refs/heads/main', true, false],
    ['workflow_dispatch', 'refs/heads/fix/example', false, false],
    ['workflow_dispatch', 'refs/heads/fix/example', true, true],
  ]) assert.equal(Boolean(evaluate(verification.if, {event_name: event, ref}, {codeql: optedIn})), expected,
    `${event} ${ref} codeql=${optedIn}`);
  assert.equal(Boolean(evaluate(verification.if,
    {event_name: 'workflow_dispatch', ref: 'refs/heads/fix/example'})), false,
    'existing checks-mode bot dispatch must remain light');
  const analysis = production.steps.find(step => step.uses?.startsWith('github/codeql-action/analyze@'));
  assert.equal(evaluate(analysis.with.upload, {job: 'codeql'}), 'always');
  for (const job of ['codeql-readonly', 'unexpected-job'])
    assert.equal(evaluate(analysis.with.upload, {job}), 'never', 'read-only analysis must not upload code scanning state');
  assert.ok(verification.steps.every(step => !/cache(?:@|\/save@)|attest@/u.test(step.uses ?? '')));
  assert.equal(verification.steps.find(step => step.uses?.startsWith('actions/checkout@')).with['persist-credentials'], false);
});


test('the extractor sysroot matches the validated CodeQL 2.27.4 proc-macro compiler', () => {
  assert.equal(security().env.CODEQL_RUST_SYSROOT_TOOLCHAIN, '1.99.0',
    'the original producer expected Rust 1.99.0 but its proc-macro server came from 1.94.0');
});

test('the producer provides standalone Rust contexts before discovery and archives their provenance', () => {
  const job = security();
  const prepare = job.steps.find(step => step.name === 'Prepare standalone Rust analysis contexts');
  assert.ok(prepare, 'scripts and the direct-rustc worker need ancestor analysis manifests');
  assert.equal(prepare.if, "matrix.language == 'rust'");
  assert.match(prepare.run, /python3 scripts\/codeql-rust-projects\.py --output/);
  assert.ok(job.steps.indexOf(prepare) < job.steps.findIndex(step => step.uses?.startsWith('github/codeql-action/init@')));
  const archive = job.steps.find(step => step.name === 'Preserve Rust analysis contexts');
  assert.equal(archive?.if, "${{ !cancelled() && matrix.language == 'rust' }}");
  assert.match(archive.with.path, /codeql-rust-contexts\/\*\*\/Cargo\.lock/);
  assert.doesNotMatch(archive.with.path, /\/target\//, 'compiled dependency caches are not provenance artifacts');
});

const projectAdapter = new URL('../../../scripts/codeql-rust-projects.py', import.meta.url);
const checkPython = (body) => {
  const code = `import importlib.util, json, tempfile, pathlib, os, sys\nsys.dont_write_bytecode = True\nspec = importlib.util.spec_from_file_location('adapter', ${JSON.stringify(projectAdapter.pathname)})\na = importlib.util.module_from_spec(spec)\nspec.loader.exec_module(a)\n` + body;
  const result = spawnSync('python3', ['-c', code], { encoding: 'utf8', timeout: 10000 });
  assert.equal(result.signal, null, result.stderr);
  assert.equal(result.status, 0, result.stdout + result.stderr);
};

test('Rust script contexts retain incompatible exact pins, feature aliases, editions and helper ownership', {timeout: 20000}, () => {
  checkPython(String.raw`
with tempfile.TemporaryDirectory(prefix='rust-contexts-') as directory:
    root = pathlib.Path(directory)
    scripts = root / 'scripts'
    scripts.mkdir()
    def script(name, manifest):
        path = scripts / name
        path.write_text('//! ' + '\n//! '.join(['\x60\x60\x60cargo', *manifest.splitlines(), '\x60\x60\x60']) + '\nfn main() {}\n')
        return path
    first = script('first.rs', '[package]\nedition="2024"\n[dependencies]\njson={package="serde_json",version="=1.0.151",features=["preserve_order"]}')
    second = script('second.rs', '[package]\nedition="2024"\n[dependencies]\njson={package="serde_json",version="=1.0.154",features=["preserve_order"]}')
    (scripts / 'plain.rs').write_text('#!/usr/bin/env rust-script\nfn main() {}\n')
    (scripts / 'helper.rs').write_text('use super::*;\nfn fixture() { let s = r#"fn main() {}"#; }\n')
    before = {p: p.read_bytes() for p in scripts.iterdir()}
    groups, sources = a.contexts(root)
    assert len(groups) == 3
    assert len(sources) == 4
    assert {g['edition'] for g in groups} == {'2021', '2024'}
    pins = set()
    for i, group in enumerate(groups):
        manifest = a.tomllib.loads(a.collector_manifest(group, 'fixture-' + str(i)))
        assert manifest['dependencies'] == group['dependencies']
        assert manifest['package']['edition'] == group['edition']
        assert manifest['workspace'] == {}
        if group['dependencies']:
            pins.add(group['dependencies']['json']['version'])
    assert pins == {'=1.0.151', '=1.0.154'}
    assert all('helper.rs' not in path for g in groups for path in g['sources'])
    assert before == {p: p.read_bytes() for p in scripts.iterdir()}
    first.write_text('//! \x60\x60\x60cargo\n//! [dependencies]\n//! x = {path="../unknown"}\n//! \x60\x60\x60\n')
    try:
        a.contexts(root)
        raise AssertionError('unsupported relative dependency silently lost')
    except ValueError as e:
        assert 'explicit context adapter' in str(e)
`);
});

test('Cargo graph conversion preserves real aliases, features, build cfgs and compiled proc-macros', {timeout: 20000}, () => {
  checkPython(String.raw`
with tempfile.TemporaryDirectory(prefix='rust-graph-') as directory:
    base = pathlib.Path(directory)
    macro = base / 'derive.so'
    macro.write_bytes(b'fixture dylib')
    def package(pid, kind='lib', build=False):
        targets = [dict(name=pid,kind=[kind],src_path=str(base/pid/'lib.rs'),edition='2024')]
        if build:
            targets.append(dict(name='build',kind=['custom-build']))
        return dict(id=pid,name=pid,version='1.2.3',manifest_path=str(base/pid/'Cargo.toml'),targets=targets)
    def dep(pid, name, kind=None):
        return dict(pkg=pid,name=name,dep_kinds=[dict(kind=kind,target=None)])
    metadata = dict(packages=[package('root'),package('library',build=True),package('derive','proc-macro'),package('build-only')],
        resolve=dict(root='root',nodes=[dict(id='root',features=[],deps=[dep('library','renamed-lib')]),
        dict(id='library',features=['derive'],deps=[dep('derive','derive'),dep('build-only','builder','build')]),
        dict(id='derive',features=[],deps=[]),dict(id='build-only',features=[],deps=[])]))
    messages = [dict(reason='build-script-executed',package_id='library',cfgs=['has_generated'],env=[['GENERATED','yes']],out_dir=str(base/'out')),
        dict(reason='compiler-artifact',package_id='derive',target=dict(kind=['proc-macro']),filenames=[str(macro)])]
    graph, dependencies = a.dependency_graph(metadata,messages,'x86_64-unknown-linux-gnu')
    assert len(graph) == 2
    library = next(c for c in graph if c['display_name'] == 'library')
    derive = next(c for c in graph if c['display_name'] == 'derive')
    assert library['cfg'] == ['feature="derive"','has_generated']
    assert library['env']['GENERATED'] == 'yes'
    assert library['env']['OUT_DIR'] == str(base/'out')
    assert library['env']['CARGO_PKG_VERSION'] == '1.2.3'
    assert derive['is_proc_macro'] and derive['proc_macro_dylib_path'] == str(macro)
    assert dependencies == [dict(crate=graph.index(library),name='renamed_lib')]
    combined = [dict(deps=[])]
    mapped = a.append_graph(graph,dependencies,combined)
    assert mapped[0]['crate'] == 2
    assert combined[2]['deps'][0]['crate'] == 1
    for missing in [messages[:1],messages[1:]]:
        try:
            a.dependency_graph(metadata,missing,'host')
            raise AssertionError('missing macro/build script silently accepted')
        except ValueError:
            pass
    macro.unlink()
    try:
        a.dependency_graph(metadata,messages,'host')
        raise AssertionError('missing macro dylib accepted')
    except ValueError:
        pass
`);
});

test('Cargo metadata inactive optionals preserve the exact hosted compiled dependency graph', {timeout: 20000}, () => {
  checkPython(String.raw`
import gzip
fixture = json.loads(gzip.decompress(pathlib.Path('rust/tests/fixtures/codeql/hosted-inactive-optional-dependency.json.gz').read_bytes()))
metadata, messages = fixture['metadata'], fixture['messages']
graph, dependencies = a.dependency_graph(metadata, messages, 'x86_64-unknown-linux-gnu')
root_id = metadata['resolve']['root']
compiled = {m['target']['name'] for m in messages if m['reason'] == 'compiler-artifact' and m['package_id'] != root_id}
assert len(graph) == 13
assert {c['display_name'] for c in graph} == compiled
assert 'serde_core' not in compiled
assert 'serde_core' in {p['name'] for p in metadata['packages']}
datetime = next(c for c in graph if c['display_name'] == 'toml_datetime')
assert datetime['cfg'] == ['feature="alloc"', 'feature="default"', 'feature="std"']
assert datetime['deps'] == []
assert {d['name'] for d in dependencies} == {'regex', 'toml_edit'}
for crate in graph:
    assert all(0 <= d['crate'] < len(graph) for d in crate['deps'])
`);
});

test('optional aliases and feature forwarding retain activated dependencies and required build data', {timeout: 20000}, () => {
  checkPython(String.raw`
import gzip, copy
fixture = json.loads(gzip.decompress(pathlib.Path('rust/tests/fixtures/codeql/hosted-inactive-optional-dependency.json.gz').read_bytes()))
for label, enabled, definitions, alias, expected in [
    ('weak only', ['std'], {'std': ['serde_core?/std']}, None, False),
    ('explicit dep', ['serde'], {'serde': ['dep:serde_core']}, None, True),
    ('strong forwarding', ['serde'], {'serde': ['serde_core/std']}, None, True),
    ('nested feature', ['serialization'], {'serialization': ['serde'], 'serde': ['dep:serde_core']}, None, True),
    ('renamed inactive', ['std'], {'std': ['serialization?/std']}, 'serialization', False),
    ('renamed active', ['serde'], {'serde': ['dep:serialization']}, 'serialization', True),
    ('hyphenated alias', ['serde'], {'serde': ['dep:serialization-data']}, 'serialization-data', True),
]:
    metadata = copy.deepcopy(fixture['metadata'])
    messages = copy.deepcopy(fixture['messages'])
    parent = next(p for p in metadata['packages'] if p['name'] == 'toml_datetime')
    node = next(n for n in metadata['resolve']['nodes'] if n['id'] == parent['id'])
    parent['features'], node['features'] = definitions, enabled
    if alias:
        parent['dependencies'][0]['rename'] = alias
        node['deps'][0]['name'] = alias.replace('-', '_')
    optional = next(p for p in metadata['packages'] if p['name'] == 'serde_core')
    # Even if another build unit compiled this package, a weak optional edge stays inactive.
    messages.append(dict(reason='build-script-executed',package_id=optional['id'],cfgs=['generated'],env=[],out_dir='/fixture/generated'))
    graph, _ = a.dependency_graph(metadata, messages, 'x86_64-unknown-linux-gnu')
    assert ('serde_core' in {c['display_name'] for c in graph}) == expected, label
    if expected:
        crate = next(c for c in graph if c['display_name'] == 'serde_core')
        assert 'generated' in crate['cfg']
        assert crate['env']['OUT_DIR'] == '/fixture/generated'
        messages.pop()
        try:
            a.dependency_graph(metadata, messages, 'x86_64-unknown-linux-gnu')
            raise AssertionError('activated optional build data silently lost: ' + label)
        except ValueError as error:
            assert 'Build script output absent' in str(error), str(error)
# A required dependency cannot disappear because its feature name is absent.
metadata = copy.deepcopy(fixture['metadata'])
parent = next(p for p in metadata['packages'] if p['name'] == 'toml_datetime')
parent['dependencies'][0]['optional'] = False
try:
    a.dependency_graph(metadata, fixture['messages'], 'x86_64-unknown-linux-gnu')
    raise AssertionError('required dependency silently lost')
except ValueError as error:
    assert 'Build script output absent' in str(error)
`);
});

test('direct-rustc worker context uses the real WASM target and source paths', {timeout: 20000}, () => {
  checkPython(String.raw`
root = pathlib.Path.cwd()
project = a.wasm_project(root, '/verified/sysroot', '/verified/rust-src')
crate = project['crates'][0]
assert crate['target'] == 'wasm32-unknown-unknown'
assert crate['edition'] == '2024'
assert crate['deps'] == []
assert pathlib.Path(crate['root_module']).is_file()
assert str(root/'rust/src') in crate['source']['include_dirs']
assert project['sysroot_src'] == '/verified/rust-src'
with tempfile.TemporaryDirectory() as directory:
    fake = pathlib.Path(directory)
    (fake/'js/wasm-worker').mkdir(parents=True)
    (fake/'js/wasm-worker/build.sh').write_text('rustc --edition=2024 --target other --crate-type cdylib')
    try:
        a.wasm_project(fake,'sysroot','source')
        raise AssertionError('changed worker target accepted')
    except ValueError:
        pass
`);
});


test('context publication waits for verified dependency builds and preserves original source paths', {timeout: 20000}, () => {
  checkPython(String.raw`
with tempfile.TemporaryDirectory(prefix='rust-context-publication-') as directory:
    base = pathlib.Path(directory)
    root = base/'checkout'
    (root/'scripts').mkdir(parents=True)
    (root/'js/wasm-worker/src').mkdir(parents=True)
    source = root/'scripts/maintained.rs'
    source.write_text('//! \x60\x60\x60cargo\n//! [dependencies]\n//! fixture="=1.2.3"\n//! \x60\x60\x60\nfn main() {}\n')
    (root/'js/wasm-worker/src/lib.rs').write_text('#![no_std]\n')
    (root/'js/wasm-worker/build.sh').write_text('rustc --edition=2024 --target wasm32-unknown-unknown --crate-type cdylib')
    original = source.read_bytes()
    rust_src = base/'rust-src'
    rust_src.mkdir()
    os.environ.update(CODEQL_RUST_SYSROOT_TOOLCHAIN='1.99.0',CODEQL_EXTRACTOR_RUST_OPTION_SYSROOT=str(base/'sysroot'),
        CODEQL_EXTRACTOR_RUST_OPTION_SYSROOT_SRC=str(rust_src))
    calls = []
    failing = True
    def command(argv, output, env):
        calls.append(argv)
        assert argv[:3] == ['rustup','run','1.99.0']
        assert env['CARGO_BUILD_JOBS'] == '1' and env['CARGO_INCREMENTAL'] == '0'
        if argv[3] == 'rustc':
            return 'host: x86_64-unknown-linux-gnu\n'
        if argv[4] == 'metadata':
            package = dict(id='fixture',name='fixture',version='1.2.3',manifest_path=str(base/'fixture/Cargo.toml'),
                targets=[dict(name='fixture',kind=['lib'],src_path=str(base/'fixture/lib.rs'),edition='2021')])
            node = dict(id='fixture',features=[],deps=[])
            return json.dumps(dict(packages=[package],resolve=dict(root='collector',nodes=[node,dict(id='collector',features=[],
                deps=[dict(pkg='fixture',name='fixture',dep_kinds=[dict(kind=None,target=None)])])])) )
        assert argv[4] == 'check' and '--locked' in argv and '--lib' in argv
        assert argv[argv.index('--target') + 1] == 'x86_64-unknown-linux-gnu'
        if failing:
            raise RuntimeError('fixture dependency compilation failed')
        return ''
    a.command = command
    try:
        a.prepare(root,base/'failed')
        raise AssertionError('dependency failure reported success')
    except RuntimeError as e:
        assert 'dependency compilation failed' in str(e)
    assert not (root/'scripts/rust-project.json').exists()
    assert not (root/'js/wasm-worker/rust-project.json').exists()
    assert source.read_bytes() == original
    failing = False
    a.prepare(root,base/'verified')
    project = json.loads((root/'scripts/rust-project.json').read_text())
    own = next(c for c in project['crates'] if c['is_workspace_member'])
    assert own['root_module'] == str(source)
    assert own['deps'] == [dict(crate=0,name='fixture')]
    assert own['cfg'] == ['test']
    assert pathlib.Path(own['env']['CARGO_MANIFEST_DIR']).is_dir()
    assert source.read_bytes() == original
    receipt = json.loads((base/'verified/contexts.json').read_text())
    assert len(receipt['projects']) == 2
    try:
        a.prepare(root,base/'overwrite')
        raise AssertionError('existing manifest overwritten')
    except ValueError as e:
        assert 'Refusing to overwrite' in str(e)
`);
});


test('optional Rust macro observation reuses the read-only producer without replacing security queries', () => {
  const workflow = YAML.parse(readFileSync('.github/workflows/security.yml', 'utf8'));
  assert.equal(workflow.on.workflow_dispatch.inputs['macro-contexts'].default, false);
  const steps = workflow.jobs.codeql.steps;
  const probe = steps.find(step => step.name === 'Observe Rust macro expansion contexts');
  assert.equal(probe.if, "${{ github.job == 'codeql-readonly' && matrix.language == 'rust' && inputs.macro-contexts }}");
  assert.equal(probe['timeout-minutes'], 4);
  assert.equal(probe.env.CODEQL_BINARY, '${{ steps.codeql-init.outputs.codeql-path }}');
  assert.equal(probe.env.CODEQL_DATABASES, '${{ steps.codeql-analyze.outputs.db-locations }}');
  const init = steps.find(step => step.id === 'codeql-init');
  const analysis = steps.find(step => step.id === 'codeql-analyze');
  assert.ok(init.uses.startsWith('github/codeql-action/init@'));
  assert.ok(analysis.uses.startsWith('github/codeql-action/analyze@'));
  assert.equal(init.with.queries, undefined, 'diagnostics must not replace the security suite');
  assert.ok(steps.indexOf(probe) > steps.indexOf(analysis));
  const gate = steps.find(step => step.name === 'Reject unresolved CodeQL findings');
  assert.ok(steps.indexOf(gate) > steps.indexOf(probe));
  assert.equal(gate.if, '${{ !cancelled() }}', 'failed diagnostics must still check original SARIF');
  assert.ok(probe.run.includes('codeql-results/macro-contexts'));
  assert.equal(workflow.jobs['codeql-readonly'].steps, steps);
});

test('macro observation preserves actual command failures and rejects incomplete tables without local compilation', {timeout: 20000}, () => {
  const adapter = new URL('../../../experiments/issue-1580/codeql-macro-contexts/observe.py', import.meta.url);
  const code = String.raw`
import importlib.util, json, pathlib, sys, tempfile
sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location('probe', sys.argv[1])
a = importlib.util.module_from_spec(spec)
spec.loader.exec_module(a)
with tempfile.TemporaryDirectory(prefix='macro-probe-') as temporary:
    root = pathlib.Path(temporary)
    database = root / 'database with spaces'
    database.mkdir()
    (database / 'db-rust').mkdir()
    (database / 'codeql-database.yml').write_text('primaryLanguage: rust\n')
    codeql = root / 'codeql'
    codeql.write_text('''#!/usr/bin/env python3
import json, pathlib, sys
root=pathlib.Path(__file__).parent
mode=(root/'mode').read_text()
args=sys.argv[1:]
with (root/'calls.jsonl').open('a') as stream:
    print(json.dumps(args),file=stream)
if args[0]=='version':
    print(json.dumps({'version':'fixture-only'}))
elif args[:2]==['resolve','ram']:
    assert '--ram=4096' in args and '--off-heap-ram=0' in args
    assert pathlib.Path(next(a.removeprefix('--dataset=') for a in args if a.startswith('--dataset='))).name=='db-rust'
    assert '--format=json' in args
    if mode=='memory-failure':
        print('actual fixture memory failure',file=sys.stderr)
        sys.exit(6)
    values={'undersized-heap':['-J-Xmx1800M'],'excessive-heap':['-J-Xmx8G'],
            'missing-heap':[],'malformed-memory':{'heap':'unknown'},
            'nonzero-offheap':['-J-Xmx2400M','--off-heap-ram=512']}
    print(json.dumps(values.get(mode,['-J-Xmx2400M'])))
elif args[:2]==['query','run']:
    assert '--threads=2' in args and '--ram=4096' in args and '--max-disk-cache=4096' in args
    assert '--keep-full-cache' in args, 'the following payload query reuses the same database cache'
    assert '--warnings=error' in args
    assert '--off-heap-ram=0' in args, 'the child evaluator must resolve the same heap-only policy'
    assert not any(a.startswith('-J') for a in args), 'launcher heap flags do not survive child sizing'
    assert pathlib.Path(args[-1]).read_bytes()==(root/'query.ql').read_bytes()
    if mode=='query-failure':
        print('actual fixture query failure',file=sys.stderr)
        sys.exit(7)
    if mode=='query-warning':
        print('WARNING: fixture runtime allocation incomplete',file=sys.stderr)
    pathlib.Path(next(a.removeprefix('--output=') for a in args if a.startswith('--output='))).write_bytes(b'fixture BQRS')
elif args[:2]==['bqrs','decode']:
    assert '--entities=all' in args and '--format=json' in args
    if mode=='decode-failure':
        sys.exit(9)
    row=[{'id':1},'rust/src/included.rs',4,1,4,12,'expanded','inside-expansion']
    table={'columns':[{}]*8,'tuples':[] if mode=='empty' else [row[:-1] if mode=='malformed' else row]}
    pathlib.Path(next(a.removeprefix('--output=') for a in args if a.startswith('--output='))).write_text(json.dumps({'#select':table}))
else:
    sys.exit(10)
''')
    codeql.chmod(0o755)
    pack = root / 'qlpacks/codeql/rust-queries/0.1.47'
    pack.mkdir(parents=True)
    manifest = pack / 'qlpack.yml'
    manifest.write_text('name: codeql/rust-queries\nversion: 0.1.47\n')
    (root / 'query.ql').write_bytes(pathlib.Path(sys.argv[1]).with_name('MacroContexts.ql').read_bytes())
    before = {p.relative_to(pack):p.read_bytes() for p in pack.rglob('*') if p.is_file()}
    for mode in ['success','memory-failure','undersized-heap','excessive-heap','missing-heap','malformed-memory','nonzero-offheap',
                 'query-failure','query-warning','decode-failure','empty','malformed','missing-database','ambiguous-pack']:
        (root / 'mode').write_text(mode)
        (root / 'calls.jsonl').write_text('')
        output = root / ('output-' + mode)
        other = root / 'qlpacks/codeql/rust-queries/other'
        if mode=='ambiguous-pack':
            other.mkdir()
            (other / 'qlpack.yml').write_bytes(manifest.read_bytes())
        try:
            a.observe(codeql, {'rust':str(root/'absent' if mode=='missing-database' else database)}, output)
            assert mode=='success', mode
        except (ValueError, RuntimeError):
            assert mode!='success', mode
        result = json.loads((output/'manifest.json').read_text())
        assert result['complete']==(mode=='success'), mode
        assert result['semanticCoverageProven'] is False
        assert result['independentAuthorshipCredit']==0
        assert before=={p.relative_to(pack):p.read_bytes() for p in pack.rglob('*') if p.is_file()}, mode
        calls=[json.loads(line) for line in (root/'calls.jsonl').read_text().splitlines()]
        if mode in ['memory-failure','undersized-heap','excessive-heap','missing-heap','malformed-memory','nonzero-offheap']:
            assert not any(call[:2]==['query','run'] for call in calls), mode
        if mode=='memory-failure':
            assert result['commands'][-1]['exitCode']==6
            assert (output/'memory.stderr').read_text()=='actual fixture memory failure\n'
        if mode=='query-warning':
            assert result['queryRuntimeDiagnostics']==['WARNING: fixture runtime allocation incomplete']
            assert result['commands'][-1]['name']=='query'
            assert not any(call[:2]==['bqrs','decode'] for call in calls)
            assert (output/'macro-contexts.bqrs').read_bytes()==b'fixture BQRS'
        if mode=='query-failure':
            assert result['commands'][-1]['exitCode']==7
            assert (output/'query.stderr').read_text()=='actual fixture query failure\n'
        if mode=='decode-failure':
            assert result['commands'][-1]['exitCode']==9
        if mode=='success':
            assert result['queryRuntimeDiagnostics']==[]
            assert result['resolvedMemory']=={'totalMiB':4096,'offHeapMiB':0,'heapMiB':2400,'arguments':['-J-Xmx2400M']}
            assert [command['name'] for command in result['commands']]==['version','memory','query','decode']
            assert result['rows']==1
            assert (output/'macro-contexts.bqrs').read_bytes()==b'fixture BQRS'
`;
  const result = spawnSync('python3', ['-c', code, adapter.pathname], { encoding: 'utf8', timeout: 15000 });
  assert.equal(result.signal, null, result.stderr);
  assert.equal(result.status, 0, result.stdout + result.stderr);
});
