from pathlib import Path
import hashlib,json,subprocess
root=Path('/tmp/gh-issue-solver-1794900398048');out=Path('/tmp/issue-1580-research/query-import-php-diff-review');out.mkdir(exist_ok=False)
head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()
paths=['.github/workflows/security.yml','data/benchmarks/fast-pull-request-retained-jobs.json','experiments/issue-1580/codeql-included-string-observation.test.mjs','experiments/issue-1580/codeql-macro-contexts/IncludedStringPayloads.ql','experiments/issue-1580/codeql-macro-contexts/observe.py','experiments/issue-1580/template-php-input-integrity.test.mjs','experiments/issue-1580/fixtures/php-template/source.json','experiments/issue-1580/fixtures/php-template/scripts/check-version-modification.php','experiments/issue-1580/fixtures/php-template/scripts/src/ReleaseNotes.php','rust/src/seed.rs','rust/src/protocol.rs','rust/tests/web/issue-1580-template-observations.test.mjs']
r=subprocess.run(['git','diff','d48b09883ac91cd1d2319bea2afbaf0df2df8f86',head,'--',*paths],cwd=root,capture_output=True);assert r.returncode==0,r.stderr.decode()
(out/'repair.diff').write_bytes(r.stdout);lines=r.stdout.decode().splitlines(keepends=True);parts=[]
for start in range(0,len(lines),180):
    name='part-'+str(len(parts)+1)+'.diff';raw=''.join(lines[start:start+180]).encode();(out/name).write_bytes(raw);parts.append({'file':name,'startLine':start+1,'endLine':min(start+180,len(lines)),'sha256':hashlib.sha256(raw).hexdigest()})
summary={'head':head,'base':'d48b09883ac91cd1d2319bea2afbaf0df2df8f86','paths':paths,'lines':len(lines),'rawSha256':hashlib.sha256(r.stdout).hexdigest(),'parts':parts,'literalReviewComplete':False,'wholePrReviewComplete':False,'independentAuthorshipCredit':0};(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps({'head':head,'paths':len(paths),'lines':len(lines),'parts':len(parts)}))
