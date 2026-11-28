from pathlib import Path
from datetime import datetime, timezone
import gzip, hashlib, json, subprocess
root = Path('/tmp/gh-issue-solver-1794900398048')
directory = Path('/tmp/issue-1580-research/link-record-warning-inventory')
directory.mkdir(exist_ok=False)
result = subprocess.run(['git', 'ls-files', '-z', '--cached', '--others', '--exclude-standard'], cwd=root, capture_output=True, check=True)
(directory / 'committable-files.bin').write_bytes(result.stdout)
code = "import {checkListedFiles} from './scripts/check-file-size.mjs'; import {readFileSync} from 'node:fs'; const listed=new Set(readFileSync('/tmp/issue-1580-research/link-record-warning-inventory/committable-files.bin','utf8').split('\\0').filter(Boolean)); process.stdout.write(JSON.stringify(checkListedFiles(process.cwd(),listed)));"
result = subprocess.run(['node', '--max-old-space-size=192', '--input-type=module', '-e', code], cwd=root, capture_output=True)
(directory / 'inventory.json').write_bytes(result.stdout)
(directory / 'stderr.log').write_bytes(result.stderr)
assert result.returncode == 0
data = json.loads(result.stdout)
assert not data['violations'] and not data['embeddedDataViolations']
assert not any(row['file'] == 'rust/src/link_store.rs' for row in data['warnings'])
evidence = root / 'experiments/issue-1580/evidence/checks/link-record-warning-inventory'
evidence.mkdir(exist_ok=False)
for p in directory.iterdir():
    (evidence / (p.name + '.gz')).write_bytes(gzip.compress(p.read_bytes(), mtime=0))
outcome = {'observedAt': datetime.now(timezone.utc).isoformat(), 'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip(), 'warningCount': len(data['warnings']), 'violations': 0, 'embeddedDataViolations': 0, 'linkStoreWarningResolved': True, 'limitsChanged': False, 'sourceInventorySha256': hashlib.sha256((directory / 'inventory.json').read_bytes()).hexdigest(), 'fullIssueComplete': False, 'independentAuthorshipCredit': 0}
(evidence / 'outcome.json').write_text(json.dumps(outcome, indent=2) + '\n')
(evidence / 'inventory-link-record-warning-checkpoint.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(outcome))
for row in data['warnings']:
    print(json.dumps(row))
