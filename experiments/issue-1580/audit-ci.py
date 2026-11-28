#!/usr/bin/env python3
"""Bounded, sequential CI evidence collection; no build or test execution."""
import gzip, hashlib, json, re, subprocess
from pathlib import Path

root = Path('experiments/issue-1580/evidence/ci')
root.mkdir(parents=True, exist_ok=True)
repo = 'link-assistant/formal-ai'

def gh(*args):
    result = subprocess.run(['gh', *args], capture_output=True, text=True)
    if result.returncode:
        raise RuntimeError(result.stderr.strip())
    return result.stdout

def api(endpoint):
    return json.loads(gh('api', endpoint))

workflows = json.loads(gh('api', f'repos/{repo}/actions/workflows?per_page=100', '--paginate', '--slurp'))
latest = []
for page in workflows:
    for workflow in page['workflows']:
        runs = api(f'repos/{repo}/actions/workflows/{workflow["id"]}/runs?branch=main&per_page=1')['workflow_runs']
        row = {'workflow': workflow['path'], 'name': workflow['name'], 'state': workflow['state'], 'run': runs[0] if runs else None}
        latest.append(row)
        print(workflow['name'], runs[0]['conclusion'] if runs else 'no main run', flush=True)
(root / 'latest-workflows.json').write_text(json.dumps(latest, indent=2) + '\n')

ids = {43805800854, 43805800653, 43738765561, 43715780047, 43707981518, 43706083454, 42624594054, 42621251488, 41768761745, 43813238887}
ids.update(row['run']['id'] for row in latest if row['run'] and row['run']['conclusion'] not in ('success', 'skipped'))
summaries = []
for run_id in sorted(ids):
    run = api(f'repos/{repo}/actions/runs/{run_id}')
    pages = json.loads(gh('api', f'repos/{repo}/actions/runs/{run_id}/jobs?per_page=100', '--paginate', '--slurp'))
    jobs = [job for page in pages for job in page['jobs']]
    (root / f'{run_id}.json').write_text(json.dumps({'run': run, 'jobs': jobs}, indent=2) + '\n')
    log = root / f'{run_id}.log'
    if not log.exists() and not log.with_suffix('.log.gz').exists():
        with log.open('w') as output:
            result = subprocess.run(['gh', 'run', 'view', str(run_id), '--repo', repo, '--log'], stdout=output, stderr=subprocess.PIPE, text=True)
        if result.returncode:
            log.write_text('LOG_UNAVAILABLE: ' + result.stderr)
    if log.exists():
        raw = log.read_bytes()
        with gzip.open(log.with_suffix('.log.gz'), 'wb') as output:
            output.write(raw)
        log.unlink()
    else:
        with gzip.open(log.with_suffix('.log.gz'), 'rb') as input_file:
            raw = input_file.read()
    lines = raw.decode('utf8', errors='replace').splitlines()
    interesting = set()
    for index, line in enumerate(lines):
        if re.search(r'##\[error\]|##\[warning\]|\.\.\. FAILED|^.*not ok |panicked at|error\[E\d+\]|test result: FAILED|exceeded|timed out|[Ee]rror:', line):
            interesting.update(range(max(0, index-3), min(len(lines), index+15)))
    excerpts = '\n'.join(f'{index+1}: {lines[index]}' for index in sorted(interesting))
    # Excerpts are separated into reviewable chunks under 1500 lines.
    excerpts_lines = excerpts.splitlines()
    for part in range(0, len(excerpts_lines), 1000):
        with gzip.open(root / f'{run_id}-excerpts-{part//1000+1}.txt.gz', 'wt', encoding='utf8') as output:
            output.write('\n'.join(excerpts_lines[part:part+1000])+'\n')
    failures = [{'id': job['id'], 'name': job['name'], 'conclusion': job['conclusion']} for job in jobs if job['conclusion'] not in ('success', 'skipped')]
    summaries.append({'id':run_id, 'name':run['name'], 'sha':run['head_sha'], 'created':run['created_at'], 'conclusion':run['conclusion'], 'jobs':failures, 'logSha256':hashlib.sha256(raw).hexdigest(), 'logLines':len(lines)})
    print(run_id, run['name'], run['conclusion'], len(failures), 'failed jobs', flush=True)
(root / 'summary.json').write_text(json.dumps(summaries, indent=2)+'\n')
(root / 'complete').write_text('complete\n')
