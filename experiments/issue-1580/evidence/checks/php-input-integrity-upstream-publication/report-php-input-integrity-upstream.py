from pathlib import Path
from datetime import datetime, timezone
import gzip,hashlib,json,shutil,subprocess
root=Path('/tmp/gh-issue-solver-1794900398048');scratch=Path('/tmp/issue-1580-research')
base=scratch/'php-input-integrity-upstream-publication';base.mkdir(exist_ok=False)
repo='link-foundation/php-ai-driven-development-pipeline-template';records=[]
def run(name,argv,parsed=False):
    result=subprocess.run(argv,cwd=root,capture_output=True)
    record={'name':name,'argv':argv,'exitCode':result.returncode,'observedAt':datetime.now(timezone.utc).isoformat()}
    for label in ['stdout','stderr']:
        raw=getattr(result,label);filename=name+'.'+label+'.gz';packed=gzip.compress(raw,mtime=0);(base/filename).write_bytes(packed)
        record[label]={'file':filename,'rawSha256':hashlib.sha256(raw).hexdigest(),'archiveSha256':hashlib.sha256(packed).hexdigest(),'bytes':len(raw),'lines':len(raw.splitlines())}
    records.append(record);(base/'records.json').write_text(json.dumps(records,indent=2)+'\n')
    assert result.returncode==0,(name,result.stderr.decode(errors='replace')[-2000:])
    return json.loads(result.stdout) if parsed else result.stdout.decode()
publication=json.loads((scratch/'query-import-php-publication/outcome.json').read_text());assert publication['complete']
commit='ac9cd6a55e3ea7d8738e6122f623d7676666630d'
assert run('evidence-commit',['gh','api','repos/link-assistant/formal-ai/commits/'+commit],True)['sha']==commit
pages=run('fresh-issues',['gh','api','repos/'+repo+'/issues?state=all&per_page=100','--paginate','--slurp'],True)
issues=[row for page in pages for row in page if 'pull_request' not in row]
titles={row['title']:row for row in issues};out=[]
for name,title in [('version','Version guard treats malformed or unavailable manifest evidence as unchanged'),('notes','Release-note truncation emits invalid UTF-8 at complete character boundaries')]:
    body=(scratch/'php-upstream-reports'/ (name+'.md')).read_text();assert commit in body and 'versionFromJson' not in body
    path=base/(name+'.md');path.write_text(body)
    if title in titles:
        out.append({'kind':name,'created':False,'url':titles[title]['html_url']});continue
    url=run('create-'+name,['gh','issue','create','--repo',repo,'--title',title,'--body-file',str(path)]).strip()
    assert url.startswith('https://github.com/'+repo+'/issues/')
    out.append({'kind':name,'created':True,'url':url})
comments=run('fresh-budget-comments',['gh','api','repos/'+repo+'/issues/15/comments','--paginate','--slurp'],True)
matching=[row for page in comments for row in page if commit in row['body'] and 'Seven finite process-group observations' in row['body']]
budget=base/'budget-comment.md';shutil.copyfile(scratch/'php-upstream-reports/budget-comment.md',budget)
if matching:out.append({'kind':'budget','created':False,'url':matching[0]['html_url']})
else:
    url=run('comment-budget',['gh','issue','comment','15','--repo',repo,'--body-file',str(budget)]).strip()
    assert url.startswith('https://github.com/'+repo+'/issues/15#issuecomment-')
    out.append({'kind':'budget','created':True,'url':url})
result={'complete':True,'reports':out,'sourceRepository':repo,'evidenceCommit':commit,'upstreamRepairProven':False,'fullIssueComplete':False,'independentAuthorshipCredit':0}
(base/'outcome.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
