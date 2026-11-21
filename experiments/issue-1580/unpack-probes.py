from zipfile import ZipFile
from pathlib import Path
import os
root=Path('/tmp/issue-1580-research')
with ZipFile(root/'main-binary.zip') as z:
    target=root/'formal-ai-main'
    target.write_bytes(z.read('formal-ai')); target.chmod(0o755)
with ZipFile(root/'latest-draft.zip') as z:
    for name in ['attempt-outcome.json','attempt-policy.txt','attempt.stderr.txt','formal-ai-attempt-receipt.json','formal-ai-attempt-identity.json','draft-selection.json']:
        text=z.read(name).decode(); (root/('draft-'+name)).write_text(text); print(name,text[:14000])
source=(root/'native-probe.py').read_text().replace('/usr/local/bin/formal-ai',str(root/'formal-ai-main'))
source=source.replace("value.get('choices')", "[{k:v for k,v in c['message'].items() if k in ['content','tool_calls']} for c in value.get('choices',[])]")
(root/'native-current-probe.py').write_text(source)
