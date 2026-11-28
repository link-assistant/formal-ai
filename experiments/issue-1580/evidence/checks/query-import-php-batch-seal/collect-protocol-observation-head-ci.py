from pathlib import Path
from datetime import datetime, timezone
import gzip, hashlib, json, os, subprocess, sys, time, traceback

root = Path('/tmp/gh-issue-solver-1794900398048')
publication = Path('/tmp/issue-1580-research/protocol-observation-publication/outcome.json')
scratch = Path('/tmp/issue-1580-research/protocol-observation-head-ci')
if len(sys.argv) == 1:
    scratch.mkdir(exist_ok=False)
    with (scratch / 'worker.log').open('wb') as stream:
        process = subprocess.Popen([sys.executable, __file__, 'worker'], cwd=root, stdout=stream, stderr=subprocess.STDOUT, start_new_session=True)
    print(json.dumps({'pid': process.pid, 'scratch': str(scratch)}))
    sys.exit(0)
published = json.loads(publication.read_text())
assert published['complete'] and published['pushExecuted']
head = published['head']
commit_time = datetime.fromisoformat(subprocess.check_output(['git', 'show', '-s', '--format=%cI', head], cwd=root, text=True).strip())
base = root / 'experiments/issue-1580/evidence/ci' / ('native-families-' + head[:8])
base.mkdir(exist_ok=False)
records = []
saved = set()
previous_state = None
early_saved = set()

def run(name, arguments, parsed=False, checked=True):
    result = subprocess.run(arguments, cwd=root, capture_output=True)
    row = {'name': name, 'argv': arguments, 'exitCode': result.returncode}
    for label, raw in [('stdout', result.stdout), ('stderr', result.stderr)]:
        filename = name + '.' + label + '.gz'
        packed = gzip.compress(raw, mtime=0)
        (base / filename).write_bytes(packed)
        row[label] = {'file': filename, 'rawSha256': hashlib.sha256(raw).hexdigest(), 'archiveSha256': hashlib.sha256(packed).hexdigest(), 'bytes': len(raw), 'lines': len(raw.splitlines())}
    records.append(row)
    (base / 'records.json').write_text(json.dumps(records, separators=(',', ':')) + '\n')
    if checked:
        assert result.returncode == 0, (name, result.stderr.decode(errors='replace')[-2000:])
    return json.loads(result.stdout) if parsed else result.stdout

try:
    complete = False
    for tick in range(120):
        runs = run(f'observation-{tick:03}', ['gh', 'run', 'list', '--repo', 'link-assistant/formal-ai', '--branch', 'issue-1580-8d61a32a61e1', '--limit', '100', '--json', 'databaseId,workflowName,status,conclusion,headSha,createdAt,updatedAt,url'], True)
        current = [row for row in runs if row['headSha'] == head]
        assert all(datetime.fromisoformat(row['createdAt'].replace('Z', '+00:00')) >= commit_time for row in current)
        failed = [row for row in current if row['status'] == 'completed' and row['conclusion'] not in ['success', 'skipped']]
        for row in failed:
            if row['databaseId'] not in saved:
                run('failed-run-' + str(row['databaseId']), ['gh', 'run', 'view', str(row['databaseId']), '--repo', 'link-assistant/formal-ai', '--log'])
                saved.add(row['databaseId'])
        for row in current:
            if 'native' not in row['workflowName'].lower():
                continue
            if row['databaseId'] in early_saved or row['status'] != 'completed':
                continue
            jobs = run('early-jobs-' + str(row['databaseId']), ['gh', 'run', 'view', str(row['databaseId']), '--repo', 'link-assistant/formal-ai', '--json', 'jobs'], True)['jobs']
            for job in jobs:
                if 'registry' in job['name'].lower() or job['conclusion'] in ['failure', 'cancelled']:
                    run('early-job-' + str(job['databaseId']), ['gh', 'run', 'view', str(row['databaseId']), '--repo', 'link-assistant/formal-ai', '--job', str(job['databaseId']), '--log'])
            early_saved.add(row['databaseId'])
        status = {'head': head, 'observedAt': datetime.now(timezone.utc).isoformat(), 'workflowCount': len(current), 'completed': sum(row['status'] == 'completed' for row in current), 'success': sum(row['conclusion'] == 'success' for row in current), 'failed': [{'id': row['databaseId'], 'name': row['workflowName'], 'conclusion': row['conclusion'], 'url': row['url']} for row in failed], 'runs': current, 'fullIssueComplete': False}
        (base / 'latest.json').write_text(json.dumps(status, indent=2) + '\n')
        print(json.dumps({key: value for key, value in status.items() if key != 'runs'}), flush=True)
        if len(current) >= 27 and all(row['status'] == 'completed' for row in current):
            complete = True
            break
        time.sleep(30)
    if complete:
        for row in current:
            run('full-run-' + str(row['databaseId']), ['gh', 'run', 'view', str(row['databaseId']), '--repo', 'link-assistant/formal-ai', '--log'])
            if row['workflowName'] not in ['CI/CD Pipeline', 'Layered CI', 'Security'] and 'native' not in row['workflowName'].lower():
                continue
            jobs = run('jobs-' + str(row['databaseId']), ['gh', 'run', 'view', str(row['databaseId']), '--repo', 'link-assistant/formal-ai', '--json', 'jobs'], True)['jobs']
            for job in jobs:
                name = job['name'].lower()
                if 'codeql' in name or 'native' in name or 'registry' in name or 'javascript' in name or 'js tier' in name or name == 'pipeline status' or 'fast pull request' in name or job['conclusion'] in ['failure','cancelled']:
                    run('job-' + str(job['databaseId']), ['gh', 'run', 'view', str(row['databaseId']), '--repo', 'link-assistant/formal-ai', '--job', str(job['databaseId']), '--log'])
        run('pull-request-checks', ['gh', 'pr', 'checks', '1581', '--repo', 'link-assistant/formal-ai', '--json', 'name,state,bucket,workflow,link'], True, False)
    outcome = {'complete': complete, 'head': head, 'workflowCount': len(current), 'success': sum(row['conclusion'] == 'success' for row in current), 'failed': status['failed'], 'fullIssueComplete': False, 'independentAuthorshipCredit': 0}
    (base / 'outcome.json').write_text(json.dumps(outcome, indent=2) + '\n')
    (scratch / 'outcome.json').write_text(json.dumps(outcome, indent=2) + '\n')
    print(json.dumps(outcome), flush=True)
except BaseException:
    traceback.print_exc()
    (scratch / 'failure.json').write_text(json.dumps({'head': head, 'complete': False, 'traceback': traceback.format_exc()}) + '\n')
    sys.exit(1)
