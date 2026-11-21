"""Inspect an unavailable historical run without assuming why it did not run."""
from pathlib import Path
import json, subprocess, sys
root=Path('docs/case-studies/issue-1582/raw-data/ci')
run_id=sys.argv[1] if len(sys.argv)>1 else '43904194152'
args=['gh','api',f'repos/link-assistant/formal-ai/actions/runs/{run_id}']
result=subprocess.run(args,text=True,capture_output=True)
(root/f'{run_id}-run-full.json').write_text(result.stdout if result.returncode==0 else result.stderr)
if result.returncode: raise SystemExit(result.returncode)
run=json.loads(result.stdout)
print(json.dumps({key:run.get(key) for key in ['id','name','event','status','conclusion','head_sha','check_suite_url','html_url']}))
url=run.get('check_suite_url')
if url:
 result=subprocess.run(['gh','api',url+'/check-runs','--paginate','--slurp'],text=True,capture_output=True)
 (root/f'{run_id}-check-runs-full.json').write_text(result.stdout if result.returncode==0 else result.stderr)
 print('check-runs exit',result.returncode)
 if result.returncode==0:
  for page in json.loads(result.stdout):
   for check in page.get('check_runs',[]):
    print(json.dumps({'name':check['name'],'conclusion':check['conclusion'],'output':check.get('output')}))
    if check.get('output',{}).get('annotations_count'):
     annotations=subprocess.run(['gh','api',check['output']['annotations_url'],'--paginate','--slurp'],text=True,capture_output=True)
     (root/f'{run_id}-{check["id"]}-annotations.json').write_text(annotations.stdout if annotations.returncode==0 else annotations.stderr)
