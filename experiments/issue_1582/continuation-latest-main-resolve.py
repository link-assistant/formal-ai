from pathlib import Path
import re, subprocess
path=Path('docs/requirements-traceability.md')
source=path.read_text()
source,count=re.subn(r'<<<<<<< HEAD\n([\s\S]*?)=======\n([\s\S]*?)>>>>>>> origin/main\n',lambda match:match[1]+match[2],source)
assert count==1,count
path.write_text(source)
# Start the generator from main's tighter ratchet, then measure the union.
path=Path('data/meta/progressive-plan-ratchet.lino')
path.write_bytes(subprocess.check_output(['git','show','origin/main:'+str(path)]))
