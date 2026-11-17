"""Retain current-head run metadata and, on request, every completed run log."""
from pathlib import Path
import json, subprocess, sys

repo = 'link-assistant/formal-ai'
branch = 'issue-1582-79aa864de807'
sha = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
root = Path('docs/case-studies/issue-1582/raw-data/ci')
root.mkdir(parents=True, exist_ok=True)
pages = json.loads(subprocess.check_output([
    'gh', 'api', f'repos/{repo}/actions/runs?branch={branch}&per_page=100',
    '--paginate', '--slurp',
], text=True))
runs = [run for page in pages for run in page['workflow_runs'] if run['head_sha'] == sha]
selected_run = next((int(arg.split('=', 1)[1]) for arg in sys.argv if arg.startswith('--run=')), None)
log_limit = next((int(arg.split('=', 1)[1]) for arg in sys.argv if arg.startswith('--limit-logs=')), len(runs))
downloaded = 0
(root / f'repair-runs-{sha[:12]}.json').write_text(json.dumps(runs, indent=2) + '\n')
for run in runs:
    print(json.dumps({key: run.get(key) for key in ['id', 'name', 'event', 'head_sha', 'created_at', 'status', 'conclusion', 'html_url']}))
    if '--logs' not in sys.argv or run['status'] != 'completed':
        continue
    if selected_run is not None and run['id'] != selected_run:
        continue
    prefix = root / str(run['id'])
    status_path = prefix.with_suffix('.log-status.json')
    if status_path.exists():
        previous = json.loads(status_path.read_text())
        if previous.get('conclusion') == run['conclusion'] and (previous.get('exit') == 0 or run['conclusion'] == 'action_required'):
            continue
    if downloaded >= log_limit:
        continue
    downloaded += 1
    with prefix.with_suffix('.log').open('w') as output:
        result = subprocess.run(['gh', 'run', 'view', str(run['id']), '--repo', repo, '--log'], stdout=output, stderr=subprocess.PIPE, text=True)
    status_path.write_text(json.dumps({
        'id': run['id'], 'sha': sha, 'created_at': run['created_at'],
        'conclusion': run['conclusion'], 'exit': result.returncode,
        'stderr': result.stderr, 'url': run['html_url'],
    }, indent=2) + '\n')
print(json.dumps({'head': sha, 'runs': len(runs), 'pending': sum(run['status'] != 'completed' for run in runs), 'nonPassing': sum(run['status'] == 'completed' and run['conclusion'] != 'success' for run in runs)}))
