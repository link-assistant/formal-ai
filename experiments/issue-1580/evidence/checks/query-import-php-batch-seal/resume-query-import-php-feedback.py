from pathlib import Path
from datetime import datetime,timezone
import gzip,hashlib,json,shutil,subprocess
root=Path('/tmp/gh-issue-solver-1794900398048');out=Path('/tmp/issue-1580-research/query-import-php-feedback')
records=json.loads((out/'records.json').read_text())
assert records[-1]['name']=='pull-request-diff' and records[-1]['exitCode']==1
argv=['gh','pr','view','1581','--repo','link-assistant/formal-ai','--json','isDraft,headRefOid,baseRefOid,state,title,body']
r=subprocess.run(argv,cwd=root,capture_output=True)
for label,raw in [('stdout',r.stdout),('stderr',r.stderr)]:(out/('pull-request.'+label+'.gz')).write_bytes(gzip.compress(raw,mtime=0))
assert r.returncode==0,r.stderr.decode();pull=json.loads(r.stdout)
assert subprocess.check_output(['git','rev-parse','origin/issue-1580-8d61a32a61e1'],cwd=root,text=True).strip()==pull['headRefOid']
merge_base=subprocess.check_output(['git','merge-base',pull['baseRefOid'],pull['headRefOid']],cwd=root,text=True).strip()
diff=out/'published-local.diff'
with diff.open('wb') as stream:r=subprocess.run(['git','diff',merge_base,pull['headRefOid']],cwd=root,stdout=stream,stderr=subprocess.PIPE)
assert r.returncode==0,r.stderr.decode()
digest=hashlib.sha256();size=lines=0
with diff.open('rb') as stream,gzip.GzipFile(filename='',mode='wb',fileobj=(out/'published-local.diff.gz').open('wb'),mtime=0) as packed:
    for raw in stream:digest.update(raw);size+=len(raw);lines+=1;packed.write(raw)
diff.unlink()
main=subprocess.check_output(['git','rev-parse','origin/main'],cwd=root,text=True).strip();ancestor=subprocess.run(['git','merge-base','--is-ancestor',main,'HEAD'],cwd=root).returncode==0
counts={};new=[]
for label,known in [('issue-comments',1),('conversation',11),('inline',0),('reviews',0)]:
    rows=[row for page in json.loads(gzip.decompress((out/(label+'.stdout.gz')).read_bytes())) for row in page];counts[label]=len(rows)
    if len(rows)>known:new.append(label)
summary={'observedAt':datetime.now(timezone.utc).isoformat(),'main':main,'mainAncestor':ancestor,'counts':counts,'newFeedbackKinds':new,'isDraft':pull['isDraft'],'publishedHead':pull['headRefOid'],'githubDiffUnavailable':'HTTP 406 maximum 300 files; full local diff against verified PR merge base retained instead','diff':{'mergeBase':merge_base,'file':'published-local.diff.gz','rawSha256':digest.hexdigest(),'bytes':size,'lines':lines,'literalReviewComplete':False},'fullIssueComplete':False,'independentAuthorshipCredit':0}
(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
