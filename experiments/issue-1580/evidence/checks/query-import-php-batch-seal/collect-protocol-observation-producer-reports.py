from pathlib import Path
from datetime import datetime, timezone
import gzip, hashlib, json, subprocess, sys, time, traceback

root = Path('/tmp/gh-issue-solver-1794900398048')
scratch = Path('/tmp/issue-1580-research/protocol-observation-producer-original-reports')
published = json.loads(Path('/tmp/issue-1580-research/protocol-observation-publication/outcome.json').read_text())
base = root / ('experiments/issue-1580/evidence/ci/codeql-producer-' + published['head'][:8])
if len(sys.argv) == 1:
    scratch.mkdir(exist_ok=False)
    with (scratch / 'worker.log').open('wb') as stream:
        child = subprocess.Popen([sys.executable, __file__, 'worker'], cwd=root,
                                 stdout=stream, stderr=subprocess.STDOUT, start_new_session=True)
    print(json.dumps({'pid': child.pid, 'directory': str(scratch)}))
    sys.exit(0)
base.mkdir(exist_ok=False)
records = []
def run(name, argv, parsed=False):
    result = subprocess.run(argv, cwd=root, capture_output=True)
    record = {'name': name, 'argv': argv, 'exitCode': result.returncode,
              'observedAt': datetime.now(timezone.utc).isoformat()}
    for label, raw in [('stdout', result.stdout), ('stderr', result.stderr)]:
        name = record['name'] + '.' + label + '.gz'
        packed = gzip.compress(raw, mtime=0)
        (base / name).write_bytes(packed)
        record[label] = {'file': name, 'rawSha256': hashlib.sha256(raw).hexdigest(),
                         'archiveSha256': hashlib.sha256(packed).hexdigest(),
                         'bytes': len(raw), 'lines': len(raw.splitlines())}
    records.append(record)
    (base / 'records.json').write_text(json.dumps(records, separators=(',', ':')) + '\n')
    assert result.returncode == 0, (record['name'], result.stderr.decode(errors='replace')[-2000:])
    return json.loads(result.stdout) if parsed else result.stdout
try:
    head = published['head']
    dispatch = Path('/tmp/issue-1580-research/protocol-observation-publication/dispatch-security.stdout').read_text().strip()
    rid = int(dispatch.rsplit('/', 1)[1])
    latest = root / ('experiments/issue-1580/evidence/ci/native-families-' + head[:8] + '/latest.json')
    finished = False
    for tick in range(120):
        if latest.exists():
            observed = json.loads(latest.read_text())
            assert observed['head'] == head
            found = [row for row in observed['runs'] if row['databaseId'] == rid]
            if found and found[0]['status'] == 'completed':
                finished = True
                break
        time.sleep(30)
    assert finished, 'Existing finite CI observer has not recorded producer completion'
    state = run('state', ['gh', 'run', 'view', str(rid), '--repo', 'link-assistant/formal-ai',
                '--json', 'event,headSha,status,conclusion,jobs,url,createdAt,updatedAt'], True)
    assert state['headSha'] == head and state['status'] == 'completed' and state['event'] == 'workflow_dispatch'
    run('full-log', ['gh', 'run', 'view', str(rid), '--repo', 'link-assistant/formal-ai', '--log'])
    for job in state['jobs']:
        run('job-' + str(job['databaseId']), ['gh', 'run', 'view', str(rid), '--repo', 'link-assistant/formal-ai',
            '--job', str(job['databaseId']), '--log'])
    pages = run('artifacts', ['gh', 'api', f'repos/link-assistant/formal-ai/actions/runs/{rid}/artifacts',
                '--paginate', '--slurp'], True)
    artifacts = [artifact for page in pages for artifact in page['artifacts']]
    copied = []
    for artifact in artifacts:
        if not artifact['name'].startswith(('codeql-results-', 'codeql-rust-contexts-')):
            continue
        assert not artifact['expired']
        destination = scratch / 'download' / str(artifact['id'])
        destination.mkdir(parents=True)
        run('download-' + str(artifact['id']), ['gh', 'run', 'download', str(rid), '--repo',
            'link-assistant/formal-ai', '--name', artifact['name'], '--dir', str(destination)])
        for source in sorted(destination.rglob('*')):
            if not source.is_file():
                continue
            assert not source.is_symlink()
            raw = source.read_bytes()
            relative = source.relative_to(destination).as_posix()
            name = 'producer-' + str(artifact['id']) + '-' + relative.replace('/', '_') + '.gz'
            packed = gzip.compress(raw, mtime=0)
            (base / name).write_bytes(packed)
            copied.append({'artifact': artifact, 'relativePath': relative, 'file': name,
                'rawSha256': hashlib.sha256(raw).hexdigest(), 'archiveSha256': hashlib.sha256(packed).hexdigest(),
                'bytes': len(raw), 'lines': len(raw.splitlines())})
    assert len([row for row in copied if row['relativePath'].endswith('.sarif')]) == 2
    (base / 'producer-files.json').write_text(json.dumps(copied, indent=2) + '\n')
    outcome = {'complete': True, 'head': head, 'runId': rid, 'url': state['url'], 'conclusion': state['conclusion'],
               'producerFiles': len(copied), 'fullIssueComplete': False, 'independentAuthorshipCredit': 0}
    (base / 'outcome.json').write_text(json.dumps(outcome, indent=2) + '\n')
    (scratch / 'outcome.json').write_text(json.dumps(outcome, indent=2) + '\n')
    print(json.dumps(outcome), flush=True)
except BaseException:
    traceback.print_exc()
    (scratch / 'failure.json').write_text(json.dumps({'complete': False, 'traceback': traceback.format_exc()}) + '\n')
    sys.exit(1)
