from pathlib import Path
import gzip, hashlib, json, os, subprocess, sys, traceback

root = Path('/tmp/gh-issue-solver-1794900398048')
scratch = Path('/tmp/issue-1580-research/typescript-currency-repair')
if len(sys.argv) == 1:
    scratch.mkdir(exist_ok=False)
    with (scratch / 'worker.log').open('wb') as output:
        process = subprocess.Popen([sys.executable, __file__, 'worker'], cwd=root, stdout=output, stderr=subprocess.STDOUT, start_new_session=True)
    print(json.dumps({'pid': process.pid, 'scratch': str(scratch)}))
    sys.exit(0)
records = []
environment = dict(os.environ, NODE_OPTIONS='--max-old-space-size=192')
def run(name, argv, expected=0):
    with (scratch / (name + '.stdout')).open('wb') as output, (scratch / (name + '.stderr')).open('wb') as errors:
        result = subprocess.run(argv, cwd=root, env=environment, stdout=output, stderr=errors)
    row = {'name': name, 'argv': argv, 'exitCode': result.returncode}
    records.append(row)
    (scratch / 'records.json').write_text(json.dumps(records, indent=2) + '\n')
    print(json.dumps(row), flush=True)
    assert result.returncode == expected, row

try:
    assert subprocess.check_output(['git', 'branch', '--show-current'], cwd=root, text=True).strip() == 'issue-1580-8d61a32a61e1'
    target = root / 'ts/worker/formal_ai_worker_web_formalize.ts'
    source = root / 'js/worker/formal_ai_worker_web_formalize.js'
    before = target.read_bytes()
    source_before = source.read_bytes()
    (scratch / 'before.ts.gz').write_bytes(gzip.compress(before, mtime=0))
    run('reproduce-current-tree-drift', ['node', 'scripts/translate-es.mjs', '--check'], 1)
    assert (scratch / 'reproduce-current-tree-drift.stdout').read_text().strip() == 'drifted: ts/worker/formal_ai_worker_web_formalize.ts'
    run('maintained-generator', ['node', 'scripts/translate-es.mjs', '--write'])
    assert source.read_bytes() == source_before
    modified = subprocess.check_output(['git', 'diff', '--name-only', '--', 'js', 'ts'], cwd=root, text=True).splitlines()
    assert modified == ['ts/worker/formal_ai_worker_web_formalize.ts'], modified
    run('complete-tree-currency', ['node', 'scripts/translate-es.mjs', '--check'])
    run('translator-regression', ['node', '--test', '--test-concurrency=1', 'rust/tests/web/translate-es.test.mjs'])
    run('page-formalization-regression', ['node', '--test', '--test-concurrency=1', 'rust/tests/web/page-formalization-route.test.mjs'])
    run('whitespace', ['git', 'diff', '--check'])
    with (scratch / 'projection.diff').open('wb') as output:
        subprocess.run(['git', 'diff', '--', 'ts/worker/formal_ai_worker_web_formalize.ts'], cwd=root, stdout=output, check=True)
    outcome = {'passed': True, 'sourceUnchanged': True, 'onlyGeneratedProjectionChanged': True, 'beforeSha256': hashlib.sha256(before).hexdigest(), 'afterSha256': hashlib.sha256(target.read_bytes()).hexdigest(), 'generator': 'scripts/translate-es.mjs --write', 'failedRun': 44848472242, 'originalLogLines': [13034, 13035, 13036], 'localNativeExecution': False, 'fullIssueComplete': False, 'independentAuthorshipCredit': 0}
    (scratch / 'outcome.json').write_text(json.dumps(outcome, indent=2) + '\n')
    evidence = root / 'experiments/issue-1580/evidence/checks/typescript-currency-repair'
    evidence.mkdir(exist_ok=False)
    for item in scratch.iterdir():
        if item.is_file() and item.name != 'worker.log':
            raw = item.read_bytes()
            (evidence / (item.name + '.gz')).write_bytes(gzip.compress(raw, mtime=0))
    (evidence / 'outcome.json').write_text(json.dumps(outcome, indent=2) + '\n')
    (evidence / 'repair-typescript-currency.py').write_bytes(Path(__file__).read_bytes())
    (evidence / 'README.md').write_text('Layered CI 44848472242 reports the generated web formalization TypeScript twin stale at original full-log lines 13034–13036. Reproduce the failure against the whole current tree, regenerate with the maintained writer, and verify that exactly one generated TypeScript projection changes while JavaScript stays byte-identical. Retain the failing replay and passing complete-tree currency and existing translator/page regressions separately. No local Rust execution or complete issue acceptance is claimed.\n')
    print(json.dumps(outcome), flush=True)
except BaseException:
    traceback.print_exc()
    (scratch / 'failure.json').write_text(json.dumps({'passed': False, 'traceback': traceback.format_exc()}) + '\n')
    sys.exit(1)
