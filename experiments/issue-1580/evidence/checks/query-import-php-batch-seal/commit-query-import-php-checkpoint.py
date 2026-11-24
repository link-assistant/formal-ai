from pathlib import Path
import gzip,hashlib,json,shutil,subprocess
root=Path('/tmp/gh-issue-solver-1794900398048');scratch=Path('/tmp/issue-1580-research')
seal=root/'experiments/issue-1580/evidence/checks/query-import-php-batch-seal'
assert json.loads((seal/'outcome.json').read_text())['passed']
compressed=[]
for path in sorted(seal.glob('*.diff')):
    raw=path.read_bytes();packed=gzip.compress(raw,mtime=0);name=path.name+'.gz'
    (seal/name).write_bytes(packed);path.unlink()
    compressed.append({'originalFile':path.name,'file':name,'rawSha256':hashlib.sha256(raw).hexdigest(),'archiveSha256':hashlib.sha256(packed).hexdigest()})
if compressed:
    (seal/'repair-diff-archive-mapping.json').write_text(json.dumps(compressed,indent=2)+'\n')
failure=scratch/'commit-query-import-php-checkpoint.log'
if failure.exists() and 'trailing whitespace' in failure.read_text():
    (seal/'first-commit-check-failure.log.gz').write_bytes(gzip.compress(failure.read_bytes(),mtime=0))
for name in ['docs/case-studies/issue-1580/source-warning-extraction-checkpoint.md','experiments/issue-1580/work-plan.md']:
    path=root/name;text=path.read_text();text=text.replace('ratchet. The full replay finishes 140/141 passed or skipped; its only failure is','ratchet. The full replay finishes 140/141 passed or skipped; its only\nfailure is');path.write_text(text)
allowed=['docs/case-studies/issue-1580/source-warning-extraction-checkpoint.md','experiments/issue-1580/work-plan.md',
 'experiments/issue-1580/evidence/checks/included-string-fingerprint-all-gates/',
 'experiments/issue-1580/evidence/checks/query-import-php-batch-validation/',
 'experiments/issue-1580/evidence/checks/query-import-php-batch-seal/',
 'experiments/issue-1580/evidence/checks/receipt-filenames/',
 'experiments/issue-1580/evidence/ci/codeql-producer-d48b0988/',
 'experiments/issue-1580/evidence/ci/native-families-d48b0988/']
assert subprocess.check_output(['git','branch','--show-current'],cwd=root,text=True).strip()=='issue-1580-8d61a32a61e1'
rows=subprocess.check_output(['git','status','--porcelain','--untracked-files=normal'],cwd=root,text=True).splitlines()
assert all(any(row[3:]==name or name.endswith('/') and row[3:].startswith(name) for name in allowed) for row in rows),rows
for filename in ['assess-protocol-observation-results.py','collect-protocol-observation-head-ci.py','collect-protocol-observation-producer-reports.py','commit-query-import-php-checkpoint.py']:
    shutil.copyfile(scratch/filename,seal/filename)
subprocess.run(['git','diff','--check'],cwd=root,check=True)
subprocess.run(['git','add','--',*allowed],cwd=root,check=True)
subprocess.run(['git','diff','--cached','--check'],cwd=root,check=True)
subprocess.run(['git','commit','-m','Retain complete CI failure evidence and PHP regression checkpoint','-m','Preserve original failed gate replays alongside the focused filename repair, fresh feedback and bounded source-diff review. Native and query runtime qualification remains pending; complete issue acceptance is still open.','-m','Formal-AI-Independent-Authorship: 0'],cwd=root,check=True)
assert not subprocess.check_output(['git','status','--porcelain'],cwd=root,text=True).strip()
head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()
result={'head':head,'sourceHead':json.loads((seal/'outcome.json').read_text())['sourceHead'],'treeClean':True,'fullIssueComplete':False}
(scratch/'query-import-php-publication/expected-head.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result))
