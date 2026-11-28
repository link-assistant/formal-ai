"""Stream and preserve cited/latest successful or skipped runs without building."""
import gzip, hashlib, json, re, subprocess
from pathlib import Path

root=Path('experiments/issue-1580/evidence/ci')
repo='link-assistant/formal-ai'
issue=json.loads(Path('/tmp/issue-1580-research/issue-current.json').read_text())
ids={int(value) for value in re.findall(r'actions/runs/(\d+)',issue['body'])}
latest=json.loads((root/'latest-workflows.json').read_text())
ids.update(row['run']['id'] for row in latest if row['run'])
def gh(*args):
    value=subprocess.run(['gh',*args],capture_output=True,text=True)
    if value.returncode: raise RuntimeError(value.stderr)
    return value.stdout
summary=[]
for run_id in sorted(ids):
    metadata=root/f'{run_id}.json'
    if not metadata.exists():
        run=json.loads(gh('api',f'repos/{repo}/actions/runs/{run_id}'))
        pages=json.loads(gh('api',f'repos/{repo}/actions/runs/{run_id}/jobs?per_page=100','--paginate','--slurp'))
        metadata.write_text(json.dumps({'run':run,'jobs':[job for page in pages for job in page['jobs']]},separators=(',',':'))+'\n')
    value=json.loads(metadata.read_text()); run=value['run']
    archive=root/f'{run_id}.log.gz'
    if not archive.exists():
        temporary=Path('/tmp/issue-1580-research/current-run.log')
        with temporary.open('wb') as output:
            result=subprocess.run(['gh','run','view',str(run_id),'--repo',repo,'--log'],stdout=output,stderr=subprocess.PIPE)
        if result.returncode: temporary.write_bytes(b'LOG_UNAVAILABLE: '+result.stderr)
        with temporary.open('rb') as source,gzip.open(archive,'wb') as output:
            while block:=source.read(65536): output.write(block)
        temporary.unlink()
    findings=[]; digest=hashlib.sha256(); count=0
    with gzip.open(archive,'rb') as source:
        for count,line in enumerate(source,1):
            digest.update(line)
            text=line.decode('utf8',errors='replace').rstrip()
            if re.search(r'##\[(?:warning|error)\]|LOG_UNAVAILABLE|[Ff]alse (?:positive|negative)|test result: FAILED|timed out|panicked at',text):
                findings.append({'line':count,'text':text})
    (root/f'{run_id}-annotations.jsonl').write_text(''.join(json.dumps(row,ensure_ascii=False)+'\n' for row in findings))
    summary.append({'run':run_id,'name':run['name'],'sha':run['head_sha'],'created':run['created_at'],'conclusion':run['conclusion'],'lines':count,'logSha256':digest.hexdigest(),'annotations':len(findings)})
    print(run_id,run['name'],run['conclusion'],count,len(findings),flush=True)
(root/'all-cited-and-latest.json').write_text(json.dumps(summary,indent=2)+'\n')
