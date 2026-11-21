from pathlib import Path
import gzip, hashlib, json, subprocess
root = Path('/tmp/gh-issue-solver-1794900398048')
base = root / 'experiments/issue-1580/evidence/checks/world-algorithm-warning-inventory'
data = json.loads(gzip.decompress((base / 'inventory.json.gz').read_bytes()))
paths = ['rust/src/link_store.rs', 'rust/src/es_meta.rs', 'rust/src/calculation.rs',
    'rust/src/world_model.rs', 'rust/src/algorithm_discovery.rs']
parents = []
for relative in paths:
    path = root / relative
    assert path.is_file(), relative
    assert not any(row['file'] == relative for row in data['warnings']), relative
    raw = path.read_bytes(); lines = len(raw.splitlines())
    assert lines < 900, (relative, lines)
    parents.append({'file': relative, 'lines': lines, 'sha256': hashlib.sha256(raw).hexdigest()})
original = (base / 'outcome.json').read_bytes()
(base / 'initial-outcome.json.gz').write_bytes(gzip.compress(original, mtime=0))
outcome = json.loads(original)
outcome['verifiedParentPaths'] = parents
outcome['selectedParentVerificationHead'] = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip()
outcome['initialAssertionCoverageCorrected'] = True
(base / 'outcome.json').write_text(json.dumps(outcome, indent=2) + '\n')
(base / Path(__file__).name).write_bytes(Path(__file__).read_bytes())
source = Path('/tmp/issue-1580-research/inventory-world-algorithm-warning-checkpoint.py')
value = source.read_text()
for old, new in [('rust/src/codegen/es.rs', 'rust/src/es_meta.rs'), ('rust/src/calc/solve.rs', 'rust/src/calculation.rs'),
                 ('rust/src/algorithmization.rs', 'rust/src/algorithm_discovery.rs')]:
    assert value.count(old) == 1, old
    value = value.replace(old, new)
source.write_text(value)
(base / 'corrected-inventory-helper.py').write_text(value)
(base / 'parent-verification-correction.json').write_text(json.dumps({
    'originalInventoryCountsStillValid': True, 'initialAssertionUsedThreeNonexistentPaths': True,
    'actualFiveParentPathsNowCheckedForExistenceWarningAbsenceAndLineCount': True,
    'originalOutcomeSha256': hashlib.sha256(original).hexdigest(), 'warningCount': len(data['warnings']),
    'fullIssueComplete': False, 'independentAuthorshipCredit': 0}, indent=2) + '\n')
print(json.dumps({'actualParentPathsVerified': parents, 'warningCount': len(data['warnings']),
    'initialAssertionCoverageCorrected': True, 'fullIssueComplete': False, 'independentAuthorshipCredit': 0}))
