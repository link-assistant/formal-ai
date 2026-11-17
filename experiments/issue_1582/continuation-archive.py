"""Archive continuation evidence with deterministic compression and streamed hashes."""
from pathlib import Path
import gzip, hashlib, json, shutil
root=Path('docs/case-studies/issue-1582')
local=root/'raw-data/local/continuation';local.mkdir(parents=True,exist_ok=True)
def retain(source,target):
 target.parent.mkdir(parents=True,exist_ok=True)
 if source.suffix in {'.log','.diff','.ts','.js','.base64'} or source.stat().st_size>120000:
  if target.exists():target.unlink()
  target=target.with_name(target.name+'.gz')
  with source.open('rb') as incoming,target.open('wb') as output:
   with gzip.GzipFile(filename='',mode='wb',fileobj=output,mtime=0) as compressed:shutil.copyfileobj(incoming,compressed,65536)
 else:shutil.copyfile(source,target)
temporary=Path('/tmp/issue-1582-review')
for source in temporary.rglob('*'):
 if source.is_file() and source.name not in {'meta-language.tar.gz','shellcheck.tar.xz'}:
  retain(source,local/source.relative_to(temporary))
source=Path('/tmp/issue-1582-parser-install.log')
if source.exists():retain(source,local/source.name)
for source in Path('experiments/formal_ai_subagent/sandboxes/gates').glob('*.log'):retain(source,local/'gates'/source.name)
def measured(stream):
 digest=hashlib.sha256();size=lines=0;last=b''
 while chunk:=stream.read(65536):
  digest.update(chunk);size+=len(chunk);lines+=chunk.count(b'\n');last=chunk[-1:]
 return size,digest.hexdigest(),lines+int(bool(last) and last!=b'\n')
index=[]
for path in sorted((root/'raw-data').rglob('*')):
 if not path.is_file():continue
 with path.open('rb') as stream:size,digest,_=measured(stream)
 row={'path':str(path.relative_to(root)),'bytes':size,'sha256':digest}
 if path.suffix=='.gz':
  with gzip.open(path,'rb') as stream:size,digest,lines=measured(stream)
  row.update(decodedBytes=size,decodedSha256=digest,decodedLines=lines)
 index.append(row)
(root/'evidence-index.json').write_text('{"schema":"issue-1582-evidence-v1","files":[\n'+',\n'.join(json.dumps(row,separators=(',',':')) for row in index)+'\n]}\n')
print(json.dumps({'files':len(index),'bytes':sum(row['bytes'] for row in index)}))
