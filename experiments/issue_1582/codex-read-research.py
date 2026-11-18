"""Archive primary upstream sources and print only relevant bounded snippets."""
import gzip, subprocess
from pathlib import Path
root=Path('docs/case-studies/issue-1582/raw-data/research')
sha=subprocess.check_output(['gh','api','repos/openai/codex/commits/main','--jq','.sha'],text=True).strip()
(root/'codex-source-sha.txt').write_text(sha+'\n')
for path in ['codex-rs/config/src/config_toml.rs','codex-rs/protocol/src/models.rs']:
 data=subprocess.check_output(['gh','api',f'repos/openai/codex/contents/{path}?ref={sha}','-H','Accept: application/vnd.github.raw+json'])
 name='codex-'+Path(path).name
 with gzip.GzipFile(filename='',mode='wb',fileobj=(root/(name+'.gz')).open('wb'),mtime=0) as out:out.write(data)
 lines=data.decode().splitlines()
 print(path,sha)
 for i,line in enumerate(lines):
  if any(term in line for term in ['pub fn read_only(', 'SandboxMode::ReadOnly =>', 'pub struct SandboxReadOnly','pub enum NetworkSandboxPolicy']):
   for number in range(max(0,i-3),min(len(lines),i+28)):print(f'{number+1}: {lines[number]}')
