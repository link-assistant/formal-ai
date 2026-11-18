"""Inventory every artifact attached to the selected repair-head runs."""
from pathlib import Path
import hashlib, json, subprocess, sys, zipfile
sha = next((arg.split('=', 1)[1] for arg in sys.argv if arg.startswith('--sha=')), None)
if sha is None:
    sha = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
root = Path('docs/case-studies/issue-1582/raw-data/ci')
runs = json.loads((root / f'repair-runs-{sha[:12]}.json').read_text())
inventory_file = root / f'artifacts-{sha[:12]}.json'
if '--download' in sys.argv:
    inventory = json.loads(inventory_file.read_text())
    for run in inventory:
        for artifact in run['artifacts']:
            target = root / f"artifact-{artifact['id']}.zip"
            with target.open('wb') as output:
                result = subprocess.run(['gh', 'api', f"repos/link-assistant/formal-ai/actions/artifacts/{artifact['id']}/zip"], stdout=output, check=True)
            assert zipfile.is_zipfile(target), target
            with zipfile.ZipFile(target) as archive:
                assert archive.testzip() is None
                members = [{'path': item.filename, 'bytes': item.file_size} for item in archive.infolist()]
            with target.open('rb') as source:
                digest = hashlib.file_digest(source, 'sha256').hexdigest()
            receipt = {'run': run['run'], 'sha': sha, 'artifact': artifact, 'sha256': digest, 'members': members}
            target.with_suffix('.json').write_text(json.dumps(receipt, indent=2) + '\n')
            print(json.dumps({'artifact': artifact['id'], 'bytes': target.stat().st_size, 'sha256': digest, 'members': len(members)}))
    sys.exit(0)
inventory = []
for run in runs:
    pages = json.loads(subprocess.check_output([
        'gh', 'api', f"repos/link-assistant/formal-ai/actions/runs/{run['id']}/artifacts?per_page=100",
        '--paginate', '--slurp',
    ], text=True))
    inventory.append({'run': run['id'], 'sha': sha, 'artifacts': [artifact for page in pages for artifact in page['artifacts']]})
inventory_file.write_text(json.dumps(inventory, indent=2) + '\n')
artifacts = [item for run in inventory for item in run['artifacts']]
print(json.dumps({'sha': sha, 'runs': len(inventory), 'artifacts': len(artifacts), 'bytes': sum(item['size_in_bytes'] for item in artifacts)}))
for artifact in artifacts:
    print(json.dumps({key: artifact[key] for key in ['id', 'name', 'size_in_bytes', 'expired', 'archive_download_url']}))
