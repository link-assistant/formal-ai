from pathlib import Path
from datetime import datetime,timezone
import gzip,hashlib,json,subprocess
root=Path('/tmp/gh-issue-solver-1794900398048');out=Path('/tmp/issue-1580-research/query-import-php-feedback');out.mkdir(exist_ok=False)
records=[]
def run(name,argv,parsed=False):
    r=subprocess.run(argv,cwd=root,capture_output=True);record={'name':name,'argv':argv,'exitCode':r.returncode,'observedAt':datetime.now(timezone.utc).isoformat()}
    for label,raw in [('stdout',r.stdout),('stderr',r.stderr)]:
        filename=name+'.'+label+'.gz';packed=gzip.compress(raw,mtime=0);(out/filename).write_bytes(packed);record[label]={'file':filename,'rawSha256':hashlib.sha256(raw).hexdigest(),'archiveSha256':hashlib.sha256(packed).hexdigest(),'bytes':len(raw),'lines':len(raw.splitlines())}
    records.append(record);(out/'records.json').write_text(json.dumps(records,indent=2)+'\n');assert r.returncode==0,r.stderr.decode()
    return json.loads(r.stdout) if parsed else r.stdout.decode()
run('fetch-main',['git','fetch','origin','main']);main=run('main',['git','rev-parse','origin/main']).strip()
ancestor=subprocess.run(['git','merge-base','--is-ancestor',main,'HEAD'],cwd=root).returncode==0
counts={}
for label,path,known in [('issue-comments','issues/1580/comments',1),('conversation','issues/1581/comments',11),('inline','pulls/1581/comments',0),('reviews','pulls/1581/reviews',0)]:
    pages=run(label,['gh','api','repos/link-assistant/formal-ai/'+path,'--paginate','--slurp'],True);rows=[row for page in pages for row in page];counts[label]=len(rows)
    if len(rows)>known:
        (out/(label+'-new-readable.txt')).write_text('\n\n'.join(f"ID {row['id']} {row.get('html_url','')}\n{row.get('body','')}" for row in rows[known:]));print('NEW FEEDBACK',label,len(rows)-known)
run('pull-request-diff',['gh','pr','diff','1581','--repo','link-assistant/formal-ai'])
pull=run('pull-request',['gh','pr','view','1581','--repo','link-assistant/formal-ai','--json','isDraft,headRefOid,state,title,body'],True)
summary={'observedAt':datetime.now(timezone.utc).isoformat(),'main':main,'mainAncestor':ancestor,'counts':counts,'isDraft':pull['isDraft'],'publishedHead':pull['headRefOid'],'fullIssueComplete':False,'independentAuthorshipCredit':0};(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
