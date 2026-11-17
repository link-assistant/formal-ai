"""Retain downloaded/input/output evidence with hashes, using bounded compression."""
from pathlib import Path
import gzip, hashlib, json, shutil
root=Path('docs/case-studies/issue-1582')
local=root/'raw-data/local';local.mkdir(parents=True,exist_ok=True)
for p in [*Path('/tmp').glob('issue-1582-*.log'), *Path('/tmp').glob('issue-1582-*.diff')]:
 if p.name in ['issue-1582-archive-evidence.log','issue-1582-prepare-final-gates.log']:continue
 shutil.copyfile(p,local/p.name)
gates=Path('experiments/formal_ai_subagent/sandboxes/gates')
for p in gates.glob('*.log'):
 target=local/'gates'/p.name;target.parent.mkdir(exist_ok=True);shutil.copyfile(p,target)
# Preserve the complete older branch session log without adding another giant
# uncompressed text file. Keep its existing tracked original for history.
for p in Path('dev/log/issues/1582').rglob('solve.log'):
 shutil.copyfile(p,local/'prior-pr-session.log')
for p in (root/'raw-data').rglob('*'):
 if p.is_file() and p.suffix not in ['.gz','.zip','.png'] and (p.suffix in ['.log','.diff'] or p.stat().st_size>120000 or len(p.read_bytes().splitlines())>1400):
  compressed=p.with_name(p.name+'.gz')
  with p.open('rb') as source,compressed.open('wb') as target:
   with gzip.GzipFile(filename='',mode='wb',fileobj=target,mtime=0) as out:shutil.copyfileobj(source,out,65536)
  p.unlink()
index=[]
for p in sorted((root/'raw-data').rglob('*')):
 if not p.is_file():continue
 data=p.read_bytes();entry={'path':str(p.relative_to(root)),'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()}
 if p.suffix=='.gz':
  decoded=gzip.decompress(data);entry.update(decodedBytes=len(decoded),decodedSha256=hashlib.sha256(decoded).hexdigest(),decodedLines=len(decoded.splitlines()))
 index.append(entry)
(root/'evidence-index.json').write_text('{"schema":"issue-1582-evidence-v1","files":[\n'+',\n'.join(json.dumps(entry,separators=(',',':')) for entry in index)+'\n]}\n')
print(json.dumps({'files':len(index),'bytes':sum(x['bytes'] for x in index)}))
